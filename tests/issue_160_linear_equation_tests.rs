//! Regression tests for issue #160: single-variable linear equations.

use link_calculator::{
    types::{Rational, ValueKind},
    Calculator,
};

#[test]
fn solves_issue_160_examples() {
    for (input, expected) in [
        ("x*2 = 123", "x = 61.5"),
        ("2 * x + 3 = 11", "x = 4"),
        ("10 = y / 3 + 1", "y = 27"),
        ("2 * (x + 3) = 10", "x = 2"),
        ("2 * x + 3 = x + 11", "x = 8"),
    ] {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal(input);

        assert!(
            result.success,
            "expected success for {input:?}, got {:?}",
            result.error
        );
        assert_eq!(result.result, expected);
        assert!(
            result.lino_interpretation.contains('='),
            "equation lino should contain '=': {}",
            result.lino_interpretation
        );
    }
}

#[test]
fn calculate_with_value_returns_structured_equation_solution() {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc.calculate_with_value("2 * x + 3 = 11").unwrap();

    assert_eq!(value.to_display_string(), "x = 4");
    assert_eq!(
        value.kind,
        ValueKind::EquationSolution {
            variable: "x".to_string(),
            value: Rational::from_integer(4),
        }
    );
    assert_eq!(lino, "(((2 * x) + 3) = 11)");
    assert!(
        steps.iter().any(|step| step == "Solve linear equation:"),
        "steps should include equation solving details: {steps:?}"
    );
}

#[test]
fn preserves_numeric_equality_checks() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 + 1 = 2");

    assert!(result.success);
    assert_eq!(result.result, "true");
}
