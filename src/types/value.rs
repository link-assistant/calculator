//! Value type representing typed values with units.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::CalculatorError;
use crate::types::{CurrencyDatabase, DateTime, Decimal, Unit};

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
    /// A decimal number.
    Number(Decimal),
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
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.add_numbers(*a, *b, other, currency_db)
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
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.subtract_numbers(*a, *b, other, currency_db)
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
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                let result = *a * *b;
                // Keep the unit if one side has a unit
                let unit = if self.unit != Unit::None {
                    self.unit.clone()
                } else {
                    other.unit.clone()
                };
                Ok(Value::number_with_unit(result, unit))
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
            (ValueKind::Duration { seconds }, ValueKind::Number(n)) => {
                if n.is_zero() {
                    return Err(CalculatorError::DivisionByZero);
                }
                let result_secs = (*seconds as f64) / n.to_f64();
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
            ValueKind::Duration { seconds } => Value::duration(-seconds),
            _ => self.clone(),
        }
    }

    /// Returns the type name for error messages.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ValueKind::Number(_) => "number",
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
            ValueKind::DateTime(dt) => dt.to_string(),
            ValueKind::Duration { seconds } => format_duration(*seconds),
            ValueKind::Boolean(b) => b.to_string(),
        }
    }

    /// Returns true if this is a number.
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self.kind, ValueKind::Number(_))
    }

    /// Returns the decimal value if this is a number.
    #[must_use]
    pub fn as_number(&self) -> Option<Decimal> {
        match &self.kind {
            ValueKind::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the decimal value if this is a number (alias for as_number).
    #[must_use]
    pub fn as_decimal(&self) -> Option<Decimal> {
        self.as_number()
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
