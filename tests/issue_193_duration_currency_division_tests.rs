//! Regression tests for issue #193.
//!
//! A raw date difference should behave as a numeric day count when divided by
//! a currency expression. Explicit duration-unit conversions, such as hours,
//! should also divide by currency as their numeric unit count instead of
//! preserving the duration unit in the result.

use link_calculator::Calculator;

#[test]
fn issue_193_date_difference_divided_by_inr_product_uses_day_count() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(((2026-08-08) - (2026-06-17)) / (30 * (3500 INR)))");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "0.000495238095238095");
    assert_eq!(result.fraction.as_deref(), Some("13/26250"));
}

#[test]
fn issue_193_localized_currency_name_uses_same_duration_day_count_rule() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(((2026-08-08) - (2026-06-17)) / (30 * (3500 рупий)))");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "0.000495238095238095");
    assert_eq!(result.fraction.as_deref(), Some("13/26250"));
}

#[test]
fn issue_193_explicit_duration_unit_divided_by_currency_becomes_number() {
    let mut calc = Calculator::new();
    let result =
        calc.calculate_internal("(((2026-08-08) - (2026-06-17)) as hours) / (30 * (3500 INR))");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "0.01188571428571429");
    assert_eq!(result.fraction.as_deref(), Some("52/4375"));
}
