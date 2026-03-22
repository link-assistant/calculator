//! Unit types for values.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a unit of measurement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Unit {
    /// No unit (dimensionless).
    #[default]
    None,
    /// Currency unit (e.g., USD, EUR, TON crypto).
    Currency(String),
    /// Time duration unit.
    Duration(DurationUnit),
    /// Data size unit (e.g., KB, MiB, GB).
    DataSize(DataSizeUnit),
    /// Mass/weight unit (e.g., kg, ton, lb).
    Mass(MassUnit),
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

/// Data size units — both decimal (SI, powers of 1000) and binary (IEC, powers of 1024).
///
/// SI units follow IEC 80000-13 / the International System of Units.
/// Binary units follow IEC 80000-13 (published January 29, 1999).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSizeUnit {
    // ---- Bits (smallest common unit) ----
    /// 1 bit
    Bit,
    // ---- Decimal (SI) bit units ----
    /// 1 kilobit = 1,000 bits
    Kilobit,
    /// 1 megabit = 1,000,000 bits
    Megabit,
    /// 1 gigabit = 1,000,000,000 bits
    Gigabit,
    /// 1 terabit = 1,000,000,000,000 bits
    Terabit,
    /// 1 petabit = 1,000,000,000,000,000 bits
    Petabit,
    // ---- Binary (IEC) bit units ----
    /// 1 kibibit = 1,024 bits
    Kibibit,
    /// 1 mebibit = 1,048,576 bits
    Mebibit,
    /// 1 gibibit = 1,073,741,824 bits
    Gibibit,
    /// 1 tebibit = 1,099,511,627,776 bits
    Tebibit,
    /// 1 pebibit = 1,125,899,906,842,624 bits
    Pebibit,
    // ---- Bytes ----
    /// 1 byte = 8 bits
    Byte,
    // ---- Decimal (SI) byte units ----
    /// 1 kilobyte = 1,000 bytes (SI)
    Kilobyte,
    /// 1 megabyte = 1,000,000 bytes (SI)
    Megabyte,
    /// 1 gigabyte = 1,000,000,000 bytes (SI)
    Gigabyte,
    /// 1 terabyte = 1,000,000,000,000 bytes (SI)
    Terabyte,
    /// 1 petabyte = 1,000,000,000,000,000 bytes (SI)
    Petabyte,
    // ---- Binary (IEC) byte units ----
    /// 1 kibibyte = 1,024 bytes (IEC)
    Kibibyte,
    /// 1 mebibyte = 1,048,576 bytes (IEC)
    Mebibyte,
    /// 1 gibibyte = 1,073,741,824 bytes (IEC)
    Gibibyte,
    /// 1 tebibyte = 1,099,511,627,776 bytes (IEC)
    Tebibyte,
    /// 1 pebibyte = 1,125,899,906,842,624 bytes (IEC)
    Pebibyte,
}

impl DataSizeUnit {
    /// Returns the number of bits this unit represents.
    /// Uses exact integer arithmetic via u128 to avoid floating-point precision errors.
    #[must_use]
    pub const fn bits(self) -> u128 {
        match self {
            Self::Bit => 1,
            // Decimal bit units (powers of 1000)
            Self::Kilobit => 1_000,
            Self::Megabit => 1_000_000,
            Self::Gigabit => 1_000_000_000,
            Self::Terabit => 1_000_000_000_000,
            Self::Petabit => 1_000_000_000_000_000,
            // Binary bit units (powers of 1024)
            Self::Kibibit => 1_024,
            Self::Mebibit => 1_048_576,
            Self::Gibibit => 1_073_741_824,
            Self::Tebibit => 1_099_511_627_776,
            Self::Pebibit => 1_125_899_906_842_624,
            // Byte units: multiply byte count by 8
            Self::Byte => 8,
            // Decimal byte units
            Self::Kilobyte => 8_000,
            Self::Megabyte => 8_000_000,
            Self::Gigabyte => 8_000_000_000,
            Self::Terabyte => 8_000_000_000_000,
            Self::Petabyte => 8_000_000_000_000_000,
            // Binary byte units
            Self::Kibibyte => 8_192,
            Self::Mebibyte => 8_388_608,
            Self::Gibibyte => 8_589_934_592,
            Self::Tebibyte => 8_796_093_022_208,
            Self::Pebibyte => 9_007_199_254_740_992,
        }
    }

    /// Converts a value from this unit to another data size unit.
    ///
    /// Conversion goes through bits as the canonical base unit.
    #[must_use]
    pub fn convert(self, value: f64, to: Self) -> f64 {
        let from_bits = self.bits() as f64;
        let to_bits = to.bits() as f64;
        value * from_bits / to_bits
    }

    /// Returns the standard abbreviation for this unit.
    #[must_use]
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Bit => "b",
            Self::Kilobit => "Kb",
            Self::Megabit => "Mb",
            Self::Gigabit => "Gb",
            Self::Terabit => "Tb",
            Self::Petabit => "Pb",
            Self::Kibibit => "Kib",
            Self::Mebibit => "Mib",
            Self::Gibibit => "Gib",
            Self::Tebibit => "Tib",
            Self::Pebibit => "Pib",
            Self::Byte => "B",
            Self::Kilobyte => "KB",
            Self::Megabyte => "MB",
            Self::Gigabyte => "GB",
            Self::Terabyte => "TB",
            Self::Petabyte => "PB",
            Self::Kibibyte => "KiB",
            Self::Mebibyte => "MiB",
            Self::Gibibyte => "GiB",
            Self::Tebibyte => "TiB",
            Self::Pebibyte => "PiB",
        }
    }

    /// Parses a string into a `DataSizeUnit`, returning `None` if not recognized.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            // Bits
            "b" | "bit" | "bits" => Some(Self::Bit),
            // Decimal bit units
            "Kb" | "kbit" | "kilobit" | "kilobits" => Some(Self::Kilobit),
            "Mb" | "mbit" | "megabit" | "megabits" => Some(Self::Megabit),
            "Gb" | "gbit" | "gigabit" | "gigabits" => Some(Self::Gigabit),
            "Tb" | "tbit" | "terabit" | "terabits" => Some(Self::Terabit),
            "Pb" | "pbit" | "petabit" | "petabits" => Some(Self::Petabit),
            // Binary bit units
            "Kib" | "kibit" | "kibibit" | "kibibits" => Some(Self::Kibibit),
            "Mib" | "mibit" | "mebibit" | "mebibits" => Some(Self::Mebibit),
            "Gib" | "gibit" | "gibibit" | "gibibits" => Some(Self::Gibibit),
            "Tib" | "tibit" | "tebibit" | "tebibits" => Some(Self::Tebibit),
            "Pib" | "pibit" | "pebibit" | "pebibits" => Some(Self::Pebibit),
            // Bytes
            "B" | "byte" | "bytes" => Some(Self::Byte),
            // Decimal byte units
            "KB" | "kB" | "kilobyte" | "kilobytes" => Some(Self::Kilobyte),
            "MB" | "megabyte" | "megabytes" => Some(Self::Megabyte),
            "GB" | "gigabyte" | "gigabytes" => Some(Self::Gigabyte),
            "TB" | "terabyte" | "terabytes" => Some(Self::Terabyte),
            "PB" | "petabyte" | "petabytes" => Some(Self::Petabyte),
            // Binary byte units
            "KiB" | "kibibyte" | "kibibytes" => Some(Self::Kibibyte),
            "MiB" | "mebibyte" | "mebibytes" => Some(Self::Mebibyte),
            "GiB" | "gibibyte" | "gibibytes" => Some(Self::Gibibyte),
            "TiB" | "tebibyte" | "tebibytes" => Some(Self::Tebibyte),
            "PiB" | "pebibyte" | "pebibytes" => Some(Self::Pebibyte),
            _ => None,
        }
    }
}

/// Mass/weight units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MassUnit {
    /// 1 milligram = 0.001 grams
    Milligram,
    /// 1 gram
    Gram,
    /// 1 kilogram = 1000 grams
    Kilogram,
    /// 1 metric ton = 1000 kilograms
    MetricTon,
    /// 1 pound ≈ 453.592 grams
    Pound,
    /// 1 ounce ≈ 28.3495 grams
    Ounce,
}

impl MassUnit {
    /// Returns the number of grams this unit represents (as f64).
    #[must_use]
    pub fn grams(self) -> f64 {
        match self {
            Self::Milligram => 0.001,
            Self::Gram => 1.0,
            Self::Kilogram => 1000.0,
            Self::MetricTon => 1_000_000.0,
            Self::Pound => 453.592_37,
            Self::Ounce => 28.349_523,
        }
    }

    /// Converts a value from this unit to another mass unit.
    #[must_use]
    pub fn convert(self, value: f64, to: Self) -> f64 {
        value * self.grams() / to.grams()
    }

    /// Returns the standard abbreviation for this unit.
    #[must_use]
    pub const fn abbreviation(self) -> &'static str {
        match self {
            Self::Milligram => "mg",
            Self::Gram => "g",
            Self::Kilogram => "kg",
            Self::MetricTon => "t",
            Self::Pound => "lb",
            Self::Ounce => "oz",
        }
    }

    /// Parses a string into a `MassUnit`, returning `None` if not recognized.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mg" | "milligram" | "milligrams" => Some(Self::Milligram),
            "g" | "gram" | "grams" => Some(Self::Gram),
            "kg" | "kgs" | "kilogram" | "kilograms" => Some(Self::Kilogram),
            // Metric ton: includes "ton" (singular) which is ambiguous with TON cryptocurrency.
            // The ambiguity is resolved at a higher level (NumberGrammar::parse_unit_with_alternatives)
            // which detects when both mass and currency interpretations are valid and surfaces
            // them as alternative interpretations to the user.
            "t" | "ton" | "tons" | "tonne" | "tonnes" | "metric_ton" | "metric_tons" => {
                Some(Self::MetricTon)
            }
            "lb" | "lbs" | "pound" | "pounds" => Some(Self::Pound),
            "oz" | "ounce" | "ounces" => Some(Self::Ounce),
            _ => None,
        }
    }
}

impl std::fmt::Display for MassUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

impl Unit {
    /// Creates a currency unit.
    pub fn currency(code: &str) -> Self {
        Self::Currency(code.to_uppercase())
    }

    /// Creates a data size unit.
    #[must_use]
    pub const fn data_size(unit: DataSizeUnit) -> Self {
        Self::DataSize(unit)
    }

    /// Creates a mass unit.
    #[must_use]
    pub const fn mass(unit: MassUnit) -> Self {
        Self::Mass(unit)
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

    /// Checks if the unit is a data size unit.
    #[must_use]
    pub fn is_data_size(&self) -> bool {
        matches!(self, Self::DataSize(_))
    }

    /// Checks if the unit is a mass unit.
    #[must_use]
    pub fn is_mass(&self) -> bool {
        matches!(self, Self::Mass(_))
    }

    /// Checks if two units are in the same category (both currencies, both mass, etc.).
    ///
    /// `Unit::None` is treated as compatible with any category.
    #[must_use]
    pub fn is_same_category(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::None, _)
                | (_, Self::None)
                | (Self::Currency(_), Self::Currency(_))
                | (Self::Duration(_), Self::Duration(_))
                | (Self::DataSize(_), Self::DataSize(_))
                | (Self::Mass(_), Self::Mass(_))
                | (Self::Custom(_), Self::Custom(_))
        )
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
            (Self::DataSize(_), Self::DataSize(_)) => {
                // Data sizes can be added/subtracted (with conversion between units)
                matches!(op, "+" | "-")
            }
            (Self::Mass(_), Self::Mass(_)) => {
                // Mass units can be added/subtracted (with conversion between units)
                matches!(op, "+" | "-")
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
            Self::DataSize(d) => d.abbreviation().to_string(),
            Self::Mass(m) => m.abbreviation().to_string(),
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
            Self::DataSize(d) => write!(f, "{d}"),
            Self::Mass(m) => write!(f, "{m}"),
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

impl fmt::Display for DataSizeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

impl DurationUnit {
    /// Parses a string into a `DurationUnit`, returning `None` if not recognized.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ms" | "millisecond" | "milliseconds" => Some(Self::Milliseconds),
            "s" | "sec" | "secs" | "second" | "seconds" => Some(Self::Seconds),
            "min" | "mins" | "minute" | "minutes" => Some(Self::Minutes),
            "h" | "hr" | "hrs" | "hour" | "hours" => Some(Self::Hours),
            "d" | "day" | "days" => Some(Self::Days),
            "w" | "week" | "weeks" => Some(Self::Weeks),
            "mo" | "month" | "months" => Some(Self::Months),
            "y" | "yr" | "yrs" | "year" | "years" => Some(Self::Years),
            _ => None,
        }
    }

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
    #[allow(clippy::float_cmp)]
    fn test_duration_conversion() {
        assert_eq!(DurationUnit::Minutes.to_secs(2.0), 120.0);
        assert_eq!(DurationUnit::Hours.secs_to_unit(3600.0), 1.0);
    }

    #[test]
    fn test_data_size_unit_display() {
        assert_eq!(DataSizeUnit::Kilobyte.to_string(), "KB");
        assert_eq!(DataSizeUnit::Kibibyte.to_string(), "KiB");
        assert_eq!(DataSizeUnit::Megabyte.to_string(), "MB");
        assert_eq!(DataSizeUnit::Mebibyte.to_string(), "MiB");
        assert_eq!(Unit::DataSize(DataSizeUnit::Kilobyte).to_string(), "KB");
    }

    #[test]
    fn test_data_size_unit_parse() {
        assert_eq!(DataSizeUnit::parse("KB"), Some(DataSizeUnit::Kilobyte));
        assert_eq!(DataSizeUnit::parse("KiB"), Some(DataSizeUnit::Kibibyte));
        assert_eq!(DataSizeUnit::parse("MB"), Some(DataSizeUnit::Megabyte));
        assert_eq!(DataSizeUnit::parse("MiB"), Some(DataSizeUnit::Mebibyte));
        assert_eq!(
            DataSizeUnit::parse("mebibytes"),
            Some(DataSizeUnit::Mebibyte)
        );
        assert_eq!(
            DataSizeUnit::parse("kilobytes"),
            Some(DataSizeUnit::Kilobyte)
        );
        assert_eq!(DataSizeUnit::parse("invalid"), None);
        assert_eq!(DataSizeUnit::parse("USD"), None);
    }

    #[test]
    fn test_data_size_bits() {
        assert_eq!(DataSizeUnit::Byte.bits(), 8);
        assert_eq!(DataSizeUnit::Kilobyte.bits(), 8_000);
        assert_eq!(DataSizeUnit::Kibibyte.bits(), 8_192);
        assert_eq!(DataSizeUnit::Megabyte.bits(), 8_000_000);
        assert_eq!(DataSizeUnit::Mebibyte.bits(), 8_388_608);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_data_size_conversion_within_si() {
        // 741 KB -> MB = 0.741 MB (exact)
        let result = DataSizeUnit::Kilobyte.convert(741.0, DataSizeUnit::Megabyte);
        assert!((result - 0.741).abs() < 1e-10);
    }

    #[test]
    fn test_data_size_conversion_within_iec() {
        // 741 KiB -> MiB = 741 / 1024 ≈ 0.72363281...
        let result = DataSizeUnit::Kibibyte.convert(741.0, DataSizeUnit::Mebibyte);
        let expected = 741.0 / 1024.0;
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_data_size_conversion_cross_standard() {
        // 741 KB -> MiB = 741_000 / 1_048_576 ≈ 0.706863...
        let result = DataSizeUnit::Kilobyte.convert(741.0, DataSizeUnit::Mebibyte);
        let expected = 741_000.0 / 1_048_576.0;
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_data_size_unit_compatibility() {
        let kb = Unit::DataSize(DataSizeUnit::Kilobyte);
        let mib = Unit::DataSize(DataSizeUnit::Mebibyte);
        assert!(kb.is_compatible_for_operation(&mib, "+"));
        assert!(kb.is_compatible_for_operation(&mib, "-"));
        assert!(!kb.is_compatible_for_operation(&mib, "*"));
    }
}
