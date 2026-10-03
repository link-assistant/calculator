//! Tests for issue #224: `35usd- 200cny- 1hkd` failed with
//! "Cannot convert HKD to USD: No exchange rate available".
//!
//! Root cause: a fresh `Calculator` only knew a handful of hardcoded fallback
//! rates (USD, EUR, GBP, JPY, CHF, CNY, RUB, INR, CLF, VND, KZT). Every other
//! currency — HKD, AUD, CAD, SEK, … — only worked after the live ECB fetch in
//! the browser succeeded, even though the repository already ships the daily
//! central-bank rates in `data/currency/*.lino`.
//!
//! Fix: `build.rs` embeds the latest row of every `data/currency/*.lino` file
//! and `CurrencyDatabase::new()` uses it for currencies that have no
//! fallback rate at all. Live API rates still override these bundled ones.

use link_calculator::Calculator;

fn parse_amount(result: &str, currency: &str) -> f64 {
    result
        .trim()
        .strip_suffix(currency)
        .unwrap_or_else(|| panic!("result {result:?} should end with {currency}"))
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("result {result:?} should be numeric: {e}"))
}

/// The exact input reported in issue #224.
#[test]
fn test_issue_224_exact_input() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("35usd- 200cny- 1hkd");
    assert!(
        result.success,
        "35usd- 200cny- 1hkd should calculate, got error: {:?}",
        result.error
    );
    // 35 USD - 200 CNY (~27.6 USD) - 1 HKD (~0.13 USD) ≈ 7.3 USD.
    let value = parse_amount(&result.result, "USD");
    assert!(
        (5.0..10.0).contains(&value),
        "expected roughly 7.3 USD, got {}",
        result.result
    );
}

/// The same expression with spaces around the operators.
#[test]
fn test_issue_224_spaced_input() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("35 USD - 200 CNY - 1 HKD");
    assert!(result.success, "got error: {:?}", result.error);
    assert!(result.result.ends_with("USD"), "got {}", result.result);
}

/// The bundled rate is shown in the steps with its real source and date.
#[test]
fn test_issue_224_hkd_conversion_shows_bundled_rate_source() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("1 HKD in USD");
    assert!(result.success, "got error: {:?}", result.error);
    // The HKD peg keeps 1 HKD within 0.1275..0.1290 USD.
    let value = parse_amount(&result.result, "USD");
    assert!((0.12..0.14).contains(&value), "got {}", result.result);

    let steps = result.steps.join("\n");
    assert!(
        steps.contains("frankfurter.dev (ECB)"),
        "steps should name the bundled ECB source:\n{steps}"
    );
    assert!(
        !steps.contains("date: unknown"),
        "bundled rates carry their real date:\n{steps}"
    );
}

/// Every USD-quoted ECB currency shipped in `data/currency` converts out of the box.
#[test]
fn test_issue_224_all_bundled_ecb_currencies_convert() {
    for code in [
        "AUD", "BRL", "CAD", "CZK", "DKK", "HKD", "HUF", "KRW", "MXN", "NOK", "NZD", "PLN", "SEK",
        "SGD", "TRY", "ZAR",
    ] {
        let mut calculator = Calculator::new();
        let input = format!("100 {code} in USD");
        let result = calculator.calculate_internal(&input);
        assert!(
            result.success,
            "{input} should convert with bundled rates, got error: {:?}",
            result.error
        );
        assert!(
            result.result.ends_with("USD"),
            "{input} -> {}",
            result.result
        );

        // Cross conversion between two non-USD currencies triangulates via USD.
        let input = format!("100 {code} in EUR");
        let result = calculator.calculate_internal(&input);
        assert!(result.success, "{input} failed: {:?}", result.error);
    }
}

/// Bundled rates only fill gaps: existing hardcoded defaults are unchanged.
#[test]
fn test_issue_224_bundled_rates_do_not_replace_existing_defaults() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("1 USD in RUB");
    assert!(result.success, "got error: {:?}", result.error);
    assert_eq!(result.result, "89.5 RUB");
}

/// Live API rates still override bundled rates.
#[test]
fn test_issue_224_api_rates_override_bundled_rates() {
    let mut calculator = Calculator::new();
    calculator.update_rates_from_api("usd", "2026-10-02", r#"{"hkd": 8.0}"#);
    let result = calculator.calculate_internal("8 HKD in USD");
    assert!(result.success, "got error: {:?}", result.error);
    assert_eq!(result.result, "1 USD");
}

/// "SEK" (Swedish krona) collided with German "sek" (seconds): uppercase ISO
/// codes are currencies, lowercase "sek" stays a duration unless the
/// conversion target says otherwise.
#[test]
fn test_issue_224_sek_currency_vs_german_seconds() {
    let mut calculator = Calculator::new();

    let result = calculator.calculate_internal("100 SEK");
    assert!(result.success, "got error: {:?}", result.error);
    assert_eq!(result.result, "100 SEK");

    let result = calculator.calculate_internal("100 sek in usd");
    assert!(result.success, "got error: {:?}", result.error);
    assert!(result.result.ends_with("USD"), "got {}", result.result);

    let result = calculator.calculate_internal("100 USD in SEK");
    assert!(result.success, "got error: {:?}", result.error);
    assert!(result.result.ends_with("SEK"), "got {}", result.result);

    let result = calculator.calculate_internal("10 sek");
    assert!(result.success, "got error: {:?}", result.error);
    assert_eq!(result.result, "10 seconds");
}
