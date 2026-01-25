#!/usr/bin/env python3
"""
Download historical exchange rates from multiple sources and convert to .lino format.

Sources:
- Frankfurter API (https://frankfurter.dev/) - ECB data from 1999, 30+ currencies (no RUB)
- CBR API (http://cbr.ru/) - Russian Central Bank data from 1992, RUB rates

Output format (.lino - links notation):
    rate:
      from USD
      to RUB
      value 89.50
      date 2026-01-25
      source 'frankfurter.dev'
"""

import json
import os
import sys
import time
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional


# Currency pairs to download - popular pairs that users commonly need
FRANKFURTER_PAIRS = [
    # USD base pairs
    ("USD", "EUR"),
    ("USD", "GBP"),
    ("USD", "JPY"),
    ("USD", "CHF"),
    ("USD", "CNY"),
    ("USD", "CAD"),
    ("USD", "AUD"),
    ("USD", "NZD"),
    ("USD", "SEK"),
    ("USD", "NOK"),
    ("USD", "DKK"),
    ("USD", "PLN"),
    ("USD", "CZK"),
    ("USD", "HUF"),
    ("USD", "TRY"),
    ("USD", "MXN"),
    ("USD", "BRL"),
    ("USD", "INR"),
    ("USD", "KRW"),
    ("USD", "SGD"),
    ("USD", "HKD"),
    ("USD", "ZAR"),
    # EUR base pairs (for common non-USD conversions)
    ("EUR", "USD"),
    ("EUR", "GBP"),
    ("EUR", "JPY"),
    ("EUR", "CHF"),
    # GBP base pairs
    ("GBP", "USD"),
    ("GBP", "EUR"),
]

# CBR currency codes for RUB pairs
CBR_CURRENCIES = {
    "R01235": "USD",  # US Dollar
    "R01239": "EUR",  # Euro (from 1999)
    "R01035": "GBP",  # British Pound
    "R01820": "JPY",  # Japanese Yen
    "R01775": "CHF",  # Swiss Franc
    "R01375": "CNY",  # Chinese Yuan
}


def fetch_json(url: str, max_retries: int = 3) -> Optional[dict]:
    """Fetch JSON from URL with retries."""
    for attempt in range(max_retries):
        try:
            with urllib.request.urlopen(url, timeout=30) as response:
                return json.loads(response.read().decode('utf-8'))
        except Exception as e:
            if attempt < max_retries - 1:
                time.sleep(1)
            else:
                print(f"  Error fetching {url}: {e}", file=sys.stderr)
                return None


def fetch_xml(url: str, max_retries: int = 3) -> Optional[ET.Element]:
    """Fetch XML from URL with retries."""
    for attempt in range(max_retries):
        try:
            with urllib.request.urlopen(url, timeout=30) as response:
                content = response.read().decode('windows-1251')
                return ET.fromstring(content)
        except Exception as e:
            if attempt < max_retries - 1:
                time.sleep(1)
            else:
                print(f"  Error fetching {url}: {e}", file=sys.stderr)
                return None


def write_lino_rate(output_dir: Path, from_curr: str, to_curr: str,
                    date: str, rate: float, source: str):
    """Write a single rate to a .lino file."""
    # Create directory structure: data/currency/{from}/{to}/
    rate_dir = output_dir / from_curr.lower() / to_curr.lower()
    rate_dir.mkdir(parents=True, exist_ok=True)

    # File name: {date}.lino
    file_path = rate_dir / f"{date}.lino"

    # Write in links notation format
    content = f"""rate:
  from {from_curr.upper()}
  to {to_curr.upper()}
  value {rate}
  date {date}
  source '{source}'
"""

    file_path.write_text(content)


def download_frankfurter_rates(output_dir: Path, start_date: str, end_date: str):
    """Download rates from Frankfurter API (ECB data)."""
    print(f"\nDownloading Frankfurter rates from {start_date} to {end_date}...")

    for from_curr, to_curr in FRANKFURTER_PAIRS:
        print(f"  {from_curr} -> {to_curr}...", end=" ", flush=True)

        url = f"https://api.frankfurter.app/{start_date}..{end_date}?from={from_curr}&to={to_curr}"
        data = fetch_json(url)

        if data and "rates" in data:
            rates = data["rates"]
            count = 0
            for date_str, day_rates in rates.items():
                if to_curr in day_rates:
                    rate = day_rates[to_curr]
                    write_lino_rate(output_dir, from_curr, to_curr, date_str, rate, "frankfurter.dev (ECB)")
                    count += 1
            print(f"{count} rates")
        else:
            print("no data")

        # Be nice to the API
        time.sleep(0.2)


def download_cbr_rates(output_dir: Path, start_date: str, end_date: str):
    """Download RUB rates from Russian Central Bank API."""
    print(f"\nDownloading CBR rates from {start_date} to {end_date}...")

    # Convert dates to CBR format (DD/MM/YYYY)
    start_dt = datetime.strptime(start_date, "%Y-%m-%d")
    end_dt = datetime.strptime(end_date, "%Y-%m-%d")

    cbr_start = start_dt.strftime("%d/%m/%Y")
    cbr_end = end_dt.strftime("%d/%m/%Y")

    for cbr_code, currency in CBR_CURRENCIES.items():
        print(f"  RUB -> {currency}...", end=" ", flush=True)

        url = f"http://www.cbr.ru/scripts/XML_dynamic.asp?date_req1={cbr_start}&date_req2={cbr_end}&VAL_NM_RQ={cbr_code}"
        root = fetch_xml(url)

        if root is not None:
            count = 0
            for record in root.findall("Record"):
                date_attr = record.get("Date")
                value_elem = record.find("Value")
                nominal_elem = record.find("Nominal")

                if date_attr and value_elem is not None and value_elem.text:
                    # Parse CBR date format (DD.MM.YYYY)
                    try:
                        dt = datetime.strptime(date_attr, "%d.%m.%Y")
                        date_str = dt.strftime("%Y-%m-%d")
                    except ValueError:
                        continue

                    # Parse value (uses comma as decimal separator)
                    rate_str = value_elem.text.replace(",", ".")
                    try:
                        rate = float(rate_str)
                    except ValueError:
                        continue

                    # Handle nominal (e.g., rate for 100 JPY)
                    nominal = 1
                    if nominal_elem is not None and nominal_elem.text:
                        try:
                            nominal = int(nominal_elem.text)
                        except ValueError:
                            nominal = 1

                    # CBR gives rate as: N RUB = 1 foreign currency (adjusted by nominal)
                    # We want: 1 RUB = X foreign currency
                    # So the rate from RUB to foreign is: nominal / rate
                    rub_to_foreign = nominal / rate

                    # Write RUB -> foreign
                    write_lino_rate(output_dir, "RUB", currency, date_str, rub_to_foreign, "cbr.ru (Central Bank of Russia)")

                    # Also write foreign -> RUB (inverse)
                    foreign_to_rub = rate / nominal
                    write_lino_rate(output_dir, currency, "RUB", date_str, foreign_to_rub, "cbr.ru (Central Bank of Russia)")

                    count += 1
            print(f"{count} rates")
        else:
            print("no data")

        # Be nice to the API
        time.sleep(0.3)


def main():
    # Determine output directory
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    output_dir = repo_root / "data" / "currency"

    print(f"Output directory: {output_dir}")

    # Get date range from arguments or use defaults
    # Default: last 5 years of data (a reasonable amount for a calculator)
    today = datetime.now()
    default_end = today.strftime("%Y-%m-%d")
    default_start = (today - timedelta(days=5*365)).strftime("%Y-%m-%d")

    if len(sys.argv) >= 3:
        start_date = sys.argv[1]
        end_date = sys.argv[2]
    else:
        start_date = default_start
        end_date = default_end

    print(f"Date range: {start_date} to {end_date}")

    # Download from both sources
    download_frankfurter_rates(output_dir, start_date, end_date)
    download_cbr_rates(output_dir, start_date, end_date)

    # Count total files
    total_files = sum(1 for _ in output_dir.rglob("*.lino"))
    print(f"\nTotal .lino files: {total_files}")


if __name__ == "__main__":
    main()
