//! Link Calculator - A grammar-based expression parser and calculator.
//!
//! This library provides a WebAssembly-compatible calculator that supports:
//! - `DateTime` parsing and arithmetic
//! - Decimal numbers with units
//! - Currency conversions with temporal awareness
//! - Links notation for expression representation
//!
//! # Example
//!
//! ```
//! use link_calculator::Calculator;
//!
//! let mut calculator = Calculator::new();
//! let result = calculator.calculate_internal("2 + 3");
//! assert!(result.success);
//! assert_eq!(result.result, "5");
//! ```

#![allow(clippy::module_inception)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::use_self)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::if_not_else)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::format_push_string)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::match_same_arms)]

pub mod crypto_api;
pub mod currency_api;
pub mod error;
pub mod grammar;
pub mod lino;
pub mod types;
pub mod wasm;

use error::{CalculatorError, ErrorInfo};
use grammar::ExpressionParser;
use types::{Expression, Value, ValueKind};
use wasm_bindgen::prelude::*;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Data for plotting a function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlotData {
    /// X-axis values.
    pub x_values: Vec<f64>,
    /// Y-axis values.
    pub y_values: Vec<f64>,
    /// Label for the plot (e.g., "sin(x)/x").
    pub label: String,
    /// X-axis label.
    pub x_label: String,
    /// Y-axis label.
    pub y_label: String,
}

/// A single calculation step with i18n support.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationStep {
    /// The translation key for this step type.
    pub key: String,
    /// Parameters for interpolation in the translated message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<std::collections::HashMap<String, String>>,
    /// The raw (English) text for fallback.
    pub text: String,
}

impl CalculationStep {
    /// Creates a new step with a translation key, params, and fallback text.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        params: Option<std::collections::HashMap<String, String>>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            params,
            text: text.into(),
        }
    }

    /// Creates a simple step with just text (no translation key).
    #[must_use]
    pub fn text_only(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            key: String::new(),
            params: None,
            text,
        }
    }
}

/// Repeating decimal notation formats.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepeatingDecimalFormats {
    /// Vinculum notation with overline: 0.3̅
    pub vinculum: String,
    /// Parenthesis notation: 0.(3)
    pub parenthesis: String,
    /// Ellipsis notation: 0.333...
    pub ellipsis: String,
    /// LaTeX notation: 0.\overline{3}
    pub latex: String,
    /// Fraction representation: 1/3
    pub fraction: String,
}

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

/// Result of a calculation operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationResult {
    /// The computed value as a string.
    pub result: String,
    /// The input interpreted in links notation format.
    pub lino_interpretation: String,
    /// Alternative links notation interpretations the user can switch between.
    /// The first element is always the currently selected (default) interpretation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_lino: Option<Vec<String>>,
    /// Step-by-step explanation of the calculation (raw text for backwards compatibility).
    pub steps: Vec<String>,
    /// Step-by-step explanation with i18n support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps_i18n: Option<Vec<CalculationStep>>,
    /// Whether the calculation was successful.
    pub success: bool,
    /// Error message if calculation failed (raw text for backwards compatibility).
    pub error: Option<String>,
    /// Error information for i18n support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_info: Option<ErrorInfo>,
    /// Link to create an issue for unrecognized input.
    pub issue_link: Option<String>,
    /// LaTeX representation of the input (for rendering mathematical formulas).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex_input: Option<String>,
    /// LaTeX representation of the result (for rendering mathematical formulas).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex_result: Option<String>,
    /// Whether this is a symbolic result (e.g., indefinite integral).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symbolic: Option<bool>,
    /// Plot data points for graphing (x, y pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot_data: Option<PlotData>,
    /// Repeating decimal notations (if the result is a repeating decimal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeating_decimal: Option<RepeatingDecimalFormats>,
    /// Fraction representation of the result (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fraction: Option<String>,
    /// Whether the result represents a live (auto-updating) time expression.
    /// When `true`, the frontend should periodically re-calculate the expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_live_time: Option<bool>,
}

impl CalculationResult {
    /// Creates a successful calculation result.
    #[must_use]
    pub fn success(result: String, lino: String, steps: Vec<String>) -> Self {
        Self {
            result,
            lino_interpretation: lino,
            alternative_lino: None,
            steps,
            steps_i18n: None,
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }

    /// Creates a successful calculation result with rational value information.
    #[must_use]
    pub fn success_with_value(value: &Value, lino: String, steps: Vec<String>) -> Self {
        let result = value.to_display_string();

        // Extract repeating decimal and fraction info if available
        let (repeating_decimal, fraction) = if let Some(rational) = value.as_rational() {
            let fraction = if !rational.is_integer() {
                Some(rational.to_fraction_string())
            } else {
                None
            };

            let repeating =
                rational
                    .to_repeating_decimal_notation()
                    .map(|rd| RepeatingDecimalFormats {
                        vinculum: rd.to_vinculum_notation(),
                        parenthesis: rd.to_parenthesis_notation(),
                        ellipsis: rd.to_ellipsis_notation(),
                        latex: rd.to_latex(),
                        fraction: rational.to_fraction_string(),
                    });

            (repeating, fraction)
        } else {
            (None, None)
        };

        Self {
            result,
            lino_interpretation: lino,
            alternative_lino: None,
            steps,
            steps_i18n: None,
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal,
            fraction,
            is_live_time: None,
        }
    }

    /// Creates a successful calculation result with i18n step info.
    #[must_use]
    pub fn success_with_i18n(
        result: String,
        lino: String,
        steps: Vec<String>,
        steps_i18n: Vec<CalculationStep>,
    ) -> Self {
        Self {
            result,
            lino_interpretation: lino,
            alternative_lino: None,
            steps,
            steps_i18n: Some(steps_i18n),
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }

    /// Creates a successful calculation result with LaTeX formatting.
    #[must_use]
    pub fn success_with_latex(
        result: String,
        lino: String,
        steps: Vec<String>,
        latex_input: Option<String>,
        latex_result: Option<String>,
    ) -> Self {
        Self {
            result,
            lino_interpretation: lino,
            alternative_lino: None,
            steps,
            steps_i18n: None,
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input,
            latex_result,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }

    /// Creates a symbolic result (e.g., for indefinite integrals).
    #[must_use]
    pub fn symbolic(
        expression: &str,
        result: String,
        latex_input: String,
        latex_result: String,
        plot_data: Option<PlotData>,
    ) -> Self {
        Self {
            result,
            lino_interpretation: expression.to_string(),
            alternative_lino: None,
            steps: vec![
                format!("Input: {}", expression),
                "Computed symbolic result".to_string(),
            ],
            steps_i18n: None,
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input: Some(latex_input),
            latex_result: Some(latex_result),
            is_symbolic: Some(true),
            plot_data,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }

    /// Creates a failed calculation result.
    #[must_use]
    pub fn failure(error: String, input: &str) -> Self {
        let issue_link = generate_issue_link(input, &error);
        Self {
            result: String::new(),
            lino_interpretation: String::new(),
            alternative_lino: None,
            steps: Vec::new(),
            steps_i18n: None,
            success: false,
            error: Some(error),
            error_info: None,
            issue_link: Some(issue_link),
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }

    /// Creates a failed calculation result with i18n error info.
    #[must_use]
    pub fn failure_with_i18n(error: &CalculatorError, input: &str) -> Self {
        let error_string = error.to_string();
        let issue_link = generate_issue_link(input, &error_string);
        Self {
            result: String::new(),
            lino_interpretation: String::new(),
            alternative_lino: None,
            steps: Vec::new(),
            steps_i18n: None,
            success: false,
            error: Some(error_string),
            error_info: Some(error.to_error_info()),
            issue_link: Some(issue_link),
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
        }
    }
}

/// Generates a GitHub issue link for unrecognized input.
fn generate_issue_link(input: &str, error: &str) -> String {
    let title = format!("Unrecognized input: {}", truncate(input, 50));
    let body = format!(
        "## Input that failed to parse\n\n```\n{}\n```\n\n## Error message\n\n```\n{}\n```\n\n## Expected behavior\n\nPlease describe what you expected the calculator to do with this input.",
        input, error
    );
    format!(
        "https://github.com/link-assistant/calculator/issues/new?title={}&body={}",
        urlencoding_encode(&title),
        urlencoding_encode(&body)
    )
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            '\n' => result.push_str("%0A"),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    result
}

/// The main calculator struct.
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Calculator {
    parser: ExpressionParser,
}

#[wasm_bindgen]
impl Calculator {
    /// Creates a new Calculator instance.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        // Initialize panic hook for better error messages in WASM
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        Self {
            parser: ExpressionParser::new(),
        }
    }

    /// Plans a calculation without executing it, returning a JSON string.
    ///
    /// Parses the expression to determine:
    /// - Links notation interpretation (default and alternatives)
    /// - Which rate sources are required (ECB, CBR, Crypto)
    /// - Which currency codes are referenced
    /// - Whether the expression contains live time references
    ///
    /// The worker should call this first, fetch the required rate sources,
    /// then call `execute()` to get the actual result.
    #[wasm_bindgen]
    pub fn plan(&self, input: &str) -> String {
        let plan = self.plan_internal(input);
        serde_json::to_string(&plan).unwrap_or_else(|e| {
            format!(
                r#"{{"success":false,"error":"Serialization error: {}"}}"#,
                e
            )
        })
    }

    /// Executes a calculation, returning a JSON string with the full result.
    ///
    /// This is the same as `calculate()` but named to clarify the plan→execute pipeline.
    /// The worker should call `plan()` first to determine required rate sources,
    /// fetch them, then call `execute()`.
    #[wasm_bindgen]
    pub fn execute(&mut self, input: &str) -> String {
        let result = self.calculate_internal(input);
        serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(
                r#"{{"success":false,"error":"Serialization error: {}"}}"#,
                e
            )
        })
    }

    /// Calculates the result of an expression, returning a JSON string.
    ///
    /// Kept for backwards compatibility. Equivalent to `execute()`.
    #[wasm_bindgen]
    pub fn calculate(&mut self, input: &str) -> String {
        self.execute(input)
    }

    /// Returns the version of the calculator.
    #[wasm_bindgen]
    #[must_use]
    pub fn version() -> String {
        VERSION.to_string()
    }

    /// Updates exchange rates from API response. Returns the number of rates updated.
    /// Args: `base` (e.g., "USD"), `date` (e.g., "2026-01-25"), `rates_json` (e.g., `{"eur": 0.92}`).
    #[wasm_bindgen]
    pub fn update_rates_from_api(&mut self, base: &str, date: &str, rates_json: &str) -> usize {
        let rates: std::collections::HashMap<String, f64> = match serde_json::from_str(rates_json) {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let base_upper = base.to_uppercase();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut count = 0;

        for (target, rate) in rates {
            let target_upper = target.to_uppercase();

            if base_upper == target_upper {
                continue;
            } // Skip same currency

            let info = types::ExchangeRateInfo::new(rate, currency_api::API_SOURCE, date)
                .with_fetched_at(&timestamp);

            self.parser
                .currency_db_mut()
                .set_rate_with_info(&base_upper, &target_upper, info);

            count += 1;
        }

        count
    }

    /// Updates RUB exchange rates from the Central Bank of Russia (cbr.ru) API response.
    /// Returns the number of rates updated.
    ///
    /// The CBR rates format: `{"usd": 76.63, "eur": 90.58, "inr": 0.842, ...}`
    /// where each value represents "1 CURRENCY = X RUB".
    ///
    /// These rates take priority over ECB/Frankfurter rates for RUB conversions,
    /// since CBR provides official RUB rates directly (no cross-rate needed).
    ///
    /// Args: `date` (e.g., "2026-02-25"), `rates_json` (currency_code → RUB amount).
    #[wasm_bindgen]
    pub fn update_cbr_rates_from_api(&mut self, date: &str, rates_json: &str) -> usize {
        let rates: std::collections::HashMap<String, f64> = match serde_json::from_str(rates_json) {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut count = 0;

        for (currency, rub_per_unit) in rates {
            let currency_upper = currency.to_uppercase();

            // Skip RUB itself
            if currency_upper == "RUB" {
                continue;
            }

            // Store: 1 CURRENCY = rub_per_unit RUB
            let info =
                types::ExchangeRateInfo::new(rub_per_unit, currency_api::CBR_API_SOURCE, date)
                    .with_fetched_at(&timestamp);

            self.parser
                .currency_db_mut()
                .set_rate_with_info(&currency_upper, "RUB", info);

            count += 1;
        }

        count
    }

    /// Updates cryptocurrency exchange rates from API response.
    /// Returns the number of rates updated.
    ///
    /// Args: `base` (fiat currency, e.g., "USD"), `date` (e.g., "2026-01-25"),
    /// `rates_json` (e.g., `{"TON": 5.42, "BTC": 95000.0}`).
    #[wasm_bindgen]
    pub fn update_crypto_rates_from_api(
        &mut self,
        base: &str,
        date: &str,
        rates_json: &str,
    ) -> usize {
        let rates: std::collections::HashMap<String, f64> = match serde_json::from_str(rates_json) {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let base_upper = base.to_uppercase();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut count = 0;

        for (ticker, price) in rates {
            let ticker_upper = ticker.to_uppercase();
            // Store rate as: 1 ticker = price base_currency
            let info = types::ExchangeRateInfo::new(price, crypto_api::COINGECKO_SOURCE, date)
                .with_fetched_at(&timestamp);

            self.parser
                .currency_db_mut()
                .set_rate_with_info(&ticker_upper, &base_upper, info);

            count += 1;
        }

        count
    }
}

impl Calculator {
    /// Internal planning method — parses expression and determines requirements.
    pub fn plan_internal(&self, input: &str) -> CalculationPlan {
        let input = input.trim();
        if input.is_empty() {
            return CalculationPlan {
                expression: String::new(),
                lino_interpretation: String::new(),
                alternative_lino: None,
                required_sources: Vec::new(),
                currencies: Vec::new(),
                is_live_time: false,
                success: false,
                error: Some("Empty input".to_string()),
            };
        }

        match self.parser.parse(input) {
            Ok(expr) => {
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

                // If we have any non-ECB fiat currency conversion that involves
                // a currency *and* a different currency, we likely need ECB for
                // triangulation. For example, TON→USD needs crypto + ecb.
                if currencies.len() >= 2 && !sources_set.is_empty() {
                    // Any multi-currency expression benefits from ECB as base
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
            Err(e) => CalculationPlan {
                expression: input.to_string(),
                lino_interpretation: String::new(),
                alternative_lino: None,
                required_sources: Vec::new(),
                currencies: Vec::new(),
                is_live_time: false,
                success: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// Internal calculation method that returns a proper Result type.
    pub fn calculate_internal(&mut self, input: &str) -> CalculationResult {
        // Try to parse the expression to generate alternative interpretations
        // and detect live time expressions before evaluation.
        let parsed_expr = self.parser.parse(input).ok();
        let alternatives = parsed_expr.as_ref().and_then(Expression::alternative_lino);
        let is_live_time = parsed_expr
            .as_ref()
            .is_some_and(Expression::contains_live_time);

        let mut result = match self.parser.parse_and_evaluate(input) {
            Ok((value, steps, lino)) => {
                let mut r = CalculationResult::success_with_value(&value, lino, steps);
                // Also check if the value itself is a live time datetime
                let value_is_live =
                    matches!(&value.kind, ValueKind::DateTime(dt) if dt.is_live_time());
                if is_live_time || value_is_live {
                    r.is_live_time = Some(true);
                }
                r
            }
            Err(CalculatorError::SymbolicResult {
                expression,
                result,
                latex_input,
                latex_result,
            }) => {
                // Generate plot data for the integrand function
                let plot_data = self.generate_plot_data_for_integral(input);
                CalculationResult::symbolic(
                    &expression,
                    result,
                    latex_input,
                    latex_result,
                    plot_data,
                )
            }
            Err(e) => CalculationResult::failure_with_i18n(&e, input),
        };

        // Attach alternative interpretations if available
        result.alternative_lino = alternatives;

        result
    }

    /// Generates plot data for an integral expression.
    fn generate_plot_data_for_integral(&mut self, input: &str) -> Option<PlotData> {
        // Try to parse and extract the integrand for plotting
        let expr = self.parser.parse(input).ok()?;

        if let types::Expression::IndefiniteIntegral {
            integrand,
            variable,
        } = expr
        {
            // Generate plot points for the integrand
            let mut x_values = Vec::new();
            let mut y_values = Vec::new();

            // Generate points from -10 to 10 with 200 steps
            let num_points: i32 = 200;
            let x_min = -10.0;
            let x_max = 10.0;
            let step = (x_max - x_min) / f64::from(num_points);

            for i in 0..=num_points {
                let x = f64::from(i).mul_add(step, x_min);

                // Skip x = 0 for functions like sin(x)/x to avoid division issues
                if x.abs() < 1e-10 {
                    // For sin(x)/x, the limit at x=0 is 1
                    x_values.push(x);
                    y_values.push(1.0);
                    continue;
                }

                // Try to evaluate the integrand at this point
                if let Ok(y_val) = self.evaluate_at_point(&integrand, &variable, x) {
                    if y_val.is_finite() {
                        x_values.push(x);
                        y_values.push(y_val);
                    }
                }
            }

            if !x_values.is_empty() {
                return Some(PlotData {
                    x_values,
                    y_values,
                    label: format!("{}", integrand),
                    x_label: variable.clone(),
                    y_label: format!("f({})", variable),
                });
            }
        }

        None
    }

    /// Evaluates an expression at a specific point.
    fn evaluate_at_point(
        &mut self,
        expr: &types::Expression,
        var: &str,
        value: f64,
    ) -> Result<f64, CalculatorError> {
        let substituted = Self::substitute_variable(expr, var, value);
        let result = self.parser.evaluate(&substituted)?;
        result
            .as_decimal()
            .map(|d| d.to_f64())
            .ok_or_else(|| CalculatorError::eval("Expected numeric result"))
    }

    /// Substitutes a variable with a numeric value in an expression.
    fn substitute_variable(expr: &types::Expression, var: &str, value: f64) -> types::Expression {
        use types::{Decimal, Expression};

        match expr {
            Expression::Variable(name) if name == var => {
                Expression::number(Decimal::from_f64(value))
            }
            Expression::Variable(_)
            | Expression::Number { .. }
            | Expression::DateTime(_)
            | Expression::Now => expr.clone(),
            Expression::Until(inner) => {
                Expression::Until(Box::new(Self::substitute_variable(inner, var, value)))
            }
            Expression::Binary { left, op, right } => Expression::binary(
                Self::substitute_variable(left, var, value),
                *op,
                Self::substitute_variable(right, var, value),
            ),
            Expression::Negate(inner) => {
                Expression::negate(Self::substitute_variable(inner, var, value))
            }
            Expression::Group(inner) => {
                Expression::group(Self::substitute_variable(inner, var, value))
            }
            Expression::Power { base, exponent } => Expression::power(
                Self::substitute_variable(base, var, value),
                Self::substitute_variable(exponent, var, value),
            ),
            Expression::FunctionCall { name, args } => Expression::function_call(
                name.clone(),
                args.iter()
                    .map(|a| Self::substitute_variable(a, var, value))
                    .collect(),
            ),
            Expression::AtTime { value: v, time } => Expression::at_time(
                Self::substitute_variable(v, var, value),
                Self::substitute_variable(time, var, value),
            ),
            Expression::IndefiniteIntegral {
                integrand,
                variable,
            } => Expression::indefinite_integral(
                Self::substitute_variable(integrand, var, value),
                variable.clone(),
            ),
            Expression::UnitConversion {
                value: v,
                target_unit,
            } => Expression::unit_conversion(
                Self::substitute_variable(v, var, value),
                target_unit.clone(),
            ),
            Expression::Equality { left, right } => Expression::equality(
                Self::substitute_variable(left, var, value),
                Self::substitute_variable(right, var, value),
            ),
        }
    }

    /// Parses an expression without evaluating it.
    pub fn parse(&self, input: &str) -> Result<types::Expression, CalculatorError> {
        self.parser.parse(input)
    }

    /// Evaluates a parsed expression.
    pub fn evaluate(&mut self, expr: &types::Expression) -> Result<Value, CalculatorError> {
        self.parser.evaluate(expr)
    }

    /// Loads a historical exchange rate from .lino format content.
    ///
    /// The .lino format for rates:
    /// ```text
    /// rate:
    ///   from USD
    ///   to EUR
    ///   value 0.92
    ///   date 2026-01-25
    ///   source 'frankfurter.dev (ECB)'
    /// ```
    pub fn load_rate_from_lino(&mut self, content: &str) -> Result<(), String> {
        let mut from_currency: Option<String> = None;
        let mut to_currency: Option<String> = None;
        let mut value: Option<f64> = None;
        let mut date: Option<String> = None;
        let mut source: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line == "rate:" {
                continue;
            }

            if let Some(rest) = line.strip_prefix("from ") {
                from_currency = Some(rest.trim().to_uppercase());
            } else if let Some(rest) = line.strip_prefix("to ") {
                to_currency = Some(rest.trim().to_uppercase());
            } else if let Some(rest) = line.strip_prefix("value ") {
                value = rest.trim().parse().ok();
            } else if let Some(rest) = line.strip_prefix("date ") {
                date = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("source ") {
                // Remove quotes from source
                let src = rest.trim();
                let src = src.trim_start_matches('\'').trim_end_matches('\'');
                let src = src.trim_start_matches('"').trim_end_matches('"');
                source = Some(src.to_string());
            }
        }

        let from = from_currency.ok_or("Missing 'from' currency")?;
        let to = to_currency.ok_or("Missing 'to' currency")?;
        let rate_value = value.ok_or("Missing 'value'")?;
        let rate_date = date.ok_or("Missing 'date'")?;
        let rate_source = source.unwrap_or_else(|| "unknown".to_string());

        // Create ExchangeRateInfo and add to the database
        let rate_info = types::ExchangeRateInfo::new(rate_value, rate_source, rate_date.clone());

        self.parser
            .currency_db_mut()
            .set_historical_rate_with_info(&from, &to, &rate_date, rate_info);

        Ok(())
    }

    /// Loads multiple historical exchange rates from a batch of .lino content.
    /// Each rate should be separated by double newlines or start with "rate:".
    pub fn load_rates_batch(&mut self, contents: &[&str]) -> Result<usize, String> {
        let mut loaded = 0;
        for content in contents {
            if self.load_rate_from_lino(content).is_ok() {
                loaded += 1;
            }
            // Silently skip invalid rate files
        }
        Ok(loaded)
    }

    /// Loads historical exchange rates from a consolidated .lino format.
    ///
    /// Supports both the new format (conversion/rates) and legacy format (rates/data):
    ///
    /// New format:
    /// ```text
    /// conversion:
    ///   from USD
    ///   to EUR
    ///   source 'frankfurter.dev (ECB)'
    ///   rates:
    ///     2021-01-25 0.8234
    ///     2021-02-01 0.8315
    ///     ...
    /// ```
    ///
    /// Legacy format:
    /// ```text
    /// rates:
    ///   from USD
    ///   to EUR
    ///   source 'frankfurter.dev (ECB)'
    ///   data:
    ///     2021-01-25 0.8234
    ///     2021-02-01 0.8315
    ///     ...
    /// ```
    pub fn load_rates_from_consolidated_lino(&mut self, content: &str) -> Result<usize, String> {
        let mut from_currency: Option<String> = None;
        let mut to_currency: Option<String> = None;
        let mut source: Option<String> = None;
        let mut in_data_section = false;
        let mut loaded = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Handle 'rates:' based on context:
            // - If we haven't parsed from_currency yet, it's a root marker (legacy format), skip it
            // - If we have parsed from_currency, it's the data section marker (new format)
            if trimmed == "rates:" {
                if from_currency.is_some() {
                    // New format: 'rates:' marks the start of data section
                    in_data_section = true;
                }
                // Either way, continue to next line (skip the 'rates:' line itself)
                continue;
            }

            // Skip root marker for new format
            if trimmed == "conversion:" {
                continue;
            }

            // Check for legacy data section marker
            if trimmed == "data:" {
                in_data_section = true;
                continue;
            }

            if in_data_section {
                // Parse date and value: "2021-01-25 0.8234"
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Some(from), Some(to)) = (from_currency.as_ref(), to_currency.as_ref()) {
                        let date = parts[0];
                        if let Ok(value) = parts[1].parse::<f64>() {
                            let rate_source =
                                source.clone().unwrap_or_else(|| "unknown".to_string());
                            let rate_info =
                                types::ExchangeRateInfo::new(value, rate_source, date.to_string());
                            self.parser
                                .currency_db_mut()
                                .set_historical_rate_with_info(from, to, date, rate_info);
                            loaded += 1;
                        }
                    }
                }
            } else {
                // Parse header section
                if let Some(rest) = trimmed.strip_prefix("from ") {
                    from_currency = Some(rest.trim().to_uppercase());
                } else if let Some(rest) = trimmed.strip_prefix("to ") {
                    to_currency = Some(rest.trim().to_uppercase());
                } else if let Some(rest) = trimmed.strip_prefix("source ") {
                    // Remove quotes from source
                    let src = rest.trim();
                    let src = src.trim_start_matches('\'').trim_end_matches('\'');
                    let src = src.trim_start_matches('"').trim_end_matches('"');
                    source = Some(src.to_string());
                }
            }
        }

        if loaded == 0 {
            Err("No rates loaded from consolidated file".to_string())
        } else {
            Ok(loaded)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for private helper functions that can only be tested here.
    // Public API tests have been moved to tests/ directory to keep this file
    // under the 1000-line limit.

    #[test]
    fn test_issue_link_generation() {
        let link = generate_issue_link("invalid input", "Parse error");
        assert!(link.contains("github.com"));
        assert!(link.contains("issues/new"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 5), "hello");
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn test_lino_rates_used_in_historical_conversion() {
        let mut calc = Calculator::new();
        // Load rates in the new .lino format
        let content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192";

        let result = calc.load_rates_from_consolidated_lino(content);
        assert!(result.is_ok());

        // Verify the historical rate can be retrieved from the database (uses private `parser` field)
        let date = types::DateTime::from_date(chrono::NaiveDate::from_ymd_opt(2021, 2, 8).unwrap());
        let rate = calc
            .parser
            .currency_db()
            .get_historical_rate("USD", "RUB", &date);
        assert!(rate.is_some());
        let rate_value = rate.unwrap();
        assert!((rate_value - 74.2602).abs() < 0.001);
    }
}
