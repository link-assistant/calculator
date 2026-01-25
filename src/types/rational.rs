//! Rational number type for exact fractional arithmetic.
//!
//! This module provides a `Rational` type that represents exact fractions
//! using numerator/denominator pairs. This allows for perfect precision
//! in calculations like `(1/3)*3 = 1` instead of `0.9999...`.

use num_rational::Ratio;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

use crate::types::Decimal;

/// A rational number represented as a fraction (numerator/denominator).
///
/// This type maintains exact precision by storing the numerator and denominator
/// separately and only performing the division when displaying or converting
/// to a decimal approximation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rational {
    /// The underlying rational number.
    #[serde(serialize_with = "serialize_ratio", deserialize_with = "deserialize_ratio")]
    inner: Ratio<i128>,
}

fn serialize_ratio<S>(ratio: &Ratio<i128>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Serialize as a tuple (numerator, denominator)
    let tuple = (*ratio.numer(), *ratio.denom());
    tuple.serialize(serializer)
}

fn deserialize_ratio<'de, D>(deserializer: D) -> Result<Ratio<i128>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (numer, denom): (i128, i128) = Deserialize::deserialize(deserializer)?;
    Ok(Ratio::new(numer, denom))
}

impl Rational {
    /// Creates a new rational number from an integer.
    #[must_use]
    pub fn from_integer(n: i128) -> Self {
        Self {
            inner: Ratio::from_integer(n),
        }
    }

    /// Creates a new rational number from numerator and denominator.
    ///
    /// The fraction is automatically reduced to lowest terms.
    ///
    /// # Panics
    ///
    /// Panics if denominator is zero.
    #[must_use]
    pub fn new(numer: i128, denom: i128) -> Self {
        assert!(denom != 0, "Denominator cannot be zero");
        Self {
            inner: Ratio::new(numer, denom),
        }
    }

    /// Creates a rational number from a Decimal value.
    ///
    /// This converts the decimal to a fraction by using powers of 10.
    #[must_use]
    pub fn from_decimal(d: Decimal) -> Self {
        // Parse the decimal string to extract integer and fractional parts
        let s = d.to_string();

        if let Some(dot_pos) = s.find('.') {
            // Has fractional part
            let sign = if s.starts_with('-') { -1i128 } else { 1i128 };
            let s_abs = s.trim_start_matches('-');
            let dot_pos_abs = if s.starts_with('-') { dot_pos - 1 } else { dot_pos };

            let int_part = &s_abs[..dot_pos_abs];
            let frac_part = &s_abs[dot_pos_abs + 1..];

            // Calculate denominator based on decimal places
            let decimal_places = frac_part.len() as u32;
            let denom = 10i128.pow(decimal_places);

            // Parse parts
            let int_val: i128 = if int_part.is_empty() {
                0
            } else {
                int_part.parse().unwrap_or(0)
            };
            let frac_val: i128 = if frac_part.is_empty() {
                0
            } else {
                frac_part.parse().unwrap_or(0)
            };

            let numer = sign * (int_val * denom + frac_val);
            Self::new(numer, denom)
        } else {
            // Integer value
            let n: i128 = s.parse().unwrap_or(0);
            Self::from_integer(n)
        }
    }

    /// Creates a rational number from an f64.
    ///
    /// This uses a rational approximation.
    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        if value.is_nan() || value.is_infinite() {
            return Self::from_integer(0);
        }

        // Use continued fraction expansion for better precision
        let (numer, denom) = Self::f64_to_rational_continued_fraction(value, 1_000_000_000);
        Self::new(numer, denom)
    }

    /// Converts an f64 to a rational using continued fraction expansion.
    fn f64_to_rational_continued_fraction(value: f64, max_denom: i128) -> (i128, i128) {
        if value == 0.0 {
            return (0, 1);
        }

        let sign = if value < 0.0 { -1 } else { 1 };
        let value = value.abs();

        let mut h_prev = 0i128;
        let mut k_prev = 1i128;
        let mut h_curr = 1i128;
        let mut k_curr = 0i128;

        let mut x = value;

        for _ in 0..50 {
            let a = x.floor() as i128;

            let h_next = a.saturating_mul(h_curr).saturating_add(h_prev);
            let k_next = a.saturating_mul(k_curr).saturating_add(k_prev);

            if k_next > max_denom {
                break;
            }

            h_prev = h_curr;
            k_prev = k_curr;
            h_curr = h_next;
            k_curr = k_next;

            let frac = x - (a as f64);
            if frac.abs() < 1e-15 {
                break;
            }

            x = 1.0 / frac;
        }

        (sign * h_curr, k_curr)
    }

    /// Returns the numerator.
    #[must_use]
    pub fn numer(&self) -> i128 {
        *self.inner.numer()
    }

    /// Returns the denominator.
    #[must_use]
    pub fn denom(&self) -> i128 {
        *self.inner.denom()
    }

    /// Returns true if this is an integer (denominator is 1).
    #[must_use]
    pub fn is_integer(&self) -> bool {
        self.inner.is_integer()
    }

    /// Returns true if this is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    /// Returns true if this is negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.inner.is_negative()
    }

    /// Returns the absolute value.
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            inner: self.inner.abs(),
        }
    }

    /// Converts to f64 (may lose precision).
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        (*self.inner.numer() as f64) / (*self.inner.denom() as f64)
    }

    /// Converts to a Decimal.
    #[must_use]
    pub fn to_decimal(&self) -> Decimal {
        if self.is_integer() {
            Decimal::new(self.numer() as i64)
        } else {
            Decimal::from_f64(self.to_f64())
        }
    }

    /// Returns zero.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            inner: Ratio::zero(),
        }
    }

    /// Returns one.
    #[must_use]
    pub fn one() -> Self {
        Self {
            inner: Ratio::one(),
        }
    }

    /// Checked division that returns None on division by zero.
    #[must_use]
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            Some(Self {
                inner: self.inner.clone() / other.inner.clone(),
            })
        }
    }

    /// Converts the rational to a display string.
    ///
    /// If the rational is an integer, returns just the integer.
    /// Otherwise, returns the decimal representation.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        if self.is_integer() {
            self.numer().to_string()
        } else {
            // Check if this is a simple repeating decimal that should show as integer
            // For example, 0.9999... should show as 1
            let approx = self.to_decimal();
            approx.normalize().to_string()
        }
    }

    /// Returns a fractional representation (e.g., "1/3").
    #[must_use]
    pub fn to_fraction_string(&self) -> String {
        if self.is_integer() {
            self.numer().to_string()
        } else {
            format!("{}/{}", self.numer(), self.denom())
        }
    }

    /// Detects if the decimal representation is a repeating decimal
    /// and returns the notation (e.g., "0.3̅" or "0.(3)").
    ///
    /// Returns None if not a repeating decimal or if detection failed.
    #[must_use]
    pub fn to_repeating_decimal_notation(&self) -> Option<RepeatingDecimal> {
        if self.is_integer() {
            return None;
        }

        let numer = self.numer().abs();
        let denom = self.denom().abs();
        let is_negative = self.is_negative();

        // Use Floyd's algorithm to detect the repeating pattern
        detect_repeating_decimal(numer as u128, denom as u128, is_negative)
    }
}

impl Default for Rational {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl FromStr for Rational {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try parsing as a fraction first (e.g., "1/3")
        if let Some((numer_str, denom_str)) = s.split_once('/') {
            let numer: i128 = numer_str.trim().parse().map_err(|e| format!("{e}"))?;
            let denom: i128 = denom_str.trim().parse().map_err(|e| format!("{e}"))?;
            if denom == 0 {
                return Err("Denominator cannot be zero".to_string());
            }
            return Ok(Self::new(numer, denom));
        }

        // Try parsing as a decimal
        let d: Decimal = s.parse().map_err(|e| format!("{e}"))?;
        Ok(Self::from_decimal(d))
    }
}

impl From<i64> for Rational {
    fn from(value: i64) -> Self {
        Self::from_integer(value as i128)
    }
}

impl From<i32> for Rational {
    fn from(value: i32) -> Self {
        Self::from_integer(value as i128)
    }
}

impl From<Decimal> for Rational {
    fn from(value: Decimal) -> Self {
        Self::from_decimal(value)
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            inner: self.inner + other.inner,
        }
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            inner: self.inner - other.inner,
        }
    }
}

impl Mul for Rational {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            inner: self.inner * other.inner,
        }
    }
}

impl Div for Rational {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            inner: self.inner / other.inner,
        }
    }
}

impl Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self {
        Self { inner: -self.inner }
    }
}

/// Represents a repeating decimal with its components.
#[derive(Debug, Clone, PartialEq)]
pub struct RepeatingDecimal {
    /// Whether the number is negative.
    pub is_negative: bool,
    /// The integer part.
    pub integer_part: String,
    /// The non-repeating part after the decimal point.
    pub non_repeating: String,
    /// The repeating part (repetend).
    pub repeating: String,
}

impl RepeatingDecimal {
    /// Formats using vinculum notation (overline): 0.3̅
    #[must_use]
    pub fn to_vinculum_notation(&self) -> String {
        let sign = if self.is_negative { "-" } else { "" };

        if self.repeating.is_empty() {
            // Not actually repeating
            if self.non_repeating.is_empty() {
                format!("{}{}", sign, self.integer_part)
            } else {
                format!("{}{}.{}", sign, self.integer_part, self.non_repeating)
            }
        } else {
            // Add combining overline character (U+0305) after each repeating digit
            let repetend_with_overline: String = self
                .repeating
                .chars()
                .map(|c| format!("{}\u{0305}", c))
                .collect();

            if self.non_repeating.is_empty() {
                format!("{}{}.{}", sign, self.integer_part, repetend_with_overline)
            } else {
                format!(
                    "{}{}.{}{}",
                    sign, self.integer_part, self.non_repeating, repetend_with_overline
                )
            }
        }
    }

    /// Formats using parenthesis notation: 0.(3)
    #[must_use]
    pub fn to_parenthesis_notation(&self) -> String {
        let sign = if self.is_negative { "-" } else { "" };

        if self.repeating.is_empty() {
            if self.non_repeating.is_empty() {
                format!("{}{}", sign, self.integer_part)
            } else {
                format!("{}{}.{}", sign, self.integer_part, self.non_repeating)
            }
        } else if self.non_repeating.is_empty() {
            format!("{}{}.({})", sign, self.integer_part, self.repeating)
        } else {
            format!(
                "{}{}.{}({})",
                sign, self.integer_part, self.non_repeating, self.repeating
            )
        }
    }

    /// Formats using ellipsis notation: 0.333...
    #[must_use]
    pub fn to_ellipsis_notation(&self) -> String {
        let sign = if self.is_negative { "-" } else { "" };

        if self.repeating.is_empty() {
            if self.non_repeating.is_empty() {
                format!("{}{}", sign, self.integer_part)
            } else {
                format!("{}{}.{}", sign, self.integer_part, self.non_repeating)
            }
        } else {
            // Show a few repetitions then ellipsis
            let repetitions = self.repeating.repeat(3.min(10 / self.repeating.len().max(1)));
            if self.non_repeating.is_empty() {
                format!("{}{}.{}...", sign, self.integer_part, repetitions)
            } else {
                format!("{}{}.{}{}...", sign, self.integer_part, self.non_repeating, repetitions)
            }
        }
    }

    /// Formats using LaTeX: 0.\overline{3}
    #[must_use]
    pub fn to_latex(&self) -> String {
        let sign = if self.is_negative { "-" } else { "" };

        if self.repeating.is_empty() {
            if self.non_repeating.is_empty() {
                format!("{}{}", sign, self.integer_part)
            } else {
                format!("{}{}.{}", sign, self.integer_part, self.non_repeating)
            }
        } else if self.non_repeating.is_empty() {
            format!("{}{}.\\overline{{{}}}", sign, self.integer_part, self.repeating)
        } else {
            format!(
                "{}{}.{}\\overline{{{}}}",
                sign, self.integer_part, self.non_repeating, self.repeating
            )
        }
    }
}

/// Detects repeating pattern in a fraction's decimal expansion.
fn detect_repeating_decimal(
    numerator: u128,
    denominator: u128,
    is_negative: bool,
) -> Option<RepeatingDecimal> {
    use std::collections::HashMap;

    if denominator == 0 {
        return None;
    }

    let integer_part = numerator / denominator;
    let mut remainder = numerator % denominator;

    if remainder == 0 {
        // No decimal part
        return Some(RepeatingDecimal {
            is_negative,
            integer_part: integer_part.to_string(),
            non_repeating: String::new(),
            repeating: String::new(),
        });
    }

    let mut digits = Vec::new();
    let mut remainder_positions: HashMap<u128, usize> = HashMap::new();
    let mut repeat_start = None;

    // Perform long division, tracking remainders
    const MAX_DIGITS: usize = 1000;

    while remainder != 0 && digits.len() < MAX_DIGITS {
        if let Some(&pos) = remainder_positions.get(&remainder) {
            repeat_start = Some(pos);
            break;
        }

        remainder_positions.insert(remainder, digits.len());

        remainder *= 10;
        let digit = remainder / denominator;
        digits.push((digit as u8 + b'0') as char);
        remainder %= denominator;
    }

    if let Some(start) = repeat_start {
        let non_repeating: String = digits[..start].iter().collect();
        let repeating: String = digits[start..].iter().collect();

        Some(RepeatingDecimal {
            is_negative,
            integer_part: integer_part.to_string(),
            non_repeating,
            repeating,
        })
    } else if remainder == 0 {
        // Terminating decimal
        Some(RepeatingDecimal {
            is_negative,
            integer_part: integer_part.to_string(),
            non_repeating: digits.iter().collect(),
            repeating: String::new(),
        })
    } else {
        // Too long, couldn't detect pattern
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_creation() {
        let r = Rational::new(1, 3);
        assert_eq!(r.numer(), 1);
        assert_eq!(r.denom(), 3);
    }

    #[test]
    fn test_rational_from_integer() {
        let r = Rational::from_integer(42);
        assert!(r.is_integer());
        assert_eq!(r.numer(), 42);
        assert_eq!(r.denom(), 1);
    }

    #[test]
    fn test_rational_simplification() {
        let r = Rational::new(2, 6);
        assert_eq!(r.numer(), 1);
        assert_eq!(r.denom(), 3);
    }

    #[test]
    fn test_one_third_times_three() {
        let one_third = Rational::new(1, 3);
        let three = Rational::from_integer(3);
        let result = one_third * three;
        assert!(result.is_integer());
        assert_eq!(result.numer(), 1);
        assert_eq!(result.to_display_string(), "1");
    }

    #[test]
    fn test_two_thirds_times_three() {
        let two_thirds = Rational::new(2, 3);
        let three = Rational::from_integer(3);
        let result = two_thirds * three;
        assert!(result.is_integer());
        assert_eq!(result.numer(), 2);
    }

    #[test]
    fn test_one_sixth_times_six() {
        let one_sixth = Rational::new(1, 6);
        let six = Rational::from_integer(6);
        let result = one_sixth * six;
        assert!(result.is_integer());
        assert_eq!(result.numer(), 1);
    }

    #[test]
    fn test_repeating_decimal_one_third() {
        let r = Rational::new(1, 3);
        let rd = r.to_repeating_decimal_notation().unwrap();
        assert_eq!(rd.integer_part, "0");
        assert_eq!(rd.non_repeating, "");
        assert_eq!(rd.repeating, "3");
        assert_eq!(rd.to_parenthesis_notation(), "0.(3)");
    }

    #[test]
    fn test_repeating_decimal_one_sixth() {
        let r = Rational::new(1, 6);
        let rd = r.to_repeating_decimal_notation().unwrap();
        assert_eq!(rd.integer_part, "0");
        assert_eq!(rd.non_repeating, "1");
        assert_eq!(rd.repeating, "6");
        assert_eq!(rd.to_parenthesis_notation(), "0.1(6)");
    }

    #[test]
    fn test_repeating_decimal_one_seventh() {
        let r = Rational::new(1, 7);
        let rd = r.to_repeating_decimal_notation().unwrap();
        assert_eq!(rd.integer_part, "0");
        assert_eq!(rd.non_repeating, "");
        assert_eq!(rd.repeating, "142857");
        assert_eq!(rd.to_parenthesis_notation(), "0.(142857)");
    }

    #[test]
    fn test_terminating_decimal() {
        let r = Rational::new(1, 4);
        let rd = r.to_repeating_decimal_notation().unwrap();
        assert_eq!(rd.repeating, "");
        assert_eq!(rd.non_repeating, "25");
    }

    #[test]
    fn test_fraction_string() {
        let r = Rational::new(1, 3);
        assert_eq!(r.to_fraction_string(), "1/3");
    }

    #[test]
    fn test_from_decimal() {
        let d = Decimal::from_f64(0.5);
        let r = Rational::from_decimal(d);
        assert_eq!(r.numer(), 1);
        assert_eq!(r.denom(), 2);
    }

    #[test]
    fn test_checked_div_by_zero() {
        let a = Rational::from_integer(10);
        let zero = Rational::zero();
        assert!(a.checked_div(&zero).is_none());
    }

    #[test]
    fn test_negative_rational() {
        let r = Rational::new(-1, 3);
        assert!(r.is_negative());
        let rd = r.to_repeating_decimal_notation().unwrap();
        assert!(rd.is_negative);
    }
}
