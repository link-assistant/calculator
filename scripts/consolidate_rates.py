#!/usr/bin/env python3
"""
Consolidate individual .lino rate files into a single file per currency pair.

This script reads all date-based .lino files from data/currency/{from}/{to}/*.lino
and consolidates them into a single data/currency/{from}-{to}.lino file.

The new format stores all historical rates for a currency pair in a single file:

rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315
    ...
"""

import os
import re
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple


def parse_lino_file(file_path: Path) -> Tuple[str, str, str, float, str]:
    """Parse a single .lino rate file and extract its data.

    Returns: (from_currency, to_currency, date, value, source)
    """
    content = file_path.read_text()

    from_curr = None
    to_curr = None
    date = None
    value = None
    source = None

    for line in content.strip().split('\n'):
        line = line.strip()
        if line.startswith('from '):
            from_curr = line[5:].strip().upper()
        elif line.startswith('to '):
            to_curr = line[3:].strip().upper()
        elif line.startswith('date '):
            date = line[5:].strip()
        elif line.startswith('value '):
            try:
                value = float(line[6:].strip())
            except ValueError:
                pass
        elif line.startswith('source '):
            # Extract source, removing quotes
            src = line[7:].strip()
            src = src.strip("'\"")
            source = src

    return (from_curr, to_curr, date, value, source)


def consolidate_currency_pair(pair_dir: Path) -> Dict[str, List[Tuple[str, float]]]:
    """Read all .lino files in a directory and return consolidated data.

    Returns: dict with 'from', 'to', 'source', 'rates' (list of (date, value) tuples)
    """
    lino_files = list(pair_dir.glob('*.lino'))
    if not lino_files:
        return None

    rates = []
    from_curr = None
    to_curr = None
    source = None

    for lino_file in lino_files:
        try:
            fc, tc, date, value, src = parse_lino_file(lino_file)
            if fc and tc and date and value is not None:
                from_curr = fc
                to_curr = tc
                if src:
                    source = src
                rates.append((date, value))
        except Exception as e:
            print(f"  Warning: Error parsing {lino_file}: {e}", file=sys.stderr)

    if not rates:
        return None

    # Sort rates by date
    rates.sort(key=lambda x: x[0])

    return {
        'from': from_curr,
        'to': to_curr,
        'source': source or 'unknown',
        'rates': rates
    }


def write_consolidated_lino(output_path: Path, data: dict):
    """Write consolidated rate data to a single .lino file."""
    lines = [
        "rates:",
        f"  from {data['from']}",
        f"  to {data['to']}",
        f"  source '{data['source']}'",
        "  data:"
    ]

    for date, value in data['rates']:
        lines.append(f"    {date} {value}")

    output_path.write_text('\n'.join(lines) + '\n')


def main():
    # Determine directories
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    currency_dir = repo_root / "data" / "currency"

    if not currency_dir.exists():
        print(f"Error: Currency directory not found: {currency_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Currency directory: {currency_dir}")

    # Find all currency pair directories (e.g., usd/eur, chf/rub)
    consolidated_count = 0
    total_files_removed = 0

    # Get all base currency directories
    base_dirs = [d for d in currency_dir.iterdir() if d.is_dir()]

    for base_dir in sorted(base_dirs):
        from_curr = base_dir.name.upper()

        # Get all target currency directories
        target_dirs = [d for d in base_dir.iterdir() if d.is_dir()]

        for target_dir in sorted(target_dirs):
            to_curr = target_dir.name.upper()

            # Count .lino files
            lino_files = list(target_dir.glob('*.lino'))
            if not lino_files:
                continue

            print(f"  {from_curr}/{to_curr}: {len(lino_files)} files...", end=" ", flush=True)

            # Consolidate
            data = consolidate_currency_pair(target_dir)
            if data:
                # Write consolidated file at currency_dir level
                output_file = currency_dir / f"{from_curr.lower()}-{to_curr.lower()}.lino"
                write_consolidated_lino(output_file, data)

                # Remove old files and directory
                for lino_file in lino_files:
                    lino_file.unlink()
                    total_files_removed += 1

                # Try to remove the now-empty directory
                try:
                    target_dir.rmdir()
                except OSError:
                    pass  # Directory not empty or other error

                consolidated_count += 1
                print(f"-> {output_file.name} ({len(data['rates'])} rates)")
            else:
                print("skipped (no valid data)")

    # Clean up empty base directories
    for base_dir in base_dirs:
        if base_dir.is_dir():
            # Check if directory is empty
            remaining = list(base_dir.iterdir())
            if not remaining:
                try:
                    base_dir.rmdir()
                    print(f"  Removed empty directory: {base_dir.name}")
                except OSError:
                    pass

    print(f"\nConsolidation complete:")
    print(f"  Currency pairs consolidated: {consolidated_count}")
    print(f"  Individual files removed: {total_files_removed}")

    # Count final .lino files
    final_files = list(currency_dir.glob('*.lino'))
    print(f"  Final consolidated files: {len(final_files)}")


if __name__ == "__main__":
    main()
