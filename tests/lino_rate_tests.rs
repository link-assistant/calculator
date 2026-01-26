//! Tests that verify .lino rate files are properly loaded and used in calculations.
//!
//! These tests ensure that:
//! 1. Rates from .lino files can be loaded into the calculator
//! 2. Currency conversions actually use those loaded rates
//! 3. Historical rates work with the "at" date syntax

use link_calculator::Calculator;

/// Test that we can load rates from a .lino file and use them in calculations.
/// Uses the "Feb 8, 2021" date format which is parsed as a `DateTime` correctly.
#[test]
fn test_load_lino_rates_and_use_in_conversion() {
    let mut calculator = Calculator::new();

    // Load USD to RUB rates in the new .lino format
    let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192";

    let result = calculator.load_rates_from_consolidated_lino(lino_content);
    assert!(result.is_ok(), "Should load rates successfully");
    assert_eq!(result.unwrap(), 2, "Should load 2 rates");

    // Use month name format since ISO date format (YYYY-MM-DD) is tokenized
    // as number-minus-number-minus-number instead of a date
    let calc_result = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
    assert!(
        calc_result.success,
        "Historical conversion should succeed: {:?}",
        calc_result.error
    );

    // Check that the steps show we're using the loaded rate, not the default
    let steps_text = calc_result.steps.join("\n");
    assert!(
        steps_text.contains("cbr.ru") || steps_text.contains("74.26"),
        "Should use the loaded rate from cbr.ru, not default. Steps: {steps_text}"
    );

    // Parse the result to get the numeric value
    let result_str = calc_result.result.replace(" RUB", "").replace(',', "");
    let result_value: f64 = result_str
        .trim()
        .parse()
        .expect("Should parse result as number");

    // The rate for 2021-02-08 is 74.2602, so 1 USD should = ~74.26 RUB
    assert!(
        (result_value - 74.2602).abs() < 0.01,
        "1 USD at 2021-02-08 should be ~74.26 RUB (from .lino file), got {result_value}. Steps: {steps_text}"
    );
}

/// Test that different dates return different historical rates.
/// Note: Uses month name format (e.g., "Jan 25, 2021") as ISO date format
/// (YYYY-MM-DD) is currently tokenized as arithmetic, not as a date.
#[test]
fn test_different_dates_use_different_rates() {
    let mut calculator = Calculator::new();

    // Load multiple dates of USD to EUR rates
    let lino_content = "conversion:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  rates:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402";

    calculator
        .load_rates_from_consolidated_lino(lino_content)
        .expect("Should load rates");

    // Test first date - using month name format
    let result1 = calculator.calculate_internal("(0 EUR + 1 USD) at Jan 25, 2021");
    assert!(result1.success);
    let val1_str = result1.result.replace(" EUR", "").replace(',', "");
    let val1: f64 = val1_str.trim().parse().expect("Should parse");
    assert!(
        (val1 - 0.8234).abs() < 0.001,
        "Rate on Jan 25, 2021 should be 0.8234, got {val1}"
    );

    // Test second date - using month name format
    let result2 = calculator.calculate_internal("(0 EUR + 1 USD) at Feb 8, 2021");
    assert!(result2.success);
    let val2_str = result2.result.replace(" EUR", "").replace(',', "");
    let val2: f64 = val2_str.trim().parse().expect("Should parse");
    assert!(
        (val2 - 0.8402).abs() < 0.001,
        "Rate on Feb 8, 2021 should be 0.8402, got {val2}"
    );

    // Rates should be different
    assert!(
        (val1 - val2).abs() > 0.01,
        "Different dates should have different rates"
    );
}

/// Test loading rates for multiple currency pairs.
#[test]
fn test_multiple_currency_pairs() {
    let mut calculator = Calculator::new();

    // Load EUR to GBP rates
    let eur_gbp_content = "conversion:
  from EUR
  to GBP
  source 'ecb.europa.eu'
  rates:
    2021-02-08 0.8765";

    // Load USD to JPY rates
    let usd_jpy_content = "conversion:
  from USD
  to JPY
  source 'boj.or.jp'
  rates:
    2021-02-08 105.25";

    calculator
        .load_rates_from_consolidated_lino(eur_gbp_content)
        .expect("Should load EUR/GBP");
    calculator
        .load_rates_from_consolidated_lino(usd_jpy_content)
        .expect("Should load USD/JPY");

    // Test EUR to GBP conversion - using month name format
    let result1 = calculator.calculate_internal("(0 GBP + 1 EUR) at Feb 8, 2021");
    assert!(result1.success);
    let val1_str = result1.result.replace(" GBP", "").replace(',', "");
    let val1: f64 = val1_str.trim().parse().expect("Should parse");
    assert!(
        (val1 - 0.8765).abs() < 0.001,
        "EUR->GBP rate should be 0.8765, got {val1}"
    );

    // Test USD to JPY conversion - using month name format
    let result2 = calculator.calculate_internal("(0 JPY + 1 USD) at Feb 8, 2021");
    assert!(result2.success);
    let val2_str = result2.result.replace(" JPY", "").replace(',', "");
    let val2: f64 = val2_str.trim().parse().expect("Should parse");
    assert!(
        (val2 - 105.25).abs() < 0.1,
        "USD->JPY rate should be 105.25, got {val2}"
    );
}

/// Test that the inverse rate is also available after loading.
#[test]
fn test_inverse_rate_available() {
    let mut calculator = Calculator::new();

    // Load USD to RUB rate
    let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru'
  rates:
    2021-02-08 74.2602";

    calculator
        .load_rates_from_consolidated_lino(lino_content)
        .expect("Should load rates");

    // Test forward conversion: USD -> RUB (using month name format)
    let result1 = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
    assert!(result1.success);

    // Test inverse conversion: RUB -> USD (using month name format)
    let result2 = calculator.calculate_internal("(0 USD + 100 RUB) at Feb 8, 2021");
    assert!(result2.success);
    let val_str = result2.result.replace(" USD", "").replace(',', "");
    let val: f64 = val_str.trim().parse().expect("Should parse");

    // 100 RUB at 74.2602 rate = 100 / 74.2602 ≈ 1.347 USD
    let expected = 100.0 / 74.2602;
    assert!(
        (val - expected).abs() < 0.01,
        "100 RUB should be ~{expected:.3} USD, got {val}"
    );
}

/// Test that rate source and date are shown in calculation steps.
#[test]
fn test_rate_info_shown_in_steps() {
    let mut calculator = Calculator::new();

    let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602";

    calculator
        .load_rates_from_consolidated_lino(lino_content)
        .expect("Should load rates");

    // Use month name format for the date
    let result = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
    assert!(result.success);

    let steps_text = result.steps.join("\n");

    // Should show the source from the loaded .lino file
    assert!(
        steps_text.contains("cbr.ru"),
        "Steps should contain rate source 'cbr.ru'. Steps: {steps_text}"
    );

    // Should show the rate value
    assert!(
        steps_text.contains("74.26"),
        "Steps should contain exchange rate 74.26. Steps: {steps_text}"
    );
}

/// Test loading the legacy .lino format (rates/data) still works.
#[test]
fn test_legacy_lino_format() {
    let mut calculator = Calculator::new();

    // Legacy format: rates: as root, data: for rates
    let lino_content = "rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315";

    let result = calculator.load_rates_from_consolidated_lino(lino_content);
    assert!(result.is_ok(), "Should load legacy format");
    assert_eq!(result.unwrap(), 2, "Should load 2 rates");

    // Verify the rate is used - using month name format for the date
    let calc_result = calculator.calculate_internal("(0 EUR + 1 USD) at Jan 25, 2021");
    assert!(calc_result.success);
    let val_str = calc_result.result.replace(" EUR", "").replace(',', "");
    let val: f64 = val_str.trim().parse().expect("Should parse");
    assert!(
        (val - 0.8234).abs() < 0.001,
        "Rate should be 0.8234, got {val}"
    );
}

/// Test arithmetic with currency conversion uses correct rates.
#[test]
fn test_currency_arithmetic_with_loaded_rates() {
    let mut calculator = Calculator::new();

    let lino_content = "conversion:
  from USD
  to EUR
  source 'test'
  rates:
    2021-02-08 0.85";

    calculator
        .load_rates_from_consolidated_lino(lino_content)
        .expect("Should load rates");

    // Test: 100 USD + 50 EUR at Feb 8, 2021
    // First, 50 EUR needs to be converted to USD
    // 50 EUR = 50 / 0.85 USD ≈ 58.82 USD
    // Total = 100 + 58.82 = 158.82 USD
    let result = calculator.calculate_internal("(100 USD + 50 EUR) at Feb 8, 2021");
    assert!(result.success, "Arithmetic with conversion should succeed");
    let val_str = result.result.replace(" USD", "").replace(',', "");
    let val: f64 = val_str.trim().parse().expect("Should parse");

    let expected = 100.0 + (50.0 / 0.85);
    assert!(
        (val - expected).abs() < 0.1,
        "100 USD + 50 EUR should be ~{expected:.2} USD, got {val}"
    );
}

/// Test subtraction with currency conversion.
#[test]
fn test_currency_subtraction_with_loaded_rates() {
    let mut calculator = Calculator::new();

    let lino_content = "conversion:
  from USD
  to EUR
  source 'test'
  rates:
    2021-02-08 0.85";

    calculator
        .load_rates_from_consolidated_lino(lino_content)
        .expect("Should load rates");

    // Test: 100 USD - 34 EUR at Feb 8, 2021
    // 34 EUR = 34 / 0.85 USD ≈ 40 USD
    // Result = 100 - 40 = 60 USD
    let result = calculator.calculate_internal("(100 USD - 34 EUR) at Feb 8, 2021");
    assert!(
        result.success,
        "Subtraction with conversion should succeed: {:?}",
        result.error
    );
    let val_str = result.result.replace(" USD", "").replace(',', "");
    let val: f64 = val_str.trim().parse().expect("Should parse");

    let expected = 100.0 - (34.0 / 0.85);
    assert!(
        (val - expected).abs() < 0.1,
        "100 USD - 34 EUR should be ~{expected:.2} USD, got {val}"
    );
}

/// Test that ISO date format (YYYY-MM-DD) in "at" clause is currently parsed
/// as arithmetic (subtraction), not as a date. This is a known limitation.
/// Users should use month name format (e.g., "Feb 8, 2021") for historical
/// date queries.
#[test]
fn test_iso_date_format_limitation() {
    let mut calculator = Calculator::new();

    // Note: "2021-02-08" after "at" is tokenized as 2021 - 02 - 08 = 2011
    // This is a known limitation of the current lexer.
    let result = calculator.calculate_internal("(0 RUB + 1 USD) at 2021-02-08");
    assert!(result.success);

    // The expression is evaluated as: (0 RUB + 1 USD) at 2011
    // which fails because 2011 (a number) is not a valid DateTime
    // Actually it seems to succeed, let's check what happens

    // The current implementation might handle this differently
    // This test documents the current behavior
    let steps_text = result.steps.join("\n");

    // The "at" clause evaluates to a number (2021 - 02 - 08 = 2011)
    // which is likely treated as year-only or causes fallback to default rates
    // Document the actual behavior here:
    assert!(
        steps_text.contains("default") || steps_text.contains("89.5"),
        "ISO dates are not properly parsed in at clause - falls back to default rate. Steps: {steps_text}"
    );
}
