//! Tests for Issue #73: `10 RUB + 10 USD + 11 INR` uses hardcoded rates instead of CBR.
//!
//! Root causes identified:
//! 1. The CBR API (`cbr.ru/scripts/XML_daily.asp`) does NOT return CORS headers, so
//!    browser-based WASM cannot directly fetch from it. The `fetch_cbr_rates()` call
//!    in the web worker fails silently with a CORS error.
//! 2. The `data/currency/*.lino` files are NOT served via GitHub Pages because they
//!    are not copied to `web/public/` during the CI build.
//! 3. INR was missing from `CBR_CURRENCIES` in `scripts/download_historical_rates.py`,
//!    so no `inr-rub.lino` file existed.
//!
//! Fixes implemented:
//! - Added INR to `CBR_CURRENCIES` in download script (inr-rub.lino will be generated).
//! - Added copy step in `web-build` CI job to copy data/currency/ to web/public/data/currency/.
//! - Added fallback in web worker to load .lino files from GitHub Pages when CBR CORS fails.
//!
//! These tests verify the Rust-level behavior after rates are injected, which is what
//! happens after either the direct CBR fetch OR the .lino file fallback succeeds.

use link_calculator::Calculator;

/// Verify that the expression `10 RUB + 10 USD + 11 INR` gives the correct result
/// when CBR rates are loaded (simulating what the web worker does after fetching rates).
///
/// With CBR rates (approximately 2026-02-26):
/// - 1 USD ≈ 76.47 RUB
/// - 1 INR ≈ 0.8408 RUB (100 INR = 84.08 RUB)
///
/// Expected: 10 + (10 × 76.47) + (11 × 0.8408) ≈ 783.93 RUB
/// Old hardcoded result was: 916.38 RUB (using 1 USD = 89.5 RUB, 1 INR ≈ 1.034 RUB)
#[test]
fn test_issue_73_expression_with_cbr_rates() {
    let mut calc = Calculator::new();

    // Load CBR rates as the web worker would after a successful CBR fetch or .lino fallback
    // These approximate real CBR rates as of 2026-02-26
    let cbr_rates = r#"{"usd": 76.47, "eur": 90.32, "inr": 0.8408}"#;
    calc.update_cbr_rates_from_api("2026-02-26", cbr_rates);

    let result = calc.calculate_internal("10 RUB + 10 USD + 11 INR");
    assert!(
        result.success,
        "10 RUB + 10 USD + 11 INR should succeed with CBR rates"
    );

    // With CBR rates: 10 + 764.7 + 9.2488 = 783.9488 RUB
    let result_f64: f64 = result
        .result
        .trim_end_matches(" RUB")
        .parse()
        .expect("Result should be a number in RUB");

    // The correct result should be roughly 783-784 RUB, NOT the hardcoded 916 RUB
    assert!(
        result_f64 > 770.0 && result_f64 < 800.0,
        "Result should be ~783 RUB with CBR rates, got {result_f64} RUB. The old hardcoded result was ~916 RUB."
    );
}

/// Verify that steps reference CBR as the source for the issue #73 expression.
#[test]
fn test_issue_73_steps_show_cbr_source() {
    let mut calc = Calculator::new();

    let cbr_rates = r#"{"usd": 76.47, "inr": 0.8408}"#;
    calc.update_cbr_rates_from_api("2026-02-26", cbr_rates);

    let result = calc.calculate_internal("10 RUB + 10 USD + 11 INR");
    assert!(result.success, "Calculation should succeed");

    let steps_str = result.steps.join("\n");

    // Both USD→RUB and INR→RUB should reference CBR
    assert!(
        steps_str.contains("cbr.ru (Central Bank of Russia)"),
        "Steps should reference CBR as the rate source, got steps: {steps_str}"
    );
}

/// Verify that INR→RUB uses a direct CBR rate (not via USD triangulation)
/// after CBR rates are loaded with both USD and INR.
#[test]
fn test_issue_73_inr_rub_no_triangulation_after_cbr() {
    let mut calc = Calculator::new();

    // Load both USD and INR directly from CBR
    let cbr_rates = r#"{"usd": 76.47, "inr": 0.8408}"#;
    calc.update_cbr_rates_from_api("2026-02-26", cbr_rates);

    let result = calc.calculate_internal("0 RUB + 11 INR");
    assert!(result.success, "11 INR → RUB should succeed");

    let steps_str = result.steps.join("\n");

    // With CBR direct rate, the step should NOT say "(via USD)"
    assert!(
        !steps_str.contains("via USD"),
        "With direct CBR INR rate, should not triangulate via USD. Steps: {steps_str}"
    );
}

/// Verify USD→RUB rate uses CBR value (not hardcoded 89.5) for issue #73 expression.
#[test]
fn test_issue_73_usd_rub_rate_is_cbr_not_hardcoded() {
    let mut calc = Calculator::new();

    // CBR rate: 1 USD = 76.47 RUB
    // Hardcoded rate: 1 USD = 89.5 RUB
    let cbr_rates = r#"{"usd": 76.47}"#;
    calc.update_cbr_rates_from_api("2026-02-26", cbr_rates);

    let result = calc.calculate_internal("0 RUB + 10 USD");
    assert!(result.success, "10 USD → RUB should succeed");

    let result_f64: f64 = result
        .result
        .trim_end_matches(" RUB")
        .parse()
        .expect("Result should be a number");

    // Should be 764.7, not 895.0 (hardcoded)
    assert!(
        (result_f64 - 764.7).abs() < 1.0,
        "10 USD should be ~764.7 RUB using CBR rate 76.47, not {result_f64} RUB (hardcoded would be 895.0)"
    );
}

/// Regression test: verify the hardcoded default gives the WRONG answer (916 RUB)
/// to confirm we understand the issue and that our fix makes it right.
#[test]
fn test_issue_73_hardcoded_default_gives_wrong_result() {
    // Without CBR rates loaded, Calculator uses hardcoded defaults
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("10 RUB + 10 USD + 11 INR");
    assert!(
        result.success,
        "Calculation should succeed even with hardcoded rates"
    );

    let result_f64: f64 = result
        .result
        .trim_end_matches(" RUB")
        .parse()
        .expect("Result should be a number");

    // With hardcoded 1 USD = 89.5 RUB, result is ~916 RUB - this is WRONG
    // With CBR rates (~76.47 RUB/USD), result would be ~784 RUB - this is correct
    // This test documents the existing wrong behavior
    assert!(
        result_f64 > 900.0,
        "Without CBR rates, the hardcoded result should be ~916 RUB, got {result_f64} RUB"
    );
}

/// Verify that loading INR rate from a typical .lino file entry works correctly.
/// The CBR .lino format stores INR rates as: 1 INR = X RUB (after nominal division).
/// The download script divides by nominal (100 for INR): 84.0766 / 100 = 0.840766 RUB/INR.
#[test]
fn test_issue_73_inr_lino_rate_format() {
    let mut calc = Calculator::new();

    // This simulates what the fallback code in the web worker does after reading
    // the inr-rub.lino file (latest rate: 1 INR = 0.840766 RUB)
    let cbr_rates = r#"{"inr": 0.840766}"#;
    calc.update_cbr_rates_from_api("2026-02-26", cbr_rates);

    let result = calc.calculate_internal("0 RUB + 100 INR");
    assert!(result.success, "100 INR → RUB should succeed");

    let result_f64: f64 = result
        .result
        .trim_end_matches(" RUB")
        .parse()
        .expect("Result should be a number");

    // 100 INR × 0.840766 ≈ 84.0766 RUB
    assert!(
        (result_f64 - 84.0766).abs() < 1.0,
        "100 INR should be ~84.0766 RUB, got {result_f64} RUB"
    );
}
