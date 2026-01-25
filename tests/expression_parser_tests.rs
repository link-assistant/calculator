//! Unit tests for the expression parser module.

use link_calculator::grammar::ExpressionParser;
use link_calculator::types::{Decimal, Expression, Unit};

#[test]
fn test_parse_simple_number() {
    let parser = ExpressionParser::new();
    let expr = parser.parse("42").unwrap();
    assert!(matches!(expr, Expression::Number { .. }));
}

#[test]
fn test_parse_addition() {
    let parser = ExpressionParser::new();
    let expr = parser.parse("2 + 3").unwrap();
    assert!(matches!(expr, Expression::Binary { .. }));
}

#[test]
fn test_parse_currency() {
    let parser = ExpressionParser::new();
    let expr = parser.parse("100 USD").unwrap();
    if let Expression::Number { value, unit } = expr {
        assert_eq!(value, Decimal::new(100));
        assert_eq!(unit, Unit::currency("USD"));
    } else {
        panic!("Expected Number expression");
    }
}

#[test]
fn test_parse_currency_subtraction() {
    let parser = ExpressionParser::new();
    let expr = parser.parse("84 USD - 34 EUR").unwrap();
    assert!(matches!(expr, Expression::Binary { .. }));
}

#[test]
fn test_evaluate_simple() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("2 + 3").unwrap();
    assert_eq!(value.to_display_string(), "5");
}

#[test]
fn test_evaluate_multiplication() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("4 * 5").unwrap();
    assert_eq!(value.to_display_string(), "20");
}

#[test]
fn test_evaluate_precedence() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("2 + 3 * 4").unwrap();
    assert_eq!(value.to_display_string(), "14"); // 2 + (3 * 4) = 14
}

#[test]
fn test_evaluate_parentheses() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("(2 + 3) * 4").unwrap();
    assert_eq!(value.to_display_string(), "20"); // (2 + 3) * 4 = 20
}

#[test]
fn test_evaluate_negation() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("-5 + 3").unwrap();
    assert_eq!(value.to_display_string(), "-2");
}

#[test]
fn test_datetime_subtraction() {
    let parser = ExpressionParser::new();
    let result = parser.parse_and_evaluate("(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
    assert!(result.is_ok());
    let (value, _, _) = result.unwrap();
    // Should be approximately 1 day, 20 hours, 8 minutes
    assert!(value.to_display_string().contains("day"));
}

#[test]
fn test_lino_representation() {
    let parser = ExpressionParser::new();
    let (_, _, lino) = parser.parse_and_evaluate("84 USD - 34 EUR").unwrap();
    assert!(lino.contains("84 USD"));
    assert!(lino.contains("34 EUR"));
}

// Tests for math functions
#[test]
fn test_parse_function_call() {
    let parser = ExpressionParser::new();
    let expr = parser.parse("sin(0)").unwrap();
    assert!(matches!(expr, Expression::FunctionCall { .. }));
}

#[test]
fn test_evaluate_sin() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("sin(0)").unwrap();
    assert_eq!(value.to_display_string(), "0");
}

#[test]
fn test_evaluate_sqrt() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("sqrt(16)").unwrap();
    assert_eq!(value.to_display_string(), "4");
}

#[test]
fn test_evaluate_power() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("2^3").unwrap();
    assert_eq!(value.to_display_string(), "8");
}

#[test]
fn test_evaluate_pi() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("pi()").unwrap();
    let pi = value.as_decimal().unwrap().to_f64();
    assert!((pi - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_evaluate_complex_expression() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("2 + sin(0) * 3").unwrap();
    assert_eq!(value.to_display_string(), "2");
}

#[test]
fn test_evaluate_nested_functions() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("sqrt(abs(-16))").unwrap();
    assert_eq!(value.to_display_string(), "4");
}

#[test]
fn test_evaluate_function_with_expression() {
    let parser = ExpressionParser::new();
    let (value, _, _) = parser.parse_and_evaluate("sqrt(4 + 12)").unwrap();
    assert_eq!(value.to_display_string(), "4");
}
