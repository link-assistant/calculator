//! Error types for the Link Calculator.

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
