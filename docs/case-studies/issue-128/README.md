# Case Study: Issue #128 — Calendar Month Arithmetic and Duration Unit Display

## Summary

**Issue URL:** https://github.com/link-assistant/calculator/issues/128  
**Version:** 0.13.0  
**Status:** Fixed in PR #129

Two bugs were reported via the expression `(17 февраля 2027) - 6 месяцев`:

1. **Wrong day in result** — Got `2026-08-21`, expected `2026-08-17`.  
2. **Abbreviations in output** — Steps showed `6 mo` instead of `6 months`; lino showed `(6 mo)` instead of `(6 months)`.

---

## Environment

- **Version**: 0.13.0  
- **URL**: https://link-assistant.github.io/calculator/?q=KGV4cHJlc3Npb24lMjAlMjIoMTclMjAlRDElODQlRDAlQjUlRDAlQjIlRDElODAlRDAlQjAlRDAlQkIlRDElOEYlMjAyMDI3KSUyMC0lMjA2JTIwJUQwJUJDJUQwJUI1JUQxJTgxJUQxJThGJUQxJTg2JUQwJUI1JUQwJUIyJTIyKQ
- **User Agent**: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36
- **Language**: en / ru  
- **WASM Ready**: Yes  
- **Timestamp**: 2026-04-05T13:57:56.386Z

---

## Bug 1: Wrong Day When Subtracting Months

### Input
```
(17 февраля 2027) - 6 месяцев
```

### Actual Result (Before Fix)
```
2026-08-21
```

### Expected Result
```
2026-08-17
```

### Root Cause Analysis

The issue was in `src/types/unit.rs`, method `DurationUnit::to_secs`:

```rust
// BEFORE (broken):
Self::Months => value * 2_592_000.0, // 30 days approximation
Self::Years  => value * 31_536_000.0, // 365 days approximation
```

When computing `date - 6 months`, the code converted `6 months` to **seconds** (6 × 30 × 86400 = 15,552,000 s = 180 days), then subtracted those seconds from the date.

```
2027-02-17 - 180 days = 2026-08-21   ← WRONG (180 days, not 6 calendar months)
2027-02-17 - 6 calendar months = 2026-08-17   ← CORRECT
```

The difference is 4 days because the traversed period includes February (28 days) + partial months. A fixed 30-day approximation does not match the real calendar.

### Sequence of Events

1. User types `17 февраля 2027 - 6 месяцев` (Russian: "17 February 2027 - 6 months").
2. `DurationUnit::parse("месяцев")` → `DurationUnit::Months`.
3. In `value.rs` `subtract_at_date`, the code called `dur_unit.to_secs(6.0)` = `15,552,000`.
4. `DateTime::add_duration(-15_552_000)` → subtract 180 days → `2026-08-21`.
5. Expected: subtract 6 calendar months preserving day → `2026-08-17`.

### Fix

Added `DateTime::add_calendar_months(i32)` in `src/types/datetime.rs`:

```rust
pub fn add_calendar_months(&self, months: i32) -> Self {
    let naive = self.inner.naive_utc();
    let new_naive = if months >= 0 {
        naive.checked_add_months(Months::new(months as u32)).unwrap_or(naive)
    } else {
        naive.checked_sub_months(Months::new((-months) as u32)).unwrap_or(naive)
    };
    // ...
}
```

Added `add_calendar_months_or_duration` helper in `src/types/value.rs`:

```rust
fn add_calendar_months_or_duration(dt: &DateTime, unit: &DurationUnit, amount: f64) -> DateTime {
    match unit {
        DurationUnit::Months => dt.add_calendar_months(amount as i32),
        DurationUnit::Years  => dt.add_calendar_months((amount * 12.0) as i32),
        other => {
            let seconds = other.to_secs(amount.abs()) as i64;
            if amount >= 0.0 { dt.add_duration(seconds) } else { dt.add_duration(-seconds) }
        }
    }
}
```

All four `DateTime ± duration_unit` match arms in `add_at_date` / `subtract_at_date` now call this helper.

**Clamping behavior** (by `chrono::Months`): if the day does not exist in the target month (e.g. January 31 + 1 month), the result is clamped to the last day of that month (Feb 28/29).

---

## Bug 2: Duration Units Displayed as Abbreviations

### Observed Output (Before Fix)
- Steps: `Literal value: 6 mo`
- Lino: `((2027-02-17) - (6 mo))`
- Compute: `2027-02-17 - 6 mo`

### Expected Output
- Steps: `Literal value: 6 months`
- Lino: `((2027-02-17) - (6 months))`
- Compute: `2027-02-17 - 6 months`

### Root Cause

`DurationUnit` implemented `Display` using short abbreviations:

```rust
// BEFORE (broken):
impl fmt::Display for DurationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Milliseconds => "ms",
            Self::Seconds      => "s",
            Self::Minutes      => "min",
            Self::Hours        => "h",
            Self::Days         => "d",
            Self::Weeks        => "w",
            Self::Months       => "mo",
            Self::Years        => "y",
        };
        write!(f, "{s}")
    }
}
```

The `to_display_string()` in `Value` calls `format!("{} {}", r_str, self.unit)`, which invokes `DurationUnit::Display`, producing `"6 mo"`.

### Fix

Updated `DurationUnit::Display` to use full English words:

```rust
// AFTER (fixed):
Self::Milliseconds => "milliseconds",
Self::Seconds      => "seconds",
Self::Minutes      => "minutes",
Self::Hours        => "hours",
Self::Days         => "days",
Self::Weeks        => "weeks",
Self::Months       => "months",
Self::Years        => "years",
```

Updated the existing `test_unit_display` unit test to expect `"hours"` instead of `"h"`.

---

## Files Changed

| File | Change |
|------|--------|
| `src/types/datetime.rs` | Added `chrono::Months` import; added `add_calendar_months(i32)` method |
| `src/types/value.rs` | Added `DurationUnit` import; added `add_calendar_months_or_duration` helper; updated all 4 date±duration match arms to use it |
| `src/types/unit.rs` | Updated `DurationUnit::Display` to full English words; updated `test_unit_display` test |
| `tests/issue_128_calendar_month_arithmetic_tests.rs` | New test file with 8 tests covering both bugs |

---

## Tests Added

All 8 tests in `tests/issue_128_calendar_month_arithmetic_tests.rs`:

1. `test_issue_128_exact_day_preserved_russian` — verifies `17 февраля 2027 - 6 месяцев = 2026-08-17`
2. `test_issue_128_exact_day_preserved_english` — verifies `17 February 2027 - 6 months = 2026-08-17`
3. `test_issue_128_exact_day_preserved_add_months` — verifies `17 August 2026 + 6 months = 2027-02-17`
4. `test_issue_128_month_end_clamping` — verifies `31 January 2027 + 1 month = 2027-02-28`
5. `test_issue_128_year_arithmetic_preserves_day` — verifies `17 февраля 2024 + 1 год = 2025-02-17`
6. `test_issue_128_subtract_year_preserves_day` — verifies `15 марта 2025 - 2 года = 2023-03-15`
7. `test_issue_128_steps_show_full_word_months` — verifies steps contain `"6 months"` not `"6 mo"`
8. `test_issue_128_russian_steps_show_full_word_months` — Russian input shows full English word in steps

---

## Verification

```
cargo test   # all tests pass (0 failures)
```

Full test suite: **752 tests pass, 0 fail**.

---

## Related Work

- Issue #125 (PR #127): Added support for parsing Russian date arithmetic expressions. That fix enabled parsing `17 февраля 2027 - 6 месяцев` correctly but left the day-of-month arithmetic bug in place (tests only checked `starts_with("2026-08")`).
- `chrono` crate: `chrono::Months` type and `NaiveDateTime::checked_add_months` / `checked_sub_months` were introduced in chrono 0.4.22. The project uses 0.4.43, so no version bump was needed.

---

## Conclusion

The root cause of the wrong-day bug was a fundamental design issue: month arithmetic was reduced to a fixed-seconds approximation (30 days/month), losing all calendar semantics. The fix introduces proper calendar month arithmetic via `chrono::Months` while keeping second-based arithmetic for all non-month/year units.

The abbreviation bug was a one-line fix in `DurationUnit::Display`.
