//! Dimensional arithmetic on physical units (issue #235).
//!
//! Multiplication, division and integer powers combine unit products;
//! addition, subtraction, comparison and conversion require equal dimensions
//! and convert exactly through the unit database factors.
//!
//! Affine temperatures follow Numbat: `20 °C` is read as 293.15 K, and a value
//! displayed in `°C`/`°F` (the result of a conversion) only converts further;
//! any arithmetic on it is rejected.

use super::Value;
use crate::error::CalculatorError;
use crate::types::{Dimension, PhysicalUnit, Rational, Unit, ValueKind};

/// Whether a unit takes part in dimensional algebra.
fn is_dimensional(unit: &Unit) -> bool {
    matches!(
        unit,
        Unit::Physical(_) | Unit::Mass(_) | Unit::Duration(_) | Unit::DataSize(_)
    )
}

/// The affine unit of a value displayed in `°C`/`°F`, if any.
fn affine_unit(value: &Value) -> Option<&PhysicalUnit> {
    match &value.unit {
        Unit::Physical(unit) if unit.has_affine() => Some(unit),
        _ => None,
    }
}

fn affine_error(value: &Value, operation: &str) -> CalculatorError {
    CalculatorError::domain(format!(
        "cannot {operation} {}: affine temperature units only convert; \
         convert to K first (e.g. `{} in K`)",
        value.to_display_string(),
        value.to_display_string()
    ))
}

impl Value {
    /// The numeric amount and unit product of a dimensional value.
    fn physical_parts(&self) -> Option<(Rational, PhysicalUnit)> {
        if !matches!(self.kind, ValueKind::Number(_) | ValueKind::Rational(_)) {
            return None;
        }
        Some((self.to_rational()?, PhysicalUnit::from_unit(&self.unit)?))
    }

    /// The amount in coherent SI units and the dimension of a value with a
    /// dimensional unit (`30 °` → `(0.5236, angle)`).
    pub(crate) fn physical_base(&self) -> Option<(Rational, Dimension)> {
        if !is_dimensional(&self.unit) {
            return None;
        }
        let (amount, unit) = self.physical_parts()?;
        Some((to_base(amount, &unit), unit.dimension()))
    }

    /// Builds a value from an amount and an unsimplified unit product.
    pub(crate) fn from_physical(amount: Rational, unit: &PhysicalUnit) -> Self {
        let (unit, scale) = unit.simplify();
        Self::rational_with_unit(amount * scale, unit.into_unit())
    }

    /// Rejects arithmetic on a value displayed in an affine unit.
    pub(crate) fn reject_affine(&self, operation: &str) -> Result<(), CalculatorError> {
        match affine_unit(self) {
            Some(_) => Err(affine_error(self, operation)),
            None => Ok(()),
        }
    }

    /// Whether an operation between the two values uses dimensional algebra:
    /// one side is a physical unit, or both carry dimensional units.
    fn uses_physical_algebra(&self, other: &Self) -> bool {
        let physical = |value: &Self| matches!(value.unit, Unit::Physical(_));
        (physical(self)
            || physical(other)
            || (is_dimensional(&self.unit) && is_dimensional(&other.unit)))
            && self.physical_parts().is_some()
            && other.physical_parts().is_some()
    }

    /// Multiplies or divides dimensional values (`10 m × 10 m`, `5 V / 2 Ω`).
    pub(super) fn physical_product(
        &self,
        other: &Self,
        divide: bool,
    ) -> Result<Option<Self>, CalculatorError> {
        let both_durations = matches!(
            (&self.unit, &other.unit),
            (Unit::Duration(_), Unit::Duration(_))
        );
        if both_durations || !self.uses_physical_algebra(other) {
            // `1 / (2 kg)` is `0.5 kg^-1`.
            let reciprocal = divide
                && self.unit == Unit::None
                && is_dimensional(&other.unit)
                && self.physical_parts().is_some()
                && other.physical_parts().is_some();
            if !reciprocal {
                return Ok(None);
            }
        }
        let operation = if divide { "divide" } else { "multiply" };
        self.reject_affine(operation)?;
        other.reject_affine(operation)?;
        let (a, left) = self.physical_parts().expect("checked above");
        let (b, right) = other.physical_parts().expect("checked above");
        if divide {
            if b.is_zero() {
                return Err(CalculatorError::DivisionByZero);
            }
            return Ok(Some(Self::from_physical(
                a / b,
                &left.product(&right.powi(-1)),
            )));
        }
        Ok(Some(Self::from_physical(a * b, &left.product(&right))))
    }

    /// Adds or subtracts dimensional values of equal dimension, converting the
    /// right operand to the left operand's unit (`1 ft + 1 in`).
    pub(super) fn physical_sum(
        &self,
        other: &Self,
        subtract: bool,
    ) -> Option<Result<Self, CalculatorError>> {
        let operation = if subtract { "subtract" } else { "add" };
        if let Err(error) = self
            .reject_affine(operation)
            .and_then(|()| other.reject_affine(operation))
        {
            return Some(Err(error));
        }
        if self.unit == other.unit
            || self.unit == Unit::None
            || other.unit == Unit::None
            || matches!((&self.unit, &other.unit), (Unit::Mass(_), Unit::Mass(_)))
            || !self.uses_physical_algebra(other)
        {
            return None;
        }
        Some(self.physical_sum_unchecked(other, subtract))
    }

    fn physical_sum_unchecked(
        &self,
        other: &Self,
        subtract: bool,
    ) -> Result<Self, CalculatorError> {
        let operation = if subtract { "subtract" } else { "add" };
        let (a, left) = self.physical_parts().expect("checked by caller");
        let (b, right) = other.physical_parts().expect("checked by caller");
        if left.dimension() != right.dimension() {
            // An exact zero has every dimension (`4 m + 0 kg`).
            if b.is_zero() {
                return Ok(self.clone());
            }
            if a.is_zero() && !subtract {
                return Ok(other.clone());
            }
            return Err(CalculatorError::unit_mismatch(
                operation,
                &self.unit.display_name(),
                &other.unit.display_name(),
            ));
        }
        let b_in_left = b * right.factor() / left.factor();
        let amount = if subtract {
            a - b_in_left
        } else {
            a + b_in_left
        };
        Ok(Self::rational_with_unit(amount, self.unit.clone()))
    }

    /// Raises a dimensional value to an integer power (`(2 m)^2` → `4 m^2`).
    pub(crate) fn physical_power(&self, power: i32) -> Option<Result<Self, CalculatorError>> {
        if !is_dimensional(&self.unit) {
            return None;
        }
        let (amount, unit) = self.physical_parts()?;
        if let Err(error) = self.reject_affine("raise to a power") {
            return Some(Err(error));
        }
        if power < 0 && amount.is_zero() {
            return Some(Err(CalculatorError::DivisionByZero));
        }
        Some(Ok(Self::from_physical(
            amount.pow_i32(power),
            &unit.powi(power),
        )))
    }

    /// Takes the `n`th root of a dimensional value (`sqrt(4 m^2)` → `2 m`).
    ///
    /// Fails with a non-integer dimension error when a unit power is not a
    /// multiple of `n` (`sqrt(2 m)`).
    pub(crate) fn physical_root(&self, n: i32) -> Option<Result<Self, CalculatorError>> {
        if !is_dimensional(&self.unit) {
            return None;
        }
        let (amount, unit) = self.physical_parts()?;
        Some(self.physical_root_unchecked(&amount, &unit, n))
    }

    fn physical_root_unchecked(
        &self,
        amount: &Rational,
        unit: &PhysicalUnit,
        n: i32,
    ) -> Result<Self, CalculatorError> {
        self.reject_affine("take a root of")?;
        let (simplified, scale) = unit.simplify();
        let amount = amount.clone() * scale;
        let root_unit = simplified.root(n).ok_or_else(|| {
            CalculatorError::domain(format!(
                "root of {} has a non-integer dimension",
                self.to_display_string()
            ))
        })?;
        if amount.is_negative() && n % 2 == 0 {
            return Err(CalculatorError::domain("even root of a negative quantity"));
        }
        let root = exact_root(&amount, n).unwrap_or_else(|| {
            let magnitude = amount.abs().to_f64().powf(1.0 / f64::from(n));
            let signed = if amount.is_negative() {
                -magnitude
            } else {
                magnitude
            };
            Rational::from_f64(signed)
        });
        Ok(Self::rational_with_unit(root, root_unit.into_unit()))
    }

    /// Whether two dimensional values of equal dimension denote the same quantity
    /// (`2 m == 200 cm`).
    pub(super) fn physical_eq(&self, other: &Self) -> Option<bool> {
        if self.unit == other.unit || !self.uses_physical_algebra(other) {
            return None;
        }
        let (a, left) = self.physical_parts()?;
        let (b, right) = other.physical_parts()?;
        if left.dimension() != right.dimension() {
            return Some(false);
        }
        Some(to_base(a, &left) == to_base(b, &right))
    }

    /// Converts a value to a physical target unit (`1 ft to cm`, `25 °C in °F`).
    ///
    /// A value in metres converted to a duration reads `m` as minutes: `5m`
    /// is a length, but `5m in seconds` asks for a duration.
    pub(super) fn physical_conversion(
        &self,
        target: &Unit,
    ) -> Option<Result<Self, CalculatorError>> {
        let target_is_physical = matches!(target, Unit::Physical(_));
        let source_is_physical = matches!(self.unit, Unit::Physical(_));
        let crosses_legacy = is_dimensional(&self.unit)
            && is_dimensional(target)
            && std::mem::discriminant(&self.unit) != std::mem::discriminant(target);
        if !(target_is_physical || source_is_physical || crosses_legacy) {
            return None;
        }
        let target_unit = PhysicalUnit::from_unit(target)?;
        let (amount, source) = self.physical_parts()?;
        if self.unit == Unit::None {
            return Some(Ok(Self::rational_with_unit(amount, target.clone())));
        }
        let source = if is_metre(&source) && matches!(target, Unit::Duration(_)) {
            PhysicalUnit::parse("min").expect("minute is a database unit")
        } else {
            source
        };
        if source.dimension() != target_unit.dimension() {
            return Some(Err(CalculatorError::InvalidOperation(format!(
                "Cannot convert {} to {}: dimensions {} and {} differ",
                self.unit.display_name(),
                target.display_name(),
                source.dimension(),
                target_unit.dimension()
            ))));
        }
        let base = to_base(amount, &source);
        let mut converted = base / target_unit.factor();
        if let Some(offset) = target_unit.affine_offset() {
            converted = converted - offset;
        }
        Some(Ok(Self::rational_with_unit(converted, target.clone())))
    }
}

/// Normalizes a quantity in an affine unit to kelvin (`20 °C` → `293.15 K`).
///
/// Other units are returned unchanged.
#[must_use]
pub fn normalize_affine(amount: Rational, unit: &Unit) -> (Rational, Unit) {
    let Unit::Physical(physical) = unit else {
        return (amount, unit.clone());
    };
    if physical.affine_offset().is_none() {
        return (amount, unit.clone());
    }
    let base = to_base(amount, physical);
    let kelvin = PhysicalUnit::parse("K").expect("kelvin is a database unit");
    (base, Unit::Physical(kelvin))
}

/// The amount in coherent SI units, applying an affine offset.
fn to_base(amount: Rational, unit: &PhysicalUnit) -> Rational {
    let shifted = match unit.affine_offset() {
        Some(offset) => amount + offset,
        None => amount,
    };
    shifted * unit.factor()
}

fn is_metre(unit: &PhysicalUnit) -> bool {
    matches!(unit.terms(), [term] if term.symbol == "m" && term.power == 1)
}

/// The exact `n`th root of a rational whose numerator and denominator are
/// perfect powers (`sqrt(9/4)` → `3/2`).
fn exact_root(value: &Rational, n: i32) -> Option<Rational> {
    let root_of = |x: i128| -> Option<i128> {
        let magnitude = x.unsigned_abs() as f64;
        let guess = magnitude.powf(1.0 / f64::from(n)).round() as i128;
        let exponent = u32::try_from(n).ok()?;
        (guess - 1..=guess + 1)
            .filter(|candidate| *candidate >= 0)
            .find(|candidate| candidate.checked_pow(exponent) == Some(x.abs()))
            .map(|candidate| if x < 0 { -candidate } else { candidate })
    };
    let numer = root_of(value.numer())?;
    let denom = root_of(value.denom())?;
    Some(Rational::new(numer, denom))
}
