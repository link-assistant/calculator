//! Regression tests for issue #187: dividing compatible duration units.
//!
//! The original report showed `8 часов / 30 минут` evaluating to
//! `0.2666666666666667 hours`, because the raw numbers were divided and the
//! left-hand duration unit was preserved. Duration divided by duration should
//! be a unitless ratio after converting both sides to the same base unit.

use link_calculator::Calculator;

#[test]
fn issue_187_russian_hours_divided_by_minutes_is_unitless_ratio() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("8 часов / 30 минут");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "16");
    assert_eq!(result.lino_interpretation, "((8 hours) / (30 minutes))");
}

#[test]
fn issue_187_english_hours_divided_by_minutes_is_unitless_ratio() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("8 hours / 30 minutes");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "16");
}

#[test]
fn issue_187_reverse_duration_division_converts_units() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("30 minutes / 8 hours");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "0.0625");
}
