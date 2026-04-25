//! Value type representing typed values with units.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::CalculatorError;
use crate::types::{CurrencyDatabase, DateTime, Decimal, DurationUnit, Rational, Unit};

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
        currency_db: &mut CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        self.add_at_date(other, currency_db, None)
    }

    /// Adds two values with optional date context for historical currency conversion.
    pub fn add_at_date(
        &self,
        other: &Self,
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
    ) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational + Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                self.add_rationals(a.clone(), b.clone(), other, currency_db, date)
            }
            // Number + Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.add_numbers(*a, *b, other, currency_db, date)
            }
            // Mixed: convert Decimal to Rational
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                let b_rat = Rational::from_decimal(*b);
                self.add_rationals(a.clone(), b_rat, other, currency_db, date)
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                let a_rat = Rational::from_decimal(*a);
                self.add_rationals(a_rat, b.clone(), other, currency_db, date)
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
            // DateTime + number-with-duration-unit (e.g. "now + 10 days")
            (ValueKind::DateTime(dt), ValueKind::Rational(r))
                if matches!(other.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = other.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        r.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
            }
            (ValueKind::DateTime(dt), ValueKind::Number(n))
                if matches!(other.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = other.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        n.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
            }
            // number-with-duration-unit + DateTime (commutative, e.g. "10 days + now")
            (ValueKind::Rational(r), ValueKind::DateTime(dt))
                if matches!(self.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = self.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        r.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
            }
            (ValueKind::Number(n), ValueKind::DateTime(dt))
                if matches!(self.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = self.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        n.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
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
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
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
                // Use historical rate if date is provided
                let converted = if let Some(dt) = date {
                    currency_db.convert_at_date(b_dec.to_f64(), c2, c1, dt)?
                } else {
                    currency_db.convert(b_dec.to_f64(), c2, c1)?
                };
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a_dec + converted_decimal, c1))
            }
            // Mass + different mass unit (convert to first unit's type)
            (Unit::Mass(m1), Unit::Mass(m2)) if m1 != m2 => {
                let a_val = a.to_f64();
                let b_val = b.to_f64();
                let b_converted = m2.convert(b_val, *m1);
                let result = Decimal::from_f64(a_val + b_converted);
                Ok(Value::number_with_unit(result, Unit::Mass(*m1)))
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
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::number(a + b)),
            (Unit::None, unit) | (unit, Unit::None) => {
                Ok(Value::number_with_unit(a + b, unit.clone()))
            }
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Ok(Value::currency(a + b, c1)),
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                // Convert c2 to c1, using historical rate if date is provided
                let converted = if let Some(dt) = date {
                    currency_db.convert_at_date(b.to_f64(), c2, c1, dt)?
                } else {
                    currency_db.convert(b.to_f64(), c2, c1)?
                };
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
        currency_db: &mut CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        self.subtract_at_date(other, currency_db, None)
    }

    /// Subtracts two values with optional date context for historical currency conversion.
    pub fn subtract_at_date(
        &self,
        other: &Self,
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
    ) -> Result<Self, CalculatorError> {
        match (&self.kind, &other.kind) {
            // Rational - Rational
            (ValueKind::Rational(a), ValueKind::Rational(b)) => {
                self.subtract_rationals(a.clone(), b.clone(), other, currency_db, date)
            }
            // Number - Number (legacy)
            (ValueKind::Number(a), ValueKind::Number(b)) => {
                self.subtract_numbers(*a, *b, other, currency_db, date)
            }
            // Mixed: convert Decimal to Rational
            (ValueKind::Rational(a), ValueKind::Number(b)) => {
                let b_rat = Rational::from_decimal(*b);
                self.subtract_rationals(a.clone(), b_rat, other, currency_db, date)
            }
            (ValueKind::Number(a), ValueKind::Rational(b)) => {
                let a_rat = Rational::from_decimal(*a);
                self.subtract_rationals(a_rat, b.clone(), other, currency_db, date)
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
            // DateTime - number-with-duration-unit (e.g. "now - 10 days")
            (ValueKind::DateTime(dt), ValueKind::Rational(r))
                if matches!(other.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = other.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        -r.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
            }
            (ValueKind::DateTime(dt), ValueKind::Number(n))
                if matches!(other.unit, Unit::Duration(_)) =>
            {
                if let Unit::Duration(dur_unit) = other.unit {
                    Ok(Value::datetime(add_calendar_months_or_duration(
                        dt,
                        dur_unit,
                        -n.to_f64(),
                    )))
                } else {
                    unreachable!()
                }
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
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
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
                // Use historical rate if date is provided
                let converted = if let Some(dt) = date {
                    currency_db.convert_at_date(b_dec.to_f64(), c2, c1, dt)?
                } else {
                    currency_db.convert(b_dec.to_f64(), c2, c1)?
                };
                let converted_decimal = Decimal::from_f64(converted);
                Ok(Value::currency(a_dec - converted_decimal, c1))
            }
            // Mass - different mass unit (convert to first unit's type)
            (Unit::Mass(m1), Unit::Mass(m2)) if m1 != m2 => {
                let a_val = a.to_f64();
                let b_val = b.to_f64();
                let b_converted = m2.convert(b_val, *m1);
                let result = Decimal::from_f64(a_val - b_converted);
                Ok(Value::number_with_unit(result, Unit::Mass(*m1)))
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
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, &other.unit) {
            (Unit::None, Unit::None) => Ok(Value::number(a - b)),
            (unit, Unit::None) => Ok(Value::number_with_unit(a - b, unit.clone())),
            (Unit::None, unit) => Ok(Value::number_with_unit(a - b, unit.clone())),
            (Unit::Currency(c1), Unit::Currency(c2)) if c1 == c2 => Ok(Value::currency(a - b, c1)),
            (Unit::Currency(c1), Unit::Currency(c2)) => {
                // Convert c2 to c1, using historical rate if date is provided
                let converted = if let Some(dt) = date {
                    currency_db.convert_at_date(b.to_f64(), c2, c1, dt)?
                } else {
                    currency_db.convert(b.to_f64(), c2, c1)?
                };
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

    /// Converts this value to the given unit.
    ///
    /// Supports conversion between data size units (KB, KiB, MB, MiB, etc.)
    /// and currency conversions (USD → EUR, etc.).
    pub fn convert_to_unit(
        &self,
        target_unit: &Unit,
        currency_db: &mut CurrencyDatabase,
    ) -> Result<Self, CalculatorError> {
        self.convert_to_unit_at_date(target_unit, currency_db, None)
    }

    /// Converts this value to the given unit, using a historical exchange rate if `date` is provided.
    pub fn convert_to_unit_at_date(
        &self,
        target_unit: &Unit,
        currency_db: &mut CurrencyDatabase,
        date: Option<&DateTime>,
    ) -> Result<Self, CalculatorError> {
        match (&self.unit, target_unit) {
            // Data size to data size conversion
            (Unit::DataSize(from), Unit::DataSize(to)) => {
                let value_f64 = self.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation(
                        "data size conversion requires a numeric value".into(),
                    )
                })?;
                let result = from.convert(value_f64.to_f64(), *to);
                Ok(Value::number_with_unit(
                    Decimal::from_f64(result),
                    Unit::DataSize(*to),
                ))
            }
            // Currency to currency conversion
            (Unit::Currency(from), Unit::Currency(to)) => {
                let amount = self.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation(
                        "currency conversion requires a numeric value".into(),
                    )
                })?;
                let converted = if let Some(dt) = date {
                    currency_db.convert_at_date(amount.to_f64(), from, to, dt)?
                } else {
                    currency_db.convert(amount.to_f64(), from, to)?
                };
                Ok(Value::currency(Decimal::from_f64(converted), to))
            }
            // Mass to mass conversion
            (Unit::Mass(from), Unit::Mass(to)) => {
                let value_f64 = self.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation(
                        "mass conversion requires a numeric value".into(),
                    )
                })?;
                let result = from.convert(value_f64.to_f64(), *to);
                Ok(Value::number_with_unit(
                    Decimal::from_f64(result),
                    Unit::Mass(*to),
                ))
            }
            // Dimensionless value: just apply the target unit (e.g. "5 as MB")
            (Unit::None, Unit::DataSize(_)) => {
                let value_f64 = self.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation(
                        "unit conversion requires a numeric value".into(),
                    )
                })?;
                Ok(Value::number_with_unit(value_f64, target_unit.clone()))
            }
            // Dimensionless value: just apply the mass target unit (e.g. "5 as kg")
            (Unit::None, Unit::Mass(_)) => {
                let value_f64 = self.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation(
                        "unit conversion requires a numeric value".into(),
                    )
                })?;
                Ok(Value::number_with_unit(value_f64, target_unit.clone()))
            }
            // DateTime timezone conversion (e.g., "6 PM GMT as MSK")
            (_, Unit::Timezone(tz_abbrev)) => {
                if let ValueKind::DateTime(dt) = &self.kind {
                    let target_offset =
                        DateTime::parse_tz_abbreviation(tz_abbrev).ok_or_else(|| {
                            CalculatorError::parse(format!("Unknown timezone: {tz_abbrev}"))
                        })?;
                    let converted = dt.with_timezone_offset(target_offset, tz_abbrev);
                    Ok(Value::datetime(converted))
                } else {
                    Err(CalculatorError::InvalidOperation(format!(
                        "Cannot convert {} to timezone {}; only DateTime values can be converted to a timezone",
                        self.to_display_string(),
                        tz_abbrev
                    )))
                }
            }
            (from_unit, to_unit) => Err(CalculatorError::InvalidOperation(format!(
                "Cannot convert {} to {}",
                from_unit.display_name(),
                to_unit.display_name()
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

    /// Alias for `as_number`.
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

    /// Converts this value to a Rational if numeric (clones Rational, converts Decimal).
    #[must_use]
    pub fn to_rational(&self) -> Option<Rational> {
        match &self.kind {
            ValueKind::Rational(r) => Some(r.clone()),
            ValueKind::Number(d) => Some(Rational::from_decimal(*d)),
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

/// Applies a signed duration to a `DateTime`, using calendar arithmetic for
/// months and years and second-based arithmetic for all other units.
///
/// `amount` is positive for addition and negative for subtraction.
fn add_calendar_months_or_duration(dt: &DateTime, unit: DurationUnit, amount: f64) -> DateTime {
    match unit {
        DurationUnit::Months => dt.add_calendar_months(amount as i32),
        DurationUnit::Years => dt.add_calendar_months((amount * 12.0) as i32),
        other => {
            let seconds = other.to_secs(amount.abs()) as i64;
            if amount >= 0.0 {
                dt.add_duration(seconds)
            } else {
                dt.add_duration(-seconds)
            }
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
mod tests;
