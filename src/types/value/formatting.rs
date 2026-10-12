//! Numeric access for calculator values.
use super::{Value, ValueKind};
use crate::types::{Decimal, Rational};

impl Value {
    /// Returns the type name for error messages.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ValueKind::Number(_) => "number",
            ValueKind::Rational(_) | ValueKind::Ratio { .. } => "number",
            ValueKind::Percent(_) => "percent",
            ValueKind::Uncertainty { .. } => "uncertainty",
            ValueKind::DateTime(_) => "datetime",
            ValueKind::Duration { .. } => "duration",
            ValueKind::Boolean(_) => "boolean",
            ValueKind::Comparison { .. } => "comparison result",
            ValueKind::EquationSolution { .. }
            | ValueKind::EquationSolutions { .. }
            | ValueKind::SymbolicEquationSolution { .. } => "equation solution",
        }
    }

    /// Returns true if this is a number (either Decimal or Rational).
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(
            self.kind,
            ValueKind::Number(_)
                | ValueKind::Rational(_)
                | ValueKind::Percent(_)
                | ValueKind::Ratio { .. }
        )
    }

    /// Returns the decimal value if this is a number.
    #[must_use]
    pub fn as_number(&self) -> Option<Decimal> {
        match &self.kind {
            ValueKind::Number(n) => Some(*n),
            ValueKind::Rational(r) | ValueKind::Percent(r) | ValueKind::Ratio { value: r, .. } => {
                Some(r.to_decimal())
            }
            _ => None,
        }
    }

    /// Alias for `as_number`.
    #[must_use]
    pub fn as_decimal(&self) -> Option<Decimal> {
        self.as_number()
    }

    /// Returns the rational value if this is a Rational.
    #[must_use]
    pub fn as_rational(&self) -> Option<&Rational> {
        match &self.kind {
            ValueKind::Rational(r) | ValueKind::Percent(r) | ValueKind::Ratio { value: r, .. } => {
                Some(r)
            }
            _ => None,
        }
    }

    /// Converts this value to a Rational if numeric (clones Rational, converts Decimal).
    #[must_use]
    pub fn to_rational(&self) -> Option<Rational> {
        match &self.kind {
            ValueKind::Rational(r) | ValueKind::Percent(r) | ValueKind::Ratio { value: r, .. } => {
                Some(r.clone())
            }
            ValueKind::Number(d) => Some(Rational::from_decimal(*d)),
            _ => None,
        }
    }

    /// Returns the fraction string representation if this is a Rational.
    #[must_use]
    pub fn to_fraction_string(&self) -> Option<String> {
        match &self.kind {
            ValueKind::Rational(r) | ValueKind::Percent(r) | ValueKind::Ratio { value: r, .. } => {
                Some(r.to_fraction_string())
            }
            _ => None,
        }
    }
}
