//! Regression tests for issue #188.
//!
//! Chained division and multiplication should expose both natural grouping
//! interpretations so the user can choose the intended denominator.

use link_calculator::Calculator;

#[test]
fn division_then_multiplication_exposes_denominator_grouping_alternative() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("253 / 16 * 3");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "47.4375");
    assert_eq!(result.lino_interpretation, "((253 / 16) * 3)");

    let alternatives = result.alternative_lino.expect("missing alternatives");
    assert_eq!(alternatives, vec!["((253 / 16) * 3)", "(253 / (16 * 3))"]);
}

#[test]
fn plan_includes_division_multiplication_interpretations() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("253 / 16 * 3");

    assert!(plan.success, "plan failed: {:?}", plan.error);
    assert_eq!(plan.lino_interpretation, "((253 / 16) * 3)");

    let alternatives = plan.alternative_lino.expect("missing alternatives");
    assert_eq!(alternatives, vec!["((253 / 16) * 3)", "(253 / (16 * 3))"]);
}
