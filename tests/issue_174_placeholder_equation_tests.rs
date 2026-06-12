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
fn rejects_unsupported_placeholder_equations() {
    for input in ["? * ? = 4", "* * * = 4", "? + = 4", "2 * = 8", "** + 2 = 4"] {
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
