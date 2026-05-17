//! Tests for issue #158: binary `%` must be parsed as remainder/modulo in
//! math-like infix expressions, while postfix percent forms must keep working.
//!
//! The original bug accepted only a valid prefix of `100 - 25 % 7`, evaluated
//! it as `100 - (25 / 100)`, and silently dropped the trailing `7`.

use link_calculator::Calculator;

fn calculate(input: &str) -> link_calculator::CalculationResult {
    Calculator::new().calculate_internal(input)
}

#[test]
fn issue_158_binary_percent_remainder_matches_reported_expression() {
    let result = calculate("100 - 25 % 7");

    assert!(
        result.success,
        "100 - 25 % 7 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "96");
    assert_eq!(result.lino_interpretation, "(100 - (25 % 7))");
    assert!(
        result.steps.iter().any(|step| step == "Compute: 25 % 7"),
        "steps should include the modulo operation, got: {:?}",
        result.steps
    );
}

#[test]
fn issue_158_binary_percent_has_multiplicative_precedence() {
    let result = calculate("100 - 25 % 7 * 2");

    assert!(
        result.success,
        "100 - 25 % 7 * 2 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "92");
    assert_eq!(result.lino_interpretation, "(100 - ((25 % 7) * 2))");
}

#[test]
fn issue_158_binary_percent_supports_negative_left_operand() {
    let result = calculate("-5 % 2");

    assert!(
        result.success,
        "-5 % 2 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "-1");
}

#[test]
fn issue_158_postfix_percent_still_works() {
    let standalone = calculate("50%");
    assert!(
        standalone.success,
        "50% should still succeed, got error: {:?}",
        standalone.error
    );
    assert_eq!(standalone.result, "0.5");

    let multiplied = calculate("3% * 50");
    assert!(
        multiplied.success,
        "3% * 50 should still succeed, got error: {:?}",
        multiplied.error
    );
    assert_eq!(multiplied.result, "1.5");
}

#[test]
fn issue_158_percent_of_still_works() {
    let result = calculate("8% of $50");

    assert!(
        result.success,
        "8% of $50 should still succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "4 USD");
    assert_eq!(result.lino_interpretation, "((8 / 100) * (50 USD))");
}

#[test]
fn issue_158_rejects_trailing_tokens_after_postfix_percent() {
    let result = calculate("50% please");

    assert!(
        !result.success,
        "50% please must not silently return {:?}",
        result.result
    );
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|error| error.contains("please")),
        "error should mention the unconsumed token, got: {:?}",
        result.error
    );
}

#[test]
fn issue_158_rejects_custom_unit_text_after_arithmetic() {
    let result = calculate("2 + 2 please");

    assert!(
        !result.success,
        "2 + 2 please must not silently return {:?}",
        result.result
    );
}
