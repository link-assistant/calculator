//! Tests for issue #128: Calendar month arithmetic and duration unit display.
//!
//! Issue: `(17 февраля 2027) - 6 месяцев` returned 2026-08-21 instead of 2026-08-17.
//!
//! Root cause 1: Month arithmetic was implemented by converting months to a fixed
//! number of seconds (30 days/month = 2,592,000 s), which does not preserve the
//! day-of-month.  2027-02-17 - 180 days = 2026-08-21 (wrong).  The correct result
//! is 2026-08-17 — subtracting calendar months must keep the same day of month.
//!
//! Root cause 2: `DurationUnit` `Display` showed `"mo"` (abbreviation) instead of
//! `"months"` (full word).  This affected the "Literal value: 6 mo" step line and
//! the lino interpretation `(6 mo)`.
//!
//! Fixes:
//! 1. Added `DateTime::add_calendar_months(i32)` that uses `chrono::Months` for
//!    proper calendar arithmetic (preserves day, clamping at month-end).
//! 2. Updated `add_at_date` / `subtract_at_date` in `value.rs` to call
//!    `add_calendar_months` when the unit is `DurationUnit::Months` or `Years`.
//! 3. Changed `DurationUnit` `Display` to use full English words.

use link_calculator::Calculator;

// ── Bug 1: exact day must be preserved when subtracting calendar months ──────

/// The original issue: Feb 17, 2027 - 6 months must be 2026-08-17, not 2026-08-21.
#[test]
fn test_issue_128_exact_day_preserved_russian() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027 - 6 месяцев");
    assert!(
        result.success,
        "17 февраля 2027 - 6 месяцев should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "2026-08-17",
        "17 февраля 2027 - 6 месяцев should be 2026-08-17 (exact day preserved), got: {}",
        result.result
    );
}

/// Same using English month name format.
#[test]
fn test_issue_128_exact_day_preserved_english() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 February 2027 - 6 months");
    assert!(
        result.success,
        "17 February 2027 - 6 months should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "2026-08-17",
        "17 February 2027 - 6 months should be 2026-08-17, got: {}",
        result.result
    );
}

/// Adding months should also preserve day.
#[test]
fn test_issue_128_exact_day_preserved_add_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 August 2026 + 6 months");
    assert!(
        result.success,
        "17 August 2026 + 6 months should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "2027-02-17",
        "17 August 2026 + 6 months should be 2027-02-17, got: {}",
        result.result
    );
}

/// Month-end clamping: Jan 31, 2027 + 1 month should give 2027-02-28 (not overflow).
#[test]
fn test_issue_128_month_end_clamping() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("31 January 2027 + 1 month");
    assert!(
        result.success,
        "31 January 2027 + 1 month should succeed, got error: {:?}",
        result.error
    );
    // chrono clamps to the last day of the month
    assert_eq!(
        result.result, "2027-02-28",
        "31 January 2027 + 1 month should be 2027-02-28 (clamped), got: {}",
        result.result
    );
}

/// Year arithmetic should also preserve day.
#[test]
fn test_issue_128_year_arithmetic_preserves_day() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2024 + 1 год");
    assert!(
        result.success,
        "17 февраля 2024 + 1 год should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "2025-02-17",
        "17 февраля 2024 + 1 год should be 2025-02-17, got: {}",
        result.result
    );
}

/// Subtracting years should preserve day.
#[test]
fn test_issue_128_subtract_year_preserves_day() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("15 марта 2025 - 2 года");
    assert!(
        result.success,
        "15 марта 2025 - 2 года should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "2023-03-15",
        "15 марта 2025 - 2 года should be 2023-03-15, got: {}",
        result.result
    );
}

// ── Bug 2: duration unit display must use full words ─────────────────────────

/// Steps should say "6 months" not "6 mo".
#[test]
fn test_issue_128_steps_show_full_word_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 February 2027 - 6 months");
    assert!(
        result.success,
        "should succeed, got error: {:?}",
        result.error
    );
    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("6 months"),
        "Steps should contain '6 months' (full word), but got:\n{steps_text}"
    );
    // "6 mo" as abbreviation should not appear (note: "6 months" is fine — check for " mo" followed by non-alpha)
    assert!(
        !steps_text.contains("6 mo\n") && !steps_text.contains("6 mo\r") && !steps_text.contains("6 mo "),
        "Steps should NOT contain '6 mo' abbreviation (without 'nths' suffix), but got:\n{steps_text}"
    );
}

/// Russian input: steps should also show full English word.
#[test]
fn test_issue_128_russian_steps_show_full_word_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027 - 6 месяцев");
    assert!(
        result.success,
        "should succeed, got error: {:?}",
        result.error
    );
    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("6 months"),
        "Steps should contain '6 months' (full English word), but got:\n{steps_text}"
    );
}
