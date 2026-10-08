use super::*;

#[test]
fn test_number_expression() {
    let expr = Expression::number(Decimal::new(42));
    assert_eq!(expr.to_string(), "42");
    assert_eq!(expr.to_lino(), "42");
}

#[test]
fn test_binary_expression() {
    let left = Expression::number(Decimal::new(2));
    let right = Expression::number(Decimal::new(3));
    let expr = Expression::binary(left, BinaryOp::Add, right);
    assert_eq!(expr.to_string(), "2 + 3");
    assert_eq!(expr.to_lino(), "(2 + 3)");
}

#[test]
fn test_complex_expression() {
    let usd = Expression::currency(Decimal::new(84), "USD");
    let eur = Expression::currency(Decimal::new(34), "EUR");
    let expr = Expression::binary(usd, BinaryOp::Subtract, eur);
    assert!(expr.to_lino().contains("84 USD"));
    assert!(expr.to_lino().contains("34 EUR"));
}

#[test]
fn test_binary_op_precedence() {
    assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
    assert_eq!(
        BinaryOp::Modulo.precedence(),
        BinaryOp::Multiply.precedence()
    );
    assert_eq!(BinaryOp::Add.precedence(), BinaryOp::Subtract.precedence());
}

#[test]
fn test_depth() {
    let simple = Expression::number(Decimal::new(1));
    assert_eq!(simple.depth(), 1);
    let binary = Expression::binary(
        Expression::number(Decimal::new(1)),
        BinaryOp::Add,
        Expression::number(Decimal::new(2)),
    );
    assert_eq!(binary.depth(), 2);
}
