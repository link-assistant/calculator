//! Calculation planning — determines data requirements without executing.
//!
//! The plan→execute pipeline:
//! 1. `Calculator::plan(expr)` parses the AST and returns a `CalculationPlan`
//!    with required rate sources, LINO interpretation, and alternatives.
//! 2. The worker fetches only the required rate sources.
//! 3. `Calculator::execute(expr)` runs the calculation with rates available.

use crate::crypto_api;
use crate::types::Expression;

/// Rate sources that the calculator can fetch data from.
///
/// These correspond to the three independent APIs the web worker integrates with:
/// - `ecb` — Fiat currency rates via Frankfurter API (European Central Bank data)
/// - `cbr` — RUB-based rates from the Central Bank of Russia
/// - `crypto` — Cryptocurrency rates via CoinGecko
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateSource {
    /// European Central Bank (fiat currencies: USD, EUR, GBP, JPY, …).
    Ecb,
    /// Central Bank of Russia (RUB-based rates).
    Cbr,
    /// CoinGecko (cryptocurrencies: BTC, ETH, TON, …).
    Crypto,
}

/// A calculation plan produced by `Calculator::plan()`.
///
/// Contains everything the worker needs to know *before* executing the
/// calculation: which rate sources to fetch, how the expression was
/// interpreted, and what alternatives exist.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationPlan {
    /// The input expression, trimmed.
    pub expression: String,
    /// Links notation interpretation of the expression (default interpretation).
    pub lino_interpretation: String,
    /// Alternative links notation interpretations, if any.
    /// The first element is always the default (same as `lino_interpretation`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_lino: Option<Vec<String>>,
    /// Rate sources that must be loaded before this expression can be executed.
    /// Empty for pure math expressions.
    pub required_sources: Vec<RateSource>,
    /// Currency codes found in the expression (e.g., `["USD", "RUB", "TON"]`).
    pub currencies: Vec<String>,
    /// Whether the expression contains a live time reference (auto-refresh needed).
    pub is_live_time: bool,
    /// Whether the expression was parsed successfully.
    pub success: bool,
    /// Error message if parsing failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Maps a currency code to the rate source(s) that provide its exchange rate.
fn currency_to_sources(code: &str) -> Vec<RateSource> {
    let upper = code.to_uppercase();

    // Check if it's a cryptocurrency (known to CoinGecko)
    if crypto_api::coingecko_id(&upper).is_some() {
        return vec![RateSource::Crypto];
    }

    // RUB needs CBR rates
    if upper == "RUB" {
        return vec![RateSource::Cbr];
    }

    // All other currencies are fiat, served by ECB/Frankfurter
    vec![RateSource::Ecb]
}

/// Creates a `CalculationPlan` from a parsed expression.
///
/// Walks the AST to collect currencies, maps them to rate sources,
/// generates LINO interpretation, and detects live time references.
pub fn create_plan(input: &str, expr: &Expression) -> CalculationPlan {
    let lino = expr.to_lino();
    let alternatives = expr.alternative_lino();
    let is_live_time = expr.contains_live_time();

    // Collect currencies from the AST and map to rate sources
    let currencies_set = expr.collect_currencies();
    let mut currencies: Vec<String> = currencies_set.iter().cloned().collect();
    currencies.sort();

    let mut sources_set = std::collections::HashSet::new();
    for code in &currencies {
        for source in currency_to_sources(code) {
            sources_set.insert(source);
        }
    }

    // If we have any multi-currency expression, add ECB for triangulation
    if currencies.len() >= 2 && !sources_set.is_empty() {
        sources_set.insert(RateSource::Ecb);
    }

    let mut required_sources: Vec<RateSource> = sources_set.into_iter().collect();
    required_sources.sort_by_key(|s| match s {
        RateSource::Ecb => 0,
        RateSource::Cbr => 1,
        RateSource::Crypto => 2,
    });

    CalculationPlan {
        expression: input.to_string(),
        lino_interpretation: lino,
        alternative_lino: alternatives,
        required_sources,
        currencies,
        is_live_time,
        success: true,
        error: None,
    }
}

/// Creates a failed `CalculationPlan` for an empty input.
pub fn empty_plan() -> CalculationPlan {
    CalculationPlan {
        expression: String::new(),
        lino_interpretation: String::new(),
        alternative_lino: None,
        required_sources: Vec::new(),
        currencies: Vec::new(),
        is_live_time: false,
        success: false,
        error: Some("Empty input".to_string()),
    }
}

/// Creates a failed `CalculationPlan` for a parse error.
pub fn error_plan(input: &str, error: &str) -> CalculationPlan {
    CalculationPlan {
        expression: input.to_string(),
        lino_interpretation: String::new(),
        alternative_lino: None,
        required_sources: Vec::new(),
        currencies: Vec::new(),
        is_live_time: false,
        success: false,
        error: Some(error.to_string()),
    }
}
