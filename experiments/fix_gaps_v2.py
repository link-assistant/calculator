#!/usr/bin/env python3
"""
Backfill daily data for Jan 12-18, 2026 to eliminate the weekly-to-daily transition gap.
The weekly entries on Jan 5 and Jan 12 already exist, but days Jan 13-16 (business days) are missing.
Jan 17-18 are weekend (no ECB data).
"""

import json
import time
import urllib.request
from pathlib import Path

FRANKFURTER_PAIRS = [
    ("USD", "EUR"), ("USD", "GBP"), ("USD", "JPY"), ("USD", "CHF"),
    ("USD", "CNY"), ("USD", "CAD"), ("USD", "AUD"), ("USD", "NZD"),
    ("USD", "SEK"), ("USD", "NOK"), ("USD", "DKK"), ("USD", "PLN"),
    ("USD", "CZK"), ("USD", "HUF"), ("USD", "TRY"), ("USD", "MXN"),
    ("USD", "BRL"), ("USD", "INR"), ("USD", "KRW"), ("USD", "SGD"),
    ("USD", "HKD"), ("USD", "ZAR"),
    ("EUR", "USD"), ("EUR", "GBP"), ("EUR", "JPY"), ("EUR", "CHF"),
    ("GBP", "USD"), ("GBP", "EUR"),
]

def fetch_json(url):
    req = urllib.request.Request(url, headers={
        "User-Agent": "calculator-rates-updater/1.0",
        "Accept": "application/json",
    })
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.loads(response.read().decode('utf-8'))

def parse_lino_file(file_path):
    header_lines = []
    rates = {}
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
    lines = list(header_lines)
    for date_str in sorted(rates.keys()):
        lines.append(f"{indent}{date_str} {rates[date_str]}")
    file_path.write_text('\n'.join(lines) + '\n')

def main():
    data_dir = Path(__file__).parent.parent / "data" / "currency"

    # Backfill Jan 5-18 to get daily data covering the last two weekly entries
    # We already have Jan 5 (weekly) and Jan 12 (weekly), need Jan 6-11 and Jan 13-16
    print("Fetching daily data for 2026-01-05..2026-01-18...")

    for from_curr, to_curr in FRANKFURTER_PAIRS:
        file_path = data_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"
        if not file_path.exists():
            continue

        print(f"  {from_curr}->{to_curr}...", end=" ", flush=True)

        url = f"https://api.frankfurter.dev/v1/2026-01-05..2026-01-18?from={from_curr}&to={to_curr}"
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

            write_lino_file(file_path, header, rates)
            print(f"{added} new entries")
        else:
            print("no data")

        time.sleep(0.2)

    print("\nDone!")

if __name__ == "__main__":
    main()
