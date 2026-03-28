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

/// Maps a currency code to the rate source that is its primary provider.
fn primary_source(code: &str) -> RateSource {
    let upper = code.to_uppercase();

    // Cryptocurrencies are provided by CoinGecko
    if crypto_api::coingecko_id(&upper).is_some() {
        return RateSource::Crypto;
    }

    // RUB is provided by CBR
    if upper == "RUB" {
        return RateSource::Cbr;
    }

    // Currencies available from CBR but not from ECB/Frankfurter
    if matches!(
        upper.as_str(),
        "VND"
            | "AZN"
            | "DZD"
            | "AMD"
            | "BHD"
            | "BYN"
            | "BOB"
            | "GEL"
            | "AED"
            | "EGP"
            | "IRR"
            | "KGS"
            | "CUP"
            | "MDL"
            | "MNT"
            | "NGN"
            | "OMR"
            | "QAR"
            | "SAR"
            | "TJS"
            | "BDT"
            | "TMT"
            | "UZS"
            | "UAH"
            | "ETB"
            | "RSD"
            | "MMK"
    ) {
        return RateSource::Cbr;
    }

    // All other fiat currencies are provided by ECB/Frankfurter
    RateSource::Ecb
}

/// Returns true if a source can also provide rates for a currency, even though
/// it is not the primary source for that currency.
///
/// This captures the real-world overlap between rate providers. The key
/// constraint is the runtime converter, which only triangulates through USD
/// as a bridge currency. A source can "also serve" a currency only when the
/// converter can actually use the rates it provides for that currency.
///
/// Coverage:
/// - CBR publishes rates for ~50 currencies against RUB, including USD.
///   Since the converter bridges through USD, CBR's USD↔RUB rate lets it
///   serve USD for any RUB-involving conversion. CBR also serves other fiat
///   currencies (EUR, GBP, etc.) for direct conversion to/from RUB.
/// - CoinGecko crypto rates are denominated in USD, so Crypto covers USD.
/// - ECB provides rates for major fiat currencies relative to EUR/USD.
fn can_also_serve(source: RateSource, code: &str, all_currencies: &[String]) -> bool {
    let upper = code.to_uppercase();
    match source {
        // CBR provides X↔RUB rates for many currencies. It can serve a fiat
        // currency if the expression involves RUB (which it must, since CBR
        // is only in the set when RUB is present). For direct RUB↔X conversion,
        // CBR has the rate. For USD specifically, CBR provides USD↔RUB which
        // the converter can use directly.
        //
        // However, CBR cannot bridge between two non-RUB fiat currencies
        // (e.g., GBP→EUR) because the converter triangulates through USD,
        // not RUB. So CBR can only serve a non-RUB fiat currency when it's
        // being converted directly to/from RUB — which happens when all
        // other currencies in the expression are either RUB or served by
        // another source.
        RateSource::Cbr => {
            // CBR always covers USD (via USD↔RUB rate)
            if upper == "USD" {
                return true;
            }
            // CBR can serve other fiat currencies for direct RUB conversion,
            // but only when all fiat currencies in the expression either are
            // RUB or are the single target fiat currency (so conversion is
            // always fiat↔RUB, never fiat↔fiat through CBR).
            let cbr_fiat = matches!(
                upper.as_str(),
                "EUR"
                    | "GBP"
                    | "JPY"
                    | "CHF"
                    | "CNY"
                    | "INR"
                    | "AUD"
                    | "CAD"
                    | "SGD"
                    | "HKD"
                    | "DKK"
                    | "SEK"
                    | "NOK"
                    | "CZK"
                    | "PLN"
                    | "HUF"
                    | "RON"
                    | "BGN"
                    | "TRY"
                    | "KRW"
                    | "BRL"
                    | "MXN"
                    | "ZAR"
                    | "IDR"
                    | "MYR"
                    | "PHP"
                    | "THB"
                    | "VND"
                    | "AZN"
                    | "DZD"
                    | "AMD"
                    | "BHD"
                    | "BYN"
                    | "BOB"
                    | "GEL"
                    | "AED"
                    | "EGP"
                    | "IRR"
                    | "KGS"
                    | "CUP"
                    | "MDL"
                    | "MNT"
                    | "NGN"
                    | "OMR"
                    | "QAR"
                    | "SAR"
                    | "TJS"
                    | "BDT"
                    | "TMT"
                    | "UZS"
                    | "UAH"
                    | "ETB"
                    | "RSD"
                    | "MMK"
            );
            if !cbr_fiat {
                return false;
            }
            // Count how many distinct ECB-primary fiat currencies are in the expression.
            // If there's only one (the one we're checking), CBR can handle the
            // direct RUB↔fiat conversion. If there are multiple, CBR can't
            // convert between them (e.g., GBP↔EUR requires ECB).
            let ecb_fiat_count = all_currencies
                .iter()
                .filter(|c| {
                    let u = c.to_uppercase();
                    u != "RUB" && crypto_api::coingecko_id(&u).is_none() && u != "USD"
                })
                .count();
            ecb_fiat_count <= 1
        }
        // CoinGecko rates are denominated in USD, so if Crypto is already
        // required, USD conversion is available without ECB.
        RateSource::Crypto => upper == "USD",
        // ECB provides rates for major fiat currencies (but not RUB or crypto).
        RateSource::Ecb => {
            !matches!(upper.as_str(), "RUB") && crypto_api::coingecko_id(&upper).is_none()
        }
    }
}

/// Creates a `CalculationPlan` from a parsed expression.
///
/// Walks the AST to collect currencies, maps them to rate sources,
/// generates LINO interpretation, and detects live time references.
pub fn create_plan(input: &str, expr: &Expression) -> CalculationPlan {
    let lino = expr.to_lino();
    let alternatives = expr.alternative_lino();
    let is_live_time = expr.contains_live_time() || expr.evaluates_to_datetime();

    // Collect currencies from the AST and map to rate sources
    let currencies_set = expr.collect_currencies();
    let mut currencies: Vec<String> = currencies_set.iter().cloned().collect();
    currencies.sort();

    // Map each currency to its primary source
    let mut sources_set = std::collections::HashSet::new();
    let mut source_currencies: std::collections::HashMap<RateSource, Vec<String>> =
        std::collections::HashMap::new();
    for code in &currencies {
        let source = primary_source(code);
        sources_set.insert(source);
        source_currencies
            .entry(source)
            .or_default()
            .push(code.to_uppercase());
    }

    // Optimization: remove a source if every currency that triggered its
    // inclusion is also covered by another source already in the set.
    // This generalizes for any target currency — if the expression converts
    // to a final currency, the remaining sources must form a path to it.
    let candidates: Vec<RateSource> = sources_set.iter().copied().collect();
    for source in &candidates {
        if let Some(codes) = source_currencies.get(source) {
            let other_sources: Vec<RateSource> =
                candidates.iter().copied().filter(|s| s != source).collect();
            // Check if every currency for this source is covered by some other source
            let all_covered = codes.iter().all(|code| {
                other_sources
                    .iter()
                    .any(|other| can_also_serve(*other, code, &currencies))
            });
            if all_covered && !other_sources.is_empty() {
                sources_set.remove(source);
            }
        }
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
