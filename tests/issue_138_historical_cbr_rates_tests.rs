//! Tests for issue #138: historical CBR rates must be used for dated RUB conversions.
//!
//! The browser CBR fallback already downloads `.lino` files with historical rows, but it
//! previously applied only the latest row as a current rate. That allowed a dated
//! expression such as `22822 рублей в рупиях на 11 апреля 2026` to use the
//! 2026-04-24 rate instead of the requested 2026-04-11 rate.

use link_calculator::Calculator;

fn parse_inr_result(result: &str) -> f64 {
    result
        .replace(" INR", "")
        .replace(',', "")
        .trim()
        .parse()
        .expect("result should parse as INR amount")
}

#[test]
fn current_cbr_rate_is_not_used_for_a_different_historical_date() {
    let mut calc = Calculator::new();
    calc.update_cbr_rates_from_api("2026-04-24", r#"{"inr": 0.7954370000000001}"#);

    let result = calc.calculate_internal("1 RUB as INR at Apr 11, 2026");

    assert!(
        !result.success,
        "A dated conversion should not silently use the current/latest rate: {:?}",
        result.steps
    );
    let error = result.error.unwrap_or_default();
    assert!(
        error.contains("No exchange rate available"),
        "Expected a missing historical rate error, got: {error}"
    );
}

#[test]
fn current_cbr_rate_is_available_for_its_own_effective_date() {
    let mut calc = Calculator::new();
    calc.update_cbr_rates_from_api("2026-04-24", r#"{"inr": 0.7954370000000001}"#);

    let result = calc.calculate_internal("1 RUB as INR at Apr 24, 2026");

    assert!(
        result.success,
        "The API response date should be available as a historical rate: {:?}",
        result.error
    );
    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("date: 2026-04-24"),
        "Steps should show the API response date. Steps:\n{steps_text}"
    );
}

#[test]
fn original_issue_expression_uses_april_11_cbr_lino_rate() {
    let mut calc = Calculator::new();
    calc.update_cbr_rates_from_api("2026-04-24", r#"{"inr": 0.7954370000000001}"#);
    calc.load_rates_from_consolidated_lino(include_str!("../data/currency/inr-rub.lino"))
        .expect("CBR INR/RUB .lino data should load");

    let result = calc.calculate_internal("22822 рублей в рупиях на 11 апреля 2026");

    assert!(
        result.success,
        "Original issue expression should succeed: {:?}",
        result.error
    );

    let steps_text = result.steps.join("\n");
    assert!(
        steps_text.contains("date: 2026-04-11"),
        "Steps should show the requested historical date. Steps:\n{steps_text}"
    );
    assert!(
        !steps_text.contains("date: 2026-04-24"),
        "Steps should not show the latest CBR rate date. Steps:\n{steps_text}"
    );

    let actual = parse_inr_result(&result.result);
    let expected = 22_822.0 * 1.203_667_816_570_654;
    assert!(
        (actual - expected).abs() < 0.000_001,
        "Expected {expected} INR from the 2026-04-11 rate, got {actual}. Steps:\n{steps_text}"
    );
}
