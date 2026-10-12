//! Physical dimensions as vectors of integer exponents (issue #235).
//!
//! Every physical unit has a dimension such as `length` (metre),
//! `length·time^-1` (speed) or `mass·length^2·time^-2` (energy).
//! Multiplication adds exponents, division subtracts them and powers scale
//! them, so `10 m × 10 m` has dimension `length^2`. Addition requires equal
//! dimensions.

use std::fmt;

/// Names of the base dimensions, in vector order.
pub const BASE_DIMENSIONS: [&str; 10] = [
    "length",
    "mass",
    "time",
    "current",
    "temperature",
    "amount",
    "luminosity",
    "information",
    "angle",
    "currency",
];

/// A dimension: the exponent of each base dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimension(pub [i32; 10]);

impl Dimension {
    /// The dimension of plain numbers.
    pub const NONE: Self = Self([0; 10]);

    /// Returns the base dimension with the given name (`length`, `mass`, …).
    #[must_use]
    pub fn base(name: &str) -> Option<Self> {
        let index = BASE_DIMENSIONS.iter().position(|base| *base == name)?;
        let mut exponents = [0; 10];
        exponents[index] = 1;
        Some(Self(exponents))
    }

    /// Whether this is the dimension of plain numbers.
    #[must_use]
    pub fn is_none(&self) -> bool {
        *self == Self::NONE
    }

    /// The dimension of a product.
    #[must_use]
    pub fn mul(&self, other: &Self) -> Self {
        let mut exponents = self.0;
        for (exponent, other) in exponents.iter_mut().zip(other.0) {
            *exponent += other;
        }
        Self(exponents)
    }

    /// The dimension of a quotient.
    #[must_use]
    pub fn div(&self, other: &Self) -> Self {
        self.mul(&other.pow(-1))
    }

    /// The dimension raised to an integer power.
    #[must_use]
    pub fn pow(&self, power: i32) -> Self {
        Self(self.0.map(|exponent| exponent * power))
    }

    /// The `n`-th root, if every exponent stays an integer.
    #[must_use]
    pub fn root(&self, n: i32) -> Option<Self> {
        if n == 0 || self.0.iter().any(|exponent| exponent % n != 0) {
            return None;
        }
        Some(Self(self.0.map(|exponent| exponent / n)))
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            return write!(f, "dimensionless");
        }
        let mut first = true;
        for (name, exponent) in BASE_DIMENSIONS.iter().zip(self.0) {
            if exponent == 0 {
                continue;
            }
            if !first {
                write!(f, "·")?;
            }
            first = false;
            if exponent == 1 {
                write!(f, "{name}")?;
            } else {
                write!(f, "{name}^{exponent}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_algebra() {
        let length = Dimension::base("length").unwrap();
        let time = Dimension::base("time").unwrap();
        let speed = length.div(&time);
        assert_eq!(speed.to_string(), "length·time^-1");
        assert_eq!(speed.mul(&time), length);
        assert_eq!(length.pow(2).root(2), Some(length));
        assert_eq!(length.root(2), None);
        assert!(length.div(&length).is_none());
    }
}
