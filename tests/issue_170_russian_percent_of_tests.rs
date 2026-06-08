//! Regression tests for issue #170: Russian percent-of connector.

use link_calculator::grammar::{Lexer, TokenKind};
use link_calculator::Calculator;

#[test]
fn issue_170_exact_russian_percent_of_input_evaluates() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("38% от 100к");

    assert!(
        result.success,
        "38% от 100к should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "38000");
    assert_eq!(result.lino_interpretation, "((38 / 100) * 100000)");
}

#[test]
fn issue_170_russian_ot_tokenizes_as_percent_of_connector() {
    let mut lexer = Lexer::new("38% от 100к");
    let tokens = lexer
        .tokenize()
        .unwrap_or_else(|err| panic!("lexing should succeed: {err}"));

    assert!(
        matches!(tokens.get(2).map(|token| &token.kind), Some(TokenKind::Of)),
        "expected `от` to tokenize as Of, got {:?}",
        tokens.get(2)
    );
}
