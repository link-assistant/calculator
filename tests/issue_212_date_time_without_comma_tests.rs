//! Regression tests for issue #212: "08 Aug 2026 22:35 - now" failed to parse.

use link_calculator::Calculator;

#[test]
fn reported_expression_is_supported() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("08 Aug 2026 22:35 - now");

    assert!(
        result.success,
        "expression should succeed: {:?}",
        result.error
    );
}

#[test]
fn date_followed_by_time_without_comma_parses() {
    let mut calculator = Calculator::new();

    for expression in [
        "08 Aug 2026 22:35",
        "08 Aug 2026 22:35 UTC",
        "Aug 08, 2026 22:35",
        "2026-08-08 22:35",
        "08 Aug 2026 10:35pm",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("2026-08-08"),
            "{expression:?} should resolve to 2026-08-08, got {:?}",
            result.result
        );
    }
}

#[test]
fn difference_between_two_date_times_without_comma() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("08 Aug 2026 22:35 UTC - 08 Aug 2026 20:35 UTC");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "2 hours");
}
