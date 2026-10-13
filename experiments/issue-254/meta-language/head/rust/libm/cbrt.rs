// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a72de5789857752dcc09a51f02374ca9ce60597583ed0387d53b2732965dd618 bytes=5494

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

// cbrt: a port of libm 0.2.16 src/math/cbrt.rs (`cbrt_round` with
// `Round::Nearest`; origin core-math src/binary64/cbrt/cbrt.c by Alexei
// Sibidanov, ported to Rust by Trevor Gross). The u64 bit manipulation is done
// with bigint; the inner `f64::fma` calls go to the libm fma port.

// meta-language:translated JavaScript import_statement items=1 sha256=6cefa3fe1679db83bb23a66acb0f089be8313535396750e89df70129c1d86125
// | import { fromBits, hf64, toBits } from './bits.js';
use crate::rust::libm::bits::{fromBits, hf64, toBits};
// meta-language:translated JavaScript import_statement items=1 sha256=1818d7f65a9383ce0d3c8e4a1f5c8f7bbba9f6260268469522adeaabe4a572a0
// | import { fma } from './fma.js';
use crate::rust::libm::fma::fma;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d6df0d3c7fcf2160e3bdf09d3bb09e91aefdab3424e7cbf8fd4f839b57b5c632
// | const MASK64 = 0xffffffffffffffffn;
pub static MASK64: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(18446744073709551615));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b0a123fde4f0b9bcb220bafea8a9d1e33404c75799d20b9122809c5370004b26
// | const SIG_MASK = 0x000fffffffffffffn;
pub static SIG_MASK: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(4503599627370495));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=9bd5c8a28bb540b68512bb10c0953e2d7d1fc77976456c1d2b887cc375b2bb0e
// | const ABS_MASK = 0x7fffffffffffffffn;
pub static ABS_MASK: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(9223372036854775807));

// meta-language:carried JavaScript lexical_declaration (type)
// | const ESCALE = [
// |   1.0,
// |   hf64('0x1.428a2f98d728bp+0'), /* 2^(1/3) */
// |   hf64('0x1.965fea53d6e3dp+0'), /* 2^(2/3) */
// | ];

// meta-language:carried JavaScript lexical_declaration (type)
// | /* the polynomial c0+c1*x+c2*x^2+c3*x^3 approximates x^(1/3) on [1,2]
// | with maximal error < 9.2e-5 (attained at x=2) */
// | const C = [
// |   hf64('0x1.1b0babccfef9cp-1'),
// |   hf64('0x1.2c9a3e94d1da5p-1'),
// |   hf64('-0x1.4dc30b1a1ddbap-3'),
// |   hf64('0x1.7a8d3e4ec9b07p-6'),
// | ];

// meta-language:carried JavaScript lexical_declaration (type)
// | const U0 = hf64('0x1.5555555555555p-2');
// meta-language:carried JavaScript lexical_declaration (type)
// | const U1 = hf64('0x1.c71c71c71c71cp-3');

// meta-language:translated JavaScript lexical_declaration items=1 sha256=185c49aa227dde028fa246ceab32be3b32380ea3d9a7cf66f8ed662164229163
// | const RSC = [1.0, -1.0, 0.5, -0.5, 0.25, -0.25];
pub static RSC: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![1f64, (-1f64), 0.5f64, (-0.5f64), 0.25f64, (-0.25f64)]);

// meta-language:carried JavaScript lexical_declaration (type)
// | /* off[Round::Nearest] */
// | const OFF = hf64('0x1p-53');

// meta-language:carried JavaScript lexical_declaration (type)
// | const P2M52 = hf64('0x1p-52');
// meta-language:carried JavaScript lexical_declaration (type)
// | const P2M60 = hf64('0x1p-60');
// meta-language:carried JavaScript lexical_declaration (type)
// | const P2M75 = hf64('0x1p-75');
// meta-language:carried JavaScript lexical_declaration (type)
// | const P2M98 = hf64('0x1p-98');

// meta-language:carried JavaScript lexical_declaration (type)
// | const AZZ_A = hf64('0x1.9b78223aa307cp+1');
// meta-language:carried JavaScript lexical_declaration (type)
// | const Y1_A = hf64('0x1.79d15d0e8d59cp+0');
// meta-language:carried JavaScript lexical_declaration (type)
// | // ~ 0x1.79d15d0e8d59b80000000000000ffc3dp+0
// | const AZZ_B = hf64('0x1.a202bfc89ddffp+2');
// meta-language:carried JavaScript lexical_declaration (type)
// | const Y1_B = hf64('0x1.de87aa837820fp+0');
// ~ 0x1.de87aa837820e80000000000001c0f08p+0

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `u64::leading_zeros` for a non-zero value below 2^64.
// |  * @param {bigint} v
// |  * @returns {number}
// |  */
// | function clz64(v) {
// |   return 64 - v.toString(2).length;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::cbrt`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function cbrt(x) {
// |   /* rm=0 for rounding to nearest, and other values for directed roundings */
// |   const hx = toBits(x);
// |   let mant = hx & SIG_MASK;
// |   const sign = Number(hx >> 63n);
// |
// |   let e = Number(hx >> 52n) & 0x7ff;
// |
// |   if (((e + 1) & 0x7ff) < 2) {
// |     const ix = hx & ABS_MASK;
// |
// |     /* 0, inf, nan: we return x + x instead of simply x,
// |     to that for x a signaling NaN, it correctly triggers
// |     the invalid exception. */
// |     if (e === 0x7ff || ix === 0n) {
// |       return x + x;
// |     }
// |
// |     const nz = clz64(ix) - 11; /* subnormal */
// |     mant <<= BigInt(nz);
// |     mant &= SIG_MASK;
// |     e = (e - (nz - 1)) >>> 0; // u32 wrapping_sub
// |   }
// |
// |   e = (e + 3072) >>> 0; // u32 wrapping_add
// |   const cvt1 = mant | (0x3ffn << 52n);
// |   let cvt5 = cvt1;
// |
// |   const et = Math.trunc(e / 3);
// |   const it = e % 3;
// |
// |   /* 2^(3k+it) <= x < 2^(3k+it+1), with 0 <= it <= 3 */
// |   cvt5 += BigInt(it) << 52n;
// |   cvt5 |= BigInt(sign) << 63n;
// |   const zz = fromBits(cvt5);
// |
// |   /* cbrt(x) = cbrt(zz)*2^(et-1365) where 1 <= zz < 8 */
// |   let isc = toBits(ESCALE[it]);
// |   isc |= BigInt(sign) << 63n;
// |   const cvt2 = isc;
// |   const z = fromBits(cvt1);
// |
// |   /* cbrt(zz) = cbrt(z)*isc, where isc encodes 1, 2^(1/3) or 2^(2/3),
// |   and 1 <= z < 2 */
// |   const r = 1.0 / z;
// |   const rr = r * RSC[(it << 1) | sign];
// |   const z2 = z * z;
// |   const c0 = C[0] + z * C[1];
// |   const c2 = C[2] + z * C[3];
// |   let y = c0 + z2 * c2;
// |   let y2 = y * y;
// |
// |   /* y is an approximation of z^(1/3) */
// |   let h = y2 * (y * r) - 1.0;
// |
// |   /* h determines the error between y and z^(1/3) */
// |   y -= (h * y) * (U0 - U1 * h);
// |
// |   /* The correction y -= (h*y)*(u0 - u1*h) corresponds to a cubic variant
// |   of Newton's method, with the function f(y) = 1-z/y^3. */
// |   y *= fromBits(cvt2);
// |
// |   /* Now y is an approximation of zz^(1/3),
// |    * and rr an approximation of 1/zz. We now perform another iteration of
// |    * Newton-Raphson, this time with a linear approximation only. */
// |   y2 = y * y;
// |   let y2l = fma(y, y, -y2);
// |
// |   /* y2 + y2l = y^2 exactly */
// |   let y3 = y2 * y;
// |   let y3l = fma(y, y2, -y3) + y * y2l;
// |
// |   /* y3 + y3l approximates y^3 with about 106 bits of accuracy */
// |   h = ((y3 - zz) + y3l) * rr;
// |   let dy = h * (y * U0);
// |
// |   /* the approximation of zz^(1/3) is y - dy */
// |   let y1 = y - dy;
// |   dy = (y - y1) - dy;
// |
// |   /* the approximation of zz^(1/3) is now y1 + dy, where |dy| < 1/2 ulp(y)
// |    * (for rounding to nearest) */
// |   let ady = Math.abs(dy);
// |
// |   /* For directed roundings, ady0 is tiny when dy is tiny, or ady0 is near
// |    * from ulp(1);
// |    * for rounding to nearest, ady0 is tiny when dy is near from 1/2 ulp(1),
// |    * or from 3/2 ulp(1). */
// |   let ady0 = Math.abs(ady - OFF);
// |   let ady1 = Math.abs(ady - (P2M52 + OFF));
// |
// |   if (ady0 < P2M75 || ady1 < P2M75) {
// |     y2 = y1 * y1;
// |     y2l = fma(y1, y1, -y2);
// |     y3 = y2 * y1;
// |     y3l = fma(y1, y2, -y3) + y1 * y2l;
// |     h = ((y3 - zz) + y3l) * rr;
// |     dy = h * (y1 * U0);
// |     y = y1 - dy;
// |     dy = (y1 - y) - dy;
// |     y1 = y;
// |     ady = Math.abs(dy);
// |     ady0 = Math.abs(ady - OFF);
// |     ady1 = Math.abs(ady - (P2M52 + OFF));
// |
// |     if (ady0 < P2M98 || ady1 < P2M98) {
// |       const azz = Math.abs(zz);
// |
// |       // `copysign(c, zz)`: zz carries the sign of x
// |       if (azz === AZZ_A) {
// |         y1 = sign !== 0 ? -Y1_A : Y1_A;
// |       }
// |
// |       if (azz === AZZ_B) {
// |         y1 = sign !== 0 ? -Y1_B : Y1_B;
// |       }
// |
// |       // The `round != Round::Nearest` table is not needed here.
// |     }
// |   }
// |
// |   let cvt3 = toBits(y1);
// |   cvt3 = (cvt3 + ((BigInt((et - 342 - 1023) >>> 0) << 52n) & MASK64)) & MASK64;
// |   const m0 = (cvt3 << 30n) & MASK64;
// |   const m1 = m0 >> 63n;
// |
// |   if ((m0 ^ m1) <= 1n << 30n) {
// |     let cvt4 = toBits(y1);
// |     cvt4 = (cvt4 + (164n << 15n)) & 0xffffffffffff0000n;
// |
// |     if (Math.abs((fromBits(cvt4) - y1) - dy) < P2M60 || Math.abs(zz) === 1.0) {
// |       cvt3 = (cvt3 + (1n << 15n)) & 0xffffffffffff0000n;
// |     }
// |   }
// |
// |   return fromBits(cvt3);
// | }
