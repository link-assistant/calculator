//! Tests for issue #132: Support factorial `!` postfix notation.
//!
//! Issue: `63!` returned `Parse error: Unexpected character '!' at position 2`.
//!
//! Root cause: The lexer in `src/grammar/lexer.rs` did not recognize `!` as a valid
//! token, hitting the catch-all error branch. The parser had no rule for postfix `!`.
//!
//! Fix: Added `Bang` token to the lexer and handled `expr!` → `factorial(expr)` in
//! the parser's `parse_unary()` method, following the same pattern as `expr%`.

use link_calculator::Calculator;

// ── Basic factorial notation ──────────────────────────────────────────────────

/// The original issue: 63! should be parsed and evaluated.
#[test]
fn test_issue_132_63_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("63!");
    assert!(
        result.success,
        "63! should succeed, got error: {:?}",
        result.error
    );
    // 63! = 26012963668938446981945847715207989052594335780736960000000000000
    // The exact value may be represented in scientific notation depending on precision
    assert!(
        !result.result.is_empty(),
        "63! result should not be empty, got: {}",
        result.result
    );
}

/// 5! = 120
#[test]
fn test_issue_132_5_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5!");
    assert!(
        result.success,
        "5! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "120",
        "5! should equal 120, got: {}",
        result.result
    );
}

/// 0! = 1 (by convention)
#[test]
fn test_issue_132_0_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("0!");
    assert!(
        result.success,
        "0! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "1",
        "0! should equal 1, got: {}",
        result.result
    );
}

/// 1! = 1
#[test]
fn test_issue_132_1_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1!");
    assert!(
        result.success,
        "1! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "1",
        "1! should equal 1, got: {}",
        result.result
    );
}

/// 10! = 3628800
#[test]
fn test_issue_132_10_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10!");
    assert!(
        result.success,
        "10! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "3628800",
        "10! should equal 3628800, got: {}",
        result.result
    );
}

// ── Factorial in expressions ─────────────────────────────────────────────────

/// (3+2)! = 5! = 120
#[test]
fn test_issue_132_expression_factorial() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(3+2)!");
    assert!(
        result.success,
        "(3+2)! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "120",
        "(3+2)! should equal 120, got: {}",
        result.result
    );
}

/// 5! + 3! = 120 + 6 = 126
#[test]
fn test_issue_132_sum_of_factorials() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5! + 3!");
    assert!(
        result.success,
        "5! + 3! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "126",
        "5! + 3! should equal 126, got: {}",
        result.result
    );
}

/// 5! * 2 = 240
#[test]
fn test_issue_132_factorial_times_number() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5! * 2");
    assert!(
        result.success,
        "5! * 2 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "240",
        "5! * 2 should equal 240, got: {}",
        result.result
    );
}

/// 6! / 4! = 720 / 24 = 30
#[test]
fn test_issue_132_factorial_division() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("6! / 4!");
    assert!(
        result.success,
        "6! / 4! should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "30",
        "6! / 4! should equal 30, got: {}",
        result.result
    );
}

// ── Backward compatibility: factorial() function notation ────────────────────

/// The existing `factorial(5)` notation must still work.
#[test]
fn test_issue_132_factorial_function_notation_unchanged() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("factorial(5)");
    assert!(
        result.success,
        "factorial(5) should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "120",
        "factorial(5) should equal 120, got: {}",
        result.result
    );
}

// ── Error cases ───────────────────────────────────────────────────────────────

/// (-1)! should return a domain error (factorial not defined for negatives).
#[test]
fn test_issue_132_negative_factorial_error() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(-1)!");
    assert!(
        !result.success,
        "(-1)! should fail, got result: {}",
        result.result
    );
}

/// 3.5! should return a domain error (factorial requires non-negative integer).
#[test]
fn test_issue_132_non_integer_factorial_error() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("3.5!");
    assert!(
        !result.success,
        "3.5! should fail, got result: {}",
        result.result
    );
}
