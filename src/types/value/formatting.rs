//! Numeric access and display for calculator values.
use super::kind::RatioFormat;
use super::{format_duration, Value, ValueKind};
use crate::types::{Decimal, Rational, Unit};

impl Value {
    /// Returns the type name for error messages.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ValueKind::Number(_) => "number",
            ValueKind::Rational(_) | ValueKind::Ratio { .. } => "number",
            ValueKind::Percent(_) => "percent",
            ValueKind::DateTime(_) => "datetime",
            ValueKind::Duration { .. } => "duration",
            ValueKind::Boolean(_) => "boolean",
            ValueKind::Comparison { .. } => "comparison result",
            ValueKind::EquationSolution { .. }
            | ValueKind::EquationSolutions { .. }
            | ValueKind::SymbolicEquationSolution { .. } => "equation solution",
        }
    }

    /// Converts the value to a display string.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        match &self.kind {
            ValueKind::Number(n) => {
                let n_str = n.normalize().to_string();
                if self.unit == Unit::None {
                    n_str
                } else {
                    format!("{} {}", n_str, self.unit)
                }
            }
            ValueKind::Rational(r) => {
                let r_str = r.to_display_string();
                if self.unit == Unit::None {
                    r_str
                } else {
                    format!("{} {}", r_str, self.unit)
                }
            }
            ValueKind::Percent(r) => format!(
                "{}%",
                (r.clone() * Rational::from_integer(100)).to_display_string()
            ),
            ValueKind::Ratio { value, format } => match format {
                RatioFormat::Fraction => value.to_fraction_string(),
                RatioFormat::Reciprocal => format!(
                    "1/{}",
                    Rational::one()
                        .checked_div(value)
                        .map_or_else(|| "undefined".into(), |ratio| ratio.to_display_string())
                ),
                RatioFormat::Multiplier => format!("{}x", value.to_display_string()),
            },
            ValueKind::DateTime(dt) => dt.to_string(),
            ValueKind::Duration { seconds } => format_duration(*seconds),
            ValueKind::Boolean(b) => b.to_string(),
            ValueKind::Comparison {
                left,
                relation,
                right,
            } => format!("{left} {relation} {right}"),
            ValueKind::EquationSolution { variable, value } => {
                format!("{variable} = {}", value.to_display_string())
            }
            ValueKind::EquationSolutions { variable, values } => values
                .iter()
                .map(|value| format!("{variable} = {}", value.to_display_string()))
                .collect::<Vec<_>>()
                .join(" or "),
            ValueKind::SymbolicEquationSolution {
                variable,
                expression,
            } => {
                format!("{variable} = {expression}")
            }
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
