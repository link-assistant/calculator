//! Error types for the Link Calculator.

use std::collections::HashMap;
use thiserror::Error;

/// Main error type for calculator operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CalculatorError {
    /// Error parsing the input expression.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Error with an unexpected token during parsing.
    #[error("Unexpected token '{found}' at position {position}, expected {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
        position: usize,
    },

    /// Error when units don't match in an operation.
    #[error("Unit mismatch: cannot {operation} '{left_unit}' and '{right_unit}'")]
    UnitMismatch {
        operation: String,
        left_unit: String,
        right_unit: String,
    },

    /// Error during evaluation.
    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    /// Division by zero error.
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid date/time format.
    #[error("Invalid datetime format: {0}")]
    InvalidDateTime(String),

    /// Invalid currency.
    #[error("Unknown currency: {0}")]
    UnknownCurrency(String),

    /// Currency conversion error.
    #[error("Cannot convert {from} to {to}: {reason}")]
    CurrencyConversion {
        from: String,
        to: String,
        reason: String,
    },

    /// No historical rate available.
    #[error("No exchange rate available for {currency} on {date}")]
    NoHistoricalRate { currency: String, date: String },

    /// Overflow error.
    #[error("Numeric overflow")]
    Overflow,

    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Empty input error.
    #[error("Empty input")]
    EmptyInput,

    /// Unknown function error.
    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    /// Invalid function arguments error.
    #[error("Invalid arguments for function '{function}': {reason}")]
    InvalidFunctionArgs { function: String, reason: String },

    /// Domain error (e.g., sqrt of negative number, log of non-positive).
    #[error("Domain error: {0}")]
    DomainError(String),
}

/// Error information for i18n support.
///
/// This struct contains all the information needed to translate an error
/// on the frontend, including the error key and any interpolation parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorInfo {
    /// The translation key for this error (e.g., "errors.divisionByZero").
    pub key: String,
    /// Parameters for interpolation in the translated message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

impl ErrorInfo {
    /// Creates a new error info with just a key.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            params: None,
        }
    }

    /// Creates a new error info with key and parameters.
    #[must_use]
    pub fn with_params(key: impl Into<String>, params: HashMap<String, String>) -> Self {
        Self {
            key: key.into(),
            params: Some(params),
        }
    }
}

impl CalculatorError {
    /// Returns the i18n error info for this error.
    ///
    /// This provides the translation key and any parameters needed
    /// for the frontend to display a localized error message.
    #[must_use]
    pub fn to_error_info(&self) -> ErrorInfo {
        match self {
            Self::ParseError(msg) => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), msg.clone());
                ErrorInfo::with_params("errors.parseError", params)
            }
            Self::UnexpectedToken {
                found,
                expected,
                position,
            } => {
                let mut params = HashMap::new();
                params.insert("found".to_string(), found.clone());
                params.insert("expected".to_string(), expected.clone());
                params.insert("position".to_string(), position.to_string());
                ErrorInfo::with_params("errors.unexpectedToken", params)
            }
            Self::UnitMismatch {
                operation,
                left_unit,
                right_unit,
            } => {
                let mut params = HashMap::new();
                params.insert("operation".to_string(), operation.clone());
                params.insert("leftUnit".to_string(), left_unit.clone());
                params.insert("rightUnit".to_string(), right_unit.clone());
                ErrorInfo::with_params("errors.unitMismatch", params)
            }
            Self::EvaluationError(msg) => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), msg.clone());
                ErrorInfo::with_params("errors.evaluationError", params)
            }
            Self::DivisionByZero => ErrorInfo::new("errors.divisionByZero"),
            Self::InvalidDateTime(format) => {
                let mut params = HashMap::new();
                params.insert("format".to_string(), format.clone());
                ErrorInfo::with_params("errors.invalidDateTime", params)
            }
            Self::UnknownCurrency(currency) => {
                let mut params = HashMap::new();
                params.insert("currency".to_string(), currency.clone());
                ErrorInfo::with_params("errors.unknownCurrency", params)
            }
            Self::CurrencyConversion { from, to, reason } => {
                let mut params = HashMap::new();
                params.insert("from".to_string(), from.clone());
                params.insert("to".to_string(), to.clone());
                params.insert("reason".to_string(), reason.clone());
                ErrorInfo::with_params("errors.currencyConversion", params)
            }
            Self::NoHistoricalRate { currency, date } => {
                let mut params = HashMap::new();
                params.insert("currency".to_string(), currency.clone());
                params.insert("date".to_string(), date.clone());
                ErrorInfo::with_params("errors.noHistoricalRate", params)
            }
            Self::Overflow => ErrorInfo::new("errors.overflow"),
            Self::InvalidOperation(msg) => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), msg.clone());
                ErrorInfo::with_params("errors.invalidOperation", params)
            }
            Self::EmptyInput => ErrorInfo::new("errors.emptyInput"),
            Self::UnknownFunction(name) => {
                let mut params = HashMap::new();
                params.insert("name".to_string(), name.clone());
                ErrorInfo::with_params("errors.unknownFunction", params)
            }
            Self::InvalidFunctionArgs { function, reason } => {
                let mut params = HashMap::new();
                params.insert("function".to_string(), function.clone());
                params.insert("reason".to_string(), reason.clone());
                ErrorInfo::with_params("errors.invalidFunctionArgs", params)
            }
            Self::DomainError(msg) => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), msg.clone());
                ErrorInfo::with_params("errors.domainError", params)
            }
        }
    }
}

impl CalculatorError {
    /// Creates a parse error with the given message.
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Creates an unexpected token error.
    #[must_use]
    pub fn unexpected_token(found: &str, expected: &str, position: usize) -> Self {
        Self::UnexpectedToken {
            found: found.to_string(),
            expected: expected.to_string(),
            position,
        }
    }

    /// Creates a unit mismatch error.
    #[must_use]
    pub fn unit_mismatch(operation: &str, left: &str, right: &str) -> Self {
        Self::UnitMismatch {
            operation: operation.to_string(),
            left_unit: left.to_string(),
            right_unit: right.to_string(),
        }
    }

    /// Creates an evaluation error with the given message.
    pub fn eval(msg: impl Into<String>) -> Self {
        Self::EvaluationError(msg.into())
    }

    /// Creates an unknown function error.
    pub fn unknown_function(name: impl Into<String>) -> Self {
        Self::UnknownFunction(name.into())
    }

    /// Creates an invalid function arguments error.
    pub fn invalid_args(function: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFunctionArgs {
            function: function.into(),
            reason: reason.into(),
        }
    }

    /// Creates a domain error.
    pub fn domain(msg: impl Into<String>) -> Self {
        Self::DomainError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error() {
        let err = CalculatorError::parse("invalid syntax");
        assert_eq!(err.to_string(), "Parse error: invalid syntax");
    }

    #[test]
    fn test_unexpected_token() {
        let err = CalculatorError::unexpected_token("+", "number", 5);
        assert!(err.to_string().contains("Unexpected token"));
        assert!(err.to_string().contains('+'));
    }

    #[test]
    fn test_unit_mismatch() {
        let err = CalculatorError::unit_mismatch("add", "USD", "hours");
        assert!(err.to_string().contains("Unit mismatch"));
        assert!(err.to_string().contains("USD"));
        assert!(err.to_string().contains("hours"));
    }

    #[test]
    fn test_division_by_zero() {
        let err = CalculatorError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }

    #[test]
    fn test_invalid_datetime() {
        let err = CalculatorError::InvalidDateTime("not a date".to_string());
        assert!(err.to_string().contains("Invalid datetime"));
    }
}
