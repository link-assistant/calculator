//! Regression tests for issue #134: partial Russian dates in date subtraction.

use link_calculator::Calculator;

#[test]
fn test_issue_134_russian_partial_dates_subtract_as_dates() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("18 апреля - 28 марта");

    assert!(
        result.success,
        "18 апреля - 28 марта should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "21 days",
        "18 апреля - 28 марта should be interpreted as a date interval"
    );
}

#[test]
fn test_issue_134_partial_russian_date_uses_current_year() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("18 апреля");

    assert!(
        result.success,
        "18 апреля should parse as a partial date, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("-04-18"),
        "18 апреля should resolve to April 18 in the current year, got: {}",
        result.result
    );
}
