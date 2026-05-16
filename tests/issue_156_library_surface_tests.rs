//! Tests for the exportable library surface (issue #156).
//!
//! These tests assert from the *outside* — exactly the way a downstream Rust
//! consumer such as `formal-ai` would consume the crate — that:
//!
//! 1. The core types are reachable through the crate root or one of the
//!    publicly re-exported modules.
//! 2. The full pipeline (parse → plan → evaluate-with-steps → re-encode as
//!    Links Notation) is callable without crossing any `pub(crate)` or
//!    private boundaries.
//! 3. The new high-level [`Calculator::calculate_with_value`] convenience
//!    preserves every artefact: AST, structured value, raw step list, LINO.
//! 4. The lower-level primitives exposed on [`ExpressionParser`]
//!    (`evaluate_with_steps`, `evaluate_expr`, `apply_binary_op`,
//!    `evaluate_at`, `evaluate_expr_with_var`) compose to reproduce the
//!    high-level result.
//! 5. The [`utils::generate_issue_link`] helper is reachable from outside.

use link_calculator::grammar::{evaluate_power, ExpressionParser, Lexer};
use link_calculator::plan::{CalculationPlan, RateSource};
use link_calculator::types::{BinaryOp, Decimal, Expression, Rational, Unit, Value, ValueKind};
use link_calculator::utils::{generate_issue_link, truncate};
use link_calculator::{CalculationResult, Calculator, VERSION};

#[test]
fn version_constant_is_reachable() {
    // The constant has to live at the crate root for the documented
    // `link_calculator::VERSION` pattern to keep working.
    assert!(!VERSION.is_empty());
    assert!(VERSION.chars().any(char::is_numeric));
}

#[test]
fn calculator_calculate_with_value_returns_every_artefact() {
    let mut calculator = Calculator::new();
    let (expression, value, steps, lino) = calculator
        .calculate_with_value("(2 + 3) * 4")
        .expect("expression must parse and evaluate");

    // AST
    assert!(matches!(expression, Expression::Binary { .. }));

    // Structured value (not just a string).
    let rational = value
        .as_rational()
        .expect("integer arithmetic should yield an exact rational");
    assert_eq!(rational.numer(), 20);
    assert_eq!(rational.denom(), 1);

    // Steps include both the input and the final result.
    assert!(steps.first().unwrap().starts_with("Input expression"));
    assert!(steps.last().unwrap().contains("Final result"));

    // Re-encoding the AST as LINO must round-trip with the lino we got back.
    assert_eq!(expression.to_lino(), lino);
}

#[test]
fn calculate_with_value_propagates_empty_input_error() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_with_value("   ");
    assert!(result.is_err());
}

#[test]
fn calculate_with_value_propagates_parse_error() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_with_value("???invalid???");
    assert!(result.is_err());
}

#[test]
fn parser_accessors_expose_underlying_expression_parser() {
    let calculator = Calculator::new();
    let parser_ref: &ExpressionParser = calculator.parser();
    // Sanity check: the borrowed parser can be used for a read-only parse.
    let expr = parser_ref.parse("1 + 2").expect("parses");
    assert!(matches!(
        expr,
        Expression::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn expression_parser_evaluate_with_steps_is_callable_directly() {
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("3 * 7").expect("parses");
    let (value, steps) = parser
        .evaluate_with_steps(&expr)
        .expect("evaluates with steps");
    let rational = value.as_rational().expect("integer result");
    assert_eq!(rational.numer(), 21);
    assert!(steps.iter().any(|s| s.contains("Compute: 3 * 7")));
}

#[test]
fn expression_parser_apply_binary_op_is_callable_directly() {
    let mut parser = ExpressionParser::new();
    let left = Value::rational(Rational::from_integer(10));
    let right = Value::rational(Rational::from_integer(4));
    let result = parser
        .apply_binary_op(&left, BinaryOp::Subtract, &right)
        .expect("subtraction");
    assert_eq!(result.as_rational().unwrap().numer(), 6);
}

#[test]
fn expression_parser_evaluate_expr_with_steps_uses_caller_buffer() {
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("8 / 2").expect("parses");
    let mut steps = vec!["before evaluation".to_string()];
    let value = parser
        .evaluate_expr_with_steps(&expr, &mut steps)
        .expect("evaluates");
    assert_eq!(value.as_rational().unwrap().numer(), 4);
    assert_eq!(steps[0], "before evaluation");
    assert!(steps.len() > 1);
}

#[test]
fn expression_parser_evaluate_expr_works_without_steps() {
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("9 - 1").expect("parses");
    let value = parser.evaluate_expr(&expr).expect("evaluates");
    assert_eq!(value.as_rational().unwrap().numer(), 8);
}

#[test]
fn expression_parser_evaluate_at_substitutes_variable() {
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("x * x + 1").expect("parses");
    let result = parser
        .evaluate_at(&expr, "x", 3.0)
        .expect("substitutes and evaluates");
    assert!((result.to_f64() - 10.0).abs() < 1e-9);
}

#[test]
fn expression_parser_evaluate_expr_with_var_keeps_value_kind() {
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("x + 1").expect("parses");
    let value = parser
        .evaluate_expr_with_var(&expr, "x", Decimal::from_f64(4.0))
        .expect("evaluates with var");
    assert!((value.as_decimal().unwrap().to_f64() - 5.0).abs() < 1e-9);
}

#[test]
fn evaluate_power_helper_is_reachable() {
    let base = Value::rational(Rational::from_integer(2));
    let exp = Value::rational(Rational::from_integer(10));
    let value = evaluate_power(&base, &exp).expect("2^10");
    assert_eq!(value.as_rational().unwrap().numer(), 1024);
}

#[test]
fn utils_generate_issue_link_is_reachable() {
    let link = generate_issue_link("foo", "bar");
    assert!(link.starts_with("https://github.com/link-assistant/calculator/issues/new"));
    assert!(link.contains("title="));
    assert!(link.contains("body="));
}

#[test]
fn utils_truncate_is_reachable() {
    assert_eq!(truncate("hello world", 5), "hello");
    assert_eq!(truncate("short", 50), "short");
}

#[test]
fn lexer_is_reachable_through_grammar_module() {
    let mut lexer = Lexer::new("1 + 2");
    let tokens = lexer.tokenize().expect("tokenizes");
    assert!(!tokens.is_empty());
}

#[test]
fn calculator_plan_returns_typed_calculation_plan() {
    let calculator = Calculator::new();
    let plan: CalculationPlan = calculator.plan_internal("100 USD + 50 EUR");
    assert!(plan.success);
    assert!(plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.currencies.contains(&"USD".to_string()));
    assert!(plan.currencies.contains(&"EUR".to_string()));
}

#[test]
fn calculate_internal_returns_calculation_result_with_steps_and_lino() {
    let mut calculator = Calculator::new();
    let result: CalculationResult = calculator.calculate_internal("2 + 3");
    assert!(result.success);
    assert_eq!(result.result, "5");
    assert_eq!(result.lino_interpretation, "(2 + 3)");
    assert!(result
        .steps
        .iter()
        .any(|s| s.starts_with("Input expression")));
    assert!(result.steps.iter().any(|s| s.contains("Final result")));
}

#[test]
fn high_level_and_low_level_paths_produce_the_same_value() {
    let input = "5 + 6";

    // High-level path
    let mut hi = Calculator::new();
    let (_, hi_value, hi_steps, hi_lino) = hi
        .calculate_with_value(input)
        .expect("high-level evaluation");

    // Low-level path
    let mut lo = ExpressionParser::new();
    let expr = lo.parse(input).expect("parses");
    let lo_lino = expr.to_lino();
    let (lo_value, lo_steps) = lo.evaluate_with_steps(&expr).expect("evaluates");

    assert_eq!(hi_value.to_display_string(), lo_value.to_display_string());
    assert_eq!(hi_steps, lo_steps);
    assert_eq!(hi_lino, lo_lino);
}

#[test]
fn structured_value_kind_is_reachable_through_public_value_kind() {
    let mut calculator = Calculator::new();
    let (_, value, _, _) = calculator
        .calculate_with_value("100 USD")
        .expect("evaluates");
    assert!(matches!(
        value.kind,
        ValueKind::Rational(_) | ValueKind::Number(_)
    ));
    assert!(matches!(value.unit, Unit::Currency(_)));
}
