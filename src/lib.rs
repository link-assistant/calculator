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
//! let calculator = Calculator::new();
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

pub mod error;
pub mod grammar;
pub mod lino;
pub mod types;
pub mod wasm;

use error::CalculatorError;
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

/// Result of a calculation operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationResult {
    /// The computed value as a string.
    pub result: String,
    /// The input interpreted in links notation format.
    pub lino_interpretation: String,
    /// Step-by-step explanation of the calculation.
    pub steps: Vec<String>,
    /// Whether the calculation was successful.
    pub success: bool,
    /// Error message if calculation failed.
    pub error: Option<String>,
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
}

impl CalculationResult {
    /// Creates a successful calculation result.
    #[must_use]
    pub fn success(result: String, lino: String, steps: Vec<String>) -> Self {
        Self {
            result,
            lino_interpretation: lino,
            steps,
            success: true,
            error: None,
            issue_link: None,
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
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
            success: true,
            error: None,
            issue_link: None,
            latex_input,
            latex_result,
            is_symbolic: None,
            plot_data: None,
        }
    }

    /// Creates a symbolic result (e.g., for indefinite integrals).
    #[must_use]
    pub fn symbolic(
        expression: String,
        result: String,
        latex_input: String,
        latex_result: String,
        plot_data: Option<PlotData>,
    ) -> Self {
        Self {
            result,
            lino_interpretation: expression.clone(),
            steps: vec![format!("Input: {}", expression), "Computed symbolic result".to_string()],
            success: true,
            error: None,
            issue_link: None,
            latex_input: Some(latex_input),
            latex_result: Some(latex_result),
            is_symbolic: Some(true),
            plot_data,
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
            success: false,
            error: Some(error),
            issue_link: Some(issue_link),
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
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
    pub fn calculate(&self, input: &str) -> String {
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
    pub fn calculate_internal(&self, input: &str) -> CalculationResult {
        match self.parser.parse_and_evaluate(input) {
            Ok((value, steps, lino)) => {
                CalculationResult::success(value.to_display_string(), lino, steps)
            }
            Err(CalculatorError::SymbolicResult {
                expression,
                result,
                latex_input,
                latex_result,
            }) => {
                // Generate plot data for the integrand function
                let plot_data = self.generate_plot_data_for_integral(input);
                CalculationResult::symbolic(expression, result, latex_input, latex_result, plot_data)
            }
            Err(e) => CalculationResult::failure(e.to_string(), input),
        }
    }

    /// Generates plot data for an integral expression.
    fn generate_plot_data_for_integral(&self, input: &str) -> Option<PlotData> {
        // Try to parse and extract the integrand for plotting
        let expr = self.parser.parse(input).ok()?;

        if let types::Expression::IndefiniteIntegral { integrand, variable } = expr {
            // Generate plot points for the integrand
            let mut x_values = Vec::new();
            let mut y_values = Vec::new();

            // Generate points from -10 to 10 with 200 steps
            let num_points = 200;
            let x_min = -10.0;
            let x_max = 10.0;
            let step = (x_max - x_min) / (num_points as f64);

            for i in 0..=num_points {
                let x = x_min + (i as f64) * step;

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
        &self,
        expr: &types::Expression,
        var: &str,
        value: f64,
    ) -> Result<f64, CalculatorError> {
        let substituted = self.substitute_variable(expr, var, value);
        let result = self.parser.evaluate(&substituted)?;
        result
            .as_decimal()
            .map(|d| d.to_f64())
            .ok_or_else(|| CalculatorError::eval("Expected numeric result"))
    }

    /// Substitutes a variable with a numeric value in an expression.
    fn substitute_variable(
        &self,
        expr: &types::Expression,
        var: &str,
        value: f64,
    ) -> types::Expression {
        use types::{Decimal, Expression};

        match expr {
            Expression::Variable(name) if name == var => {
                Expression::number(Decimal::from_f64(value))
            }
            Expression::Variable(_) => expr.clone(),
            Expression::Number { .. } => expr.clone(),
            Expression::DateTime(_) => expr.clone(),
            Expression::Binary { left, op, right } => Expression::binary(
                self.substitute_variable(left, var, value),
                *op,
                self.substitute_variable(right, var, value),
            ),
            Expression::Negate(inner) => {
                Expression::negate(self.substitute_variable(inner, var, value))
            }
            Expression::Group(inner) => {
                Expression::group(self.substitute_variable(inner, var, value))
            }
            Expression::Power { base, exponent } => Expression::power(
                self.substitute_variable(base, var, value),
                self.substitute_variable(exponent, var, value),
            ),
            Expression::FunctionCall { name, args } => Expression::function_call(
                name.clone(),
                args.iter()
                    .map(|a| self.substitute_variable(a, var, value))
                    .collect(),
            ),
            Expression::AtTime { value: v, time } => Expression::at_time(
                self.substitute_variable(v, var, value),
                self.substitute_variable(time, var, value),
            ),
            Expression::IndefiniteIntegral { integrand, variable } => {
                Expression::indefinite_integral(
                    self.substitute_variable(integrand, var, value),
                    variable.clone(),
                )
            }
        }
    }

    /// Parses an expression without evaluating it.
    pub fn parse(&self, input: &str) -> Result<types::Expression, CalculatorError> {
        self.parser.parse(input)
    }

    /// Evaluates a parsed expression.
    pub fn evaluate(&self, expr: &types::Expression) -> Result<Value, CalculatorError> {
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
        let calc = Calculator::new();
        let result = calc.calculate_internal("2 + 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_simple_subtraction() {
        let calc = Calculator::new();
        let result = calc.calculate_internal("10 - 4");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_simple_multiplication() {
        let calc = Calculator::new();
        let result = calc.calculate_internal("3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "12");
    }

    #[test]
    fn test_simple_division() {
        let calc = Calculator::new();
        let result = calc.calculate_internal("15 / 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_decimal_numbers() {
        let calc = Calculator::new();
        let result = calc.calculate_internal("3.14 + 2.86");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_negative_numbers() {
        let calc = Calculator::new();
        let result = calc.calculate_internal("-5 + 3");
        assert!(result.success);
        assert_eq!(result.result, "-2");
    }

    #[test]
    fn test_parentheses() {
        let calc = Calculator::new();
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
