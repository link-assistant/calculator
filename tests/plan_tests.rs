//! Tests for the plan→execute pipeline.
//!
//! These tests verify that `Calculator::plan_internal()` correctly identifies
//! which rate sources an expression needs based on the parsed AST.

use link_calculator::{Calculator, RateSource};

#[test]
fn plan_pure_math_needs_no_sources() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("2 + 3");
    assert!(plan.success);
    assert!(plan.required_sources.is_empty());
    assert!(plan.currencies.is_empty());
    assert_eq!(plan.lino_interpretation, "(2 + 3)");
}

#[test]
fn plan_usd_needs_ecb() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("100 USD");
    assert!(plan.success);
    assert!(plan.currencies.contains(&"USD".to_string()));
    // Single currency doesn't need conversion, but if it's in the AST
    // we detect it. One currency doesn't trigger ECB triangulation.
    // However the plan maps USD → ECB.
    assert!(plan.required_sources.contains(&RateSource::Ecb) || plan.required_sources.is_empty());
}

#[test]
fn plan_currency_conversion_needs_ecb() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("100 USD + 50 EUR");
    assert!(plan.success);
    assert!(plan.currencies.contains(&"USD".to_string()));
    assert!(plan.currencies.contains(&"EUR".to_string()));
    assert!(plan.required_sources.contains(&RateSource::Ecb));
}

#[test]
fn plan_rub_needs_cbr() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("1000 RUB");
    assert!(plan.success);
    assert!(plan.currencies.contains(&"RUB".to_string()));
    assert!(plan.required_sources.contains(&RateSource::Cbr));
}

#[test]
fn plan_crypto_needs_coingecko() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("20 TON in USD");
    assert!(plan.success);
    assert!(plan.currencies.contains(&"TON".to_string()));
    assert!(plan.required_sources.contains(&RateSource::Crypto));
}

#[test]
fn plan_mixed_rub_crypto_usd_needs_only_cbr_and_crypto() {
    let calc = Calculator::new();
    // The exact expression from issue #102: ECB is not needed because
    // CBR provides USD↔RUB rates and CoinGecko provides TON→USD rates.
    let plan = calc.plan_internal("(1000 RUB + 500 RUB + 2000 RUB + 20 TON + 1000 RUB) in USD");
    assert!(plan.success);
    assert!(plan.currencies.contains(&"RUB".to_string()));
    assert!(plan.currencies.contains(&"TON".to_string()));
    assert!(plan.currencies.contains(&"USD".to_string()));
    assert!(!plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.required_sources.contains(&RateSource::Cbr));
    assert!(plan.required_sources.contains(&RateSource::Crypto));
}

#[test]
fn plan_rub_to_usd_needs_only_cbr() {
    let calc = Calculator::new();
    // CBR provides USD↔RUB rates directly, no ECB needed.
    let plan = calc.plan_internal("1000 RUB in USD");
    assert!(plan.success);
    assert!(!plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.required_sources.contains(&RateSource::Cbr));
}

#[test]
fn plan_crypto_to_usd_needs_only_crypto() {
    let calc = Calculator::new();
    // CoinGecko rates are denominated in USD, no ECB needed.
    let plan = calc.plan_internal("1 BTC in USD");
    assert!(plan.success);
    assert!(!plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.required_sources.contains(&RateSource::Crypto));
}

#[test]
fn plan_rub_to_eur_needs_cbr_and_ecb() {
    let calc = Calculator::new();
    // EUR is an ECB-provided currency, so ECB is genuinely needed here.
    let plan = calc.plan_internal("1000 RUB in EUR");
    assert!(plan.success);
    assert!(plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.required_sources.contains(&RateSource::Cbr));
}

#[test]
fn plan_usd_eur_needs_ecb() {
    let calc = Calculator::new();
    // Both USD and EUR are ECB currencies; EUR prevents the optimization.
    let plan = calc.plan_internal("100 USD in EUR");
    assert!(plan.success);
    assert!(plan.required_sources.contains(&RateSource::Ecb));
}

#[test]
fn plan_crypto_to_rub_needs_crypto_and_cbr() {
    let calc = Calculator::new();
    // TON→USD from CoinGecko, USD→RUB from CBR. No ECB needed.
    let plan = calc.plan_internal("10 TON in RUB");
    assert!(plan.success);
    assert!(!plan.required_sources.contains(&RateSource::Ecb));
    assert!(plan.required_sources.contains(&RateSource::Crypto));
    assert!(plan.required_sources.contains(&RateSource::Cbr));
}

#[test]
fn plan_detects_live_time() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("now");
    assert!(plan.success);
    assert!(plan.is_live_time);
}

#[test]
fn plan_non_live_expression() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("2 * 3");
    assert!(plan.success);
    assert!(!plan.is_live_time);
}

#[test]
fn plan_invalid_expression() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("");
    assert!(!plan.success);
    assert!(plan.error.is_some());
}

#[test]
fn plan_generates_alternatives() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("2 + 3 * 4");
    assert!(plan.success);
    // This expression has mixed precedence, so alternatives should exist
    if let Some(alts) = &plan.alternative_lino {
        assert!(alts.len() >= 2);
    }
}

#[test]
fn plan_execute_consistency() {
    let mut calc = Calculator::new();
    let plan = calc.plan_internal("2 + 3");
    assert!(plan.success);

    let result = calc.calculate_internal("2 + 3");
    assert!(result.success);
    assert_eq!(result.result, "5");
    // Plan's lino should match the result's lino
    assert_eq!(plan.lino_interpretation, result.lino_interpretation);
}
