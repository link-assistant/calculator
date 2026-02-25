//! Tests for Issue #70: Use cbr.ru rates for RUB-related conversions.
//!
//! Issue #70: Expression `10 RUB + 10 USD + 11 INR` should use cbr.ru rates
//! for all RUB-related conversions instead of relying on ECB cross-rates via USD.
//!
//! Root cause: The calculator was using hardcoded default rates for USD→RUB
//! and triangulation via USD for INR→RUB (from ECB). The CBR API provides
//! official direct rates for all currencies vs RUB, which should be preferred.
//!
//! Fix: Added `update_cbr_rates_from_api()` that loads RUB rates from cbr.ru,
//! and a corresponding `fetch_cbr_rates()` WASM function. The web worker now
//! fetches CBR rates at startup and applies them before ECB rates so that
//! RUB conversions use official Russian Central Bank data directly.

use link_calculator::Calculator;

/// Verify `update_cbr_rates_from_api` returns the correct number of rates updated.
#[test]
fn test_update_cbr_rates_returns_count() {
    let mut calc = Calculator::new();

    // Simulate CBR rates: 1 USD = 76.63 RUB, 100 INR = 84.24 RUB → 1 INR = 0.8424 RUB
    let rates_json = r#"{"usd": 76.63, "eur": 90.58, "inr": 0.8424}"#;
    let count = calc.update_cbr_rates_from_api("2026-02-25", rates_json);

    // Should update 3 rates (and also their inverses internally)
    assert_eq!(count, 3, "Should update 3 rates from CBR response");
}

/// Verify `update_cbr_rates_from_api` skips invalid JSON.
#[test]
fn test_update_cbr_rates_invalid_json() {
    let mut calc = Calculator::new();
    let count = calc.update_cbr_rates_from_api("2026-02-25", "invalid json");
    assert_eq!(count, 0, "Invalid JSON should return 0");
}

/// Verify `update_cbr_rates_from_api` skips RUB itself.
#[test]
fn test_update_cbr_rates_skips_rub() {
    let mut calc = Calculator::new();

    // The CBR response might include RUB itself (shouldn't be stored as a pair)
    let rates_json = r#"{"rub": 1.0, "usd": 76.63}"#;
    let count = calc.update_cbr_rates_from_api("2026-02-25", rates_json);

    // Should only store USD→RUB, not RUB→RUB
    assert_eq!(count, 1, "Should skip RUB itself");
}

/// Verify that CBR rates override hardcoded defaults for USD→RUB.
#[test]
fn test_cbr_rates_override_hardcoded_usd_rub() {
    let mut calc = Calculator::new();

    // The hardcoded default is 89.5, CBR says 76.63
    let cbr_rates = r#"{"usd": 76.63}"#;
    calc.update_cbr_rates_from_api("2026-02-25", cbr_rates);

    // Now 1 USD → RUB should use CBR rate
    let result = calc.calculate_internal("0 RUB + 1 USD");
    assert!(result.success, "Calculation should succeed");
    assert_eq!(
        result.result, "76.63 RUB",
        "Should use CBR rate of 76.63 for USD→RUB"
    );
}

/// Verify that CBR rates provide direct INR→RUB without needing triangulation via USD.
#[test]
fn test_cbr_rates_provide_direct_inr_rub_rate() {
    let mut calc = Calculator::new();

    // CBR provides direct INR rate: 100 INR = 84.2448 RUB → 1 INR = 0.842448 RUB
    let cbr_rates = r#"{"usd": 76.63, "inr": 0.842448}"#;
    calc.update_cbr_rates_from_api("2026-02-25", cbr_rates);

    // 11 INR → RUB should use direct CBR rate, not ECB triangulation
    let result = calc.calculate_internal("0 RUB + 11 INR");
    assert!(result.success, "Calculation should succeed");

    // 11 * 0.842448 ≈ 9.266928 RUB
    let result_f64: f64 = result
        .result
        .trim_end_matches(" RUB")
        .parse()
        .expect("Result should be a number");
    assert!(
        (result_f64 - 9.266928).abs() < 0.01,
        "11 INR should be ~9.27 RUB using CBR direct rate, got {}",
        result_f64
    );
}

/// Verify the full issue #70 scenario: `10 RUB + 10 USD + 11 INR` uses CBR rates.
#[test]
fn test_issue_70_rub_usd_inr_uses_cbr_rates() {
    let mut calc = Calculator::new();

    // Load CBR rates as they would be fetched from cbr.ru
    // USD: 1 USD = 76.63 RUB
    // INR: 100 INR = 84.2448 RUB → 1 INR = 0.842448 RUB
    let cbr_rates = r#"{"usd": 76.63, "inr": 0.842448}"#;
    calc.update_cbr_rates_from_api("2026-02-25", cbr_rates);

    let result = calc.calculate_internal("10 RUB + 10 USD + 11 INR");
    assert!(result.success, "10 RUB + 10 USD + 11 INR should succeed");

    // Verify steps show CBR rates are used
    let steps_str = result.steps.join("\n");
    assert!(
        steps_str.contains("cbr.ru (Central Bank of Russia)"),
        "Steps should reference CBR as the rate source for RUB conversions, got: {}",
        steps_str
    );
}

/// Verify that CBR rate source is shown in steps for RUB conversions.
#[test]
fn test_cbr_rate_source_shown_in_steps() {
    let mut calc = Calculator::new();

    let cbr_rates = r#"{"usd": 76.63}"#;
    calc.update_cbr_rates_from_api("2026-02-25", cbr_rates);

    let result = calc.calculate_internal("0 RUB + 1 USD");
    assert!(result.success);

    let steps_str = result.steps.join("\n");
    assert!(
        steps_str.contains("cbr.ru (Central Bank of Russia)"),
        "Steps should show CBR as rate source, got: {}",
        steps_str
    );
    assert!(
        steps_str.contains("2026-02-25"),
        "Steps should show CBR date, got: {}",
        steps_str
    );
}

/// Verify that CBR rates eliminate triangulation via USD for cross-currency RUB pairs.
///
/// With CBR rates, INR→RUB should be a direct rate (not "(via USD)").
#[test]
fn test_cbr_rates_eliminate_triangulation_for_inr_rub() {
    let mut calc = Calculator::new();

    // Load both INR and USD rates from CBR
    let cbr_rates = r#"{"usd": 76.63, "inr": 0.842448}"#;
    calc.update_cbr_rates_from_api("2026-02-25", cbr_rates);

    let result = calc.calculate_internal("0 RUB + 11 INR");
    assert!(result.success);

    let steps_str = result.steps.join("\n");
    // Should NOT contain "via USD" since we now have a direct INR→RUB rate from CBR
    assert!(
        !steps_str.contains("via USD"),
        "Steps should NOT contain 'via USD' when direct CBR INR→RUB rate is available, got: {}",
        steps_str
    );
}
