//! Tests for issue #145: `8% of $50` should evaluate to 4 USD.
//!
//! Issue: `8% of $50` returned `0.08` instead of `4 USD`.
//!
//! Root cause: The lexer had no `Of` token, so "of" was parsed as an unknown
//! identifier and ignored.  The parser consumed `8%` (= 8/100 = 0.08) and
//! stopped, silently dropping `$50`.
//!
//! Fix: Added `Of` (`TokenKind::Of`) to the lexer and handled the
//! `expr% of rhs` pattern in `parse_unary()` — `N% of X` is desugared to
//! `(N / 100) * X`, matching the behaviour of the equivalent `N% * X` form.

use link_calculator::Calculator;

// ── Core bug reproduction ─────────────────────────────────────────────────────

/// The original issue: 8% of $50 should equal 4 USD.
#[test]
fn test_issue_145_8_percent_of_50_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("8% of $50");
    assert!(
        result.success,
        "8% of $50 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "4 USD",
        "8% of $50 should equal 4 USD, got: {}",
        result.result
    );
}

// ── Equivalent forms ─────────────────────────────────────────────────────────

/// 8% * $50 — already working; must stay 4 USD.
#[test]
fn test_issue_145_8_percent_times_50_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("8% * $50");
    assert!(
        result.success,
        "8% * $50 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "4 USD",
        "8% * $50 should equal 4 USD, got: {}",
        result.result
    );
}

// ── Percent-of with plain numbers ─────────────────────────────────────────────

/// 10% of 200 = 20
#[test]
fn test_issue_145_10_percent_of_200() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10% of 200");
    assert!(
        result.success,
        "10% of 200 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "20",
        "10% of 200 should equal 20, got: {}",
        result.result
    );
}

/// 50% of 80 = 40
#[test]
fn test_issue_145_50_percent_of_80() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("50% of 80");
    assert!(
        result.success,
        "50% of 80 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "40",
        "50% of 80 should equal 40, got: {}",
        result.result
    );
}

/// 100% of 42 = 42
#[test]
fn test_issue_145_100_percent_of_42() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100% of 42");
    assert!(
        result.success,
        "100% of 42 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "42",
        "100% of 42 should equal 42, got: {}",
        result.result
    );
}

// ── Backward compatibility: standalone % still works ─────────────────────────

/// 50% standalone should still evaluate to 0.5.
#[test]
fn test_issue_145_percent_standalone_unchanged() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("50%");
    assert!(
        result.success,
        "50% should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "0.5",
        "50% should equal 0.5, got: {}",
        result.result
    );
}

/// 3% * 50 should still equal 1.5.
#[test]
fn test_issue_145_percent_times_number_unchanged() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("3% * 50");
    assert!(
        result.success,
        "3% * 50 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "1.5",
        "3% * 50 should equal 1.5, got: {}",
        result.result
    );
}
