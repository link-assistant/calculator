//! Tests for issue #125: Russian date arithmetic expressions.
//!
//! Issue: `17 февраля 2027 - 6 месяцев` was not interpreted as
//! `(17 февраля 2027) - 6 месяцев`. Instead only `17 февраля` was parsed.
//!
//! Root cause 1: Russian month names (genitive case: февраля, января, марта, etc.)
//! were not recognized as datetime components by `looks_like_datetime()`.
//!
//! Root cause 2: When a number (17) is followed by a month-name identifier (февраля),
//! the parser treated `февраля` as an unknown unit rather than trying to parse
//! the whole token sequence as a datetime expression.
//!
//! Root cause 3: `try_parse_datetime_from_tokens` did not try progressively shorter
//! token prefixes when the full collected string failed to parse as a datetime.
//! This caused `"22 Jan 2027"` (without comma) to fail even though it should work.
//!
//! Root cause 4: Russian duration unit names (месяцев, недель, дней, etc.) were
//! not recognized in `DurationUnit::parse()`.
//!
//! Fix:
//! 1. Added Russian month names (nominative and genitive forms) to `looks_like_datetime()`.
//! 2. Added `translate_russian_months()` preprocessing in `DateTime::parse()`.
//! 3. Added `"DD Mon YYYY"` and `"Mon DD YYYY"` (without comma) date formats.
//! 4. Added Russian duration unit names to `DurationUnit::parse()`.
//! 5. Fixed `try_parse_datetime_from_tokens` to try progressively shorter token prefixes.
//! 6. Added datetime detection when a number is followed by a month-name identifier.

use link_calculator::Calculator;

// ── Core issue: "17 февраля 2027 - 6 месяцев" ─────────────────────────────────

/// The exact expression from the issue report should now work.
#[test]
fn test_issue_125_russian_date_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027 - 6 месяцев");
    assert!(
        result.success,
        "17 февраля 2027 - 6 месяцев should succeed, got error: {:?}",
        result.error
    );
    // 2027-02-17 - 6 months = 2026-08-17 (approximately; month arithmetic may vary by day)
    assert!(
        result.result.starts_with("2026-08"),
        "17 февраля 2027 - 6 месяцев should be around August 2026, got: {}",
        result.result
    );
}

/// The Links notation should show the date as a structured expression.
#[test]
fn test_issue_125_lino_notation() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027 - 6 месяцев");
    assert!(
        result.success,
        "17 февраля 2027 - 6 месяцев should succeed, got error: {:?}",
        result.error
    );
    // Should NOT be just "(17 февраля)" — that was the broken behavior
    assert_ne!(
        result.lino_interpretation, "(17 февраля)",
        "Lino should not be just '(17 февраля)': {}",
        result.lino_interpretation
    );
    // Should include a date and months subtraction
    assert!(
        result.lino_interpretation.contains("2027"),
        "Lino should contain 2027: {}",
        result.lino_interpretation
    );
}

// ── Russian month names in various grammatical forms ──────────────────────────

/// Standalone Russian date (genitive form: февраля = of February).
#[test]
fn test_issue_125_russian_date_standalone() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027");
    assert!(
        result.success,
        "17 февраля 2027 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("2027-02-17"),
        "17 февраля 2027 should parse to 2027-02-17, got: {}",
        result.result
    );
}

/// Russian date with January (nominative: январь, genitive: января).
#[test]
fn test_issue_125_russian_date_january() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("15 января 2027");
    assert!(
        result.success,
        "15 января 2027 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("2027-01-15"),
        "15 января 2027 should parse to 2027-01-15, got: {}",
        result.result
    );
}

/// Russian date with December (genitive: декабря).
#[test]
fn test_issue_125_russian_date_december() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("31 декабря 2026");
    assert!(
        result.success,
        "31 декабря 2026 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("2026-12-31"),
        "31 декабря 2026 should parse to 2026-12-31, got: {}",
        result.result
    );
}

// ── Russian duration units ─────────────────────────────────────────────────────

/// "месяцев" (genitive plural of месяц = month) should be recognized as months.
#[test]
fn test_issue_125_russian_months_unit() {
    let mut calc = Calculator::new();
    // Use an already-recognized date format to isolate duration unit parsing
    let result = calc.calculate_internal("17 февраля 2027 - 6 месяцев");
    assert!(
        result.success,
        "17 февраля 2027 - 6 месяцев should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 февраля 2027 - 6 месяцев should be around August 2026, got: {}",
        result.result
    );
}

/// "месяца" (genitive singular of месяц = month) should be recognized.
#[test]
fn test_issue_125_russian_month_singular() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 - 1 месяца");
    assert!(
        result.success,
        "2027-02-17 - 1 месяца should succeed, got error: {:?}",
        result.error
    );
}

/// "месяц" (nominative singular) should be recognized.
#[test]
fn test_issue_125_russian_month_nominative() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 - 1 месяц");
    assert!(
        result.success,
        "2027-02-17 - 1 месяц should succeed, got error: {:?}",
        result.error
    );
}

/// "дней" (genitive plural of день = day) should be recognized as days.
#[test]
fn test_issue_125_russian_days_unit() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 - 30 дней");
    assert!(
        result.success,
        "2027-02-17 - 30 дней should succeed, got error: {:?}",
        result.error
    );
}

/// "недель" (genitive plural of неделя = week) should be recognized as weeks.
#[test]
fn test_issue_125_russian_weeks_unit() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 + 2 недели");
    assert!(
        result.success,
        "2027-02-17 + 2 недели should succeed, got error: {:?}",
        result.error
    );
}

/// "лет" (genitive plural of год = year) should be recognized as years.
#[test]
fn test_issue_125_russian_years_unit() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 - 2 лет");
    assert!(
        result.success,
        "2027-02-17 - 2 лет should succeed, got error: {:?}",
        result.error
    );
}

/// "часов" (genitive plural of час = hour) should be recognized as hours.
#[test]
fn test_issue_125_russian_hours_unit() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2027-02-17 + 5 часов");
    assert!(
        result.success,
        "2027-02-17 + 5 часов should succeed, got error: {:?}",
        result.error
    );
}

// ── Date formats without comma ─────────────────────────────────────────────────

/// "Feb 17 2027" (English, without comma) should parse correctly.
#[test]
fn test_issue_125_english_date_without_comma_mon_dd_yyyy() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("Feb 17 2027 - 6 months");
    assert!(
        result.success,
        "Feb 17 2027 - 6 months should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "Feb 17 2027 - 6 months should be around August 2026, got: {}",
        result.result
    );
}

/// "17 February 2027" (English, without comma) should parse correctly.
#[test]
fn test_issue_125_english_date_without_comma_dd_month_yyyy() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 February 2027 - 6 months");
    assert!(
        result.success,
        "17 February 2027 - 6 months should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 February 2027 - 6 months should be around August 2026, got: {}",
        result.result
    );
}

/// "January 15 2027" (English, without comma) should parse correctly.
#[test]
fn test_issue_125_english_date_january_without_comma() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("January 15 2027 - 1 month");
    assert!(
        result.success,
        "January 15 2027 - 1 month should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("2026-12"),
        "January 15 2027 - 1 month should be December 2026, got: {}",
        result.result
    );
}

// ── Addition with Russian units ───────────────────────────────────────────────

/// Adding months to a Russian date expression.
#[test]
fn test_issue_125_russian_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 февраля 2027 + 3 месяца");
    assert!(
        result.success,
        "17 февраля 2027 + 3 месяца should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-05"),
        "17 февраля 2027 + 3 месяца should be around May 2027, got: {}",
        result.result
    );
}

// ── Related: date arithmetic with other month names ──────────────────────────

/// "1 января 2026 - 6 месяцев" (January 1 2026 - 6 months = July 2025).
#[test]
fn test_issue_125_russian_january_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 января 2026 - 6 месяцев");
    assert!(
        result.success,
        "1 января 2026 - 6 месяцев should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2025-07"),
        "1 января 2026 - 6 месяцев should be July 2025, got: {}",
        result.result
    );
}
