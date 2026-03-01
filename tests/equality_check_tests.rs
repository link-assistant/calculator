//! Tests for equality check expressions (issue #87).
//!
//! The calculator should support `=` as an equality assertion operator,
//! returning `true` or `false` depending on whether both sides are equal.
//!
//! Examples:
//! - `1 * (2 / 3) = (1 * 2) / 3` → `true`
//! - `1 + 1 = 3` → `false`

use link_calculator::Calculator;

/// Tests for the exact example from issue #87.
#[test]
fn test_issue_87_example() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 * (2 / 3) = (1 * 2) / 3");
    assert!(
        result.success,
        "Expected success but got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "true",
        "Expected 'true' because 1 * (2/3) equals (1*2)/3"
    );
}

#[test]
fn test_simple_equality_true() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 + 1 = 2");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_simple_equality_false() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 + 1 = 3");
    assert!(result.success);
    assert_eq!(result.result, "false");
}

#[test]
fn test_equality_with_multiplication() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("3 * 4 = 12");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_with_division() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10 / 2 = 5");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_both_sides_have_operations() {
    let mut calc = Calculator::new();
    // 2 + 3 = 1 + 4 → both equal 5 → true
    let result = calc.calculate_internal("2 + 3 = 1 + 4");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_lino_notation() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 + 1 = 2");
    assert!(result.success);
    assert!(
        result.lino_interpretation.contains('='),
        "Lino notation should contain '=': {}",
        result.lino_interpretation
    );
    assert_eq!(result.lino_interpretation, "((1 + 1) = 2)");
}

#[test]
fn test_equality_issue_87_lino_notation() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 * (2 / 3) = (1 * 2) / 3");
    assert!(result.success);
    // Both sides should be wrapped in their own lino expressions
    assert!(
        result.lino_interpretation.contains('='),
        "Lino notation should contain '=': {}",
        result.lino_interpretation
    );
}

#[test]
fn test_equality_steps_show_comparison() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2 + 2 = 4");
    assert!(result.success);
    let steps = result.steps.join("\n");
    assert!(
        steps.contains("Check equality"),
        "Steps should contain 'Check equality': {steps}"
    );
    assert!(
        steps.contains("Compare:"),
        "Steps should contain 'Compare:': {steps}"
    );
}

#[test]
fn test_equality_with_decimal() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("0.5 + 0.5 = 1");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_negative_numbers() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("-1 + -1 = -2");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_with_parentheses() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(2 + 3) * 4 = 20");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_associativity() {
    let mut calc = Calculator::new();
    // Multiplication is associative: (a * b) * c = a * (b * c)
    let result = calc.calculate_internal("(2 * 3) * 4 = 2 * (3 * 4)");
    assert!(result.success);
    assert_eq!(result.result, "true");
}

#[test]
fn test_equality_unequal_different_operations() {
    let mut calc = Calculator::new();
    // 5 - 3 = 4 - 1 → 2 ≠ 3 → false
    let result = calc.calculate_internal("5 - 3 = 4 - 1");
    assert!(result.success);
    assert_eq!(result.result, "false");
}
