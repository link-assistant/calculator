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
            Ok((value, steps, lino)) => {
                CalculationResult::success(value.to_display_string(), lino, steps)
            }
            Err(e) => CalculationResult::failure_with_i18n(&e, input),
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
}
