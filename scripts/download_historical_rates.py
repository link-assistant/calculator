#!/usr/bin/env python3
"""
Download historical exchange rates from multiple sources and convert to consolidated .lino format.

Sources:
- Frankfurter API (https://frankfurter.dev/) - ECB data from 1999, 30+ currencies (no RUB)
- CBR API (http://cbr.ru/) - Russian Central Bank data from 1992, RUB rates

Output format (consolidated .lino - one file per currency pair):
    rates:
      from USD
      to EUR
      source 'frankfurter.dev (ECB)'
      data:
        2021-01-25 0.8234
        2021-02-01 0.8315
        ...
"""

import json
import os
import sys
import time
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional, Dict, List, Tuple
from collections import defaultdict


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


def write_consolidated_lino(output_dir: Path, from_curr: str, to_curr: str,
                            rates: List[Tuple[str, float]], source: str):
    """Write all rates for a currency pair to a single consolidated .lino file."""
    # Sort rates by date
    rates_sorted = sorted(rates, key=lambda x: x[0])

    # File name: {from}-{to}.lino (e.g., usd-eur.lino)
    file_path = output_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"

    # Build content
    lines = [
        "rates:",
        f"  from {from_curr.upper()}",
        f"  to {to_curr.upper()}",
        f"  source '{source}'",
        "  data:"
    ]

    for date, rate in rates_sorted:
        lines.append(f"    {date} {rate}")

    file_path.write_text('\n'.join(lines) + '\n')
    return len(rates_sorted)


def download_frankfurter_rates(output_dir: Path, start_date: str, end_date: str) -> Dict[Tuple[str, str], List[Tuple[str, float]]]:
    """Download rates from Frankfurter API (ECB data).

    Returns a dict mapping (from, to) pairs to lists of (date, rate) tuples.
    """
    print(f"\nDownloading Frankfurter rates from {start_date} to {end_date}...")

    all_rates: Dict[Tuple[str, str], List[Tuple[str, float]]] = defaultdict(list)

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
                    all_rates[(from_curr, to_curr)].append((date_str, rate))
                    count += 1
            print(f"{count} rates")
        else:
            print("no data")

        # Be nice to the API
        time.sleep(0.2)

    # Write consolidated files
    for (from_curr, to_curr), rates in all_rates.items():
        if rates:
            count = write_consolidated_lino(output_dir, from_curr, to_curr, rates, "frankfurter.dev (ECB)")
            print(f"  -> {from_curr.lower()}-{to_curr.lower()}.lino ({count} rates)")

    return all_rates


def download_cbr_rates(output_dir: Path, start_date: str, end_date: str) -> Dict[Tuple[str, str], List[Tuple[str, float]]]:
    """Download RUB rates from Russian Central Bank API.

    Returns a dict mapping (from, to) pairs to lists of (date, rate) tuples.
    """
    print(f"\nDownloading CBR rates from {start_date} to {end_date}...")

    # Convert dates to CBR format (DD/MM/YYYY)
    start_dt = datetime.strptime(start_date, "%Y-%m-%d")
    end_dt = datetime.strptime(end_date, "%Y-%m-%d")

    cbr_start = start_dt.strftime("%d/%m/%Y")
    cbr_end = end_dt.strftime("%d/%m/%Y")

    all_rates: Dict[Tuple[str, str], List[Tuple[str, float]]] = defaultdict(list)

    for cbr_code, currency in CBR_CURRENCIES.items():
        print(f"  RUB <-> {currency}...", end=" ", flush=True)

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

                    # Store RUB -> foreign
                    all_rates[("RUB", currency)].append((date_str, rub_to_foreign))

                    # Also store foreign -> RUB (inverse)
                    foreign_to_rub = rate / nominal
                    all_rates[(currency, "RUB")].append((date_str, foreign_to_rub))

                    count += 1
            print(f"{count} rates each direction")
        else:
            print("no data")

        # Be nice to the API
        time.sleep(0.3)

    # Write consolidated files
    for (from_curr, to_curr), rates in all_rates.items():
        if rates:
            count = write_consolidated_lino(output_dir, from_curr, to_curr, rates, "cbr.ru (Central Bank of Russia)")
            print(f"  -> {from_curr.lower()}-{to_curr.lower()}.lino ({count} rates)")

    return all_rates


def main():
    # Determine output directory
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    output_dir = repo_root / "data" / "currency"

    print(f"Output directory: {output_dir}")

    # Ensure output directory exists
    output_dir.mkdir(parents=True, exist_ok=True)

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

    # Count final files
    total_files = sum(1 for _ in output_dir.glob("*.lino"))
    print(f"\nTotal consolidated .lino files: {total_files}")


if __name__ == "__main__":
    main()
