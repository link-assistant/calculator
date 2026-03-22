#!/usr/bin/env python3
"""
Backfill all ECB daily data from 2021-01-25 through 2025-11-30.
This converts the historical weekly data to complete daily data.

The Frankfurter API supports date ranges, so we fetch in 6-month chunks
to keep response sizes manageable.
"""

import json
import time
import urllib.request
from datetime import datetime, timedelta
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

def fetch_json(url, max_retries=3):
    for attempt in range(max_retries):
        try:
            req = urllib.request.Request(url, headers={
                "User-Agent": "calculator-rates-updater/1.0",
                "Accept": "application/json",
            })
            with urllib.request.urlopen(req, timeout=60) as response:
                return json.loads(response.read().decode('utf-8'))
        except Exception as e:
            if attempt < max_retries - 1:
                print(f" retry({attempt+1})...", end="", flush=True)
                time.sleep(2)
            else:
                raise

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

def generate_date_chunks(start_date, end_date, chunk_days=180):
    """Generate (start, end) date pairs in chunks."""
    chunks = []
    current = datetime.strptime(start_date, "%Y-%m-%d")
    end = datetime.strptime(end_date, "%Y-%m-%d")
    while current < end:
        chunk_end = min(current + timedelta(days=chunk_days), end)
        chunks.append((current.strftime("%Y-%m-%d"), chunk_end.strftime("%Y-%m-%d")))
        current = chunk_end + timedelta(days=1)
    return chunks

def main():
    data_dir = Path(__file__).parent.parent / "data" / "currency"

    # Generate date chunks from 2021-01-25 to 2025-11-30
    chunks = generate_date_chunks("2021-01-25", "2025-11-30", chunk_days=180)
    print(f"Will fetch {len(chunks)} date chunks x {len(FRANKFURTER_PAIRS)} pairs")
    print(f"Total API calls: ~{len(chunks) * len(FRANKFURTER_PAIRS)}")
    print()

    total_added = 0

    for chunk_idx, (chunk_start, chunk_end) in enumerate(chunks):
        print(f"\n--- Chunk {chunk_idx+1}/{len(chunks)}: {chunk_start} to {chunk_end} ---")

        for from_curr, to_curr in FRANKFURTER_PAIRS:
            file_path = data_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"
            if not file_path.exists():
                continue

            print(f"  {from_curr}->{to_curr}...", end=" ", flush=True)

            url = f"https://api.frankfurter.dev/v1/{chunk_start}..{chunk_end}?from={from_curr}&to={to_curr}"
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

                if added > 0:
                    write_lino_file(file_path, header, rates)
                total_added += added
                print(f"{added} new")
            else:
                print("no data")

            time.sleep(0.15)  # Be nice to API

    print(f"\n\nTotal new entries added: {total_added}")
    print("Done! Run check_gaps.py to verify.")

if __name__ == "__main__":
    main()
