//! Decimal number type for precise calculations.

use rust_decimal::Decimal as RustDecimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

/// A decimal number with arbitrary precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Decimal(RustDecimal);

impl Decimal {
    /// Creates a new Decimal from an integer.
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self(RustDecimal::from(value))
    }

    /// Creates a new Decimal from a float (may lose precision).
    #[must_use]
    pub fn from_f64(value: f64) -> Option<Self> {
        RustDecimal::try_from(value).ok().map(Self)
    }

    /// Returns zero.
    #[must_use]
    pub fn zero() -> Self {
        Self(RustDecimal::ZERO)
    }

    /// Returns one.
    #[must_use]
    pub fn one() -> Self {
        Self(RustDecimal::ONE)
    }

    /// Checks if the value is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Checks if the value is negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative() && !self.is_zero()
    }

    /// Returns the absolute value.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    /// Converts to f64 (may lose precision).
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.0.to_f64().unwrap_or(0.0)
    }

    /// Rounds to the specified number of decimal places.
    #[must_use]
    pub fn round(&self, dp: u32) -> Self {
        Self(self.0.round_dp(dp))
    }

    /// Normalizes the decimal (removes trailing zeros).
    #[must_use]
    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }

    /// Checked division that returns None on division by zero.
    #[must_use]
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            self.0.checked_div(other.0).map(Self)
        }
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let normalized = self.0.normalize();
        write!(f, "{normalized}")
    }
}

impl FromStr for Decimal {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RustDecimal::from_str(s).map(Self)
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<i32> for Decimal {
    fn from(value: i32) -> Self {
        Self(RustDecimal::from(value))
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_creation() {
        let d = Decimal::new(42);
        assert_eq!(d.to_string(), "42");
    }

    #[test]
    fn test_decimal_from_str() {
        let d: Decimal = "3.14159".parse().unwrap();
        assert!(d.to_string().starts_with("3.14"));
    }

    #[test]
    fn test_decimal_operations() {
        let a = Decimal::new(10);
        let b = Decimal::new(3);

        assert_eq!((a + b).to_string(), "13");
        assert_eq!((a - b).to_string(), "7");
        assert_eq!((a * b).to_string(), "30");
    }

    #[test]
    fn test_decimal_division() {
        let a = Decimal::new(10);
        let b = Decimal::new(4);
        assert_eq!((a / b).to_string(), "2.5");
    }

    #[test]
    fn test_decimal_checked_div() {
        let a = Decimal::new(10);
        let zero = Decimal::zero();
        assert!(a.checked_div(&zero).is_none());
    }

    #[test]
    fn test_decimal_is_negative() {
        let pos = Decimal::new(5);
        let neg = Decimal::new(-5);
        let zero = Decimal::zero();

        assert!(!pos.is_negative());
        assert!(neg.is_negative());
        assert!(!zero.is_negative());
    }
}
