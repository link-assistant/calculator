//! Regression tests for issue #211: "08Aug2026 22:35 - now" failed to parse
//! with "Unexpected trailing input '22' at position 10".
//!
//! The compact date form (no spaces between day, month name and year) is
//! lexed as a single date literal, so the fix for issue #212 — letting a date
//! literal absorb a trailing time — covers it too. These tests pin that
//! behaviour down.

use link_calculator::Calculator;

#[test]
fn reported_expression_is_supported() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("08Aug2026 22:35 - now");

    assert!(
        result.success,
        "expression should succeed: {:?}",
        result.error
    );
}

#[test]
fn compact_date_followed_by_time_parses() {
    let mut calculator = Calculator::new();

    for expression in [
        "08Aug2026 22:35",
        "08Aug2026 22:35 UTC",
        "8Aug2026 22:35",
        "8aug2026 10:35pm",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("2026-08-08 22:35"),
            "{expression:?} should resolve to 2026-08-08 22:35, got {:?}",
            result.result
        );
    }
}

#[test]
fn compact_date_without_time_still_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("08Aug2026");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "2026-08-08");
}

#[test]
fn difference_between_two_compact_date_times() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("08Aug2026 22:35 UTC - 08Aug2026 20:35 UTC");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "2 hours");
}
