//! Regression tests for issue #215: "17 августа 2026 года - 180 дней" failed to parse.

use link_calculator::Calculator;

#[test]
fn reported_expression_is_supported() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("17 августа 2026 года - 180 дней");

    assert!(
        result.success,
        "expression should succeed: {:?}",
        result.error
    );
    assert_eq!(result.result, "2026-02-18");
}

#[test]
fn russian_year_markers_are_accepted_after_a_date() {
    let mut calculator = Calculator::new();

    for expression in [
        "17 августа 2026 года",
        "17 августа 2026 год",
        "17 августа 2026 г",
        "17 августа 2026 г.",
        "17 August 2026 года",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
        assert_eq!(
            result.result, "2026-08-17",
            "{expression:?} should resolve to 2026-08-17"
        );
    }
}

#[test]
fn year_word_still_works_as_a_duration_unit() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("2 года + 1 год");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "3 years");
}
