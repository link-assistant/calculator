use super::Value;
use crate::error::CalculatorError;
use crate::types::{DateTime, DurationUnit, Rational, Unit};

/// Formats a duration in seconds to a human-readable string.
pub(super) fn format_duration(total_seconds: i64) -> String {
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

/// Divides compatible duration-unit values as a unitless ratio.
pub(super) fn divide_duration_units(
    left: &Value,
    right: &Value,
) -> Result<Option<Value>, CalculatorError> {
    let (Unit::Duration(left_unit), Unit::Duration(right_unit)) = (&left.unit, &right.unit) else {
        return Ok(None);
    };

    let left_amount = left.to_rational().ok_or_else(|| {
        CalculatorError::InvalidOperation("duration division requires numeric values".into())
    })?;
    let right_amount = right.to_rational().ok_or_else(|| {
        CalculatorError::InvalidOperation("duration division requires numeric values".into())
    })?;

    let left_seconds = left_amount * duration_unit_seconds(*left_unit);
    let right_seconds = right_amount * duration_unit_seconds(*right_unit);
    if right_seconds.is_zero() {
        return Err(CalculatorError::DivisionByZero);
    }

    Ok(Some(Value::rational(left_seconds / right_seconds)))
}

/// Applies a signed duration to a `DateTime`, using calendar arithmetic for
/// months and years and second-based arithmetic for all other units.
///
/// `amount` is positive for addition and negative for subtraction.
pub(super) fn add_calendar_months_or_duration(
    dt: &DateTime,
    unit: DurationUnit,
    amount: f64,
) -> DateTime {
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

fn duration_unit_seconds(unit: DurationUnit) -> Rational {
    match unit {
        DurationUnit::Milliseconds => Rational::new(1, 1000),
        DurationUnit::Seconds => Rational::one(),
        DurationUnit::Minutes => Rational::from_integer(60),
        DurationUnit::Hours => Rational::from_integer(3600),
        DurationUnit::Days => Rational::from_integer(86_400),
        DurationUnit::Weeks => Rational::from_integer(604_800),
        DurationUnit::Months => Rational::from_integer(2_592_000),
        DurationUnit::Years => Rational::from_integer(31_536_000),
    }
}
