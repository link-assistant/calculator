//! Tests for currency issues #51, #52, #53 (parent issue #54 "Currency issues").
//!
//! Issue #51: `10 рублей + $10 + 10 рупий` fails with "Unexpected character '$'"
//!   Root cause: Lexer did not handle currency symbol characters like `$`, `€`, `£`.
//!   Fix: Added currency symbol characters as identifier tokens in the lexer and
//!        parser support for prefix notation ($10 → 10 USD).
//!
//! Issue #52: `10 рублей + 10 USD + 10 рупий` fails with "Unit mismatch"
//!   Root cause: `CurrencyDatabase::parse_currency()` did not handle Russian-language
//!               currency names (рублей = RUB, рупий = INR).
//!   Fix: Added Russian grammatical forms of рубль and рупия to `parse_currency()`.
//!
//! Issue #53: `10 RUB + 10 USD + 10 INR` fails with "No exchange rate available"
//!   Root cause: INR was not in the default currency database, and no triangulation
//!               was implemented to compute cross-rates like INR→RUB via USD.
//!   Fix: Added INR to default currencies/rates and implemented USD triangulation.

use link_calculator::Calculator;

// ── Issue #51: Currency symbol prefix notation ($10, €5, etc.) ───────────────

/// Issue #51: Dollar sign prefix `$10` should parse as `10 USD`.
#[test]
fn test_issue_51_dollar_prefix_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("$10");
    assert!(
        result.success,
        "$ prefix should be supported, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("10"), "Result should contain 10");
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Issue #51: Dollar sign mixed with other currency expressions should work.
#[test]
fn test_issue_51_dollar_prefix_in_expression() {
    let mut calculator = Calculator::new();
    // $10 + 5 USD = 15 USD
    let result = calculator.calculate_internal("$10 + 5 USD");
    assert!(
        result.success,
        "$10 + 5 USD should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("15"), "Result should be 15 USD");
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Euro prefix symbol `€` should work similarly to `$`.
#[test]
fn test_issue_51_euro_prefix_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("€5");
    assert!(
        result.success,
        "€ prefix should be supported, got error: {:?}",
        result.error
    );
    assert!(result.result.contains('5'), "Result should contain 5");
    assert!(result.result.contains("EUR"), "Result should be in EUR");
}

/// British pound prefix `£` should work.
#[test]
fn test_issue_51_pound_prefix_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("£3");
    assert!(
        result.success,
        "£ prefix should be supported, got error: {:?}",
        result.error
    );
    assert!(result.result.contains('3'), "Result should contain 3");
    assert!(result.result.contains("GBP"), "Result should be in GBP");
}

/// Ruble symbol `₽` as prefix should work.
#[test]
fn test_issue_51_ruble_prefix_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("₽100");
    assert!(
        result.success,
        "₽ prefix should be supported, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("100"), "Result should contain 100");
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

/// Indian rupee symbol `₹` as prefix should work.
#[test]
fn test_issue_51_rupee_prefix_parses() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("₹10");
    assert!(
        result.success,
        "₹ prefix should be supported, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("10"), "Result should contain 10");
    assert!(result.result.contains("INR"), "Result should be in INR");
}

/// Issue #51+52 combined: Full expression from issue #51.
#[test]
fn test_issue_51_full_expression() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 рублей + $10 + 10 рупий");
    assert!(
        result.success,
        "10 рублей + $10 + 10 рупий should succeed (treat as 10 RUB + 10 USD + 10 INR), got error: {:?}",
        result.error
    );
    // Result should be in RUB (first currency)
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

// ── Issue #52: Russian language currency names ────────────────────────────────

/// Issue #52: Russian word for rubles (рублей) should be recognized as RUB.
#[test]
fn test_issue_52_russian_rublei_recognized() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 рублей + 10 RUB");
    assert!(
        result.success,
        "10 рублей + 10 RUB should succeed (рублей = RUB), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("20"), "Result should be 20 RUB");
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

/// Issue #52: Russian word рублей mixed with USD should convert correctly.
#[test]
fn test_issue_52_russian_rublei_with_usd() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 рублей + 10 USD");
    assert!(
        result.success,
        "10 рублей + 10 USD should succeed, got error: {:?}",
        result.error
    );
    // Result should be in RUB (first currency) and contain a reasonable value
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

/// Issue #52: Russian word рупий (Indian rupee) should be recognized as INR.
#[test]
fn test_issue_52_russian_rupiy_recognized() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 рупий + 10 INR");
    assert!(
        result.success,
        "10 рупий + 10 INR should succeed (рупий = INR), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("20"), "Result should be 20 INR");
    assert!(result.result.contains("INR"), "Result should be in INR");
}

/// Issue #52: Full expression from the issue: `10 рублей + 10 USD + 10 рупий`.
#[test]
fn test_issue_52_full_expression() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 рублей + 10 USD + 10 рупий");
    assert!(
        result.success,
        "10 рублей + 10 USD + 10 рупий should succeed, got error: {:?}",
        result.error
    );
    // Result should be in RUB (first currency)
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

// ── Issue #53: INR/RUB exchange rate availability ─────────────────────────────

/// Issue #53: `10 RUB + 10 USD + 10 INR` should succeed.
#[test]
fn test_issue_53_rub_usd_inr_expression() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 RUB + 10 USD + 10 INR");
    assert!(
        result.success,
        "10 RUB + 10 USD + 10 INR should succeed with available exchange rates, got error: {:?}",
        result.error
    );
    // Result should be in RUB (first currency)
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

/// INR should be a recognized currency.
#[test]
fn test_issue_53_inr_is_recognized() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("100 INR");
    assert!(
        result.success,
        "100 INR should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("100"), "Result should contain 100");
    assert!(result.result.contains("INR"), "Result should be in INR");
}

/// INR to USD conversion via default rates should work.
#[test]
fn test_issue_53_inr_to_usd_conversion() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("0 USD + 86.5 INR");
    assert!(
        result.success,
        "INR to USD conversion should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// RUB to INR triangulation via USD should work.
#[test]
fn test_issue_53_rub_to_inr_triangulation() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("0 INR + 100 RUB");
    assert!(
        result.success,
        "RUB to INR conversion (via USD triangulation) should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("INR"), "Result should be in INR");
}

/// English name "rupee" should be recognized as INR.
#[test]
fn test_issue_53_english_rupee_name() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("10 rupees + 10 INR");
    assert!(
        result.success,
        "10 rupees + 10 INR should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("20"), "Result should be 20 INR");
}
