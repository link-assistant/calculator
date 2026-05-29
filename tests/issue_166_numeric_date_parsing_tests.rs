//! Regression tests for issue #166: numeric date literals should be recognized
//! as dates inside expressions (e.g. `15.10.2025 + 180 days`) instead of being
//! split into arithmetic (`15 / 10 / 2025`) or rejected as a parse error.
//!
//! See: <https://github.com/link-assistant/calculator/issues/166>

use link_calculator::Calculator;

/// Helper: evaluate an expression and return the rendered result, asserting success.
fn eval(input: &str) -> String {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal(input);
    assert!(
        result.success,
        "expected `{input}` to succeed, got error: {:?}",
        result.error
    );
    result.result
}

// ── The exact examples from the issue ──────────────────────────────────────

#[test]
fn test_issue_166_dot_separated_date_plus_duration() {
    // The headline failing case: previously "Unexpected trailing input '.2025'".
    assert_eq!(eval("15.10.2025 + 180 days"), "2026-04-13");
}

#[test]
fn test_issue_166_slash_separated_date_plus_duration() {
    // Previously parsed as 15 / 10 / 2025 ≈ 0.0007.
    assert_eq!(eval("15/10/2025 + 180 days"), "2026-04-13");
}

// ── Standalone numeric dates (all separators) ───────────────────────────────

#[test]
fn test_iso_dash_date() {
    assert_eq!(eval("2026-01-22"), "2026-01-22");
}

#[test]
fn test_us_slash_date() {
    // MM/DD/YYYY
    assert_eq!(eval("01/22/2026"), "2026-01-22");
}

#[test]
fn test_european_slash_date() {
    // DD/MM/YYYY
    assert_eq!(eval("22/01/2026"), "2026-01-22");
}

#[test]
fn test_european_dot_date() {
    // DD.MM.YYYY (German/Russian convention)
    assert_eq!(eval("15.10.2025"), "2025-10-15");
}

#[test]
fn test_iso_dot_date() {
    // YYYY.MM.DD
    assert_eq!(eval("2025.10.15"), "2025-10-15");
}

#[test]
fn test_european_dash_date() {
    // DD-MM-YYYY (non-ISO dash ordering, common in several locales)
    assert_eq!(eval("15-10-2025"), "2025-10-15");
}

#[test]
fn test_dash_date_plus_duration() {
    assert_eq!(eval("15-10-2025 + 180 days"), "2026-04-13");
}

#[test]
fn test_us_dot_date() {
    // MM.DD.YYYY
    assert_eq!(eval("10.15.2025"), "2025-10-15");
}

#[test]
fn test_us_dash_date() {
    // MM-DD-YYYY
    assert_eq!(eval("01-22-2026"), "2026-01-22");
}

// ── Date arithmetic with durations ──────────────────────────────────────────

#[test]
fn test_iso_date_plus_duration() {
    assert_eq!(eval("2025-10-15 + 180 days"), "2026-04-13");
}

#[test]
fn test_iso_date_minus_duration() {
    assert_eq!(eval("2025-10-15 - 15 days"), "2025-09-30");
}

#[test]
fn test_dot_date_minus_duration() {
    assert_eq!(eval("15.10.2025 - 15 days"), "2025-09-30");
}

#[test]
fn test_date_difference_yields_duration() {
    // Date minus date is a duration.
    assert_eq!(eval("2025-10-15 - 2024-10-15"), "365 days");
}

// ── Ambiguity guards: ordinary arithmetic must be untouched ─────────────────

#[test]
fn test_spaced_arithmetic_still_subtracts() {
    // With spaces, the user clearly means subtraction, not a date.
    assert_eq!(eval("2026 - 1 - 22"), "2003");
}

#[test]
fn test_fraction_division_preserved() {
    assert_eq!(eval("1/3 * 3"), "1");
}

#[test]
fn test_decimal_addition_preserved() {
    assert_eq!(eval("3.14 + 2.86"), "6");
}

#[test]
fn test_two_component_slash_is_division() {
    // Only two components → not a date, still division.
    assert_eq!(eval("10/2"), "5");
}

#[test]
fn test_invalid_numeric_date_falls_back_to_arithmetic() {
    // Month 20 is not a valid date, so this stays arithmetic: 2026 - 20 - 5.
    assert_eq!(eval("2026-20-5"), "2001");
}
