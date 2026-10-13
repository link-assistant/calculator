// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a6294b3208c6541da8355b468c6e9a36a2c2dd952435ae1743e682b414911767 bytes=21859

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// Unbounded integers for the portable core's naturals and integers.
pub mod ml {
    use std::cmp::Ordering;
    use std::fmt;

    /// Sign and magnitude; the magnitude is little-endian base 2^32 without
    /// leading zero limbs, and zero is never negative.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Big {
        negative: bool,
        magnitude: Vec<u32>,
    }

    fn trim(mut magnitude: Vec<u32>) -> Vec<u32> {
        while magnitude.last() == Some(&0) {
            magnitude.pop();
        }
        magnitude
    }

    fn compare_magnitude(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    fn add_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for index in 0..a.len().max(b.len()) {
            let sum = u64::from(*a.get(index).unwrap_or(&0)) + u64::from(*b.get(index).unwrap_or(&0)) + carry;
            out.push(sum as u32);
            carry = sum >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }

    /// |a| - |b| where |a| >= |b|.
    fn sub_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (index, limb) in a.iter().enumerate() {
            let mut difference = i64::from(*limb) - i64::from(*b.get(index).unwrap_or(&0)) - borrow;
            borrow = 0;
            if difference < 0 {
                difference += 1 << 32;
                borrow = 1;
            }
            out.push(difference as u32);
        }
        trim(out)
    }

    fn mul_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, x) in a.iter().enumerate() {
            let mut carry = 0u64;
            for (j, y) in b.iter().enumerate() {
                let product = u64::from(out[i + j]) + u64::from(*x) * u64::from(*y) + carry;
                out[i + j] = product as u32;
                carry = product >> 32;
            }
            let mut index = i + b.len();
            while carry > 0 {
                let sum = u64::from(out[index]) + carry;
                out[index] = sum as u32;
                carry = sum >> 32;
                index += 1;
            }
        }
        trim(out)
    }

    /// Quotient and remainder of magnitudes; the divisor is not zero.
    fn divmod_magnitude(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let mut quotient = vec![0u32; a.len()];
        if b.len() == 1 {
            let divisor = u64::from(b[0]);
            let mut remainder = 0u64;
            for index in (0..a.len()).rev() {
                let current = (remainder << 32) | u64::from(a[index]);
                quotient[index] = (current / divisor) as u32;
                remainder = current % divisor;
            }
            return (trim(quotient), trim(vec![remainder as u32]));
        }
        let mut remainder: Vec<u32> = Vec::new();
        for bit in (0..a.len() * 32).rev() {
            let mut carry = (a[bit / 32] >> (bit % 32)) & 1;
            for limb in remainder.iter_mut() {
                let next = *limb >> 31;
                *limb = (*limb << 1) | carry;
                carry = next;
            }
            if carry != 0 {
                remainder.push(carry);
            }
            if compare_magnitude(&remainder, b) != Ordering::Less {
                remainder = sub_magnitude(&remainder, b);
                quotient[bit / 32] |= 1 << (bit % 32);
            }
        }
        (trim(quotient), remainder)
    }

    impl Big {
        fn from_parts(negative: bool, magnitude: Vec<u32>) -> Self {
            let magnitude = trim(magnitude);
            Self { negative: negative && !magnitude.is_empty(), magnitude }
        }

        pub fn zero() -> Self {
            Self::from_parts(false, Vec::new())
        }

        pub fn from_i128(value: i128) -> Self {
            let mut big = Self::from_u128(value.unsigned_abs());
            big.negative = value < 0;
            big
        }

        pub fn from_u128(mut value: u128) -> Self {
            let mut magnitude = Vec::new();
            while value > 0 {
                magnitude.push(value as u32);
                value >>= 32;
            }
            Self::from_parts(false, magnitude)
        }

        /// A decimal literal with an optional leading minus sign.
        pub fn parse(text: &str) -> Self {
            let (negative, digits) = match text.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, text),
            };
            let ten = Self::from_i128(10);
            let mut value = Self::zero();
            for digit in digits.chars() {
                let digit = digit.to_digit(10).expect("decimal literal");
                value = value.mul(&ten).add(&Self::from_i128(i128::from(digit)));
            }
            if negative { value.neg() } else { value }
        }

        pub fn is_zero(&self) -> bool {
            self.magnitude.is_empty()
        }

        pub fn is_negative(&self) -> bool {
            self.negative
        }

        /// The array index this names, if it is one.
        pub fn to_index(&self) -> Option<usize> {
            if self.negative || self.magnitude.len() > 2 {
                return None;
            }
            let value = self.magnitude.iter().rev().fold(0u64, |acc, &digit| (acc << 32) | u64::from(digit));
            usize::try_from(value).ok()
        }

        pub fn neg(&self) -> Self {
            Self::from_parts(!self.negative, self.magnitude.clone())
        }

        pub fn add(&self, other: &Self) -> Self {
            if self.negative == other.negative {
                return Self::from_parts(self.negative, add_magnitude(&self.magnitude, &other.magnitude));
            }
            match compare_magnitude(&self.magnitude, &other.magnitude) {
                Ordering::Less => Self::from_parts(other.negative, sub_magnitude(&other.magnitude, &self.magnitude)),
                _ => Self::from_parts(self.negative, sub_magnitude(&self.magnitude, &other.magnitude)),
            }
        }

        pub fn sub(&self, other: &Self) -> Self {
            self.add(&other.neg())
        }

        pub fn mul(&self, other: &Self) -> Self {
            Self::from_parts(self.negative != other.negative, mul_magnitude(&self.magnitude, &other.magnitude))
        }

        /// Division rounding toward zero; the divisor is not zero.
        pub fn divmod_trunc(&self, other: &Self) -> (Self, Self) {
            let (quotient, remainder) = divmod_magnitude(&self.magnitude, &other.magnitude);
            (
                Self::from_parts(self.negative != other.negative, quotient),
                Self::from_parts(self.negative, remainder),
            )
        }
    }

    impl Ord for Big {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.negative, other.negative) {
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (false, false) => compare_magnitude(&self.magnitude, &other.magnitude),
                (true, true) => compare_magnitude(&other.magnitude, &self.magnitude),
            }
        }
    }

    impl PartialOrd for Big {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl fmt::Display for Big {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.is_zero() {
                return formatter.write_str("0");
            }
            let billion = [1_000_000_000u32];
            let mut chunks = Vec::new();
            let mut rest = self.magnitude.clone();
            while !rest.is_empty() {
                let (quotient, remainder) = divmod_magnitude(&rest, &billion);
                chunks.push(remainder.first().copied().unwrap_or(0));
                rest = quotient;
            }
            let mut text = String::new();
            if self.negative {
                text.push('-');
            }
            text.push_str(&chunks.pop().unwrap_or(0).to_string());
            for chunk in chunks.iter().rev() {
                text.push_str(&format!("{chunk:09}"));
            }
            formatter.write_str(&text)
        }
    }

    /// Natural subtraction: truncated at zero.
    pub fn nat_sub(a: &Big, b: &Big) -> Big {
        if a > b { a.sub(b) } else { Big::zero() }
    }

    pub fn pred(a: &Big) -> Big {
        a.sub(&Big::from_i128(1))
    }

    /// Integer division with the source's rounding ("trunc", "floor" or
    /// "euclid"); by zero it aborts with the source's message or is total
    /// (x / 0 = 0, x % 0 = x).
    pub fn divide(a: &Big, b: &Big, rounding: &str, by_zero: Option<&str>, remainder: bool) -> Big {
        if b.is_zero() {
            if let Some(message) = by_zero {
                panic!("{message}");
            }
            return if remainder { a.clone() } else { Big::zero() };
        }
        let one = Big::from_i128(1);
        let (mut quotient, rest) = a.divmod_trunc(b);
        if !rest.is_zero() {
            if rounding == "floor" && rest.is_negative() != b.is_negative() {
                quotient = quotient.sub(&one);
            }
            if rounding == "euclid" && rest.is_negative() {
                quotient = if b.is_negative() { quotient.add(&one) } else { quotient.sub(&one) };
            }
        }
        if remainder { a.sub(&quotient.mul(b)) } else { quotient }
    }

    pub fn to_nat_checked(value: Big, message: &str) -> Big {
        if value.is_negative() {
            panic!("{message}");
        }
        value
    }

    pub fn clamp_nat(value: Big) -> Big {
        if value.is_negative() { Big::zero() } else { value }
    }

    /// Bounded domains for executable theorem checks.
    pub fn range(low: i128, high: i128) -> Vec<Big> {
        (low..=high).map(Big::from_i128).collect()
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * num-rational 0.4.2 `Ratio<BigInt>` over num-bigint 0.5.1, num-integer
// |  * 0.1.47 and num-traits 0.2.19, ported literally so that every result
// |  * (including unreduced values, panics and float conversions) matches Rust.
// |  *
// |  * `BigInt` is the JavaScript `bigint`. The free functions below are the
// |  * num-bigint / num-integer / num-traits methods the engine calls on it; the
// |  * operators that agree with Rust (`+`, `-`, `*`, `===`, `<`) are used as is.
// |  *
// |  * Sources (paths inside the crates):
// |  * - num-rational-0.4.2/src/lib.rs: `reduce`, `recip`, `floor`, `ceil`,
// |  *   `round`, `trunc`, `fract`, `Ord`, `arith_impl!`, `Mul`, `Div`, `Neg`,
// |  *   `Signed`, `Display`, `ToPrimitive for Ratio<BigInt>`, `ratio_to_f64`,
// |  *   `ldexp`; src/pow.rs: `Pow<u32>` / `Pow<i32>`.
// |  * - num-bigint-0.5.1/src/lib.rs: `ParseBigIntError`;
// |  *   src/biguint/convert.rs and src/bigint/convert.rs: `from_str_radix`,
// |  *   `to_f64`, `from_f64`, `to_i64`, ...; src/bigint/division.rs: the
// |  *   "attempt to divide by zero" panic; src/bigint/integer.rs: `gcd`, `lcm`,
// |  *   `div_floor`, `mod_floor`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=257cdc5e7b92a0dcbbfdbdcc5e6cbc6499a8af94c8bb727c9ce061d47762c8b6
// | import { Err, panic } from './core.js';
use crate::rust::core_::{Err, panic};

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7223992d72add909a6142c8c46152bfd1b7a88778b2fb7019b03751f5eaee17d
// | const DIVIDE_BY_ZERO = 'attempt to divide by zero';
pub const DIVIDE_BY_ZERO: &str = "attempt to divide by zero";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I64_MIN = -(1n << 63n);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I64_MAX = (1n << 63n) - 1n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const U64_MAX = (1n << 64n) - 1n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I128_MIN = -(1n << 127n);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I128_MAX = (1n << 127n) - 1n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const U128_MAX = (1n << 128n) - 1n;

// ----------------------------------------------------------------------------
// BigInt methods.

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::to_f64`: always `Some`, correctly rounded (half to even), and
// |  * infinite past `f64::MAX`, which is exactly what `Number(n)` does.
// |  * @param {bigint} n
// |  * @returns {number}
// |  */
// | export function bigintToF64(n) {
// |   return Number(n);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::from_f64`: `None` for NaN and infinities, otherwise the value
// |  * truncated toward zero (`-0.0` and `-0.5` give 0).
// |  * @param {number} x
// |  * @returns {bigint | null}
// |  */
// | export function bigintFromF64(x) {
// |   return Number.isFinite(x) ? BigInt(Math.trunc(x)) : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::to_i64`: the value as an `i64` (a `bigint`), or `null` when it
// |  * does not fit.
// |  * @param {bigint} n
// |  * @returns {bigint | null}
// |  */
// | export function bigintToI64(n) {
// |   return n >= I64_MIN && n <= I64_MAX ? n : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::to_u64`: the value as a `u64` (a `bigint`), or `null`.
// |  * @param {bigint} n
// |  * @returns {bigint | null}
// |  */
// | export function bigintToU64(n) {
// |   return n >= 0n && n <= U64_MAX ? n : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::to_i128`: the value as an `i128` (a `bigint`), or `null`.
// |  * @param {bigint} n
// |  * @returns {bigint | null}
// |  */
// | export function bigintToI128(n) {
// |   return n >= I128_MIN && n <= I128_MAX ? n : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::to_u128`: the value as a `u128` (a `bigint`), or `null`.
// |  * @param {bigint} n
// |  * @returns {bigint | null}
// |  */
// | export function bigintToU128(n) {
// |   return n >= 0n && n <= U128_MAX ? n : null;
// | }

// meta-language:translated JavaScript export_statement items=1 sha256=1b001ea29196e47c163b3aa41bb259a9342986769383d6374277d505d5a9cd08
// | /**
// |  * `Signed::abs`.
// |  * @param {bigint} n
// |  * @returns {bigint}
// |  */
// | export function bigintAbs(n) {
// |   return n < 0n ? -n : n;
// | }
pub fn bigint_abs(n: crate::ml::Big) -> crate::ml::Big {
    if (n < (crate::ml::Big::from_i128(0))) {
        n.neg()
    } else {
        n.clone()
    }
}

// meta-language:translated JavaScript export_statement items=1 sha256=7b7a374d894dd6a4761c575d995aeee0f7da2fb44084830509a81537acb7c8de
// | /**
// |  * `Signed::signum`: -1, 0 or 1.
// |  * @param {bigint} n
// |  * @returns {bigint}
// |  */
// | export function bigintSignum(n) {
// |   return n > 0n ? 1n : n < 0n ? -1n : 0n;
// | }
pub fn bigint_signum(n: crate::ml::Big) -> crate::ml::Big {
    if (n > (crate::ml::Big::from_i128(0))) {
        crate::ml::Big::from_i128(1)
    } else {
        if (n < (crate::ml::Big::from_i128(0))) {
            crate::ml::Big::from_i128(-1)
        } else {
            crate::ml::Big::from_i128(0)
        }
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::bits`: the bit length of the magnitude (0 for zero).
// |  * @param {bigint} n
// |  * @returns {number}
// |  */
// | export function bigintBits(n) {
// |   if (n === 0n) {
// |     return 0;
// |   }
// |   const hex = (n < 0n ? -n : n).toString(16);
// |   return (hex.length - 1) * 4 + (32 - Math.clz32(parseInt(hex[0], 16)));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Integer::is_even` (of the magnitude, so negative values work too).
// |  * @param {bigint} n
// |  * @returns {boolean}
// |  */
// | export function bigintIsEven(n) {
// |   return (n & 1n) === 0n;
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `a / b`: truncating division; panics "attempt to divide by zero".
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function bigintDiv(a, b) {
// |   if (b === 0n) {
// |     panic(DIVIDE_BY_ZERO);
// |   }
// |   return a / b;
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `a % b`: the remainder of truncating division (sign of `a`); panics
// |  * "attempt to divide by zero".
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function bigintRem(a, b) {
// |   if (b === 0n) {
// |     panic(DIVIDE_BY_ZERO);
// |   }
// |   return a % b;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Integer::div_rem`: `[a / b, a % b]`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {[bigint, bigint]}
// |  */
// | export function bigintDivRem(a, b) {
// |   if (b === 0n) {
// |     panic(DIVIDE_BY_ZERO);
// |   }
// |   return [a / b, a % b];
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Integer::div_mod_floor`: the floored quotient and the remainder with the
// |  * sign of `b`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {[bigint, bigint]}
// |  */
// | export function bigintDivModFloor(a, b) {
// |   const [d, m] = bigintDivRem(a, b);
// |   if ((m > 0n && b < 0n) || (m < 0n && b > 0n)) {
// |     return [d - 1n, m + b];
// |   }
// |   return [d, m];
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `Integer::div_floor`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function bigintDivFloor(a, b) {
// |   return bigintDivModFloor(a, b)[0];
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `Integer::mod_floor`: the remainder with the sign of `b`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function bigintModFloor(a, b) {
// |   return bigintDivModFloor(a, b)[1];
// | }

// meta-language:translated JavaScript export_statement items=2 sha256=7c14332e1a368c705c91fca640625015910d31660323f003492b07df58c2e96b
// | /**
// |  * `Integer::gcd`: always non-negative; `gcd(0, x) == |x|`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function gcd(a, b) {
// |   let x = a < 0n ? -a : a;
// |   let y = b < 0n ? -b : b;
// |   while (y !== 0n) {
// |     const r = x % y;
// |     x = y;
// |     y = r;
// |   }
// |   return x;
// | }
pub fn gcd(a: crate::ml::Big, b: crate::ml::Big) -> crate::ml::Big {
    {
        let x = if (a < (crate::ml::Big::from_i128(0))) {
            a.neg()
        } else {
            a.clone()
        };
        {
            let y = if (b < (crate::ml::Big::from_i128(0))) {
                b.neg()
            } else {
                b.clone()
            };
            {
                let x_2 = crate::ml_gcd_loop1(x.clone(), y.clone());
                x_2.clone()
            }
        }
    }
}

pub fn ml_gcd_loop1(mut x: crate::ml::Big, mut y: crate::ml::Big) -> crate::ml::Big {
    loop {
        return if (y != (crate::ml::Big::from_i128(0))) {
            {
                let r = crate::ml::divide(&x, &y, "trunc", Some("Division by zero"), true);
                {
                    let x_2 = y.clone();
                    {
                        let y_2 = r.clone();
                        {
                            (x, y) = (x_2.clone(), y_2.clone());
                            continue;
                        }
                    }
                }
            }
        } else {
            x.clone()
        };
    }
}

// meta-language:translated JavaScript export_statement items=1 sha256=9a9e598d7297535f4137e316b1561c6f3ecf7a469361cb5c0921adf36fd2b61f
// | /**
// |  * `Integer::lcm`: always non-negative; 0 when either side is 0.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint}
// |  */
// | export function lcm(a, b) {
// |   if (a === 0n && b === 0n) {
// |     return 0n;
// |   }
// |   const x = a < 0n ? -a : a;
// |   const y = b < 0n ? -b : b;
// |   return (x / gcd(x, y)) * y;
// | }
pub fn lcm(a: crate::ml::Big, b: crate::ml::Big) -> crate::ml::Big {
    if ((a == (crate::ml::Big::from_i128(0))) && (b == (crate::ml::Big::from_i128(0)))) {
        crate::ml::Big::from_i128(0)
    } else {
        {
            let x = if (a < (crate::ml::Big::from_i128(0))) {
                a.neg()
            } else {
                a.clone()
            };
            {
                let y = if (b < (crate::ml::Big::from_i128(0))) {
                    b.neg()
                } else {
                    b.clone()
                };
                (crate::ml::divide(&x, &gcd(x.clone(), y.clone()), "trunc", Some("Division by zero"), false)).mul(&y)
            }
        }
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `BigInt::pow` / `Pow<u32>` / `Pow<u64>`: exact, with `0^0 == 1`. The
// |  * exponent must be a non-negative integer.
// |  * @param {bigint} base
// |  * @param {number | bigint} exp
// |  * @returns {bigint}
// |  */
// | export function bigintPow(base, exp) {
// |   const e = BigInt(exp);
// |   if (e === 0n) {
// |     return 1n;
// |   }
// |   if (base === 0n || base === 1n) {
// |     return base;
// |   }
// |   if (base === -1n) {
// |     return (e & 1n) === 0n ? 1n : -1n;
// |   }
// |   return base ** e;
// | }

// ----------------------------------------------------------------------------
// Parsing.

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `num_bigint::ParseBigIntError`. Display is `toString()`, Debug is
// |  * `fmtDebug()`.
// |  */
// | export class ParseBigIntError {
// |   /** @param {'Empty' | 'InvalidDigit'} kind */
// |   constructor(kind) {
// |     this.kind = kind;
// |     Object.freeze(this);
// |   }
// |
// |   toString() {
// |     return this.kind === 'Empty'
// |       ? 'cannot parse integer from empty string'
// |       : 'invalid digit found in string';
// |   }
// |
// |   fmtDebug() {
// |     return `ParseBigIntError { kind: ${this.kind} }`;
// |   }
// |
// |   /** @param {unknown} other */
// |   equals(other) {
// |     return other instanceof ParseBigIntError && other.kind === this.kind;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (syntax)
// | const DECIMAL_DIGITS = /^[0-9_]*$/;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `s.parse::<BigInt>()` (`BigInt::from_str_radix(s, 10)`): an optional
// |  * single '-' or '+', then ASCII digits; '_' may appear anywhere except
// |  * first. Throws `new Err(ParseBigIntError)`.
// |  * @param {string} s
// |  * @returns {bigint}
// |  */
// | export function bigintFromStr(s) {
// |   let negative = false;
// |   if (s.startsWith('-')) {
// |     negative = true;
// |     if (!s.startsWith('+', 1)) {
// |       s = s.slice(1);
// |     }
// |   }
// |   // BigUint::from_str_radix
// |   if (s.startsWith('+') && !s.startsWith('+', 1)) {
// |     s = s.slice(1);
// |   }
// |   if (s === '') {
// |     throw new Err(new ParseBigIntError('Empty'));
// |   }
// |   if (s.startsWith('_') || !DECIMAL_DIGITS.test(s)) {
// |     throw new Err(new ParseBigIntError('InvalidDigit'));
// |   }
// |   const magnitude = BigInt(s.replaceAll('_', ''));
// |   return negative ? -magnitude : magnitude;
// | }

// ----------------------------------------------------------------------------
// Ratio.

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Ratio::reduce` after `Ratio::new_raw`.
// |  * @param {bigint} numer
// |  * @param {bigint} denom
// |  * @returns {Ratio}
// |  */
// | function reduced(numer, denom) {
// |   if (denom === 0n) {
// |     panic('denominator == 0');
// |   }
// |   if (numer === 0n) {
// |     return new Ratio(numer, 1n);
// |   }
// |   if (numer === denom) {
// |     return new Ratio(1n, 1n);
// |   }
// |   const g = gcd(numer, denom);
// |   const n = numer / g;
// |   const d = denom / g;
// |   return d < 0n ? new Ratio(-n, -d) : new Ratio(n, d);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `arith_impl!` (`Add`, `Sub`, `Rem`) for `Ratio op Ratio` and `Ratio op T`.
// |  * @param {Ratio} lhs
// |  * @param {Ratio | bigint} rhs
// |  * @param {(a: bigint, b: bigint) => bigint} op
// |  * @returns {Ratio}
// |  */
// | function arith(lhs, rhs, op) {
// |   if (typeof rhs === 'bigint') {
// |     return reduced(op(lhs._numer, lhs._denom * rhs), lhs._denom);
// |   }
// |   if (lhs._denom === rhs._denom) {
// |     return reduced(op(lhs._numer, rhs._numer), rhs._denom);
// |   }
// |   const l = lcm(lhs._denom, rhs._denom);
// |   const lhsNumer = lhs._numer * bigintDiv(l, lhs._denom);
// |   const rhsNumer = rhs._numer * bigintDiv(l, rhs._denom);
// |   return reduced(op(lhsNumer, rhsNumer), l);
// | }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=68b1876adab550d238bcd07907bdf3aa129f75ca74bfdab4b06216061e652684
// | /** @param {bigint} a @param {bigint} b */
// | const plus = (a, b) => a + b;
pub fn plus(a: crate::ml::Big, b: crate::ml::Big) -> crate::ml::Big {
    a.add(&b)
}
// meta-language:translated JavaScript lexical_declaration items=1 sha256=ca6a1f07f362af94ac4a9ac14bcb845874074ef18a7474da952de7a370e999ad
// | /** @param {bigint} a @param {bigint} b */
// | const minus = (a, b) => a - b;
pub fn minus(a: crate::ml::Big, b: crate::ml::Big) -> crate::ml::Big {
    a.sub(&b)
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Ordering` of two bigints as -1, 0 or 1.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {-1 | 0 | 1}
// |  */
// | function cmpBigint(a, b) {
// |   return a < b ? -1 : a > b ? 1 : 0;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Ordering::reverse` when `flip`.
// |  * @param {-1 | 0 | 1} ord
// |  * @param {boolean} flip
// |  * @returns {-1 | 0 | 1}
// |  */
// | function reverseIf(ord, flip) {
// |   return flip && ord !== 0 ? /** @type {-1 | 1} */ (-ord) : ord;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `num_rational::Ratio<BigInt>` (`BigRational`): an immutable numerator and
// |  * denominator. `Ratio.new` reduces; `Ratio.newRaw` keeps the fields as given,
// |  * and every method then behaves as num-rational does on such a value. Binary
// |  * operators take a `Ratio` or a `bigint` (Rust's `Ratio<T> op T`).
// |  */
// | export class Ratio {
// |   /**
// |    * `Ratio::new_raw`.
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    */
// |   constructor(numer, denom) {
// |     /** @readonly */
// |     this._numer = numer;
// |     /** @readonly */
// |     this._denom = denom;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `Ratio::new`: reduced, with a positive denominator; panics
// |    * "denominator == 0".
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    * @returns {Ratio}
// |    */
// |   static new(numer, denom) {
// |     return reduced(numer, denom);
// |   }
// |
// |   /**
// |    * `Ratio::new_raw`: no reduction, no checks.
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    * @returns {Ratio}
// |    */
// |   static newRaw(numer, denom) {
// |     return new Ratio(numer, denom);
// |   }
// |
// |   /**
// |    * `Ratio::from_integer` (and `From<BigInt>`): `t/1`.
// |    * @param {bigint} t
// |    * @returns {Ratio}
// |    */
// |   static fromInteger(t) {
// |     return new Ratio(t, 1n);
// |   }
// |
// |   /** `Zero::zero`: 0/1. @returns {Ratio} */
// |   static zero() {
// |     return ZERO;
// |   }
// |
// |   /** `One::one`: 1/1. @returns {Ratio} */
// |   static one() {
// |     return ONE;
// |   }
// |
// |   /** `Ratio::numer`. @returns {bigint} */
// |   numer() {
// |     return this._numer;
// |   }
// |
// |   /** `Ratio::denom`. @returns {bigint} */
// |   denom() {
// |     return this._denom;
// |   }
// |
// |   /** `Ratio::is_integer`: `denom == 1`. @returns {boolean} */
// |   isInteger() {
// |     return this._denom === 1n;
// |   }
// |
// |   /** `Zero::is_zero`: `numer == 0`. @returns {boolean} */
// |   isZero() {
// |     return this._numer === 0n;
// |   }
// |
// |   /** `One::is_one`: `numer == denom`. @returns {boolean} */
// |   isOne() {
// |     return this._numer === this._denom;
// |   }
// |
// |   /** `Signed::is_positive`. @returns {boolean} */
// |   isPositive() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     return (n > 0n && d > 0n) || (n < 0n && d < 0n);
// |   }
// |
// |   /** `Signed::is_negative`. @returns {boolean} */
// |   isNegative() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     return (n < 0n && d > 0n) || (n > 0n && d < 0n);
// |   }
// |
// |   /** `Neg`: `new_raw(-numer, denom)`. @returns {Ratio} */
// |   neg() {
// |     return new Ratio(-this._numer, this._denom);
// |   }
// |
// |   /** `Signed::abs`. @returns {Ratio} */
// |   abs() {
// |     return this.isNegative() ? this.neg() : this;
// |   }
// |
// |   /** `Signed::signum`: 1/1, 0/1 or -1/1. @returns {Ratio} */
// |   signum() {
// |     if (this.isPositive()) {
// |       return ONE;
// |     }
// |     if (this.isZero()) {
// |       return ZERO;
// |     }
// |     return ONE.neg();
// |   }
// |
// |   /**
// |    * `Ratio::recip`: swaps the fields, moving the sign to the numerator;
// |    * panics "division by zero".
// |    * @returns {Ratio}
// |    */
// |   recip() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (n === 0n) {
// |       panic('division by zero');
// |     }
// |     return n > 0n ? new Ratio(d, n) : new Ratio(-d, -n);
// |   }
// |
// |   /**
// |    * `Add`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   add(rhs) {
// |     return arith(this, rhs, plus);
// |   }
// |
// |   /**
// |    * `Sub`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   sub(rhs) {
// |     return arith(this, rhs, minus);
// |   }
// |
// |   /**
// |    * `Rem`: the remainder of truncating division, with the sign of `self`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   rem(rhs) {
// |     return arith(this, rhs, bigintRem);
// |   }
// |
// |   /**
// |    * `Mul`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   mul(rhs) {
// |     if (typeof rhs === 'bigint') {
// |       const g = gcd(this._denom, rhs);
// |       const denom = bigintDiv(this._denom, g);
// |       return reduced(this._numer * bigintDiv(rhs, g), denom);
// |     }
// |     const gcdAd = gcd(this._numer, rhs._denom);
// |     const gcdBc = gcd(this._denom, rhs._numer);
// |     return reduced(
// |       bigintDiv(this._numer, gcdAd) * bigintDiv(rhs._numer, gcdBc),
// |       bigintDiv(this._denom, gcdBc) * bigintDiv(rhs._denom, gcdAd),
// |     );
// |   }
// |
// |   /**
// |    * `Div`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   div(rhs) {
// |     if (typeof rhs === 'bigint') {
// |       const g = gcd(this._numer, rhs);
// |       const numer = bigintDiv(this._numer, g);
// |       return reduced(numer, this._denom * bigintDiv(rhs, g));
// |     }
// |     const gcdAc = gcd(this._numer, rhs._numer);
// |     const gcdBd = gcd(this._denom, rhs._denom);
// |     return reduced(
// |       bigintDiv(this._numer, gcdAc) * bigintDiv(rhs._denom, gcdBd),
// |       bigintDiv(this._denom, gcdBd) * bigintDiv(rhs._numer, gcdAc),
// |     );
// |   }
// |
// |   /**
// |    * `Pow<u32>` / `Pow<i32>` (and the 64-bit forms): `new_raw(numer^e,
// |    * denom^e)`, without reducing; 1/1 for 0; the `recip` of the positive power
// |    * for a negative exponent.
// |    * @param {number} exp an integer
// |    * @returns {Ratio}
// |    */
// |   pow(exp) {
// |     if (exp < 0) {
// |       const e = -exp;
// |       return new Ratio(bigintPow(this._numer, e), bigintPow(this._denom, e)).recip();
// |     }
// |     if (exp === 0) {
// |       return ONE;
// |     }
// |     return new Ratio(bigintPow(this._numer, exp), bigintPow(this._denom, exp));
// |   }
// |
// |   /** `Ratio::floor`: rounds toward minus infinity. @returns {Ratio} */
// |   floor() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (this.cmp(ZERO) < 0) {
// |       return new Ratio(bigintDiv(n - d + 1n, d), 1n);
// |     }
// |     return new Ratio(bigintDiv(n, d), 1n);
// |   }
// |
// |   /** `Ratio::ceil`: rounds toward plus infinity. @returns {Ratio} */
// |   ceil() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (this.cmp(ZERO) < 0) {
// |       return new Ratio(bigintDiv(n, d), 1n);
// |     }
// |     return new Ratio(bigintDiv(n + d - 1n, d), 1n);
// |   }
// |
// |   /** `Ratio::round`: to the nearest integer, half away from zero. @returns {Ratio} */
// |   round() {
// |     let fractional = this.fract();
// |     if (fractional.cmp(ZERO) < 0) {
// |       fractional = ZERO.sub(fractional);
// |     }
// |     const halfOrLarger = bigintIsEven(fractional._denom)
// |       ? fractional._numer >= fractional._denom / 2n
// |       : fractional._numer >= fractional._denom / 2n + 1n;
// |     if (halfOrLarger) {
// |       return this.cmp(ZERO) >= 0 ? this.trunc().add(ONE) : this.trunc().sub(ONE);
// |     }
// |     return this.trunc();
// |   }
// |
// |   /** `Ratio::trunc`: rounds toward zero. @returns {Ratio} */
// |   trunc() {
// |     return new Ratio(bigintDiv(this._numer, this._denom), 1n);
// |   }
// |
// |   /**
// |    * `Ratio::fract`: `new_raw(numer % denom, denom)`, with the sign of `self`.
// |    * @returns {Ratio}
// |    */
// |   fract() {
// |     return new Ratio(bigintRem(this._numer, this._denom), this._denom);
// |   }
// |
// |   /** `Ratio::to_integer`: `trunc().numer`. @returns {bigint} */
// |   toInteger() {
// |     return bigintDiv(this._numer, this._denom);
// |   }
// |
// |   /**
// |    * `ToPrimitive::to_f64` of num-rational's `num-bigint` feature (which the
// |    * calculator's build does not enable): correctly rounded, `null` for 0/0.
// |    * @returns {number | null}
// |    */
// |   toF64() {
// |     const f = ratioToF64(this._numer, this._denom);
// |     return Number.isNaN(f) ? null : f;
// |   }
// |
// |   /** `ToPrimitive::to_i64` (feature impl): `to_integer().to_i64()`. @returns {bigint | null} */
// |   toI64() {
// |     return bigintToI64(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_u64` (feature impl). @returns {bigint | null} */
// |   toU64() {
// |     return bigintToU64(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_i128` (feature impl). @returns {bigint | null} */
// |   toI128() {
// |     return bigintToI128(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_u128` (feature impl). @returns {bigint | null} */
// |   toU128() {
// |     return bigintToU128(this.toInteger());
// |   }
// |
// |   /**
// |    * `Ord::cmp`, num-rational's algorithm, which also orders unreduced and
// |    * negative-denominator values (and panics on a zero denominator where it
// |    * has to divide).
// |    * @param {Ratio} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     /** @type {Ratio} */
// |     let a = this;
// |     let b = other;
// |     let flip = false;
// |     for (;;) {
// |       if (a._denom === b._denom) {
// |         const ord = cmpBigint(a._numer, b._numer);
// |         return reverseIf(ord, flip !== a._denom < 0n);
// |       }
// |       if (a._numer === b._numer) {
// |         if (a._numer === 0n) {
// |           return 0;
// |         }
// |         const ord = cmpBigint(a._denom, b._denom);
// |         return reverseIf(ord, flip !== !(a._numer < 0n));
// |       }
// |       const [aInt, aRem] = bigintDivModFloor(a._numer, a._denom);
// |       const [bInt, bRem] = bigintDivModFloor(b._numer, b._denom);
// |       const ord = cmpBigint(aInt, bInt);
// |       if (ord !== 0) {
// |         return reverseIf(ord, flip);
// |       }
// |       if (aRem === 0n) {
// |         return reverseIf(bRem === 0n ? 0 : -1, flip);
// |       }
// |       if (bRem === 0n) {
// |         return reverseIf(1, flip);
// |       }
// |       // Compare the reciprocals of the remaining fractions in reverse.
// |       a = new Ratio(a._denom, aRem);
// |       b = new Ratio(b._denom, bRem);
// |       flip = !flip;
// |     }
// |   }
// |
// |   /**
// |    * `PartialEq`: `cmp(other) == Equal`, so 1/2 equals 2/4.
// |    * @param {unknown} other
// |    * @returns {boolean}
// |    */
// |   equals(other) {
// |     return other instanceof Ratio && this.cmp(other) === 0;
// |   }
// |
// |   /** `Display`: "numer" when `denom == 1`, else "numer/denom". @returns {string} */
// |   toString() {
// |     return this._denom === 1n ? `${this._numer}` : `${this._numer}/${this._denom}`;
// |   }
// |
// |   /** derived `Debug`. @returns {string} */
// |   fmtDebug() {
// |     return `Ratio { numer: ${this._numer}, denom: ${this._denom} }`;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const ZERO = new Ratio(0n, 1n);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const ONE = new Ratio(1n, 1n);

// ----------------------------------------------------------------------------
// num-rational's private `ratio_to_f64` and `ldexp`.

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2ae56ef9d6d48acd77669a26d41c2dc2f46307a540e8054ac38979ea7715784e
// | const MIN_POSITIVE = 2.2250738585072014e-308;
pub const MIN_POSITIVE: f64 = 2.2250738585072014e-308f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=37dd72dab009b97a0595266d0ecd28d286dd2628a7b61dceb8766f9adcd4a79f
// | const TWO_POW_53 = 9007199254740992;
pub const TWO_POW_53: f64 = 9007199254740992f64;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const EXACT = 1n << 53n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const view = new DataView(new ArrayBuffer(8));

// meta-language:translated JavaScript function_declaration items=1 sha256=73e68a189e84b0ccce1e2aeb1b907b2547563fc3b469c45b93a4de695aef997d
// | /**
// |  * `f64::is_normal`.
// |  * @param {number} x
// |  */
// | function isNormal(x) {
// |   return Number.isFinite(x) && Math.abs(x) >= MIN_POSITIVE;
// | }
pub fn is_normal(x: f64) -> bool {
    (x.is_finite() && ((x.abs()) >= (MIN_POSITIVE)))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `ratio_to_f64`. num-rational runs it on `i128` when both fields fit `i64`
// |  * and on `BigInt` otherwise; nothing overflows `i128` there, so one bigint
// |  * version gives the same bits.
// |  * @param {bigint} numer
// |  * @param {bigint} denom
// |  * @returns {number}
// |  */
// | function ratioToF64(numer, denom) {
// |   const floSign = Number(bigintSignum(numer)) / Number(bigintSignum(denom));
// |   if (!isNormal(floSign)) {
// |     return floSign;
// |   }
// |   if (numer >= -EXACT && numer <= EXACT && denom >= -EXACT && denom <= EXACT) {
// |     return Number(numer) / Number(denom);
// |   }
// |
// |   let n = bigintAbs(numer);
// |   let d = bigintAbs(denom);
// |   const diff = bigintBits(n) - bigintBits(d);
// |   if (diff > 1024) {
// |     return Infinity * floSign;
// |   }
// |   if (-diff > 1021 + 53 + 1) {
// |     return 0 * floSign;
// |   }
// |
// |   const shift = Math.max(diff, -1021) - 53 - 2;
// |   if (shift >= 0) {
// |     d <<= BigInt(shift);
// |   } else {
// |     n <<= BigInt(-shift);
// |   }
// |   let quotient = n / d;
// |   const remainder = n % d;
// |
// |   const nRoundingBits = BigInt(Math.max(bigintBits(quotient), -1021 - shift) - 53);
// |   const roundingBitMask = (1n << nRoundingBits) - 1n;
// |   const lsBit = (quotient & (1n << nRoundingBits)) !== 0n;
// |   const msRoundingBit = (quotient & (1n << (nRoundingBits - 1n))) !== 0n;
// |   const lsRoundingBits = (quotient & (roundingBitMask >> 1n)) !== 0n;
// |   if (msRoundingBit && (lsBit || lsRoundingBits || remainder !== 0n)) {
// |     quotient += 1n << nRoundingBits;
// |   }
// |   quotient &= ~roundingBitMask;
// |
// |   return ldexp(Number(quotient) * floSign, shift);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * num-rational's `ldexp`: `x * 2^exp` by editing the exponent bits.
// |  * @param {number} x
// |  * @param {number} exp
// |  * @returns {number}
// |  */
// | function ldexp(x, exp) {
// |   if (x === 0 || !Number.isFinite(x)) {
// |     return x;
// |   }
// |   const sign = x < 0 ? -1 : 1;
// |   if (exp > 3 * 1024) {
// |     return Infinity * sign;
// |   }
// |   if (exp < -3 * 1024) {
// |     return 0 * sign;
// |   }
// |
// |   let currExp;
// |   if (isNormal(x)) {
// |     view.setFloat64(0, x);
// |     currExp = (view.getUint32(0) >>> 20) & 0x7ff;
// |   } else {
// |     view.setFloat64(0, x * TWO_POW_53);
// |     currExp = ((view.getUint32(0) >>> 20) & 0x7ff) - 53;
// |   }
// |   const newExp = currExp + exp;
// |
// |   if (newExp > 0x7fe) {
// |     return Infinity * sign;
// |   }
// |   if (newExp > 0) {
// |     setExponent(newExp);
// |     return view.getFloat64(0);
// |   }
// |   if (newExp >= -53) {
// |     setExponent(newExp + 53);
// |     return view.getFloat64(0) / TWO_POW_53;
// |   }
// |   return 0 * sign;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Replaces the biased exponent of the f64 held in `view`.
// |  * @param {number} biased
// |  */
// | function setExponent(biased) {
// |   view.setUint32(0, ((view.getUint32(0) & 0x800fffff) | (biased << 20)) >>> 0);
// | }
