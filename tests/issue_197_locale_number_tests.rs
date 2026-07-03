//! Regression tests for issue #197.
//!
//! Users may enter numbers using decimal/grouping conventions from their
//! browser languages. The parser should try supported locale conventions
//! instead of rejecting comma-decimal input outright.

use link_calculator::Calculator;

#[test]
fn decimal_comma_expression_calculates() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("82,6172 / 100");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "0.826172");
    assert_eq!(result.lino_interpretation, "(82.6172 / 100)");
}

#[test]
fn ambiguous_comma_number_exposes_locale_interpretations() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("1,234 / 100");

    assert!(plan.success, "plan failed: {:?}", plan.error);
    assert_eq!(plan.lino_interpretation, "(1.234 / 100)");

    let alternatives = plan.alternative_lino.expect("missing alternatives");
    assert_eq!(alternatives, vec!["(1.234 / 100)", "(1234 / 100)"]);
}
