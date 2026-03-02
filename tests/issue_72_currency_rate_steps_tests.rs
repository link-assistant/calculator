//! Tests for Issue #72: Currency conversion steps should always show the rate source,
//! effective date, and exact rate value.
//!
//! Root cause: `Expression::UnitConversion` in `evaluate_expr_with_steps` did not
//! check `get_last_used_rate()` after calling `convert_to_unit`, so the exchange rate
//! info (source, date, value) was silently dropped from the steps output.
//!
//! Fix: After `convert_to_unit` call in the `UnitConversion` branch, check
//! `currency_db.get_last_used_rate()` and append the "Exchange rate: …" step,
//! exactly as the `Binary` branch already does.
//!
//! This applies uniformly to both fiat (USD→EUR) and crypto (ETH→EUR) conversions.

use link_calculator::Calculator;

/// Helper: inject a mock ETH/USD rate and a USD/EUR rate so that
/// `1 ETH in EUR` exercises a cross-rate lookup (ETH→USD→EUR).
fn calc_with_eth_and_eur_rates() -> Calculator {
    let mut calc = Calculator::new();
    // 1 ETH = 1625.0 USD (mock crypto rate)
    let crypto_json = r#"{"ETH": 1625.0}"#;
    calc.update_crypto_rates_from_api("USD", "2026-02-25", crypto_json);
    // 1 USD = 0.92 EUR (mock fiat rate)
    let fiat_json = r#"{"eur": 0.92}"#;
    calc.update_rates_from_api("USD", "2026-02-25", fiat_json);
    calc
}

// ── Fiat UnitConversion steps ─────────────────────────────────────────────────

/// `100 USD as EUR` — fiat-to-fiat unit conversion shows rate info in steps.
#[test]
fn test_fiat_unit_conversion_steps_show_rate_info() {
    let mut calc = Calculator::new();
    let rates_json = r#"{"eur": 0.92}"#;
    calc.update_rates_from_api("USD", "2026-02-25", rates_json);

    let result = calc.calculate_internal("100 USD as EUR");
    assert!(
        result.success,
        "100 USD as EUR should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info for fiat UnitConversion. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("source:"),
        "Steps should contain rate source for fiat UnitConversion. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("date:"),
        "Steps should contain rate date for fiat UnitConversion. Steps:\n{steps_text}"
    );
}

/// `100 USD in EUR` — using `in` keyword shows the same rate info.
#[test]
fn test_fiat_unit_conversion_in_keyword_steps_show_rate_info() {
    let mut calc = Calculator::new();
    let rates_json = r#"{"eur": 0.92}"#;
    calc.update_rates_from_api("USD", "2026-02-25", rates_json);

    let result = calc.calculate_internal("100 USD in EUR");
    assert!(
        result.success,
        "100 USD in EUR should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("2026-02-25"),
        "Steps should contain the rate date. Steps:\n{steps_text}"
    );
}

/// `1 EUR as USD` — reverse fiat direction also shows rate info.
#[test]
fn test_reverse_fiat_unit_conversion_steps_show_rate_info() {
    let mut calc = Calculator::new();
    let rates_json = r#"{"eur": 0.92}"#;
    calc.update_rates_from_api("USD", "2026-02-25", rates_json);

    let result = calc.calculate_internal("1 EUR as USD");
    assert!(
        result.success,
        "1 EUR as USD should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info for reverse fiat UnitConversion. Steps:\n{steps_text}"
    );
}

// ── Crypto UnitConversion steps ───────────────────────────────────────────────

/// `1 ETH in USD` — direct crypto-to-fiat unit conversion shows rate info.
#[test]
fn test_crypto_direct_unit_conversion_steps_show_rate_info() {
    let mut calc = Calculator::new();
    let crypto_json = r#"{"ETH": 1625.0}"#;
    calc.update_crypto_rates_from_api("USD", "2026-02-25", crypto_json);

    let result = calc.calculate_internal("1 ETH in USD");
    assert!(
        result.success,
        "1 ETH in USD should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info for crypto UnitConversion. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("source:"),
        "Steps should contain rate source. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("date:"),
        "Steps should contain rate date. Steps:\n{steps_text}"
    );
}

/// `1 ETH in EUR` — the exact expression from issue #72.
/// Crypto-to-fiat cross-rate conversion must show source, date, and exact rate in steps.
#[test]
fn test_issue_72_eth_in_eur_steps_show_rate_info() {
    let mut calc = calc_with_eth_and_eur_rates();

    let result = calc.calculate_internal("1 ETH in EUR");
    assert!(
        result.success,
        "1 ETH in EUR should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info for 1 ETH in EUR. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("source:"),
        "Steps should contain rate source. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("date:"),
        "Steps should contain rate date. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("2026-02-25"),
        "Steps should contain the rate date value. Steps:\n{steps_text}"
    );
}

/// Rate step for fiat conversion shows the exact rate value.
#[test]
fn test_fiat_unit_conversion_steps_show_exact_rate_value() {
    let mut calc = Calculator::new();
    let rates_json = r#"{"eur": 0.92}"#;
    calc.update_rates_from_api("USD", "2026-02-25", rates_json);

    let result = calc.calculate_internal("1 USD as EUR");
    assert!(
        result.success,
        "1 USD as EUR should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("0.92"),
        "Steps should show the exact rate value 0.92. Steps:\n{steps_text}"
    );
}

// ── Same-currency UnitConversion — no rate step ───────────────────────────────

/// `100 USD as USD` — same-currency conversion should NOT show an exchange rate step.
#[test]
fn test_same_currency_unit_conversion_no_rate_step() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("100 USD as USD");
    assert!(
        result.success,
        "100 USD as USD should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        !steps_text.contains("Exchange rate:"),
        "Same-currency UnitConversion should NOT show exchange rate. Steps:\n{steps_text}"
    );
}

// ── Default (hardcoded) fallback rate also shown ───────────────────────────────

/// Without loading API rates, the default hardcoded rate should still appear in steps.
#[test]
fn test_default_rate_shown_in_unit_conversion_steps() {
    let mut calc = Calculator::new();
    // No API rates loaded — falls back to default (hardcoded) rate

    let result = calc.calculate_internal("1 USD as EUR");
    assert!(
        result.success,
        "1 USD as EUR with default rate should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");

    assert!(
        steps_text.contains("Exchange rate:"),
        "Steps should contain exchange rate info even for default rates. Steps:\n{steps_text}"
    );
    assert!(
        steps_text.contains("source:"),
        "Steps should contain rate source. Steps:\n{steps_text}"
    );
}
