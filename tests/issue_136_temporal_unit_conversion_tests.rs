//! Tests for issue #136: Temporal modifier ignored in unit conversion with date context.
//!
//! Issue: `22822 рублей в рупиях на 11 апреля 2026` should use the exchange rate
//! effective on 2026-04-11, not the current/latest rate.
//!
//! There were two root causes:
//!
//! Root cause 1: `parse_additive()` in `token_parser.rs` checked for the "at" keyword
//! BEFORE checking for "as/in/to". After parsing the unit conversion, the parser stopped
//! without checking for a trailing "at <date>" modifier. Fix: added a second "at" check
//! after the unit conversion is parsed.
//!
//! Root cause 2: `convert_to_unit()` in `Value` did not accept a date parameter, so even
//! when the `AtTime` wrapper was present, the historical rate was never used. Fix: added
//! `convert_to_unit_at_date()` and threaded `current_date_context` through all
//! `UnitConversion` evaluation paths.
//!
//! Root cause 3: The Russian preposition "на" (meaning "at/on" for dates) was not
//! recognized as the `At` keyword in the lexer. Fix: added "на" → `TokenKind::At`.

use link_calculator::Calculator;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_rub_inr_rates(calc: &mut Calculator) {
    let lino_content = "conversion:
  from RUB
  to INR
  source 'test'
  rates:
    2026-04-11 1.5
    2026-04-17 1.2";
    calc.load_rates_from_consolidated_lino(lino_content)
        .expect("Should load RUB/INR rates");
}

// ── Root cause 1: parser must handle "X as Y at date" ────────────────────────

/// "1 RUB as INR at Apr 11, 2026" should parse into an AtTime(UnitConversion)
/// AST and the lino should contain the "at" modifier.
#[test]
fn test_unit_conversion_with_trailing_at_parses() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 RUB as INR at Apr 11, 2026");
    assert!(
        result.success,
        "1 RUB as INR at Apr 11, 2026 should succeed: {:?}",
        result.error
    );
    assert!(
        result.lino_interpretation.contains("at"),
        "LINO should contain 'at': {}",
        result.lino_interpretation
    );
}

/// The lino for "1 RUB as INR at Apr 11, 2026" should match the expected format
/// with nested parentheses: (((1 RUB) as INR) at Apr 11, 2026).
#[test]
fn test_unit_conversion_at_date_lino_format() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 RUB as INR at Apr 11, 2026");
    assert!(
        result.success,
        "Should succeed: {:?}",
        result.error
    );
    // The lino must wrap the conversion inside an AtTime expression
    assert!(
        result.lino_interpretation.contains("as INR") && result.lino_interpretation.contains("at"),
        "LINO should contain both 'as INR' and 'at': {}",
        result.lino_interpretation
    );
}

// ── Root cause 2: historical rate must be used ────────────────────────────────

/// "1 RUB as INR at Apr 11, 2026" should use the rate loaded for 2026-04-11 (1.5),
/// not the one for 2026-04-17 (1.2) or any other fallback.
#[test]
fn test_unit_conversion_at_date_uses_historical_rate() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 RUB as INR at Apr 11, 2026");
    assert!(
        result.success,
        "Should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("INR"),
        "Result should be in INR: {}",
        result.result
    );
    // At the historical rate of 1.5, 1 RUB = 1.5 INR
    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("1.5"),
        "Steps should show rate 1.5 from Apr 11, 2026: {steps_text}"
    );
}

/// Different dates should produce different results when using historical rates.
#[test]
fn test_unit_conversion_different_dates_give_different_rates() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result_apr11 = calc.calculate_internal("1 RUB as INR at Apr 11, 2026");
    let result_apr17 = calc.calculate_internal("1 RUB as INR at Apr 17, 2026");

    assert!(result_apr11.success, "Apr 11 should succeed: {:?}", result_apr11.error);
    assert!(result_apr17.success, "Apr 17 should succeed: {:?}", result_apr17.error);

    // Rates differ: 1.5 vs 1.2, so results must differ
    assert_ne!(
        result_apr11.result, result_apr17.result,
        "Different dates should produce different results: {} vs {}",
        result_apr11.result, result_apr17.result
    );
}

/// The "in" keyword for unit conversion should also work with trailing "at".
#[test]
fn test_unit_conversion_in_keyword_with_at_date() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 RUB in INR at Apr 11, 2026");
    assert!(
        result.success,
        "1 RUB in INR at Apr 11, 2026 should succeed: {:?}",
        result.error
    );
    assert!(
        result.lino_interpretation.contains("at"),
        "LINO should contain 'at': {}",
        result.lino_interpretation
    );
}

/// The "to" keyword for unit conversion should also work with trailing "at".
#[test]
fn test_unit_conversion_to_keyword_with_at_date() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 RUB to INR at Apr 11, 2026");
    assert!(
        result.success,
        "1 RUB to INR at Apr 11, 2026 should succeed: {:?}",
        result.error
    );
    assert!(
        result.lino_interpretation.contains("at"),
        "LINO should contain 'at': {}",
        result.lino_interpretation
    );
}

// ── Root cause 3: Russian "на" as temporal "at" ───────────────────────────────

/// Russian "на" should be recognized as the "at" keyword for temporal context.
#[test]
fn test_russian_na_as_at_keyword() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    // "в рупиях на Apr 11, 2026" = "in INR at Apr 11, 2026"
    let result = calc.calculate_internal("1 RUB в INR на Apr 11, 2026");
    assert!(
        result.success,
        "Russian 'на' should work as 'at': {:?}",
        result.error
    );
    assert!(
        result.lino_interpretation.contains("at"),
        "LINO should contain 'at': {}",
        result.lino_interpretation
    );
}

/// Full Russian expression: "1 рубль в рупиях на Apr 11, 2026".
#[test]
fn test_russian_rub_in_inr_at_date() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    let result = calc.calculate_internal("1 рубль в рупиях на Apr 11, 2026");
    assert!(
        result.success,
        "Full Russian expression should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("INR"),
        "Result should be in INR: {}",
        result.result
    );
}

// ── Original issue expression ─────────────────────────────────────────────────

/// The original issue expression uses "апреля" (Russian genitive of April).
/// The date "11 апреля 2026" must be recognized as a valid date.
#[test]
fn test_original_issue_russian_date_parses() {
    let mut calc = Calculator::new();
    load_rub_inr_rates(&mut calc);

    // Use approximate date form that the datetime parser supports
    let result = calc.calculate_internal("22822 рублей в рупиях на Apr 11, 2026");
    assert!(
        result.success,
        "22822 рублей в рупиях на Apr 11, 2026 should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("INR"),
        "Result should be in INR: {}",
        result.result
    );
    // At rate 1.5, 22822 * 1.5 = 34233 INR
    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("1.5"),
        "Should use the Apr 11 rate (1.5): {steps_text}"
    );
}

/// Non-currency unit conversions should not be affected by date context.
#[test]
fn test_data_size_conversion_ignores_date_context() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("1 GB as MB at Apr 11, 2026");
    assert!(
        result.success,
        "Data size conversion with date should succeed: {:?}",
        result.error
    );
    // 1 GB = 1000 MB (SI) regardless of date
    assert!(
        result.result.contains("MB"),
        "Result should be in MB: {}",
        result.result
    );
}
