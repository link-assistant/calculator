//! Regression coverage for issue #168.
//!
//! A standalone `.` used to lex as a zero-length number token. Because the
//! lexer did not advance, full tokenization looped until the process aborted
//! with an allocation failure.

use link_calculator::grammar::Lexer;
use link_calculator::Calculator;

#[test]
fn issue_168_lexer_rejects_standalone_dot_without_loop() {
    let mut lexer = Lexer::new(".");

    let error = lexer
        .next_token()
        .expect_err("standalone dot must be rejected, not emitted as an empty token");

    assert!(
        error.to_string().contains("Unexpected character '.'"),
        "error should mention the bare dot, got: {error}"
    );
}

#[test]
fn issue_168_calculate_with_value_rejects_bare_or_dangling_dot_inputs() {
    for input in ["2. 3", "2+2. 3+3", "2.", ". 3"] {
        let mut calculator = Calculator::new();

        let error = calculator
            .calculate_with_value(input)
            .expect_err("bare or dangling dot input must return a recoverable error");

        assert!(
            error.to_string().contains("Unexpected character '.'"),
            "{input:?} should mention the invalid dot, got: {error}"
        );
    }
}

#[test]
fn issue_168_valid_decimals_still_evaluate() {
    let mut calculator = Calculator::new();

    let (_expression, value, _steps, lino) = calculator
        .calculate_with_value("3.14 + 2.5")
        .expect("well-formed decimal expression should still evaluate");

    assert_eq!(value.to_display_string(), "5.64");
    assert_eq!(lino, "(3.14 + 2.5)");
}
