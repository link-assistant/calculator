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

pub mod currency_api;
pub mod error;
pub mod grammar;
pub mod lino;
pub mod types;
pub mod wasm;

use error::{CalculatorError, ErrorInfo};
use grammar::ExpressionParser;
use types::Value;
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

/// Result of a calculation operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationResult {
    /// The computed value as a string.
    pub result: String,
    /// The input interpreted in links notation format.
    pub lino_interpretation: String,
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
}

impl CalculationResult {
    /// Creates a successful calculation result.
    #[must_use]
    pub fn success(result: String, lino: String, steps: Vec<String>) -> Self {
        Self {
            result,
            lino_interpretation: lino,
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
        }
    }

    /// Creates a failed calculation result.
    #[must_use]
    pub fn failure(error: String, input: &str) -> Self {
        let issue_link = generate_issue_link(input, &error);
        Self {
            result: String::new(),
            lino_interpretation: String::new(),
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

    /// Calculates the result of an expression.
    ///
    /// # Arguments
    ///
    /// * `input` - The expression to calculate
    ///
    /// # Returns
    ///
    /// A JSON string containing the calculation result.
    #[wasm_bindgen]
    pub fn calculate(&mut self, input: &str) -> String {
        let result = self.calculate_internal(input);
        serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(
                r#"{{"success":false,"error":"Serialization error: {}"}}"#,
                e
            )
        })
    }

    /// Returns the version of the calculator.
    #[wasm_bindgen]
    #[must_use]
    pub fn version() -> String {
        VERSION.to_string()
    }
}

impl Calculator {
    /// Internal calculation method that returns a proper Result type.
    pub fn calculate_internal(&mut self, input: &str) -> CalculationResult {
        match self.parser.parse_and_evaluate(input) {
            Ok((value, steps, lino)) => CalculationResult::success_with_value(&value, lino, steps),
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
        }
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
            Expression::Variable(_) => expr.clone(),
            Expression::Number { .. } => expr.clone(),
            Expression::DateTime(_) => expr.clone(),
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
    ///   source 'fawazahmed0/currency-api'
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
    /// The consolidated format stores all rates for a currency pair in one file:
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

            if trimmed.is_empty() || trimmed == "rates:" {
                continue;
            }

            // Check for data section marker
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

    #[test]
    fn test_calculator_creation() {
        let calc = Calculator::new();
        assert!(!Calculator::version().is_empty());
        let _ = calc;
    }

    #[test]
    fn test_simple_addition() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2 + 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_simple_subtraction() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("10 - 4");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_simple_multiplication() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "12");
    }

    #[test]
    fn test_simple_division() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("15 / 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_decimal_numbers() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("3.14 + 2.86");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_negative_numbers() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("-5 + 3");
        assert!(result.success);
        assert_eq!(result.result, "-2");
    }

    #[test]
    fn test_parentheses() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(2 + 3) * 4");
        assert!(result.success);
        assert_eq!(result.result, "20");
    }

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
    fn test_load_rate_from_lino() {
        let mut calc = Calculator::new();
        let content = "rate:
  from USD
  to EUR
  value 0.85
  date 1999-01-04
  source 'frankfurter.dev (ECB)'";

        let result = calc.load_rate_from_lino(content);
        assert!(result.is_ok());

        // Test that the rate was loaded by doing a calculation with the historical date
        // Note: This tests that the rate is in the database, but the full
        // "at date" functionality requires the date context to be set during evaluation
    }

    #[test]
    fn test_load_rates_batch() {
        let mut calc = Calculator::new();
        let content1 = "rate:
  from USD
  to EUR
  value 0.85
  date 1999-01-04
  source 'test'";

        let content2 = "rate:
  from EUR
  to USD
  value 1.18
  date 1999-01-04
  source 'test'";

        let result = calc.load_rates_batch(&[content1, content2]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_load_rates_from_consolidated_lino() {
        let mut calc = Calculator::new();
        let content = "rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402";

        let result = calc.load_rates_from_consolidated_lino(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_load_rates_from_consolidated_lino_empty() {
        let mut calc = Calculator::new();
        let content = "rates:
  from USD
  to EUR
  source 'test'
  data:";

        let result = calc.load_rates_from_consolidated_lino(content);
        assert!(result.is_err());
    }
}
