//! Regression tests for issue #174: placeholder unknowns in linear equations.

use link_calculator::{
    types::{Rational, ValueKind},
    Calculator,
};

#[test]
fn solves_question_mark_placeholder_equations() {
    assert_equation_solutions(&[
        ("? + 2 = 4", "? = 2"),
        ("2 + ? = 4", "? = 2"),
        ("4 = ? + 2", "? = 2"),
        ("? - 2 = 4", "? = 6"),
        ("10 - ? = 4", "? = 6"),
        ("? * 2 = 8", "? = 4"),
        ("2 * ? = 8", "? = 4"),
        ("? / 2 = 4", "? = 8"),
        ("(? + 2) * 3 = 12", "? = 2"),
        ("3 * (? + 2) = 12", "? = 2"),
        ("? + ? = 10", "? = 5"),
        ("2 * ? + 3 = 11", "? = 4"),
        ("3 + 2 * ? = 11", "? = 4"),
        ("10 = ? / 3 + 1", "? = 27"),
        ("? / 3 + 1 = 10", "? = 27"),
        ("? + 2.5 = 4", "? = 1.5"),
        ("? - 0.5 = 2", "? = 2.5"),
        ("2 * (? - 1) = 6", "? = 4"),
        ("(? - 1) / 3 = 2", "? = 7"),
        ("-? + 10 = 4", "? = 6"),
        ("? + (-2) = 4", "? = 6"),
        ("2 * ? - 4 = 0", "? = 2"),
        ("0 = 2 * ? - 4", "? = 2"),
        ("? + 0 = 7", "? = 7"),
        ("1 * ? = 9", "? = 9"),
    ]);
}

#[test]
fn solves_star_placeholder_equations_by_parser_position() {
    assert_equation_solutions(&[
        ("* + 2 = 4", "* = 2"),
        ("2 + * = 4", "* = 2"),
        ("4 = * + 2", "* = 2"),
        ("* - 2 = 4", "* = 6"),
        ("10 - * = 4", "* = 6"),
        ("* * 2 = 8", "* = 4"),
        ("2 * * = 8", "* = 4"),
        ("* / 2 = 4", "* = 8"),
        ("(* + 2) * 3 = 12", "* = 2"),
        ("3 * (* + 2) = 12", "* = 2"),
        ("* + * = 10", "* = 5"),
        ("2 * * + 3 = 11", "* = 4"),
        ("3 + 2 * * = 11", "* = 4"),
        ("10 = * / 3 + 1", "* = 27"),
        ("* / 3 + 1 = 10", "* = 27"),
        ("* + 2.5 = 4", "* = 1.5"),
        ("* - 0.5 = 2", "* = 2.5"),
        ("2 * (* - 1) = 6", "* = 4"),
        ("(* - 1) / 3 = 2", "* = 7"),
        ("-* + 10 = 4", "* = 6"),
        ("* + (-2) = 4", "* = 6"),
        ("2 * * - 4 = 0", "* = 2"),
        ("0 = 2 * * - 4", "* = 2"),
        ("* + 0 = 7", "* = 7"),
        ("1 * * = 9", "* = 9"),
    ]);
}

#[test]
fn solves_compact_placeholder_equations_from_integrations() {
    assert_equation_solutions(&[
        ("?+2=4", "? = 2"),
        ("*+2=4", "* = 2"),
        ("2+?=4", "? = 2"),
        ("2+*=4", "* = 2"),
        ("?*2=8", "? = 4"),
        ("2*?=8", "? = 4"),
        ("**2=8", "* = 4"),
        ("2**=8", "* = 4"),
        ("(?+2)*3=12", "? = 2"),
        ("(*+2)*3=12", "* = 2"),
        ("-?+10=4", "? = 6"),
        ("-*+10=4", "* = 6"),
    ]);
}

#[test]
fn named_variable_equations_keep_working_across_many_shapes() {
    for variable in ["x", "y", "z"] {
        for (template, expected_value) in [
            ("{v} + 2 = 4", "2"),
            ("2 + {v} = 4", "2"),
            ("4 = {v} + 2", "2"),
            ("{v} - 2 = 4", "6"),
            ("10 - {v} = 4", "6"),
            ("{v} * 2 = 8", "4"),
            ("2 * {v} = 8", "4"),
            ("{v} / 2 = 4", "8"),
            ("({v} + 2) * 3 = 12", "2"),
            ("3 * ({v} + 2) = 12", "2"),
            ("{v} + {v} = 10", "5"),
            ("2 * {v} + 3 = 11", "4"),
            ("3 + 2 * {v} = 11", "4"),
            ("10 = {v} / 3 + 1", "27"),
            ("{v} / 3 + 1 = 10", "27"),
            ("{v} + 2.5 = 4", "1.5"),
            ("{v} - 0.5 = 2", "2.5"),
            ("2 * ({v} - 1) = 6", "4"),
            ("({v} - 1) / 3 = 2", "7"),
            ("-{v} + 10 = 4", "6"),
            ("{v} + (-2) = 4", "6"),
            ("2 * {v} - 4 = 0", "2"),
            ("0 = 2 * {v} - 4", "2"),
            ("{v} + 0 = 7", "7"),
            ("1 * {v} = 9", "9"),
        ] {
            let input = template.replace("{v}", variable);
            let expected = format!("{variable} = {expected_value}");

            assert_equation_solution(&input, &expected);
        }
    }
}

#[test]
fn solves_eighty_named_variable_school_equations_with_explicit_steps() {
    let variables = ["x", "y", "z", "a"];
    let templates = [
        "{v} + 5 = 12",
        "5 + {v} = 12",
        "{v} - 5 = 2",
        "12 - {v} = 5",
        "{v} * 5 = 35",
        "5 * {v} = 35",
        "{v} / 7 = 1",
        "({v} + 3) / 2 = 5",
        "2 * ({v} + 3) = 20",
        "3 * ({v} - 2) = 15",
        "{v} + 3 = 2 * {v} - 4",
        "5 * {v} - 3 = 2 * {v} + 18",
        "{v} / 2 + 3 = 6.5",
        "-{v} + 12 = 5",
        "0 - {v} = -7",
        "({v} - 1) / 3 + 2 = 4",
        "4 = ({v} + 5) / 3",
        "1.5 * {v} + 0.5 = 11",
        "2 * ({v} - 1.5) = 11",
        "({v} + 1) * 0.5 = 4",
    ];
    let mut case_count = 0;

    for variable in variables {
        for template in templates {
            let input = template.replace("{v}", variable);
            let expected = format!("{variable} = 7");

            assert_equation_solution_with_required_steps(&input, &expected);
            case_count += 1;
        }
    }

    assert_eq!(case_count, 80);
}

#[test]
fn solves_symbolic_multi_variable_equations() {
    assert_equation_solutions(&[
        ("x + y = 10", "x = 10 - y"),
        ("2 * x + 3 * y = 12", "x = 6 - 1.5*y"),
        ("y + x = 10", "y = 10 - x"),
        ("x + ? = 4", "? = 4 - x"),
        ("x + * = 4", "* = 4 - x"),
        ("? + * = 4", "? = 4 - *"),
    ]);
}

#[test]
fn solves_mixed_placeholder_and_named_equations_with_explicit_steps() {
    for (input, expected) in [
        ("x + ? = 4", "? = 4 - x"),
        ("x + * = 4", "* = 4 - x"),
        ("? + * = 4", "? = 4 - *"),
        ("2 * ? + x = 10", "? = 5 - 0.5*x"),
        ("2 * * + y = 12", "* = 6 - 0.5*y"),
        ("? + 2 * * = 12", "? = 12 - 2*(*)"),
        ("2 * ? + 4 * * = 20", "? = 10 - 2*(*)"),
        ("2 * x + ? = 10", "? = 10 - 2*x"),
        ("2 * x + 4 * * = 10", "* = 2.5 - 0.5*x"),
    ] {
        assert_equation_solution_with_required_steps(input, expected);
    }
}

#[test]
fn solves_quadratic_and_higher_power_equations() {
    assert_equation_solutions(&[
        ("x^2 = 4", "x = -2 or x = 2"),
        ("x^2 - 5 * x + 6 = 0", "x = 2 or x = 3"),
        ("x^2 + 5 * x + 6 = 0", "x = -3 or x = -2"),
        ("x^2 = 0", "x = 0"),
        ("x^3 = 27", "x = 3"),
        ("x^3 - x = 0", "x = -1 or x = 0 or x = 1"),
        ("x^4 = 16", "x = -2 or x = 2"),
        ("x^5 - 32 = 0", "x = 2"),
        ("x^6 - 64 = 0", "x = -2 or x = 2"),
        ("x^7 + 128 = 0", "x = -2"),
        ("(x - 2) * (x + 3) = 0", "x = -3 or x = 2"),
        ("(x - 1)^3 = 0", "x = 1"),
        ("(x - 1) * (x - 2) * (x - 3) = 0", "x = 1 or x = 2 or x = 3"),
        (
            "(2 * x - 1) * (x - 3) * (x + 2) = 0",
            "x = -2 or x = 0.5 or x = 3",
        ),
        ("2 * x^2 - 8 = 0", "x = -2 or x = 2"),
        ("?^2 = 9", "? = -3 or ? = 3"),
        ("*^2 = 9", "* = -3 or * = 3"),
        ("? * ? = 4", "? = -2 or ? = 2"),
        ("* * * = 4", "* = -2 or * = 2"),
    ]);
}

#[test]
fn solves_more_than_500_named_multi_variable_equations_with_explicit_steps() {
    let variables = ["x", "y", "z", "a", "b", "c", "m", "n", "p", "q", "r", "s"];
    let mut case_count = 0;

    for target in variables {
        for other in variables {
            if target == other {
                continue;
            }

            for total in 2..=6 {
                let input = format!("{target} + {other} = {total}");
                let expected = format!("{target} = {total} - {other}");

                assert_equation_solution_with_required_steps(&input, &expected);
                case_count += 1;
            }
        }
    }

    assert!(
        case_count > 500,
        "expected more than 500 generated multi-variable cases, got {case_count}"
    );
}

#[test]
fn solves_more_than_1000_power_equation_edge_cases_with_explicit_steps() {
    let variables = ["x", "y", "z", "a", "b", "c", "m", "n", "p", "q"];
    let mut case_count = 0;

    for variable in variables {
        for power in 2..=11 {
            for root in -5..=5 {
                let rhs = pow_i128(root, power);
                let input = format!("{variable}^{power} = {rhs}");
                let expected = expected_power_solution(variable, root, power);

                assert_polynomial_solution_with_required_steps(&input, &expected);
                case_count += 1;
            }
        }
    }

    assert!(
        case_count > 1000,
        "expected more than 1000 generated power-equation cases, got {case_count}"
    );
}

#[test]
fn calculate_with_value_returns_structured_placeholder_solution_and_trace() {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value("2 * ? + 3 = 11")
        .expect("placeholder equation should solve");

    assert_eq!(value.to_display_string(), "? = 4");
    assert_eq!(
        value.kind,
        ValueKind::EquationSolution {
            variable: "?".to_string(),
            value: Rational::from_integer(4),
        }
    );
    assert_eq!(lino, "(((2 * ?) + 3) = 11)");
    assert!(
        steps.iter().any(|step| step == "Solve linear equation:"),
        "steps should mark equation solving: {steps:?}"
    );
    assert!(
        steps.iter().any(|step| step.starts_with("Linear form:")),
        "steps should expose the normalized linear form: {steps:?}"
    );
    assert!(
        steps.iter().any(|step| step.starts_with("Solve for ?:")),
        "steps should expose the derivation step for the placeholder: {steps:?}"
    );
}

#[test]
fn calculate_with_value_returns_structured_star_placeholder_solution_and_trace() {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value("2 * * + 3 = 11")
        .expect("star placeholder equation should solve");

    assert_eq!(value.to_display_string(), "* = 4");
    assert_eq!(
        value.kind,
        ValueKind::EquationSolution {
            variable: "*".to_string(),
            value: Rational::from_integer(4),
        }
    );
    assert_eq!(lino, "(((2 * *) + 3) = 11)");
    assert!(
        steps.iter().any(|step| step == "Solve linear equation:"),
        "steps should mark equation solving: {steps:?}"
    );
    assert!(
        steps.iter().any(|step| step.starts_with("Linear form:")),
        "steps should expose the normalized linear form: {steps:?}"
    );
    assert!(
        steps.iter().any(|step| step.starts_with("Solve for *:")),
        "steps should expose the derivation step for the star placeholder: {steps:?}"
    );
}

#[test]
fn calculate_with_value_returns_structured_repeated_placeholder_solution() {
    for (input, variable, expected_value) in [
        ("? + ? = 10", "?", Rational::from_integer(5)),
        ("* + * = 10", "*", Rational::from_integer(5)),
    ] {
        let mut calc = Calculator::new();
        let (_expr, value, steps, _lino) = calc
            .calculate_with_value(input)
            .unwrap_or_else(|err| panic!("expected success for {input:?}, got {err:?}"));

        assert_eq!(
            value.kind,
            ValueKind::EquationSolution {
                variable: variable.to_string(),
                value: expected_value,
            },
            "wrong structured solution for {input:?}"
        );
        assert_required_step_labels(&steps, input);
    }
}

#[test]
fn calculate_with_value_returns_structured_symbolic_solution_and_detailed_trace() {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value("2 * x + 3 * y = 12")
        .expect("multi-variable equation should solve symbolically");

    assert_eq!(value.to_display_string(), "x = 6 - 1.5*y");
    assert_eq!(
        value.kind,
        ValueKind::SymbolicEquationSolution {
            variable: "x".to_string(),
            expression: "6 - 1.5*y".to_string(),
        }
    );
    assert_eq!(lino, "(((2 * x) + (3 * y)) = 12)");
    assert_required_step_labels(&steps, "2 * x + 3 * y = 12");
}

#[test]
fn calculate_with_value_returns_structured_polynomial_solutions_and_detailed_trace() {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value("x^2 - 5 * x + 6 = 0")
        .expect("quadratic equation should solve");

    assert_eq!(value.to_display_string(), "x = 2 or x = 3");
    assert_eq!(
        value.kind,
        ValueKind::EquationSolutions {
            variable: "x".to_string(),
            values: vec![Rational::from_integer(2), Rational::from_integer(3)],
        }
    );
    assert_eq!(lino, "((((x ^ 2) - (5 * x)) + 6) = 0)");
    assert_required_polynomial_step_labels(&steps, "x^2 - 5 * x + 6 = 0");
}

#[test]
fn rejects_unsupported_placeholder_equations() {
    for input in ["? + = 4", "2 * = 8", "** + 2 = 4"] {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal(input);

        assert!(
            !result.success,
            "expected {input:?} to be rejected, got {}",
            result.result
        );
    }
}

#[test]
fn rejects_unsupported_polynomial_equations() {
    for input in ["x^2 + y = 4", "x / x = 1", "x^2 = 2"] {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal(input);

        assert!(
            !result.success,
            "expected {input:?} to be rejected, got {}",
            result.result
        );
    }
}

fn assert_equation_solutions(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_equation_solution(input, expected);
    }
}

fn assert_equation_solution(input: &str, expected: &str) {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal(input);

    assert!(
        result.success,
        "expected success for {input:?}, got {:?}",
        result.error
    );
    assert_eq!(result.result, expected, "wrong result for {input:?}");
    assert!(
        result.lino_interpretation.contains('='),
        "equation lino should contain '=' for {input:?}: {}",
        result.lino_interpretation
    );
}

fn assert_equation_solution_with_required_steps(input: &str, expected: &str) {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value(input)
        .unwrap_or_else(|err| panic!("expected success for {input:?}, got {err:?}"));

    assert_eq!(
        value.to_display_string(),
        expected,
        "wrong result for {input:?}"
    );
    assert!(
        lino.contains('='),
        "equation lino should contain '=' for {input:?}: {lino}",
    );
    assert_required_step_labels(&steps, input);
}

fn assert_polynomial_solution_with_required_steps(input: &str, expected: &str) {
    let mut calc = Calculator::new();
    let (_expr, value, steps, lino) = calc
        .calculate_with_value(input)
        .unwrap_or_else(|err| panic!("expected success for {input:?}, got {err:?}"));

    assert_eq!(
        value.to_display_string(),
        expected,
        "wrong result for {input:?}"
    );
    assert!(
        lino.contains('='),
        "equation lino should contain '=' for {input:?}: {lino}",
    );
    assert_required_polynomial_step_labels(&steps, input);
}

fn assert_required_step_labels(steps: &[String], input: &str) {
    for label in [
        "Input expression:",
        "Solve linear equation:",
        "Original equation:",
        "Linear form:",
        "Choose target variable:",
        "Combine target coefficients:",
        "Move non-target variable terms to the right:",
        "Move constants to the right:",
        "Right side after moving terms:",
        "Isolate variable term:",
        "Divide both sides by",
        "Solve for",
        "Solution:",
        "Final result:",
    ] {
        assert!(
            steps.iter().any(|step| step.starts_with(label)),
            "steps for {input:?} should include {label:?}: {steps:?}"
        );
    }
}

fn assert_required_polynomial_step_labels(steps: &[String], input: &str) {
    for label in [
        "Input expression:",
        "Solve polynomial equation:",
        "Original equation:",
        "Move all terms to the left:",
        "Polynomial form:",
        "Polynomial degree:",
        "Choose target variable:",
        "Find real rational roots:",
        "Verify root",
        "Solutions:",
        "Solution:",
        "Final result:",
    ] {
        assert!(
            steps.iter().any(|step| step.starts_with(label)),
            "steps for {input:?} should include {label:?}: {steps:?}"
        );
    }
}

fn expected_power_solution(variable: &str, root: i128, power: u32) -> String {
    if root == 0 {
        return format!("{variable} = 0");
    }

    if power % 2 == 0 {
        let root_abs = root.abs();
        format!("{variable} = -{root_abs} or {variable} = {root_abs}")
    } else {
        format!("{variable} = {root}")
    }
}

fn pow_i128(value: i128, power: u32) -> i128 {
    let mut result = 1;
    for _ in 0..power {
        result *= value;
    }
    result
}
