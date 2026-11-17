//! Verbatim copy of num-rational 0.4.2's `num-bigint`-feature `to_f64`.

// Kept as upstream wrote it: legacy `core::f64` constants, and
// `assert_eq!(RADIX, 2)` on a constant.
#![allow(clippy::legacy_numeric_constants, clippy::eq_op)]

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};

use crate::util::R;

// ----------------------------------------------------------------------------
// Verbatim copy of num-rational 0.4.2 src/lib.rs (private items, lines
// 1497-1715), used when the `num-bigint` feature is on.

trait Bits {
    fn bits(&self) -> u64;
}

impl Bits for BigInt {
    fn bits(&self) -> u64 {
        self.bits()
    }
}

impl Bits for i128 {
    fn bits(&self) -> u64 {
        (128 - self.wrapping_abs().leading_zeros()).into()
    }
}

pub fn feature_to_f64(r: &R) -> Option<f64> {
    let float = match (r.numer().to_i64(), r.denom().to_i64()) {
        (Some(numer), Some(denom)) => ratio_to_f64(
            <i128 as From<_>>::from(numer),
            <i128 as From<_>>::from(denom),
        ),
        _ => {
            let numer: BigInt = r.numer().clone();
            let denom: BigInt = r.denom().clone();
            ratio_to_f64(numer, denom)
        }
    };
    if float.is_nan() {
        None
    } else {
        Some(float)
    }
}

#[allow(deprecated)]
fn ratio_to_f64<T: Bits + Clone + Integer + Signed + std::ops::ShlAssign<usize> + ToPrimitive>(
    numer: T,
    denom: T,
) -> f64 {
    use core::f64::{INFINITY, MANTISSA_DIGITS, MAX_EXP, MIN_EXP, RADIX};

    assert_eq!(
        RADIX, 2,
        "only floating point implementations with radix 2 are supported"
    );

    const MAX_EXACT_INT: i64 = 1i64 << MANTISSA_DIGITS;
    const MIN_EXACT_INT: i64 = -MAX_EXACT_INT;

    let flo_sign = numer.signum().to_f64().unwrap() / denom.signum().to_f64().unwrap();
    if !flo_sign.is_normal() {
        return flo_sign;
    }

    if let (Some(n), Some(d)) = (numer.to_i64(), denom.to_i64()) {
        let exact = MIN_EXACT_INT..=MAX_EXACT_INT;
        if exact.contains(&n) && exact.contains(&d) {
            return n.to_f64().unwrap() / d.to_f64().unwrap();
        }
    }

    let mut numer = numer.abs();
    let mut denom = denom.abs();
    let (is_diff_positive, absolute_diff) = match numer.bits().checked_sub(denom.bits()) {
        Some(diff) => (true, diff),
        None => (false, denom.bits() - numer.bits()),
    };

    if is_diff_positive && absolute_diff > MAX_EXP as u64 {
        return INFINITY * flo_sign;
    }
    if !is_diff_positive && absolute_diff > -MIN_EXP as u64 + MANTISSA_DIGITS as u64 + 1 {
        return 0.0 * flo_sign;
    }
    let diff = if is_diff_positive {
        absolute_diff.to_isize().unwrap()
    } else {
        -absolute_diff.to_isize().unwrap()
    };

    let shift: isize = diff.max(MIN_EXP as isize) - MANTISSA_DIGITS as isize - 2;
    if shift >= 0 {
        denom <<= shift as usize
    } else {
        numer <<= -shift as usize
    };

    let (quotient, remainder) = numer.div_rem(&denom);

    let mut quotient = quotient.to_u64().unwrap();
    let n_rounding_bits = {
        let quotient_bits = 64 - quotient.leading_zeros() as isize;
        let subnormal_bits = MIN_EXP as isize - shift;
        quotient_bits.max(subnormal_bits) - MANTISSA_DIGITS as isize
    } as usize;
    debug_assert!(n_rounding_bits == 2 || n_rounding_bits == 3);
    let rounding_bit_mask = (1u64 << n_rounding_bits) - 1;

    let ls_bit = quotient & (1u64 << n_rounding_bits) != 0;
    let ms_rounding_bit = quotient & (1u64 << (n_rounding_bits - 1)) != 0;
    let ls_rounding_bits = quotient & (rounding_bit_mask >> 1) != 0;
    if ms_rounding_bit && (ls_bit || ls_rounding_bits || !remainder.is_zero()) {
        quotient += 1u64 << n_rounding_bits;
    }
    quotient &= !rounding_bit_mask;

    let q_float = quotient as f64 * flo_sign;
    ldexp(q_float, shift as i32)
}

#[allow(deprecated)]
fn ldexp(x: f64, exp: i32) -> f64 {
    use core::f64::{INFINITY, MANTISSA_DIGITS, MAX_EXP, RADIX};

    assert_eq!(
        RADIX, 2,
        "only floating point implementations with radix 2 are supported"
    );

    const EXPONENT_MASK: u64 = 0x7ff << 52;
    const MAX_UNSIGNED_EXPONENT: i32 = 0x7fe;
    const MIN_SUBNORMAL_POWER: i32 = MANTISSA_DIGITS as i32;

    if x.is_zero() || x.is_infinite() || x.is_nan() {
        return x;
    }

    if exp > 3 * MAX_EXP {
        return INFINITY * x.signum();
    } else if exp < -3 * MAX_EXP {
        return 0.0 * x.signum();
    }

    let (bits, curr_exp) = if !x.is_normal() {
        let normal_x = x * 2f64.powi(MIN_SUBNORMAL_POWER);
        let bits = normal_x.to_bits();
        (
            bits,
            ((bits & EXPONENT_MASK) >> 52) as i32 - MIN_SUBNORMAL_POWER,
        )
    } else {
        let bits = x.to_bits();
        let curr_exp = (bits & EXPONENT_MASK) >> 52;
        (bits, curr_exp as i32)
    };

    let new_exp = curr_exp + exp;

    if new_exp > MAX_UNSIGNED_EXPONENT {
        INFINITY * x.signum()
    } else if new_exp > 0 {
        let new_bits = (bits & !EXPONENT_MASK) | ((new_exp as u64) << 52);
        f64::from_bits(new_bits)
    } else if new_exp >= -(MANTISSA_DIGITS as i32) {
        let new_exp = new_exp + MIN_SUBNORMAL_POWER;
        debug_assert!(new_exp >= 0);
        let new_bits = (bits & !EXPONENT_MASK) | ((new_exp as u64) << 52);
        f64::from_bits(new_bits) * 2f64.powi(-MIN_SUBNORMAL_POWER)
    } else {
        0.0 * x.signum()
    }
}
