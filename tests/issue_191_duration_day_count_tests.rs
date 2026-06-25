//! Regression tests for issue #191.
//!
//! Date subtraction produces a raw duration. When that duration is used in
//! scalar arithmetic, users expect its day count, so expressions like
//! `(8 августа - 17 июня) / 30 * 3500` should evaluate as
//! `52 / 30 * 3500` instead of failing as duration multiplication.

use link_calculator::Calculator;

#[test]
fn issue_191_russian_date_difference_divided_by_number_multiplies_as_day_count() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(8 августа - 17 июня) / 30 * 3500");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "6066.66666666667");
    assert_eq!(result.fraction.as_deref(), Some("18200/3"));
}

#[test]
fn issue_191_day_count_arithmetic_preserves_currency_unit() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("((8 августа - 17 июня) / 30 * 3500 рупий)");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "6066.66666666667 INR");
    assert_eq!(result.fraction.as_deref(), Some("18200/3"));
}

#[test]
fn issue_191_raw_duration_can_be_explicitly_converted_to_days() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(8 августа - 17 июня) as days");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "52 days");
}

#[test]
fn issue_191_raw_duration_can_be_explicitly_converted_to_number() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(8 августа - 17 июня) as number");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "52");
}

#[test]
fn issue_191_divided_duration_can_be_explicitly_labeled_as_days() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("((8 августа - 17 июня) / 30) as days");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "1.733333333333333 days");
    assert_eq!(result.fraction.as_deref(), Some("26/15"));
}
