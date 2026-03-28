//! Rational number type for exact fractional arithmetic.
//!
//! This module provides a `Rational` type that represents exact fractions
//! using arbitrary-precision `BigInt` numerator/denominator pairs. This allows
//! for perfect precision in calculations like `(1/3)*3 = 1` and exact
//! representation of large numbers like `10^100`.

use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, Pow, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

use crate::types::Decimal;

/// A rational number represented as a fraction (numerator/denominator)
/// with arbitrary-precision integers.
///
/// This type maintains exact precision by storing the numerator and denominator
/// separately and only performing the division when displaying or converting
/// to a decimal approximation. Using `BigInt` allows representing numbers
/// of any size exactly, such as `10^100`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rational {
    /// The underlying rational number with arbitrary-precision integers.
    #[serde(
        serialize_with = "serialize_ratio",
        deserialize_with = "deserialize_ratio"
    )]
    inner: Ratio<BigInt>,
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

fn serialize_ratio<S>(ratio: &Ratio<BigInt>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tuple = (ratio.numer().to_string(), ratio.denom().to_string());
    tuple.serialize(serializer)
}

fn deserialize_ratio<'de, D>(deserializer: D) -> Result<Ratio<BigInt>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (numer_str, denom_str): (String, String) = Deserialize::deserialize(deserializer)?;
    let numer: BigInt = numer_str
        .parse()
        .map_err(|e| serde::de::Error::custom(format!("invalid numerator: {e}")))?;
    let denom: BigInt = denom_str
        .parse()
        .map_err(|e| serde::de::Error::custom(format!("invalid denominator: {e}")))?;
    Ok(Ratio::new(numer, denom))
}

impl Rational {
    /// Creates a new rational number from an i128 integer.
    #[must_use]
    pub fn from_integer(n: i128) -> Self {
        Self {
            inner: Ratio::from_integer(BigInt::from(n)),
        }
    }

    /// Creates a new rational number from a `BigInt`.
    #[must_use]
    pub fn from_bigint(n: BigInt) -> Self {
        Self {
            inner: Ratio::from_integer(n),
        }
    }

    /// Creates a new rational number from i128 numerator and denominator.
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
            inner: Ratio::new(BigInt::from(numer), BigInt::from(denom)),
        }
    }

    /// Creates a new rational number from `BigInt` numerator and denominator.
    ///
    /// # Panics
    ///
    /// Panics if denominator is zero.
    #[must_use]
    pub fn new_bigint(numer: BigInt, denom: BigInt) -> Self {
        assert!(!denom.is_zero(), "Denominator cannot be zero");
        Self {
            inner: Ratio::new(numer, denom),
        }
    }

    /// Creates a rational number from a Decimal value.
    ///
    /// This converts the decimal to a fraction by using powers of 10.
    #[must_use]
    pub fn from_decimal(d: Decimal) -> Self {
        let s = d.to_string();

        if let Some(dot_pos) = s.find('.') {
            let sign: BigInt = if s.starts_with('-') {
                BigInt::from(-1)
            } else {
                BigInt::from(1)
            };
            let s_abs = s.trim_start_matches('-');
            let dot_pos_abs = if s.starts_with('-') {
                dot_pos - 1
            } else {
                dot_pos
            };

            let int_part = &s_abs[..dot_pos_abs];
            let frac_part = &s_abs[dot_pos_abs + 1..];

            let decimal_places = frac_part.len() as u32;
            let denom: BigInt = BigInt::from(10).pow(decimal_places);

            let int_val: BigInt = if int_part.is_empty() {
                BigInt::zero()
            } else {
                int_part.parse().unwrap_or_else(|_| BigInt::zero())
            };
            let frac_val: BigInt = if frac_part.is_empty() {
                BigInt::zero()
            } else {
                frac_part.parse().unwrap_or_else(|_| BigInt::zero())
            };

            let numer = sign * (&int_val * &denom + frac_val);
            Self::new_bigint(numer, denom)
        } else {
            let n: BigInt = s.parse().unwrap_or_else(|_| BigInt::zero());
            Self::from_bigint(n)
        }
    }

    /// Creates a rational number from an f64.
    ///
    /// This uses a rational approximation via continued fraction expansion.
    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        if value.is_nan() || value.is_infinite() {
            return Self::from_integer(0);
        }

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
            #[allow(clippy::cast_possible_truncation)]
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

    /// Returns the numerator as a `BigInt` reference.
    #[must_use]
    pub fn numer_bigint(&self) -> &BigInt {
        self.inner.numer()
    }

    /// Returns the denominator as a `BigInt` reference.
    #[must_use]
    pub fn denom_bigint(&self) -> &BigInt {
        self.inner.denom()
    }

    /// Returns the numerator, truncated to i128.
    /// For numbers exceeding i128 range, this saturates.
    #[must_use]
    pub fn numer(&self) -> i128 {
        self.inner.numer().to_i128().unwrap_or_else(|| {
            if self.inner.numer().is_negative() {
                i128::MIN
            } else {
                i128::MAX
            }
        })
    }

    /// Returns the denominator, truncated to i128.
    /// For numbers exceeding i128 range, this saturates.
    #[must_use]
    pub fn denom(&self) -> i128 {
        self.inner.denom().to_i128().unwrap_or(i128::MAX)
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

    /// Converts to f64 (may lose precision for large or very precise numbers).
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        let n = self.inner.numer().to_f64().unwrap_or(f64::INFINITY);
        let d = self.inner.denom().to_f64().unwrap_or(1.0);
        n / d
    }

    /// Converts to a Decimal (may lose precision for very large numbers).
    #[must_use]
    pub fn to_decimal(&self) -> Decimal {
        if self.is_integer() {
            if let Some(n) = self.inner.numer().to_i64() {
                return Decimal::new(n);
            }
        }
        Decimal::from_f64(self.to_f64())
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

    /// Raises this rational to an integer power (exact computation).
    ///
    /// For negative exponents, computes the reciprocal raised to the positive power.
    #[must_use]
    pub fn pow_i32(&self, exp: i32) -> Self {
        if exp == 0 {
            return Self::one();
        }
        if exp > 0 {
            #[allow(clippy::cast_sign_loss)]
            let exp_u32 = exp as u32;
            Self {
                inner: Pow::pow(&self.inner, exp_u32),
            }
        } else {
            // Negative exponent: (a/b)^(-n) = (b/a)^n
            let recip = self.inner.recip();
            #[allow(clippy::cast_sign_loss)]
            let exp_u32 = (-exp) as u32;
            Self {
                inner: Pow::pow(&recip, exp_u32),
            }
        }
    }

    /// Checked division that returns None on division by zero.
    #[must_use]
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            Some(Self {
                inner: &self.inner / &other.inner,
            })
        }
    }

    /// Converts the rational to a display string.
    ///
    /// If the rational is an integer, returns the exact integer string
    /// (using arbitrary precision, so `10^100` displays all 101 digits).
    /// Otherwise, returns the decimal representation.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        if self.is_integer() {
            self.inner.numer().to_string()
        } else {
            let approx = self.to_decimal();
            approx.normalize().to_string()
        }
    }

    /// Returns a fractional representation (e.g., "1/3").
    #[must_use]
    pub fn to_fraction_string(&self) -> String {
        if self.is_integer() {
            self.inner.numer().to_string()
        } else {
            format!("{}/{}", self.inner.numer(), self.inner.denom())
        }
    }

    /// Detects if the decimal representation is a repeating decimal
    /// and returns the notation (e.g., "0.3̅" or "0.(3)").
    ///
    /// Returns None if not a repeating decimal, if detection failed,
    /// or if the numbers are too large for the detection algorithm.
    #[must_use]
    pub fn to_repeating_decimal_notation(&self) -> Option<RepeatingDecimal> {
        if self.is_integer() {
            return None;
        }

        let numer_abs = self.inner.numer().abs();
        let denom_abs = self.inner.denom().abs();
        let is_negative = self.is_negative();

        let numer_u128 = numer_abs.to_u128()?;
        let denom_u128 = denom_abs.to_u128()?;

        detect_repeating_decimal(numer_u128, denom_u128, is_negative)
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
        Self::from_integer(i128::from(value))
    }
}

impl From<i32> for Rational {
    fn from(value: i32) -> Self {
        Self::from_integer(i128::from(value))
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
            use std::fmt::Write;
            let mut repetend_with_overline = String::with_capacity(self.repeating.len() * 2);
            for c in self.repeating.chars() {
                write!(repetend_with_overline, "{c}\u{0305}").unwrap();
            }

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
            let repetitions = self
                .repeating
                .repeat(3.min(10 / self.repeating.len().max(1)));
            if self.non_repeating.is_empty() {
                format!("{}{}.{}...", sign, self.integer_part, repetitions)
            } else {
                format!(
                    "{}{}.{}{}...",
                    sign, self.integer_part, self.non_repeating, repetitions
                )
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
            format!(
                "{}{}.\\overline{{{}}}",
                sign, self.integer_part, self.repeating
            )
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
    const MAX_DIGITS: usize = 1000;

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

    #[test]
    fn test_big_power() {
        let ten = Rational::from_integer(10);
        let result = ten.pow_i32(100);
        assert!(result.is_integer());
        let s = result.to_display_string();
        assert!(s.starts_with('1'));
        assert_eq!(s.len(), 101); // 1 followed by 100 zeros
        assert!(s.chars().skip(1).all(|c| c == '0'));
    }

    #[test]
    fn test_negative_power() {
        let two = Rational::from_integer(2);
        let result = two.pow_i32(-1);
        assert_eq!(result.numer(), 1);
        assert_eq!(result.denom(), 2);
    }
}
