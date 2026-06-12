use serde::{Deserialize, Serialize};

use crate::types::{DateTime, Decimal, Rational};

/// Different kinds of values the calculator can work with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueKind {
    /// A decimal number (for compatibility and complex operations).
    Number(Decimal),
    /// A rational number for exact fractional arithmetic.
    Rational(Rational),
    /// A date and/or time.
    DateTime(DateTime),
    /// A duration (difference between two datetimes).
    Duration {
        /// Duration in seconds.
        seconds: i64,
    },
    /// A boolean value.
    Boolean(bool),
    /// A solved single-variable equation.
    EquationSolution {
        /// The solved variable name.
        variable: String,
        /// The value assigned to the variable.
        value: Rational,
    },
    /// A solved linear equation whose result still contains other variables.
    SymbolicEquationSolution {
        /// The solved variable name.
        variable: String,
        /// The symbolic expression assigned to the variable.
        expression: String,
    },
}
