//! Exact percentage arithmetic and dimensionless ratio presentation.
use super::kind::RatioFormat;
use super::{Value, ValueKind};
use crate::error::CalculatorError;
use crate::types::{Rational, Unit};

impl Value {
    /// Creates a percentage from its exact fraction (`1/10` displays `10%`).
    #[must_use]
    pub fn percent(fraction: Rational) -> Self {
        Self {
            kind: ValueKind::Percent(fraction),
            unit: Unit::None,
        }
    }

    pub(crate) fn ratio(value: Rational, format: RatioFormat) -> Self {
        Self {
            kind: ValueKind::Ratio { value, format },
            unit: Unit::None,
        }
    }

    pub(super) const fn is_percent(&self) -> bool {
        matches!(self.kind, ValueKind::Percent(_))
    }

    pub(super) const fn has_ratio_display(&self) -> bool {
        matches!(self.kind, ValueKind::Percent(_) | ValueKind::Ratio { .. })
    }

    pub(super) fn scalar(&self) -> Self {
        if self.has_ratio_display() {
            Self::rational_with_unit(
                self.to_rational().expect("numeric ratio"),
                self.unit.clone(),
            )
        } else {
            self.clone()
        }
    }

    pub(super) fn additive_ratio(
        &self,
        other: &Self,
        subtract: bool,
    ) -> Option<Result<Self, CalculatorError>> {
        if !self.has_ratio_display() && !other.has_ratio_display() {
            return None;
        }
        Some((|| {
            let left = self.to_rational().ok_or_else(|| {
                CalculatorError::InvalidOperation(
                    "percentage arithmetic requires numeric operands".into(),
                )
            })?;
            let right = other.to_rational().ok_or_else(|| {
                CalculatorError::InvalidOperation(
                    "percentage arithmetic requires numeric operands".into(),
                )
            })?;
            let right = if subtract { -right } else { right };
            if self.is_percent() {
                if self.unit != Unit::None || other.unit != Unit::None {
                    return Err(CalculatorError::InvalidOperation(
                        "cannot add a unit to a percentage".into(),
                    ));
                }
                Ok(Self::percent(left + right))
            } else if other.is_percent() {
                Ok(Self::rational_with_unit(
                    left * (Rational::one() + right),
                    self.unit.clone(),
                ))
            } else if self.unit == other.unit {
                Ok(Self::rational_with_unit(left + right, self.unit.clone()))
            } else {
                Err(CalculatorError::InvalidOperation(
                    "ratio arithmetic requires compatible units".into(),
                ))
            }
        })())
    }
}

/// Evaluates ratio constructors introduced by the percentage grammar.
/// Returning `None` lets ordinary mathematical functions use their evaluator.
pub fn evaluate_ratio_function(
    name: &str,
    args: &[Value],
) -> Option<Result<Value, CalculatorError>> {
    if !matches!(
        name,
        "percent"
            | "as_percent"
            | "as_fraction"
            | "as_reciprocal"
            | "as_multiplier"
            | "as_decimal"
            | "percent_of"
    ) {
        return None;
    }
    Some((|| {
        let [value] = args else {
            return Err(CalculatorError::invalid_args(
                name,
                "expected one numeric argument",
            ));
        };
        if name == "percent_of" {
            return Ok(value.clone());
        }
        if value.unit != Unit::None {
            return Err(CalculatorError::InvalidOperation(
                "ratio conversion requires a unitless number".into(),
            ));
        }
        let fraction = value
            .to_rational()
            .ok_or_else(|| CalculatorError::invalid_args(name, "expected numeric argument"))?;
        Ok(match name {
            "percent" => Value::percent(fraction / Rational::from_integer(100)),
            "as_percent" => Value::percent(fraction),
            "as_fraction" => Value::ratio(fraction, RatioFormat::Fraction),
            "as_multiplier" => Value::ratio(fraction, RatioFormat::Multiplier),
            "as_reciprocal" => {
                if fraction.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                Value::ratio(fraction, RatioFormat::Reciprocal)
            }
            _ => Value::rational(fraction),
        })
    })())
}
