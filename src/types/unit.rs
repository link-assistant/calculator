//! Unit types for values.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a unit of measurement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Unit {
    /// No unit (dimensionless).
    #[default]
    None,
    /// Currency unit (e.g., USD, EUR).
    Currency(String),
    /// Time duration unit.
    Duration(DurationUnit),
    /// Custom unit.
    Custom(String),
}

/// Duration units for time calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

impl Unit {
    /// Creates a currency unit.
    pub fn currency(code: &str) -> Self {
        Self::Currency(code.to_uppercase())
    }

    /// Checks if the unit is a currency.
    #[must_use]
    pub fn is_currency(&self) -> bool {
        matches!(self, Self::Currency(_))
    }

    /// Checks if the unit is a duration.
    #[must_use]
    pub fn is_duration(&self) -> bool {
        matches!(self, Self::Duration(_))
    }

    /// Checks if two units are compatible for the given operation.
    #[must_use]
    pub fn is_compatible_for_operation(&self, other: &Self, op: &str) -> bool {
        match (self, other) {
            (Self::None, _) | (_, Self::None) => true,
            (Self::Currency(a), Self::Currency(b)) => {
                // Currencies can be added/subtracted (with conversion)
                // but not multiplied/divided together
                match op {
                    "+" | "-" => true,
                    "*" | "/" => a == b || matches!(other, Self::None),
                    _ => false,
                }
            }
            (Self::Duration(a), Self::Duration(b)) => {
                // Durations can be added/subtracted if same unit
                // For different units, we'd need conversion
                match op {
                    "+" | "-" => a == b,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Returns the display name of the unit.
    #[must_use]
    pub fn display_name(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Currency(code) => code.clone(),
            Self::Duration(d) => d.to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Currency(code) => write!(f, "{code}"),
            Self::Duration(d) => write!(f, "{d}"),
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl fmt::Display for DurationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
            Self::Minutes => "min",
            Self::Hours => "h",
            Self::Days => "d",
            Self::Weeks => "w",
            Self::Months => "mo",
            Self::Years => "y",
        };
        write!(f, "{s}")
    }
}

impl DurationUnit {
    /// Converts a duration to seconds.
    #[must_use]
    pub fn to_secs(self, value: f64) -> f64 {
        match self {
            Self::Milliseconds => value / 1000.0,
            Self::Seconds => value,
            Self::Minutes => value * 60.0,
            Self::Hours => value * 3600.0,
            Self::Days => value * 86400.0,
            Self::Weeks => value * 604_800.0,
            Self::Months => value * 2_592_000.0, // 30 days approximation
            Self::Years => value * 31_536_000.0, // 365 days approximation
        }
    }

    /// Converts seconds to this duration unit.
    #[must_use]
    pub fn secs_to_unit(self, seconds: f64) -> f64 {
        match self {
            Self::Milliseconds => seconds * 1000.0,
            Self::Seconds => seconds,
            Self::Minutes => seconds / 60.0,
            Self::Hours => seconds / 3600.0,
            Self::Days => seconds / 86400.0,
            Self::Weeks => seconds / 604_800.0,
            Self::Months => seconds / 2_592_000.0,
            Self::Years => seconds / 31_536_000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_display() {
        assert_eq!(Unit::None.to_string(), "");
        assert_eq!(Unit::currency("USD").to_string(), "USD");
        assert_eq!(Unit::Duration(DurationUnit::Hours).to_string(), "h");
    }

    #[test]
    fn test_unit_compatibility() {
        let usd = Unit::currency("USD");
        let eur = Unit::currency("EUR");
        let none = Unit::None;

        assert!(usd.is_compatible_for_operation(&eur, "+"));
        assert!(usd.is_compatible_for_operation(&none, "*"));
        assert!(!usd.is_compatible_for_operation(&eur, "*"));
    }

    #[test]
    fn test_duration_conversion() {
        assert_eq!(DurationUnit::Minutes.to_secs(2.0), 120.0);
        assert_eq!(DurationUnit::Hours.secs_to_unit(3600.0), 1.0);
    }
}
