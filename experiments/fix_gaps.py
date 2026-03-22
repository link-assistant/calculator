#!/usr/bin/env python3
"""
Fix gaps in ECB currency data files:
1. Remove bad entries from Jan 22/25 2026 (from broken pipeline)
2. Fetch correct daily data from Frankfurter API for the gap period
3. Merge into existing files

Also update the download script to handle the transition better.
"""

import json
import time
import urllib.request
from datetime import datetime
from pathlib import Path

# ECB pairs that need fixing
FRANKFURTER_PAIRS = [
    ("USD", "EUR"), ("USD", "GBP"), ("USD", "JPY"), ("USD", "CHF"),
    ("USD", "CNY"), ("USD", "CAD"), ("USD", "AUD"), ("USD", "NZD"),
    ("USD", "SEK"), ("USD", "NOK"), ("USD", "DKK"), ("USD", "PLN"),
    ("USD", "CZK"), ("USD", "HUF"), ("USD", "TRY"), ("USD", "MXN"),
    ("USD", "BRL"), ("USD", "INR"), ("USD", "KRW"), ("USD", "SGD"),
    ("USD", "HKD"), ("USD", "ZAR"), ("USD", "CLF"),
    ("EUR", "USD"), ("EUR", "GBP"), ("EUR", "JPY"), ("EUR", "CHF"),
    ("GBP", "USD"), ("GBP", "EUR"),
]

# Dates that have bad data from the broken pipeline
BAD_DATES = {"2026-01-22", "2026-01-25"}

def fetch_json(url):
    """Fetch JSON from URL."""
    req = urllib.request.Request(url, headers={
        "User-Agent": "calculator-rates-updater/1.0",
        "Accept": "application/json",
    })
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.loads(response.read().decode('utf-8'))

def parse_lino_file(file_path):
    """Parse a .lino file into header lines and rate entries."""
    header_lines = []
    rates = {}  # date -> rate_value_str

    content = file_path.read_text()
    lines = content.rstrip('\n').split('\n')

    in_data = False
    for line in lines:
        stripped = line.strip()
        if in_data:
            if stripped and stripped[0].isdigit() and len(stripped) >= 10 and stripped[4] == '-':
                date_str = stripped.split()[0]
                rate_str = stripped.split()[1] if len(stripped.split()) > 1 else ""
                rates[date_str] = rate_str
            else:
                header_lines.append(line)
        else:
            header_lines.append(line)
            if stripped in ("rates:", "data:"):
                in_data = True

    return header_lines, rates

def write_lino_file(file_path, header_lines, rates, indent="    "):
    """Write a .lino file from header and rates."""
    lines = list(header_lines)
    for date_str in sorted(rates.keys()):
        lines.append(f"{indent}{date_str} {rates[date_str]}")
    file_path.write_text('\n'.join(lines) + '\n')

def main():
    data_dir = Path(__file__).parent.parent / "data" / "currency"

    # Step 1: Remove bad entries from all ECB files
    print("Step 1: Removing bad entries from ECB files...")
    bad_removed = 0
    for from_curr, to_curr in FRANKFURTER_PAIRS:
        file_path = data_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"
        if not file_path.exists():
            continue

        header, rates = parse_lino_file(file_path)
        removed = 0
        for bad_date in BAD_DATES:
            if bad_date in rates:
                print(f"  {file_path.name}: removing bad entry {bad_date} = {rates[bad_date]}")
                del rates[bad_date]
                removed += 1

        if removed:
            write_lino_file(file_path, header, rates)
            bad_removed += removed

    print(f"  Removed {bad_removed} bad entries total.\n")

    # Step 2: Fetch correct daily data for Jan 20-25, 2026
    # The gap is from the last weekly entry (Jan 19) to the first daily entry (Jan 26)
    # ECB doesn't publish on weekends, so business days are: Mon 19 (have), Tue 20, Wed 21, Thu 22, Fri 23
    # Jan 24-25 are Saturday/Sunday (no ECB data expected)
    print("Step 2: Fetching correct daily data for 2026-01-19..2026-01-25...")

    for from_curr, to_curr in FRANKFURTER_PAIRS:
        file_path = data_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"
        if not file_path.exists():
            continue

        print(f"  {from_curr}->{to_curr}...", end=" ", flush=True)

        url = f"https://api.frankfurter.dev/v1/2026-01-19..2026-01-25?from={from_curr}&to={to_curr}"
        try:
            data = fetch_json(url)
        except Exception as e:
            print(f"ERROR: {e}")
            continue

        if data and "rates" in data:
            header, rates = parse_lino_file(file_path)
            added = 0
            for date_str, day_rates in data["rates"].items():
                if to_curr in day_rates:
                    rate = day_rates[to_curr]
                    if date_str not in rates:
                        rates[date_str] = str(rate)
                        added += 1
                    else:
                        # Date exists — check if it's different (correcting bad data)
                        existing = float(rates[date_str])
                        if abs(existing - rate) / max(abs(existing), abs(rate)) > 0.01:
                            print(f"correcting {date_str}: {rates[date_str]} -> {rate}", end=" ", flush=True)
                            rates[date_str] = str(rate)
                            added += 1

            write_lino_file(file_path, header, rates)
            print(f"{added} new entries")
        else:
            print("no data")

        time.sleep(0.2)

    print("\nDone! Run check_gaps.py to verify.")

if __name__ == "__main__":
    main()
