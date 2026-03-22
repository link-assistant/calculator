#!/usr/bin/env python3
"""
Check all .lino currency data files for gaps in their date ranges.

For ECB (Frankfurter) files: historical data is weekly, recent data is daily.
For CBR files: data is daily (business days).

The checker handles:
- Weekly section: gaps up to 9 days are normal (7-day spacing + possible 2-day holiday)
- Daily section: gaps up to 4 days are normal (weekends + possible holiday)
- Known holiday gaps (New Year, Christmas) where central banks don't publish
"""

import os
import sys
from datetime import datetime, timedelta
from pathlib import Path

def parse_lino_dates(file_path):
    """Parse dates and rates from a .lino file."""
    dates = []
    source = None
    with open(file_path) as f:
        in_data = False
        for line in f:
            stripped = line.strip()
            if "source" in stripped:
                source = stripped
            if in_data:
                if stripped and stripped[0].isdigit() and len(stripped) >= 10 and stripped[4] == '-':
                    parts = stripped.split()
                    date_str = parts[0]
                    rate = float(parts[1]) if len(parts) > 1 else None
                    dates.append((date_str, rate))
            elif stripped in ("rates:", "data:"):
                in_data = True
    return dates, source

def is_holiday_gap(d1, d2):
    """Check if a gap spans a known holiday period where no rates are published."""
    gap_days = (d2 - d1).days

    # New Year holidays (late Dec -> early/mid Jan)
    # Covers both ECB (Dec 25-Jan 1) and CBR (Dec 31-Jan 9)
    if d1.month == 12 and d1.day >= 24 and d2.month == 1 and d2.day <= 15:
        return True
    # Christmas week (Dec 24-26 in many countries)
    if d1.month == 12 and d2.month == 12 and d1.day >= 22 and d2.day <= 31:
        if gap_days <= 7:
            return True
    # Easter holidays (Good Friday + Easter Monday, typically late March/April)
    # ECB is closed for these — results in 5-day gaps (Thu -> Tue)
    if gap_days == 5 and d1.month in (3, 4) and d2.month in (3, 4):
        # 5-day gap in March/April is almost certainly Easter
        return True
    return False

def classify_frequency(dates):
    """Classify each date entry as weekly or daily based on surrounding context.

    Returns the index where daily data starts. Everything before is weekly.
    If all data appears to be daily or weekly, returns appropriately.
    """
    if len(dates) < 5:
        return len(dates)  # Too few to classify

    # Calculate gaps
    gaps = []
    for i in range(1, len(dates)):
        d1 = datetime.strptime(dates[i-1][0], "%Y-%m-%d")
        d2 = datetime.strptime(dates[i][0], "%Y-%m-%d")
        gaps.append((d2 - d1).days)

    # Use a sliding window to detect when data transitions from weekly to daily.
    # Weekly data has typical gaps of 7 days, daily has gaps of 1-3 days.
    window = 10
    for i in range(len(gaps) - window):
        window_gaps = gaps[i:i+window]
        avg_gap = sum(window_gaps) / len(window_gaps)
        if avg_gap <= 3.0:
            # This window looks daily. The transition is roughly at index i.
            return i

    return len(dates)  # All weekly

def check_file_gaps(file_path, verbose=False):
    """Check a single .lino file for gaps."""
    dates, source = parse_lino_dates(file_path)
    if not dates:
        return []

    is_cbr = source and "cbr" in source.lower()
    gaps = []

    if is_cbr:
        # CBR data is daily throughout
        max_gap = 5  # Allow up to 5 days (weekends + possible holiday)
        for i in range(1, len(dates)):
            d1 = datetime.strptime(dates[i-1][0], "%Y-%m-%d")
            d2 = datetime.strptime(dates[i][0], "%Y-%m-%d")
            gap_days = (d2 - d1).days
            if gap_days > max_gap and not is_holiday_gap(d1, d2):
                gaps.append({
                    'from': dates[i-1][0],
                    'to': dates[i][0],
                    'gap_days': gap_days,
                    'type': 'daily',
                })
    else:
        # ECB file - find transition from weekly to daily
        transition_idx = classify_frequency(dates)

        if verbose:
            if transition_idx < len(dates):
                print(f"    Transition at index {transition_idx}: {dates[transition_idx][0]}")
            else:
                print(f"    All weekly data")

        # Check weekly section (before transition)
        max_weekly_gap = 9  # 7 days + 2 for holidays
        for i in range(1, transition_idx):
            d1 = datetime.strptime(dates[i-1][0], "%Y-%m-%d")
            d2 = datetime.strptime(dates[i][0], "%Y-%m-%d")
            gap_days = (d2 - d1).days
            if gap_days > max_weekly_gap and not is_holiday_gap(d1, d2):
                gaps.append({
                    'from': dates[i-1][0],
                    'to': dates[i][0],
                    'gap_days': gap_days,
                    'type': 'weekly',
                })

        # Check daily section (after transition)
        max_daily_gap = 4  # weekends + possible holiday
        for i in range(max(1, transition_idx + 1), len(dates)):
            d1 = datetime.strptime(dates[i-1][0], "%Y-%m-%d")
            d2 = datetime.strptime(dates[i][0], "%Y-%m-%d")
            gap_days = (d2 - d1).days
            if gap_days > max_daily_gap and not is_holiday_gap(d1, d2):
                gaps.append({
                    'from': dates[i-1][0],
                    'to': dates[i][0],
                    'gap_days': gap_days,
                    'type': 'daily',
                })

    return gaps

def check_date_order(file_path):
    """Check if dates are in strictly ascending order."""
    dates, _ = parse_lino_dates(file_path)
    issues = []
    for i in range(1, len(dates)):
        d1 = datetime.strptime(dates[i-1][0], "%Y-%m-%d")
        d2 = datetime.strptime(dates[i][0], "%Y-%m-%d")
        if d2 <= d1:
            issues.append({
                'type': 'order',
                'prev': dates[i-1],
                'curr': dates[i],
                'line': i + 1,
            })
    return issues

def check_anomalous_values(file_path):
    """Check for anomalous rate values (sudden jumps > 30% between consecutive entries)."""
    dates, _ = parse_lino_dates(file_path)
    issues = []
    for i in range(1, len(dates)):
        if dates[i-1][1] is not None and dates[i][1] is not None:
            prev_rate = dates[i-1][1]
            curr_rate = dates[i][1]
            if prev_rate > 0:
                pct_change = abs(curr_rate - prev_rate) / prev_rate * 100
                if pct_change > 30:
                    issues.append({
                        'type': 'anomaly',
                        'date1': dates[i-1][0],
                        'rate1': dates[i-1][1],
                        'date2': dates[i][0],
                        'rate2': dates[i][1],
                        'pct_change': pct_change,
                    })
    return issues

def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    data_dir = Path(__file__).parent.parent / "data" / "currency"

    all_files = sorted(data_dir.glob("*.lino"))

    print(f"Checking {len(all_files)} .lino files for gaps and issues...\n")

    total_gaps = 0
    total_order_issues = 0
    total_anomalies = 0
    clean_files = 0

    for file_path in all_files:
        dates, source = parse_lino_dates(file_path)
        gaps = check_file_gaps(file_path, verbose=verbose)
        order_issues = check_date_order(file_path)
        anomalies = check_anomalous_values(file_path)

        has_issues = gaps or order_issues or anomalies

        if has_issues:
            print(f"{'='*60}")
            print(f"FILE: {file_path.name}")
            print(f"  Entries: {len(dates)}, Range: {dates[0][0]} to {dates[-1][0]}")

            if order_issues:
                print(f"  DATE ORDER ISSUES ({len(order_issues)}):")
                for issue in order_issues:
                    print(f"    {issue['prev'][0]} ({issue['prev'][1]}) -> {issue['curr'][0]} ({issue['curr'][1]})")
                total_order_issues += len(order_issues)

            if anomalies:
                print(f"  VALUE ANOMALIES ({len(anomalies)}):")
                for a in anomalies:
                    print(f"    {a['date1']} ({a['rate1']}) -> {a['date2']} ({a['rate2']}) = {a['pct_change']:.1f}% change")
                total_anomalies += len(anomalies)

            if gaps:
                print(f"  GAPS ({len(gaps)}):")
                for gap in gaps:
                    print(f"    {gap['from']} -> {gap['to']} ({gap['gap_days']} days, expected {gap['type']})")
                total_gaps += len(gaps)

            print()
        else:
            clean_files += 1
            if verbose:
                print(f"OK: {file_path.name} ({len(dates)} entries, {dates[0][0]} to {dates[-1][0]})")

    print(f"{'='*60}")
    print(f"SUMMARY:")
    print(f"  Files checked: {len(all_files)}")
    print(f"  Clean files: {clean_files}")
    print(f"  Total gaps: {total_gaps}")
    print(f"  Total order issues: {total_order_issues}")
    print(f"  Total value anomalies: {total_anomalies}")

    if total_gaps or total_order_issues or total_anomalies:
        print(f"\nISSUES FOUND - needs fixing!")
        return 1
    else:
        print(f"\nAll files look clean!")
        return 0

if __name__ == "__main__":
    sys.exit(main())
