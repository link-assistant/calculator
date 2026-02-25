//! Tests for issue #20: Unrecognized input: `2 UF + 1 USD`
//!
//! Issue #20: `2 UF + 1 USD` fails with "Unit mismatch: cannot add 'UF' and 'USD'"
//!   (and later: "Cannot convert USD to UF: No exchange rate available")
//!
//!   Root cause: UF (Unidad de Fomento, ISO 4217: CLF) was not registered in the
//!               currency database and had no exchange rate available.
//!
//!   Fix: Added CLF to default currencies with `parse_currency()` recognizing both "UF"
//!        and "CLF" as aliases, and added a default USD/CLF exchange rate.

use link_calculator::Calculator;

/// Issue #20: `2 UF + 1 USD` should succeed (main issue expression).
#[test]
fn test_issue_20_uf_plus_usd() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("2 UF + 1 USD");
    assert!(
        result.success,
        "2 UF + 1 USD should succeed (UF is CLF, a recognized currency), got error: {:?}",
        result.error
    );
    // Result should be in CLF/UF (first currency in expression)
    assert!(result.result.contains("CLF"), "Result should be in CLF");
}

/// Issue #20: `1 USD + 2 UF` should succeed (reversed operands).
#[test]
fn test_issue_20_usd_plus_uf() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("1 USD + 2 UF");
    assert!(
        result.success,
        "1 USD + 2 UF should succeed, got error: {:?}",
        result.error
    );
    // Result should be in USD (first currency)
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// UF alone should parse as CLF currency.
#[test]
fn test_issue_20_uf_alone() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("5 UF");
    assert!(
        result.success,
        "5 UF should succeed (UF is recognized as CLF), got error: {:?}",
        result.error
    );
    assert!(result.result.contains('5'), "Result should contain 5");
    assert!(result.result.contains("CLF"), "Result should be in CLF");
}

/// CLF ISO code should be recognized directly.
#[test]
fn test_issue_20_clf_iso_code() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("3 CLF");
    assert!(
        result.success,
        "3 CLF should succeed (CLF is the ISO 4217 code for Unidad de Fomento), got error: {:?}",
        result.error
    );
    assert!(result.result.contains('3'), "Result should contain 3");
    assert!(result.result.contains("CLF"), "Result should be in CLF");
}

/// CLF to USD conversion should work.
#[test]
fn test_issue_20_clf_to_usd_conversion() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("0 USD + 1 CLF");
    assert!(
        result.success,
        "CLF to USD conversion should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// UF to USD conversion via exchange rate should produce a positive result.
#[test]
fn test_issue_20_uf_to_usd_value() {
    let mut calculator = Calculator::new();
    // 1 CLF ≈ 45 USD (default rate: 1 USD = 0.022 CLF, so 1 CLF ≈ 45.45 USD)
    let result = calculator.calculate_internal("0 USD + 1 UF");
    assert!(
        result.success,
        "0 USD + 1 UF should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
    // The result should be a reasonable non-zero USD amount (approximately 45 USD per UF)
    assert!(
        !result.result.contains("0 USD"),
        "Result should be non-zero"
    );
}

/// UF mixed with EUR should also work via USD triangulation.
#[test]
fn test_issue_20_uf_plus_eur() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("1 UF + 1 EUR");
    assert!(
        result.success,
        "1 UF + 1 EUR should succeed (via USD triangulation), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("CLF"), "Result should be in CLF");
}

/// `CurrencyDatabase` should recognize CLF as a known currency.
#[test]
fn test_issue_20_clf_is_known_currency() {
    use link_calculator::types::CurrencyDatabase;
    let db = CurrencyDatabase::new();
    assert!(
        db.is_known_currency("CLF"),
        "CLF should be a known currency in the database"
    );
}

/// `CurrencyDatabase` should have a default rate for USD to CLF.
#[test]
fn test_issue_20_usd_clf_rate_exists() {
    use link_calculator::types::CurrencyDatabase;
    let db = CurrencyDatabase::new();
    let rate = db.get_rate("USD", "CLF");
    assert!(
        rate.is_some(),
        "USD to CLF exchange rate should be available"
    );
    let rate_value = rate.unwrap();
    // 1 USD should be worth between 0.015 and 0.035 CLF (roughly 28-65 USD per CLF)
    assert!(
        rate_value > 0.015 && rate_value < 0.035,
        "USD to CLF rate should be approximately 0.022 (1 CLF ≈ 45 USD), got {rate_value}"
    );
}

/// `CurrencyDatabase` should have a rate for CLF to USD (inverse rate).
#[test]
fn test_issue_20_clf_usd_rate_exists() {
    use link_calculator::types::CurrencyDatabase;
    let db = CurrencyDatabase::new();
    let rate = db.get_rate("CLF", "USD");
    assert!(
        rate.is_some(),
        "CLF to USD exchange rate should be available (inverse rate)"
    );
    let rate_value = rate.unwrap();
    // 1 CLF should be worth between 28 and 70 USD
    assert!(
        rate_value > 28.0 && rate_value < 70.0,
        "CLF to USD rate should be approximately 45 (1 CLF ≈ 45 USD), got {rate_value}"
    );
}

/// `parse_currency` should map "UF" to "CLF".
#[test]
fn test_issue_20_parse_currency_uf() {
    use link_calculator::types::CurrencyDatabase;
    assert_eq!(
        CurrencyDatabase::parse_currency("UF"),
        Some("CLF".to_string()),
        "UF should be parsed as CLF"
    );
    assert_eq!(
        CurrencyDatabase::parse_currency("uf"),
        Some("CLF".to_string()),
        "uf (lowercase) should be parsed as CLF"
    );
}

/// `parse_currency` should map "CLF" to "CLF".
#[test]
fn test_issue_20_parse_currency_clf() {
    use link_calculator::types::CurrencyDatabase;
    assert_eq!(
        CurrencyDatabase::parse_currency("CLF"),
        Some("CLF".to_string()),
        "CLF should be parsed as CLF"
    );
    assert_eq!(
        CurrencyDatabase::parse_currency("clf"),
        Some("CLF".to_string()),
        "clf (lowercase) should be parsed as CLF"
    );
}

/// Natural language "unidad de fomento" should be recognized.
#[test]
fn test_issue_20_parse_currency_natural_language() {
    use link_calculator::types::CurrencyDatabase;
    assert_eq!(
        CurrencyDatabase::parse_currency("unidad de fomento"),
        Some("CLF".to_string()),
        "unidad de fomento should be parsed as CLF"
    );
}
