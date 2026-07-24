//! Regression tests for issue #207 and its sub-issues #203-#206.

use link_calculator::Calculator;

#[test]
fn reported_time_span_expressions_are_supported() {
    let mut calculator = Calculator::new();

    for expression in [
        "days between 8th august and now",
        "((8th august - now) as days)",
        "8th of august - now",
        "days to 8th of august",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
    }
}

#[test]
fn days_between_returns_the_first_date_minus_the_second_in_days() {
    let mut calculator = Calculator::new();
    let result =
        calculator.calculate_internal("days between 8th august 2026 and 24th of july 2026");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "15 days");
}

#[test]
fn days_to_matches_target_date_minus_now_in_days() {
    let mut calculator = Calculator::new();
    let natural = calculator.calculate_internal("days to 8th of august");
    let explicit = calculator.calculate_internal("(8 august - now) as days");

    assert!(natural.success, "natural form failed: {:?}", natural.error);
    assert!(
        explicit.success,
        "explicit form failed: {:?}",
        explicit.error
    );

    let natural_days = natural
        .result
        .strip_suffix(" days")
        .expect("natural result should use days")
        .parse::<f64>()
        .expect("natural result should be numeric");
    let explicit_days = explicit
        .result
        .strip_suffix(" days")
        .expect("explicit result should use days")
        .parse::<f64>()
        .expect("explicit result should be numeric");
    assert!(
        (natural_days - explicit_days).abs() <= 2.0 / 86_400.0,
        "forms should differ by at most their evaluation time: {natural_days} vs {explicit_days}"
    );
}
