//! Value type representing typed values with units.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::CalculatorError;
use crate::types::{CurrencyDatabase, DateTime, Decimal, Rational, Unit};

/// A typed value with an optional unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    /// The kind of value.
    pub kind: ValueKind,
    /// The unit of measurement.
    pub unit: Unit,
}

/// Different kinds of values the calculator can work with.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Value {
    /// Creates a numeric value without a unit.
    #[must_use]
    pub fn number(n: Decimal) -> Self {
        Self {
            kind: ValueKind::Number(n),
            unit: Unit::None,
        }
    }

    /// Creates a numeric value with a unit.
    #[must_use]
    pub fn number_with_unit(n: Decimal, unit: Unit) -> Self {
        Self {
            kind: ValueKind::Number(n),
            unit,
        }
    }

    /// Creates a rational value without a unit.
    #[must_use]
    pub fn rational(r: Rational) -> Self {
        Self {
            kind: ValueKind::Rational(r),
            unit: Unit::None,
        }
    }

    /// Creates a rational value with a unit.
    #[must_use]
    pub fn rational_with_unit(r: Rational, unit: Unit) -> Self {
        Self {
            kind: ValueKind::Rational(r),
            unit,
        }
    }

    /// Creates a rational value from an integer.
    #[must_use]
    pub fn from_integer(n: i64) -> Self {
        Self {
            kind: ValueKind::Rational(Rational::from_integer(i128::from(n))),
            unit: Unit::None,
        }
    }

    /// Creates a rational value from an integer with a unit.
    #[must_use]
    pub fn from_integer_with_unit(n: i64, unit: Unit) -> Self {
        Self {
            kind: ValueKind::Rational(Rational::from_integer(i128::from(n))),
            unit,
        }
    }

    /// Creates a currency value.
    #[must_use]
    pub fn currency(amount: Decimal, currency_code: &str) -> Self {
        Self {
            kind: ValueKind::Number(amount),
            unit: Unit::currency(currency_code),
        }
    }

    /// Creates a datetime value.
    #[must_use]
    pub fn datetime(dt: DateTime) -> Self {
        Self {
            kind: ValueKind::DateTime(dt),
            unit: Unit::None,
        }
    }

    /// Creates a duration value.
    #[must_use]
    pub fn duration(seconds: i64) -> Self {
        Self {
            kind: ValueKind::Duration { seconds },
            unit: Unit::None,
        }
    }

    /// Creates a boolean value.
    #[must_use]
    pub fn boolean(b: bool) -> Self {
        Self {
            kind: ValueKind::Boolean(b),
            unit: Unit::None,
        }
    }

    /// Adds two values.
    pub fn add(
        &self,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational + Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                self.add_rationals(a.clone(), b.clone(), other, currency_db)
            }
            // Number + Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.add_numbers(*a, *b, other, currency_db)
            }
            // Mixed: convert Decimal to Rational
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                let b_rat = Rational::from_decimal(*b);
                self.add_rationals(a.clone(), b_rat, other, currency_db)
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                let a_rat = Rational::from_decimal(*a);
                self.add_rationals(a_rat, b.clone(), other, currency_db)
            }
            (ValueKind::DateTime(dt), ValueKind::Duration { seconds }) => {
                Ok(Value::datetime(dt.add_duration(*seconds)))
            }
            (ValueKind::Duration { seconds }, ValueKind::DateTime(dt)) => {
                // Duration + DateTime = DateTime (commutative)
                Ok(Value::datetime(dt.add_duration(*seconds)))
            }
            (ValueKind::Duration { seconds: s1 }, ValueKind::Duration { seconds: s2 }) => {
                Ok(Value::duration(s1 + s2))
            }
            _ => Err(CalculatorError::InvalidOperation(format!(
                "Cannot add {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    fn add_rationals(
        &self,
        a: Rational,
        b: Rational,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::rational(a + b)),
            (Unit::None, unit) | (unit, Unit::None) => {
                Ok(Value::rational_with_unit(a + b, unit.clone()))
            }
            // Same currency
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => {
                Ok(Value::rational_with_unit(a + b, self.unit.clone()))
            }
            // Different currencies - need conversion (uses Decimal for approximation)
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                let a_dec = a.to_decimal();
                let b_dec = b.to_decimal();
                let converted = currency_db.convert(b_dec.to_f64(), c2, c1)?;
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a_dec + converted_decimal, c1))
            }
            (u1, u2) if u1 == u2 => Ok(Value::rational_with_unit(a + b, u1.clone())),
            (u1, u2) => Err(CalculatorError::unit_mismatch(
                "add",
                &u1.display_name(),
                &u2.display_name(),
            )),
        }
    }

    fn add_numbers(
        &self,
        a: Decimal,
        b: Decimal,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::number(a + b)),
            (Unit::None, unit) | (unit, Unit::None) => {
                Ok(Value::number_with_unit(a + b, unit.clone()))
            }
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Ok(Value::currency(a + b, c1)),
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                // Convert c2 to c1
                let converted = currency_db.convert(b.to_f64(), c2, c1)?;
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a + converted_decimal, c1))
            }
            (u1, u2) if u1 == u2 => Ok(Value::number_with_unit(a + b, u1.clone())),
            (u1, u2) => Err(CalculatorError::unit_mismatch(
                "add",
                &u1.display_name(),
                &u2.display_name(),
            )),
        }
    }

    /// Subtracts two values.
    pub fn subtract(
        &self,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational - Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                self.subtract_rationals(a.clone(), b.clone(), other, currency_db)
            }
            // Number - Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.subtract_numbers(*a, *b, other, currency_db)
            }
            // Mixed: convert Decimal to Rational
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                let b_rat = Rational::from_decimal(*b);
                self.subtract_rationals(a.clone(), b_rat, other, currency_db)
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                let a_rat = Rational::from_decimal(*a);
                self.subtract_rationals(a_rat, b.clone(), other, currency_db)
            }
            (ValueKind::DateTime(dt1), ValueKind::DateTime(dt2)) => {
                let diff = dt1.subtract(dt2);
                Ok(Value::duration(diff.as_secs() as i64))
            }
            (ValueKind::DateTime(dt), ValueKind::Duration { seconds }) => {
                Ok(Value::datetime(dt.add_duration(-seconds)))
            }
            (ValueKind::Duration { seconds: s1 }, ValueKind::Duration { seconds: s2 }) => {
                Ok(Value::duration(s1 - s2))
            }
            _ => Err(CalculatorError::InvalidOperation(format!(
                "Cannot subtract {} from {}",
                other.type_name(),
                self.type_name()
            ))),
        }
    }

    fn subtract_rationals(
        &self,
        a: Rational,
        b: Rational,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::rational(a - b)),
            (unit, Unit::None) => Ok(Value::rational_with_unit(a - b, unit.clone())),
            (Unit::None, unit) => Ok(Value::rational_with_unit(a - b, unit.clone())),
            // Same currency
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => {
                Ok(Value::rational_with_unit(a - b, self.unit.clone()))
            }
            // Different currencies - need conversion (uses Decimal for approximation)
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                let a_dec = a.to_decimal();
                let b_dec = b.to_decimal();
                let converted = currency_db.convert(b_dec.to_f64(), c2, c1)?;
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a_dec - converted_decimal, c1))
            }
            (u1, u2) if u1 == u2 => Ok(Value::rational_with_unit(a - b, u1.clone())),
            (u1, u2) => Err(CalculatorError::unit_mismatch(
                "subtract",
                &u1.display_name(),
                &u2.display_name(),
            )),
        }
    }

    fn subtract_numbers(
        &self,
        a: Decimal,
        b: Decimal,
        other: &Self,
        currency_db: &CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::number(a - b)),
            (unit, Unit::None) => Ok(Value::number_with_unit(a - b, unit.clone())),
            (Unit::None, unit) => Ok(Value::number_with_unit(a - b, unit.clone())),
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Ok(Value::currency(a - b, c1)),
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                // Convert c2 to c1
                let converted = currency_db.convert(b.to_f64(), c2, c1)?;
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a - converted_decimal, c1))
            }
            (u1, u2) if u1 == u2 => Ok(Value::number_with_unit(a - b, u1.clone())),
            (u1, u2) => Err(CalculatorError::unit_mismatch(
                "subtract",
                &u1.display_name(),
                &u2.display_name(),
            )),
        }
    }

    /// Multiplies two values.
    pub fn multiply(&self, other: &Self) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational * Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                let result = a.clone() * b.clone();
                let unit = if self.unit != Unit::None {
                    self.unit.clone()
                } else {
                    other.unit.clone()
                };
                Ok(Value::rational_with_unit(result, unit))
            }
            // Number * Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                let result = *a * *b;
                let unit = if self.unit != Unit::None {
                    self.unit.clone()
                } else {
                    other.unit.clone()
                };
                Ok(Value::number_with_unit(result, unit))
            }
            // Mixed: convert Decimal to Rational
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                let b_rat = Rational::from_decimal(*b);
                let result = a.clone() * b_rat;
                let unit = if self.unit != Unit::None {
                    self.unit.clone()
                } else {
                    other.unit.clone()
                };
                Ok(Value::rational_with_unit(result, unit))
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                let a_rat = Rational::from_decimal(*a);
                let result = a_rat * b.clone();
                let unit = if self.unit != Unit::None {
                    self.unit.clone()
                } else {
                    other.unit.clone()
                };
                Ok(Value::rational_with_unit(result, unit))
            }
            _ => Err(CalculatorError::InvalidOperation(format!(
                "Cannot multiply {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Divides two values.
    pub fn divide(&self, other: &Self) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational / Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                if b.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let result = a.clone() / b.clone();

                // Handle unit division
                let unit = match (&self.unit, &other.unit) {
                    (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Unit::None,
                    (unit, Unit::None) => unit.clone(),
                    (Unit::None, _) => Unit::None,
                    (u1, u2) if u1 == u2 => Unit::None,
                    _ => self.unit.clone(),
                };

                Ok(Value::rational_with_unit(result, unit))
            }
            // Number / Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                if b.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let result = a.checked_div(b).ok_or(CalculatorError::Overflow)?;

                // Handle unit division
                let unit = match (&self.unit, &other.unit) {
                    (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Unit::None,
                    (unit, Unit::None) => unit.clone(),
                    (Unit::None, _) => Unit::None,
                    (u1, u2) if u1 == u2 => Unit::None,
                    _ => self.unit.clone(),
                };

                Ok(Value::number_with_unit(result, unit))
            }
            // Mixed: convert Decimal to Rational for exact division
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                if b.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let b_rat = Rational::from_decimal(*b);
                let result = a.clone() / b_rat;

                let unit = match (&self.unit, &other.unit) {
                    (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Unit::None,
                    (unit, Unit::None) => unit.clone(),
                    (Unit::None, _) => Unit::None,
                    (u1, u2) if u1 == u2 => Unit::None,
                    _ => self.unit.clone(),
                };

                Ok(Value::rational_with_unit(result, unit))
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                if b.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let a_rat = Rational::from_decimal(*a);
                let result = a_rat / b.clone();

                let unit = match (&self.unit, &other.unit) {
                    (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Unit::None,
                    (unit, Unit::None) => unit.clone(),
                    (Unit::None, _) => Unit::None,
                    (u1, u2) if u1 == u2 => Unit::None,
                    _ => self.unit.clone(),
                };

                Ok(Value::rational_with_unit(result, unit))
            }
            (ValueKind::Duration { seconds }, ValueKind::Number(n)) => {
                if n.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let result_secs = (*seconds as f64) / n.to_f64();
                Ok(Value::duration(result_secs as i64))
            }
            (ValueKind::Duration { seconds }, ValueKind::Rational(r)) => {
                if r.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let result_secs = (*seconds as f64) / r.to_f64();
                Ok(Value::duration(result_secs as i64))
            }
            _ => Err(CalculatorError::InvalidOperation(format!(
                "Cannot divide {} by {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Negates the value.
    #[must_use]
    pub fn negate(&self) -> Self {
        match &self.kind {
            ValueKind::Number(n) => Value::number_with_unit(-*n, self.unit.clone()),
            ValueKind::Rational(r) => Value::rational_with_unit(-r.clone(), self.unit.clone()),
            ValueKind::Duration { seconds } => Value::duration(-seconds),
            _ => self.clone(),
        }
    }

    /// Returns the type name for error messages.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ValueKind::Number(_) => "number",
            ValueKind::Rational(_) => "number",
            ValueKind::DateTime(_) => "datetime",
            ValueKind::Duration { .. } => "duration",
            ValueKind::Boolean(_) => "boolean",
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
            ValueKind::DateTime(dt) => dt.to_string(),
            ValueKind::Duration { seconds } => format_duration(*seconds),
            ValueKind::Boolean(b) => b.to_string(),
        }
    }

    /// Returns true if this is a number (either Decimal or Rational).
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self.kind, ValueKind::Number(_) | ValueKind::Rational(_))
    }

    /// Returns the decimal value if this is a number.
    #[must_use]
    pub fn as_number(&self) -> Option<Decimal> {
        match &self.kind {
            ValueKind::Number(n) => Some(*n),
            ValueKind::Rational(r) => Some(r.to_decimal()),
            _ => None,
        }
    }

    /// Returns the decimal value if this is a number (alias for as_number).
    #[must_use]
    pub fn as_decimal(&self) -> Option<Decimal> {
        self.as_number()
    }

    /// Returns the rational value if this is a Rational.
    #[must_use]
    pub fn as_rational(&self) -> Option<&Rational> {
        match &self.kind {
            ValueKind::Rational(r) => Some(r),
            _ => None,
        }
    }

    /// Returns the fraction string representation if this is a Rational.
    #[must_use]
    pub fn to_fraction_string(&self) -> Option<String> {
        match &self.kind {
            ValueKind::Rational(r) => Some(r.to_fraction_string()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (ValueKind::Number(a), ValueKind::Number(b)) => a == b && self.unit == other.unit,
            (ValueKind::Rational(a), ValueKind::Rational(b)) => a == b && self.unit == other.unit,
            // Cross-compare: convert to f64 for approximate equality
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                (a.to_f64() - b.to_f64()).abs() < 1e-10 && self.unit == other.unit
            }
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                (a.to_f64() - b.to_f64()).abs() < 1e-10 && self.unit == other.unit
            }
            (ValueKind::DateTime(a), ValueKind::DateTime(b)) => a == b,
            (ValueKind::Duration { seconds: a }, ValueKind::Duration { seconds: b }) => a == b,
            (ValueKind::Boolean(a), ValueKind::Boolean(b)) => a == b,
            _ => false,
        }
    }
}

/// Formats a duration in seconds to a human-readable string.
fn format_duration(total_seconds: i64) -> String {
    let is_negative = total_seconds < 0;
    let total_seconds = total_seconds.abs();

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!(
            "{} second{}",
            seconds,
            if seconds == 1 { "" } else { "s" }
        ));
    }

    let result = parts.join(", ");
    if is_negative {
        format!("-{result}")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_value() {
        let v = Value::number(Decimal::new(42));
        assert_eq!(v.to_display_string(), "42");
    }

    #[test]
    fn test_currency_value() {
        let v = Value::currency(Decimal::new(100), "USD");
        assert_eq!(v.to_display_string(), "100 USD");
    }

    #[test]
    fn test_number_addition() {
        let a = Value::number(Decimal::new(2));
        let b = Value::number(Decimal::new(3));
        let db = CurrencyDatabase::new();
        let result = a.add(&b, &db).unwrap();
        assert_eq!(result.to_display_string(), "5");
    }

    #[test]
    fn test_rational_addition() {
        let a = Value::rational(Rational::new(1, 3));
        let b = Value::rational(Rational::new(1, 3));
        let db = CurrencyDatabase::new();
        let result = a.add(&b, &db).unwrap();
        // 1/3 + 1/3 = 2/3
        assert_eq!(result.to_fraction_string(), Some("2/3".to_string()));
    }

    #[test]
    fn test_currency_addition_same() {
        let a = Value::currency(Decimal::new(100), "USD");
        let b = Value::currency(Decimal::new(50), "USD");
        let db = CurrencyDatabase::new();
        let result = a.add(&b, &db).unwrap();
        assert_eq!(result.to_display_string(), "150 USD");
    }

    #[test]
    fn test_division_by_zero() {
        let a = Value::number(Decimal::new(10));
        let b = Value::number(Decimal::zero());
        let result = a.divide(&b);
        assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
    }

    #[test]
    fn test_rational_division_by_zero() {
        let a = Value::rational(Rational::from_integer(10));
        let b = Value::rational(Rational::zero());
        let result = a.divide(&b);
        assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
    }

    #[test]
    fn test_one_third_times_three() {
        // This is the key test for issue #21
        let one = Value::rational(Rational::from_integer(1));
        let three = Value::rational(Rational::from_integer(3));

        // 1 / 3
        let one_third = one.divide(&three).unwrap();

        // (1/3) * 3 = 1 (exact!)
        let result = one_third.multiply(&three).unwrap();
        assert_eq!(result.to_display_string(), "1");
    }

    #[test]
    fn test_two_thirds_times_three() {
        let two = Value::rational(Rational::from_integer(2));
        let three = Value::rational(Rational::from_integer(3));

        // 2 / 3
        let two_thirds = two.divide(&three).unwrap();

        // (2/3) * 3 = 2 (exact!)
        let result = two_thirds.multiply(&three).unwrap();
        assert_eq!(result.to_display_string(), "2");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0 seconds");
        assert_eq!(format_duration(1), "1 second");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(3661), "1 hour, 1 minute, 1 second");
        assert_eq!(format_duration(86400), "1 day");
    }

    #[test]
    fn test_datetime_subtraction() {
        let dt1 = Value::datetime(DateTime::parse("2026-01-27").unwrap());
        let dt2 = Value::datetime(DateTime::parse("2026-01-25").unwrap());
        let db = CurrencyDatabase::new();
        let result = dt1.subtract(&dt2, &db).unwrap();
        assert!(matches!(result.kind, ValueKind::Duration { .. }));
    }

    #[test]
    fn test_datetime_plus_duration() {
        let dt = Value::datetime(DateTime::parse("2026-01-25").unwrap());
        let dur = Value::duration(86400); // 1 day in seconds
        let db = CurrencyDatabase::new();
        let result = dt.add(&dur, &db).unwrap();
        assert!(matches!(result.kind, ValueKind::DateTime(_)));
    }

    #[test]
    fn test_duration_plus_datetime() {
        // This is the case from issue #8: duration + datetime
        let dur = Value::duration(86400); // 1 day in seconds
        let dt = Value::datetime(DateTime::parse("2026-01-25").unwrap());
        let db = CurrencyDatabase::new();
        let result = dur.add(&dt, &db).unwrap();
        assert!(matches!(result.kind, ValueKind::DateTime(_)));
    }

    #[test]
    fn test_issue_8_expression() {
        // Test the exact expression from issue #8:
        // (Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)
        let dt1 = Value::datetime(DateTime::parse("Jan 27, 8:59am UTC").unwrap());
        let dt2 = Value::datetime(DateTime::parse("Jan 25, 12:51pm UTC").unwrap());
        let dt3 = Value::datetime(DateTime::parse("Jan 25, 12:51pm UTC").unwrap());
        let db = CurrencyDatabase::new();

        // First: dt1 - dt2 = duration
        let duration = dt1.subtract(&dt2, &db).unwrap();
        assert!(matches!(duration.kind, ValueKind::Duration { .. }));

        // Second: duration + dt3 = datetime (this was failing before the fix)
        let result = duration.add(&dt3, &db).unwrap();
        assert!(matches!(result.kind, ValueKind::DateTime(_)));
    }
}
