// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ac6becd41792833c3f0a794731326d3bc97522a3e59e8ef7f56b469bc391c169 bytes=8081

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

// fma: a port of libm 0.2.16 src/math/generic/fma.rs (`fma_round` for f64,
// origin musl src/math/fma.c).
//
// On x86_64 and aarch64 libm uses the FMA instruction, which is correctly
// rounded. This port is correctly rounded too: it follows generic/fma.rs line
// by line except for one sticky bit (see below) where the Rust code loses
// precision. The u64/u128 integer arithmetic of the Rust code is done with
// bigint, reduced modulo 2^64 wherever Rust wraps or truncates.

// meta-language:carried JavaScript import_statement (unsupported)
// | import { fromBits, scalbn, toBits } from './bits.js';

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d6df0d3c7fcf2160e3bdf09d3bb09e91aefdab3424e7cbf8fd4f839b57b5c632
// | const MASK64 = 0xffffffffffffffffn;
pub static MASK64: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(18446744073709551615));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b0a123fde4f0b9bcb220bafea8a9d1e33404c75799d20b9122809c5370004b26
// | const SIG_MASK = 0x000fffffffffffffn;
pub static SIG_MASK: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(4503599627370495));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=8871939c20085f2c91dabb7f817103a0b26f80fb7a06edda0379adaa54061937
// | const IMPLICIT_BIT = 0x0010000000000000n;
pub static IMPLICIT_BIT: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(4503599627370496));

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b156827ff9866368884373498d94748988164271b525fc71de50fa6320e72da7
// | /* Norm::<f64>::EXP_UNBIAS = EXP_BIAS + SIG_BITS + 1 */
// | const EXP_UNBIAS = 1023 + 52 + 1;
pub static EXP_UNBIAS: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| ((1023f64 + 52f64) + 1f64));
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /* Norm::<f64>::ZERO_INF_NAN = EXP_SAT - EXP_UNBIAS */
// | const ZERO_INF_NAN = 2047 - EXP_UNBIAS;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /* F::from_parts(false, BITS - 1 + EXP_BIAS, 0) = 2^63 */
// | const SCALE_F = fromBits(BigInt(63 + 1023) << 52n);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=9eb192ba94fba959ee95b43937742b3a408dbb65ea91ed8bab4c4661d10caad8
// | /* max_pow = BITS - 1 + EXP_BIAS */
// | const MAX_POW = 64 - 1 + 1023;
pub static MAX_POW: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| ((64f64 - 1f64) + 1023f64));
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /* F::from_parts(false, max_pow, 0) = 2^63 */
// | const C_MAX_POW = fromBits(BigInt(MAX_POW) << 52n);
// meta-language:carried JavaScript lexical_declaration (type)
// | const MIN_POSITIVE_NORMAL = fromBits(0x0010000000000000n);
// 2^-1022

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `u64::leading_zeros` for a non-zero value below 2^64.
// |  * @param {bigint} v
// |  * @returns {number}
// |  */
// | function clz64(v) {
// |   let n = 0;
// |   if (v >> 32n === 0n) {
// |     n += 32;
// |     v <<= 32n;
// |   }
// |   if (v >> 48n === 0n) {
// |     n += 16;
// |     v <<= 16n;
// |   }
// |   if (v >> 56n === 0n) {
// |     n += 8;
// |     v <<= 8n;
// |   }
// |   if (v >> 60n === 0n) {
// |     n += 4;
// |     v <<= 4n;
// |   }
// |   if (v >> 62n === 0n) {
// |     n += 2;
// |     v <<= 2n;
// |   }
// |   if (v >> 63n === 0n) {
// |     n += 1;
// |   }
// |   return n;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `(x << s) as u64` with `0 <= s < 64`.
// |  * @param {bigint} x
// |  * @param {number} s
// |  * @returns {bigint}
// |  */
// | function shl(x, s) {
// |   return (x << BigInt(s)) & MASK64;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `x >> s` with `0 <= s < 64`.
// |  * @param {bigint} x
// |  * @param {number} s
// |  * @returns {bigint}
// |  */
// | function shr(x, s) {
// |   return x >> BigInt(s);
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * @typedef {{ m: bigint, e: number, neg: boolean }} Norm
// |  */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Norm::from_float`: the significand with the implicit bit and a guard bit,
// |  * and the exponent such that `m * 2^e = |x|`; zero gets the sentinel exponent
// |  * `(1 << EXP_BITS) - EXP_UNBIAS`.
// |  * @param {number} x
// |  * @returns {Norm}
// |  */
// | function norm(x) {
// |   let ix = toBits(x);
// |   let e = Number((ix >> 52n) & 0x7ffn);
// |   const neg = ix >> 63n !== 0n;
// |   if (e === 0) {
// |     // Normalize subnormals by multiplication
// |     const scaled = x * SCALE_F;
// |     ix = toBits(scaled);
// |     e = Number((ix >> 52n) & 0x7ffn);
// |     e = e === 0 ? 1 << 11 : e - 63;
// |   }
// |
// |   e -= EXP_UNBIAS;
// |
// |   // Absolute value, set the implicit bit, and shift to create a guard bit
// |   ix &= SIG_MASK;
// |   ix |= IMPLICIT_BIT;
// |   ix <<= 1n;
// |
// |   return { m: ix, e, neg };
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::fma` (Rust `f64::mul_add`): `x * y + z` with a single rounding.
// |  * @param {number} x
// |  * @param {number} y
// |  * @param {number} z
// |  * @returns {number}
// |  */
// | export function fma(x, y, z) {
// |   const nx = norm(x);
// |   const ny = norm(y);
// |   const nz = norm(z);
// |
// |   if (nx.e >= ZERO_INF_NAN || ny.e >= ZERO_INF_NAN) {
// |     // Value will overflow, defer to non-fused operations.
// |     return x * y + z;
// |   }
// |
// |   if (nz.e >= ZERO_INF_NAN) {
// |     if (nz.e > ZERO_INF_NAN) {
// |       // Empty add component means we only need to multiply.
// |       return x * y;
// |     }
// |     // `z` is NaN or infinity, which sets the result.
// |     return z;
// |   }
// |
// |   // multiply: r = x * y
// |   let zhi;
// |   let zlo;
// |   const prod = nx.m * ny.m;
// |   let rlo = prod & MASK64;
// |   let rhi = prod >> 64n;
// |
// |   // Exponent result of multiplication
// |   let e = nx.e + ny.e;
// |   // Needed shift to align `z` to the multiplication result
// |   let d = nz.e - e;
// |   const sbits = 64;
// |
// |   // Scale `z`. Shift `z <<= kz`, `r >>= kr`, so `kz+kr == d`, set `e = e+kr` (== ez-kz)
// |   if (d > 0) {
// |     // The magnitude of `z` is larger than `x * y`
// |     if (d < sbits) {
// |       zlo = shl(nz.m, d);
// |       zhi = shr(nz.m, sbits - d);
// |     } else {
// |       zlo = 0n;
// |       zhi = nz.m;
// |       d -= sbits;
// |
// |       // `z`'s exponent is large enough that it now needs to be taken into account.
// |       e = nz.e - sbits;
// |
// |       if (d === 0) {
// |         // Exactly `sbits`, nothing to do
// |       } else if (d < sbits) {
// |         // Remaining shift fits within `sbits`. Leave `z` in place, shift `x * y`
// |         // and set the sticky bit from the bits shifted out of `rlo`.
// |         //
// |         // Deliberate deviation: generic/fma.rs tests the *updated* `rlo`
// |         // (`rlo = ...; rlo |= (rlo << (sbits - d)) != 0`), which drops the
// |         // sticky bit and rounds e.g. fma(1 + 2^-26, 1 - 2^-26 + 2^-52, 2^53)
// |         // to 2^53 instead of 2^53 + 2. musl's fma.c, the FMA instruction that
// |         // libm uses on x86_64 and aarch64, and IEEE 754 all give 2^53 + 2, so
// |         // this follows them.
// |         const sticky = shl(rlo, sbits - d) !== 0n ? 1n : 0n;
// |         rlo = shl(rhi, sbits - d) | shr(rlo, d) | sticky;
// |         rhi = shr(rhi, d);
// |       } else {
// |         // `z`'s magnitude is enough that `x * y` is irrelevant. It was nonzero, so set
// |         // the sticky bit.
// |         rlo = 1n;
// |         rhi = 0n;
// |       }
// |     }
// |   } else {
// |     // `z`'s magnitude once shifted fits entirely within `zlo`
// |     zhi = 0n;
// |     d = -d;
// |     if (d === 0) {
// |       // No shift needed
// |       zlo = nz.m;
// |     } else if (d < sbits) {
// |       // Shift s.t. `nz.m` fits into `zlo`
// |       const sticky = shl(nz.m, sbits - d) !== 0n ? 1n : 0n;
// |       zlo = shr(nz.m, d) | sticky;
// |     } else {
// |       // Would be entirely shifted out, only set the sticky bit
// |       zlo = 1n;
// |     }
// |   }
// |
// |   /* addition */
// |
// |   let neg = nx.neg !== ny.neg;
// |   const samesign = !neg !== nz.neg;
// |   let rhiNonzero = true;
// |
// |   if (samesign) {
// |     // r += z
// |     rlo = (rlo + zlo) & MASK64;
// |     rhi = (rhi + zhi + (rlo < zlo ? 1n : 0n)) & MASK64;
// |   } else {
// |     // r -= z
// |     const borrow = rlo < zlo ? 1n : 0n;
// |     rlo = (rlo - zlo) & MASK64;
// |     rhi = (rhi - ((zhi + borrow) & MASK64)) & MASK64;
// |     if (rhi >> 63n !== 0n) {
// |       rlo = -rlo & MASK64;
// |       rhi = (-rhi & MASK64) - (rlo !== 0n ? 1n : 0n);
// |       neg = !neg;
// |     }
// |     rhiNonzero = rhi !== 0n;
// |   }
// |
// |   /* Construct result */
// |
// |   // Shift result into `rhi`, left-aligned. Last bit is sticky
// |   if (rhiNonzero) {
// |     // `d` > 0, need to shift both `rhi` and `rlo` into result
// |     e += sbits;
// |     d = clz64(rhi) - 1;
// |     rhi = shl(rhi, d) | shr(rlo, sbits - d);
// |     // Update sticky
// |     rhi |= shl(rlo, d) !== 0n ? 1n : 0n;
// |   } else if (rlo !== 0n) {
// |     // `rhi` is zero, `rlo` is the entire result and needs to be shifted
// |     d = clz64(rlo) - 1;
// |     if (d < 0) {
// |       // Shift and set sticky
// |       rhi = (rlo >> 1n) | (rlo & 1n);
// |     } else {
// |       rhi = shl(rlo, d);
// |     }
// |   } else {
// |     // exact +/- 0.0
// |     return x * y + z;
// |   }
// |
// |   e -= d;
// |
// |   // Use int->float conversion to populate the significand.
// |   // i is in [1 << (BITS - 2), (1 << (BITS - 1)) - 1]
// |   let i = rhi;
// |
// |   if (neg) {
// |     i = -i;
// |   }
// |
// |   // `|r|` is in `[0x1p62,0x1p63]` for `f64`; Number(bigint) rounds to nearest, ties to even
// |   let r = Number(i);
// |
// |   /* Account for subnormal and rounding */
// |
// |   if (e < -(MAX_POW - 2)) {
// |     // Result is subnormal before rounding
// |     if (e === -(MAX_POW - 1)) {
// |       let c = C_MAX_POW;
// |       if (neg) {
// |         c = -c;
// |       }
// |
// |       if (r === c) {
// |         // Min normal after rounding,
// |         return r < 0 ? -MIN_POSITIVE_NORMAL : MIN_POSITIVE_NORMAL;
// |       }
// |
// |       if (shl(rhi, 52 + 1) !== 0n) {
// |         // Account for truncated bits. One bit will be lost in the `scalbn` call, add
// |         // another top bit to avoid double rounding if inexact.
// |         const iu = (rhi >> 1n) | (rhi & 1n) | (1n << 62n);
// |         i = iu;
// |
// |         if (neg) {
// |           i = -i;
// |         }
// |
// |         r = Number(i);
// |
// |         // Remove the top bit
// |         r = 2 * r - c;
// |       }
// |     } else {
// |       // Only round once when scaled
// |       d = 11 - 1;
// |       const sticky = shl(rhi, sbits - d) !== 0n ? 1n : 0n;
// |       i = ((rhi >> BigInt(d)) | sticky) << BigInt(d);
// |
// |       if (neg) {
// |         i = -i;
// |       }
// |
// |       r = Number(i);
// |     }
// |   }
// |
// |   // Use our exponent to scale the final value.
// |   return scalbn(r, e);
// | }
