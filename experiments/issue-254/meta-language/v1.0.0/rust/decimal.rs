// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4303f78492ace159fa9a3c30d180ff4ae6e5b10517839ca3c421b287bf4fa223 bytes=34671

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

/// The Math functions that Rust's f64 methods do not share with JavaScript.
pub mod ml_math {
    /// Math.round: the nearest integer, the one towards +Infinity on a tie,
    /// with the sign of a zero result from the argument.
    pub fn round(x: f64) -> f64 {
        if !x.is_finite() || x == 0.0 {
            return x;
        }
        if x > 0.0 && x < 0.5 {
            return 0.0;
        }
        if x < 0.0 && x >= -0.5 {
            return -0.0;
        }
        let floor = x.floor();
        if x - floor >= 0.5 {
            floor + 1.0
        } else {
            floor
        }
    }

    /// Math.sign, which keeps -0, 0 and NaN, where f64::signum does not.
    pub fn sign(x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            x
        }
    }

    /// Math.max of two Numbers: NaN when either is, and 0 above -0.
    pub fn max(a: f64, b: f64) -> f64 {
        if a > b {
            a
        } else if b > a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                b
            } else {
                a
            }
        } else {
            f64::NAN
        }
    }

    /// Math.min of two Numbers: NaN when either is, and -0 below 0.
    pub fn min(a: f64, b: f64) -> f64 {
        if a < b {
            a
        } else if b < a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                a
            } else {
                b
            }
        } else {
            f64::NAN
        }
    }

    pub fn max_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::NEG_INFINITY, |a, &b| max(a, b))
    }

    pub fn min_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::INFINITY, |a, &b| min(a, b))
    }

    pub fn is_integer(x: f64) -> bool {
        x.is_finite() && x.trunc() == x
    }

    pub fn is_safe_integer(x: f64) -> bool {
        is_integer(x) && x.abs() <= 9007199254740991.0
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * An exact emulation of the `rust_decimal` crate, version 1.43.0: a 96-bit
// |  * unsigned mantissa, a scale of 0 to 28 and a sign flag. Rounding, overflow,
// |  * errors and negative zero match the crate, so results are byte-identical to
// |  * the Rust engine. The arithmetic uses `BigInt` and replicates every place
// |  * where the crate's 32-bit limb algorithms round, truncate or wrap. File
// |  * references are to `rust_decimal-1.43.0/src/`.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { err, panic } from './core.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { debugStr } from './str.js';

// meta-language:translated JavaScript lexical_declaration items=1 sha256=090e99fd441b272765b948e9e53ce66f1900493078a1c8778e5473fb65dedd6f
// | const MAX_SCALE = 28;
pub const MAX_SCALE: f64 = 28f64;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const [U32, U96] = [1n << 32n, 1n << 96n];
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const [M32, U64_MASK, U96_MASK] = [U32 - 1n, (1n << 64n) - 1n, U96 - 1n];
// meta-language:translated JavaScript lexical_declaration items=1 sha256=87e4de4c815c9b6a02fe48b9e75d84dc458cdc3aa5a4122fd0cffa9498b274ad
// | /** `constants::WILL_OVERFLOW_U64` and `constants::BYTES_TO_OVERFLOW_U64`. */
// | const WILL_OVERFLOW_U64 = 1844674407370954906n;
pub static WILL_OVERFLOW_U64: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(1844674407370954906));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d503ef4e9eaa61f86eb4d26c6050a191c2687474c6614f76b25919e2825ec057
// | const BYTES_TO_OVERFLOW_U64 = 18;
pub const BYTES_TO_OVERFLOW_U64: f64 = 18f64;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `POW10[i]` is `10^i`. */
// | const POW10 = Array.from({ length: 58 }, (_, i) => 10n ** BigInt(i));
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /**
// |  * `10f64.powi(s)`: compiler-builtins `__powidf2`, exponentiation by squaring
// |  * (so the large powers are not the correctly rounded `10 ** s`).
// |  */
// | const POWI10 = Array.from({ length: MAX_SCALE + 1 }, (_, s) => {
// |   let r = 1;
// |   for (let a = 10, b = s; b > 0; b >>= 1, a *= a) if (b & 1) r *= a;
// |   return r;
// | });

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** The Display text of the unit variants of `rust_decimal::Error`. */
// | const ERROR_TEXT = {
// |   ExceedsMaximumPossibleValue: 'Number exceeds maximum value that can be represented.',
// |   LessThanMinimumPossibleValue: 'Number less than minimum value that can be represented.',
// |   Underflow: 'Number has a high precision that can not be represented.',
// | };

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `rust_decimal::Error` (error.rs). Variants: `ErrorString(_0: string)`,
// |  * `ExceedsMaximumPossibleValue`, `LessThanMinimumPossibleValue`, `Underflow`,
// |  * `ScaleExceedsMaximumPrecision(_0: number)`, `ConversionTo(_0: string)`.
// |  */
// | export class DecimalError {
// |   /**
// |    * @param {string} tag
// |    * @param {object} [fields]
// |    */
// |   constructor(tag, fields) {
// |     /** @type {string} */
// |     this.tag = tag;
// |     Object.assign(this, fields);
// |     Object.freeze(this);
// |   }
// |
// |   /** @param {string} message */
// |   static ErrorString(message) {
// |     return new DecimalError('ErrorString', { _0: message });
// |   }
// |
// |   static ExceedsMaximumPossibleValue = new DecimalError('ExceedsMaximumPossibleValue');
// |   static LessThanMinimumPossibleValue = new DecimalError('LessThanMinimumPossibleValue');
// |   static Underflow = new DecimalError('Underflow');
// |
// |   /** @param {number} scale */
// |   static ScaleExceedsMaximumPrecision(scale) {
// |     return new DecimalError('ScaleExceedsMaximumPrecision', { _0: scale });
// |   }
// |
// |   /** @param {string} typeName */
// |   static ConversionTo(typeName) {
// |     return new DecimalError('ConversionTo', { _0: typeName });
// |   }
// |
// |   /** `impl Display for Error`. */
// |   toString() {
// |     const value = /** @type {any} */ (this)._0;
// |     if (this.tag === 'ErrorString') return value;
// |     if (this.tag === 'ConversionTo') return `Error while converting to ${value}`;
// |     if (this.tag !== 'ScaleExceedsMaximumPrecision') return ERROR_TEXT[this.tag];
// |     return `Scale exceeds the maximum precision allowed: ${value} > ${MAX_SCALE}`;
// |   }
// |
// |   /** `#[derive(Debug)]`. */
// |   fmtDebug() {
// |     const value = /** @type {any} */ (this)._0;
// |     if (value === undefined) return this.tag;
// |     return `${this.tag}(${typeof value === 'string' ? debugStr(value) : value})`;
// |   }
// | }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=563f6c606c138e86b3c41ef4bcee8ff488a0dcc80d600d1e5009dec9b9556601
// | /** Internal `CalculationResult::Overflow`. */
// | const OVERFLOW = 'Overflow';
pub const OVERFLOW: &str = "Overflow";
// meta-language:translated JavaScript lexical_declaration items=1 sha256=14fd3da4e7c236e6b3a596da962361283039cca909832215d1293191ba2a72a5
// | /** Internal `CalculationResult::DivByZero`. */
// | const DIV_BY_ZERO = 'DivByZero';
pub const DIV_BY_ZERO: &str = "DivByZero";

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `rust_decimal::Decimal`: the value `(-1)^negative * magnitude / 10^places`.
// |  * Instances are frozen. A zero with the sign flag (`-0`) is representable
// |  * and prints as `-0`, as in the crate.
// |  */
// | export class RustDecimal {
// |   /**
// |    * The raw constructor (`from_parts_raw`): the sign flag is kept even for a
// |    * zero mantissa.
// |    * @param {bigint} magnitude the mantissa, `0 <= magnitude < 2^96`
// |    * @param {number} places the scale, 0 to 28
// |    * @param {boolean} negative the sign flag
// |    */
// |   constructor(magnitude, places, negative) {
// |     /** @type {bigint} */
// |     this.magnitude = magnitude;
// |     /** @type {number} */
// |     this.places = places;
// |     /** @type {boolean} */
// |     this.negative = negative;
// |     Object.freeze(this);
// |   }
// |
// |   /** @type {RustDecimal} `Decimal::ZERO` */
// |   static ZERO = new RustDecimal(0n, 0, false);
// |   /** @type {RustDecimal} `Decimal::ONE` */
// |   static ONE = new RustDecimal(1n, 0, false);
// |
// |   /**
// |    * `Decimal::from(i64)` (and, below, `Decimal::from(i32)`).
// |    * @param {number | bigint} value an integer in the i64 range
// |    */
// |   static fromI64(value) {
// |     const v = BigInt(value);
// |     return v < 0n ? new RustDecimal(-v, 0, true) : new RustDecimal(v, 0, false);
// |   }
// |
// |   /** @param {number} value */
// |   static fromI32(value) {
// |     return RustDecimal.fromI64(value);
// |   }
// |
// |   /**
// |    * `FromStr` (str.rs `parse_str_radix_10`): on failure, text containing an
// |    * 'e' or 'E' is retried with `from_scientific_lossy`. Throws
// |    * `new Err(DecimalError)`.
// |    * @param {string} s
// |    * @returns {RustDecimal}
// |    */
// |   static fromStr(s) {
// |     const parsed = parseStrRadix10(s);
// |     if (parsed instanceof RustDecimal) return parsed;
// |     if (s.includes('e') || s.includes('E')) return fromScientific(s, true);
// |     return err(parsed);
// |   }
// |
// |   /**
// |    * `Decimal::from_scientific` (decimal.rs). Throws `new Err(DecimalError)`.
// |    * @param {string} s
// |    * @returns {RustDecimal}
// |    */
// |   static fromScientific(s) {
// |     return fromScientific(s, false);
// |   }
// |
// |   /**
// |    * `Decimal::try_from(f64).ok()` (decimal.rs `from_f64` with
// |    * `remove_excess_bits`): `null` for NaN, infinities and values too large.
// |    * @param {number} x
// |    * @returns {RustDecimal | null}
// |    */
// |   static tryFromF64(x) {
// |     return fromF64(x);
// |   }
// |
// |   /** `is_zero`. */
// |   isZero() { return this.magnitude === 0n; }
// |
// |   /** `is_sign_negative`: the sign flag, so also true for `-0`. */
// |   isSignNegative() { return this.negative; }
// |
// |   /** `abs`. */
// |   abs() { return this.negative ? new RustDecimal(this.magnitude, this.places, false) : this; }
// |
// |   /** `Neg`: flips the sign flag, also of zero. */
// |   neg() { return new RustDecimal(this.magnitude, this.places, !this.negative); }
// |
// |   /** `scale`. */
// |   scale() { return this.places; }
// |
// |   /** `mantissa`: the signed mantissa (an `i128`). */
// |   mantissa() { return this.negative ? -this.magnitude : this.magnitude; }
// |
// |   /**
// |    * `ToPrimitive::to_f64` (decimal.rs): the integral and fractional parts
// |    * are converted separately, then rounded to `scale` places with
// |    * `f64::round` and `powi`. Always a number for a valid decimal.
// |    * @returns {number | null}
// |    */
// |   toF64() {
// |     const m = this.magnitude;
// |     const s = this.places;
// |     if (s === 0) return Number(this.negative ? -m : m);
// |     const precision = POW10[s];
// |     const integral = Number(m / precision);
// |     const frac = Number(m % precision) / Number(precision);
// |     if (frac === 0) return this.negative ? -integral : integral;
// |     const roundTo = POWI10[s];
// |     const rounded = roundHalfAwayFromZero((integral + frac) * roundTo) / roundTo;
// |     return this.negative ? -rounded : rounded;
// |   }
// |
// |   /**
// |    * `round_dp`: banker's rounding (`MidpointNearestEven`, decimal.rs
// |    * `round_dp_with_strategy`) to `dp` places.
// |    * @param {number} dp
// |    */
// |   roundDp(dp) {
// |     const oldScale = this.places;
// |     if (oldScale <= dp) return this;
// |     if (this.magnitude === 0n) return new RustDecimal(0n, dp, this.negative);
// |     const diff = oldScale - dp;
// |     const power = POW10[diff];
// |     let value = this.magnitude / power;
// |     const portion = this.magnitude - value * power;
// |     const cap = 5n * POW10[diff - 1];
// |     if (portion > cap || (portion === cap && (value & 1n) === 1n)) value += 1n;
// |     return fromParts(value, this.negative, dp);
// |   }
// |
// |   /** `normalize`: strips trailing zeros; every zero becomes `0`. */
// |   normalize() {
// |     if (this.magnitude === 0n) return RustDecimal.ZERO;
// |     let m = this.magnitude;
// |     let s = this.places;
// |     for (; s > 0 && m % 10n === 0n; s--) m /= 10n;
// |     return s === this.places ? this : new RustDecimal(m, s, this.negative);
// |   }
// |
// |   // The `checked_*` methods return `null` on overflow or division by zero;
// |   // the operators (arithmetic_impls.rs) panic instead, with the crate's
// |   // messages ("Addition overflowed", "Division by zero", ...).
// |
// |   /** @param {RustDecimal} other */
// |   checkedAdd(other) { return okOrNull(addSub(this, other, false)); }
// |   /** @param {RustDecimal} other */
// |   checkedSub(other) { return okOrNull(addSub(this, other, true)); }
// |   /** @param {RustDecimal} other */
// |   checkedMul(other) { return okOrNull(mul(this, other)); }
// |   /** @param {RustDecimal} other */
// |   checkedDiv(other) { return okOrNull(div(this, other)); }
// |   /** @param {RustDecimal} other */
// |   checkedRem(other) { return okOrNull(rem(this, other)); }
// |   /** @param {RustDecimal} other */
// |   add(other) { return orPanic(addSub(this, other, false), 'Addition overflowed'); }
// |   /** @param {RustDecimal} other */
// |   sub(other) { return orPanic(addSub(this, other, true), 'Subtraction overflowed'); }
// |   /** @param {RustDecimal} other */
// |   mul(other) { return orPanic(mul(this, other), 'Multiplication overflowed'); }
// |   /** @param {RustDecimal} other */
// |   div(other) { return orPanic(div(this, other), 'Division overflowed'); }
// |   /** @param {RustDecimal} other */
// |   rem(other) { return orPanic(rem(this, other), 'Division overflowed'); }
// |
// |   /**
// |    * `Ord::cmp` (ops/cmp.rs): numeric, so `-0 == 0` and `1.0 == 1`.
// |    * @param {RustDecimal} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     if (other.magnitude === 0n) return this.magnitude === 0n ? 0 : this.negative ? -1 : 1;
// |     if (this.magnitude === 0n) return other.negative ? 1 : -1;
// |     if (this.negative !== other.negative) return this.negative ? -1 : 1;
// |     let x = this.magnitude;
// |     let y = other.magnitude;
// |     if (this.places < other.places) x *= POW10[other.places - this.places];
// |     else if (this.places > other.places) y *= POW10[this.places - other.places];
// |     if (x === y) return 0;
// |     return (x < y) !== this.negative ? -1 : 1;
// |   }
// |
// |   /**
// |    * `PartialEq`: numeric equality (`cmp == Equal`).
// |    * @param {unknown} other
// |    */
// |   equals(other) {
// |     return other instanceof RustDecimal && this.cmp(other) === 0;
// |   }
// |
// |   /** `Display` (str.rs `to_str_internal`): `-0` and `-0.00` keep their sign. */
// |   toString() {
// |     const s = this.places;
// |     const digits = (this.magnitude === 0n ? '' : this.magnitude.toString()).padStart(s, '0');
// |     const whole = digits.slice(0, digits.length - s) || '0';
// |     const rep = s === 0 ? whole : `${whole}.${digits.slice(digits.length - s)}`;
// |     return this.negative ? `-${rep}` : rep;
// |   }
// |
// |   /** `Debug`: the same text as `Display`. */
// |   fmtDebug() { return this.toString(); }
// |
// |   /** `Serialize` (serde.rs, the default string form): the Display text. */
// |   toJSON() { return this.toString(); }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Decimal::from_parts`: the sign of a zero mantissa is dropped.
// |  * @param {bigint} magnitude @param {boolean} negative @param {number} scale
// |  */
// | function fromParts(magnitude, negative, scale) {
// |   return new RustDecimal(magnitude, scale, magnitude === 0n ? false : negative);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** `CalculationResult` to `Option`. @param {RustDecimal | string} result @returns {RustDecimal | null} */
// | function okOrNull(result) {
// |   return result instanceof RustDecimal ? result : null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Unwraps an operator's result, or panics with "Division by zero" or `message`.
// |  * @param {RustDecimal | string} result @param {string} message
// |  */
// | function orPanic(result, message) {
// |   return result instanceof RustDecimal ? result : panic(result === DIV_BY_ZERO ? 'Division by zero' : message);
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=71aca32b31eb7f0c43a62eb225035189d601a93aa8b4d4fd0f6bd622400bfb81
// | /** `f64::round`: half away from zero. @param {number} x */
// | function roundHalfAwayFromZero(x) {
// |   const t = Math.trunc(x);
// |   return Math.abs(x - t) >= 0.5 ? t + Math.sign(x) : t;
// | }
pub fn round_half_away_from_zero(x: f64) -> f64 {
    {
        let t = x.trunc();
        if ((((x - t)).abs()) >= (0.5f64)) {
            (t + crate::ml_math::sign(x))
        } else {
            t
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf24::rescale` (ops/common.rs): reduces a wide sum or product to 96 bits
// |  * and at most 28 places, rounding half to even with a sticky bit. `null` is
// |  * an overflow.
// |  * @param {bigint} value @param {number} scale
// |  * @returns {{ value: bigint, scale: number } | null}
// |  */
// | function rescaleBuf(value, scale) {
// |   let v = value;
// |   let target = 0;
// |   if (v >= U96) {
// |     // upper_word_index * 32 + leading bit, approximated with `* 77 >> 8`.
// |     target = (((v.toString(2).length - 97) * 77) >> 8) + 1;
// |     if (target > scale) return null;
// |   }
// |   if (target < scale - MAX_SCALE) target = scale - MAX_SCALE;
// |   if (target <= 0) return { value: v, scale };
// |   scale -= target;
// |   let sticky = false;
// |   let remainder = 0n;
// |   for (;;) {
// |     sticky = sticky || remainder !== 0n;
// |     const power = target > 8 ? POW10[9] : POW10[target];
// |     remainder = v % power;
// |     v /= power;
// |     if (target > 9) {
// |       target -= 9;
// |       continue;
// |     }
// |     if (v >= U96) {
// |       if (scale === 0) return null;
// |       target = 1;
// |       scale -= 1;
// |       continue;
// |     }
// |     const half = power >> 1n;
// |     if (half <= remainder && (half < remainder || (v & 1n) === 1n || sticky)) {
// |       v += 1n;
// |       if (v === U96) {
// |         if (scale === 0) return null;
// |         sticky = false;
// |         remainder = 0n;
// |         target = 1;
// |         scale -= 1;
// |         continue;
// |       }
// |     }
// |     return { value: v, scale };
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `add_sub_internal` (ops/add.rs).
// |  * @param {RustDecimal} d1 @param {RustDecimal} d2 @param {boolean} subtract
// |  * @returns {RustDecimal | string}
// |  */
// | function addSub(d1, d2, subtract) {
// |   if (d1.magnitude === 0n) {
// |     // 0 + x and 0 - x return x itself (with its scale; -x flips the sign).
// |     if (subtract && d2.magnitude !== 0n) return new RustDecimal(d2.magnitude, d2.places, !d2.negative);
// |     return d2;
// |   }
// |   if (d2.magnitude === 0n) return d1;
// |   const scale = Math.max(d1.places, d2.places);
// |   const a = d1.magnitude * POW10[scale - d1.places];
// |   const b = d2.magnitude * POW10[scale - d2.places];
// |   const sum = (d1.negative ? -a : a) + (d2.negative !== subtract ? -b : b);
// |   const negative = sum < 0n;
// |   const magnitude = negative ? -sum : sum;
// |   if (magnitude < U96) return fromParts(magnitude, negative, scale);
// |   const r = rescaleBuf(magnitude, scale);
// |   return r === null ? OVERFLOW : fromParts(r.value, negative, r.scale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `mul_impl` (ops/mul.rs).
// |  * @param {RustDecimal} d1 @param {RustDecimal} d2
// |  * @returns {RustDecimal | string}
// |  */
// | function mul(d1, d2) {
// |   const m1 = d1.magnitude;
// |   const m2 = d2.magnitude;
// |   if (m1 === 0n || m2 === 0n) return RustDecimal.ZERO;
// |   let scale = d1.places + d2.places;
// |   const negative = d1.negative !== d2.negative;
// |   let product = m1 * m2;
// |   if (m1 < U32 && m2 < U32) {
// |     // The 32-bit fast path rounds half to even without a sticky bit.
// |     if (scale > MAX_SCALE) {
// |       if (scale > MAX_SCALE + 19) return RustDecimal.ZERO;
// |       const power = POW10[scale - MAX_SCALE];
// |       const quotient = product / power;
// |       const remainder = product - quotient * power;
// |       product = quotient;
// |       const half = power >> 1n;
// |       if (remainder >= half && (remainder > half || (product & 1n) === 1n)) product += 1n;
// |       scale = MAX_SCALE;
// |     }
// |     return fromParts(product, negative, scale);
// |   }
// |   if (product >= U96 || scale > MAX_SCALE) {
// |     const r = rescaleBuf(product, scale);
// |     if (r === null) return OVERFLOW;
// |     product = r.value;
// |     scale = r.scale;
// |   }
// |   // `finish_mul` keeps the sign flag even when the rescaled product is zero.
// |   return new RustDecimal(product, scale, negative);
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `POWER_OVERFLOW_VALUES` (ops/common.rs) as `[hi, low64]` pairs. */
// | const POWER_OVERFLOW_VALUES = [
// |   [2576980377, 2576980377, 429496729],
// |   [687194767, 4123168604, 42949672],
// |   [2645699854, 1271310319, 4294967],
// |   [694066715, 3133608139, 429496],
// |   [2216890319, 2890341191, 42949],
// |   [2369172679, 4154504685, 4294],
// |   [4102387834, 2133437386, 429],
// |   [410238783, 4078814305, 42],
// | ].map(([lo, mid, hi]) => [hi, (BigInt(mid) << 32n) | BigInt(lo)]);

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf12::find_scale` (ops/common.rs): how many powers of ten the quotient
// |  * can take without overflowing; `null` when that leaves a negative scale.
// |  * @param {bigint} q @param {number} scale
// |  * @returns {number | null}
// |  */
// | function findScale(q, scale) {
// |   const hi = Number(q >> 64n);
// |   const low64 = q & U64_MASK;
// |   if (hi > 429496729) return scale < 0 ? null : 0;
// |   let x;
// |   if (scale > MAX_SCALE - 9) {
// |     x = MAX_SCALE - scale;
// |     if (hi < POWER_OVERFLOW_VALUES[x - 1][0]) return x + scale < 0 ? null : x;
// |   } else if (hi < 4 || (hi === 4 && low64 <= 5441186219426131129n)) {
// |     return 9;
// |   }
// |   if (hi > 42949) {
// |     if (hi > 4294967) x = hi > 42949672 ? 1 : 2;
// |     else x = hi > 429496 ? 3 : 4;
// |   } else if (hi > 429) {
// |     x = hi > 4294 ? 5 : 6;
// |   } else {
// |     x = hi > 42 ? 7 : 8;
// |   }
// |   const [maxHi, maxLow64] = POWER_OVERFLOW_VALUES[x - 1];
// |   if (hi === maxHi && low64 > maxLow64) x -= 1;
// |   return x + scale < 0 ? null : x;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `unscale_from_overflow` (ops/div.rs): the quotient overflowed 96 bits, so
// |  * drop one digit, rounding half to even with a sticky bit.
// |  * @param {bigint} value @param {number} scale @param {boolean} sticky
// |  * @returns {[bigint, number] | null}
// |  */
// | function unscaleFromOverflow(value, scale, sticky) {
// |   if (scale < 1) return null;
// |   const q = value / 10n;
// |   const remainder = value % 10n;
// |   return [remainder > 5n || (remainder === 5n && (sticky || (q & 1n) === 1n)) ? q + 1n : q, scale - 1];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `div_impl` with the `div_by_*` remainder loop and `unscale` (ops/div.rs).
// |  * Only the trailing zeros created by the loop are removed, so `10 / 4` is
// |  * `2.50`.
// |  * @param {RustDecimal} d1 @param {RustDecimal} d2
// |  * @returns {RustDecimal | string}
// |  */
// | function div(d1, d2) {
// |   if (d2.magnitude === 0n) return DIV_BY_ZERO;
// |   if (d1.magnitude === 0n) return RustDecimal.ZERO;
// |   const divisor = d2.magnitude;
// |   let scale = d1.places - d2.places;
// |   const negative = d1.negative !== d2.negative;
// |   let q = d1.magnitude / divisor;
// |   let r = d1.magnitude % divisor;
// |   let requireUnscale = false;
// |   while (r !== 0n || scale < 0) {
// |     let powerScale;
// |     if (r === 0n) {
// |       powerScale = Math.min(9, -scale);
// |     } else {
// |       requireUnscale = true;
// |       let willOverflow = true;
// |       if (scale !== MAX_SCALE) {
// |         const found = findScale(q, scale);
// |         if (found === null) return OVERFLOW;
// |         powerScale = found;
// |         willOverflow = found === 0;
// |       }
// |       if (willOverflow) {
// |         // Round the last digit half to even on `2 * remainder` vs the divisor.
// |         const twice = r << 1n;
// |         if (twice > divisor || (twice === divisor && (q & 1n) === 1n)) {
// |           q += 1n;
// |           if (q === U96) {
// |             const u = unscaleFromOverflow(q, scale, true);
// |             if (u === null) return OVERFLOW;
// |             [q, scale] = u;
// |           }
// |         }
// |         break;
// |       }
// |     }
// |     const power = POW10[powerScale];
// |     scale += powerScale;
// |     q *= power;
// |     if (q >= U96) return OVERFLOW;
// |     const scaled = r * power;
// |     q += scaled / divisor;
// |     r = scaled % divisor;
// |     if (q >= U96) {
// |       const u = unscaleFromOverflow(q, scale, r !== 0n);
// |       if (u === null) return OVERFLOW;
// |       [q, scale] = u;
// |       break;
// |     }
// |   }
// |   if (requireUnscale) {
// |     // `unscale`: 10^8 is only tried while the low word is zero; the other
// |     // low-bit pre-checks are implied by the divisibility.
// |     while ((q & M32) === 0n && scale >= 8 && q % POW10[8] === 0n) [q, scale] = [q / POW10[8], scale - 8];
// |     for (const n of [4, 2, 1]) if (scale >= n && q % POW10[n] === 0n) [q, scale] = [q / POW10[n], scale - n];
// |   }
// |   return fromParts(q, negative, scale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `rem_impl` (ops/rem.rs): the dividend's sign, and the larger of the two
// |  * scales. A dividend with the smaller scale is scaled up in 96 bits (as far
// |  * as `find_scale` allows) or in `rem_full`'s wider buffer; the limb
// |  * arithmetic, wrapping included, is reproduced.
// |  * @param {RustDecimal} d1 @param {RustDecimal} d2
// |  * @returns {RustDecimal | string}
// |  */
// | function rem(d1, d2) {
// |   if (d2.magnitude === 0n) return DIV_BY_ZERO;
// |   if (d1.magnitude === 0n) return RustDecimal.ZERO;
// |   // `cmp_internal` ignores the signs and is exact.
// |   const top = Math.max(d1.places, d2.places);
// |   const a = d1.magnitude * POW10[top - d1.places];
// |   const b = d2.magnitude * POW10[top - d2.places];
// |   if (a === b) return RustDecimal.ZERO;
// |   if (a < b) return d1;
// |   let x = d1.magnitude;
// |   let xScale = d1.places;
// |   let y = d2.magnitude;
// |   let scale = d1.places - d2.places;
// |   if (scale > 0) {
// |     // The divisor is scaled up with a wrapping u64 multiply (mod 2^96).
// |     y = (y * POW10[scale]) & U96_MASK;
// |     scale = 0;
// |   }
// |   for (;;) {
// |     if (scale < 0) {
// |       for (;;) {
// |         const powerScale = findScale(x, MAX_SCALE + scale);
// |         if (powerScale === null) return OVERFLOW;
// |         if (powerScale === 0) break;
// |         scale += powerScale;
// |         x = (x * POW10[powerScale]) & U96_MASK;
// |         if (powerScale !== 9 || scale >= 0) break;
// |       }
// |       xScale = d2.places;
// |     }
// |     if (x <= U64_MASK) {
// |       const y64 = y & U64_MASK;
// |       if (y64 === 0n) return panic('attempt to calculate the remainder with a divisor of zero');
// |       return fromParts(x % y64, d1.negative, xScale);
// |     }
// |     if (y >= U32) return remFull(x, xScale, d1.negative, y, scale);
// |     x = ((((x >> 32n) % y) << 32n) | (x & M32)) % y;
// |     if (scale >= 0) return fromParts(x, d1.negative, xScale);
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `rem_full` (ops/rem.rs), for a divisor `y` wider than 32 bits: the dividend
// |  * `x` is shifted by the divisor's leading zeros, scaled by `10^-scale` in a
// |  * 192-bit buffer plus an overflow word, then reduced by `partial_divide_64`
// |  * or `partial_divide_96` steps. The result has `xScale` and `negative`.
// |  * @param {bigint} x @param {number} xScale @param {boolean} negative @param {bigint} y @param {number} scale
// |  */
// | function remFull(x, xScale, negative, y, scale) {
// |   const yHi = y >> 64n;
// |   const shift = BigInt(Math.clz32(Number(yHi === 0n ? y >> 32n : yHi)));
// |   const low64 = (x << shift) & U64_MASK;
// |   const mid64 = x >> 32n >> (32n - shift);
// |   // The `Buf24` words, then the `overflow` word.
// |   const buf = [low64 & M32, low64 >> 32n, mid64 & M32, mid64 >> 32n, 0n, 0n, 0n];
// |   let upper = 3;
// |   while (scale < 0) {
// |     const power = POW10[Math.min(-scale, 9)];
// |     const last = upper === 6 ? 6 : Math.min(upper, 5);
// |     let t = 0n;
// |     for (let i = 0; i <= last; i++) buf[i] = (t = (t >> 32n) + buf[i] * power) & M32;
// |     // Keeps the top word's high bit clear.
// |     if (t > 0x7fffffffn) {
// |       upper += 1;
// |       buf[Math.min(upper, 6)] = t >> 32n;
// |     }
// |     scale += 9;
// |   }
// |   const divisor = (y << shift) & U64_MASK;
// |   const start = upper >= 4 && upper <= 6 ? upper - 2 : 1;
// |   if (yHi === 0n) {
// |     for (let at = start; at >= 0; at--) partialDivide64(buf, at, divisor);
// |     return fromParts((buf[0] | (buf[1] << 32n)) >> shift, negative, xScale);
// |   }
// |   const words = [divisor & M32, divisor >> 32n, (y >> 32n >> (32n - shift)) & M32];
// |   for (let at = start - 1; at >= 0; at--) partialDivide96(buf, at, words);
// |   const low = ((buf[0] | (buf[1] << 32n)) >> shift) + ((buf[2] << (32n - shift) << 32n) & U64_MASK);
// |   return fromParts(low + ((buf[2] >> shift) << 64n), negative, xScale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf16::partial_divide_64` (ops/div.rs) on the words `b[at..at + 3]`: the
// |  * remainder by a normalized 64-bit divisor replaces the low two words (the
// |  * quotient is not needed).
// |  * @param {bigint[]} b @param {number} at @param {bigint} divisor
// |  */
// | function partialDivide64(b, at, divisor) {
// |   const low64 = b[at] | (b[at + 1] << 32n);
// |   const divisorHi = divisor >> 32n;
// |   let r;
// |   if (b[at + 2] === 0n) {
// |     if (low64 < divisor) return;
// |     r = low64 % divisor;
// |   } else if (b[at + 2] >= divisorHi) {
// |     r = (low64 - ((divisor << 32n) & U64_MASK) + divisor) & U64_MASK;
// |     while (r >= divisor) r = (r + divisor) & U64_MASK;
// |   } else {
// |     const mid64 = b[at + 1] | (b[at + 2] << 32n);
// |     if (mid64 < divisorHi) return;
// |     const q = mid64 / divisorHi;
// |     const product = q * (divisor & M32);
// |     r = ((b[at] | ((mid64 - q * divisorHi) << 32n)) - product) & U64_MASK;
// |     if (r > U64_MASK - product) do r = (r + divisor) & U64_MASK; while (r >= divisor);
// |   }
// |   b[at] = r & M32;
// |   b[at + 1] = r >> 32n;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf16::partial_divide_96` (ops/div.rs) on the words `b[at..at + 4]`: the
// |  * remainder by a normalized 96-bit divisor (`words`) replaces the low three
// |  * words. The estimated quotient digit is truncated to 32 bits, as in Rust.
// |  * @param {bigint[]} b @param {number} at @param {bigint[]} words
// |  */
// | function partialDivide96(b, at, words) {
// |   const dividend = b[at + 2] | (b[at + 3] << 32n);
// |   const divisorHi = words[2];
// |   if (dividend < divisorHi) return;
// |   const quo = (dividend / divisorHi) & M32;
// |   let r = ((dividend & M32) - quo * divisorHi) & M32;
// |   let prod1 = quo * words[0];
// |   let prod2 = quo * words[1] + (prod1 >> 32n);
// |   prod1 = (prod1 & M32) | ((prod2 << 32n) & U64_MASK);
// |   prod2 >>= 32n;
// |   let num = ((b[at] | (b[at + 1] << 32n)) - prod1) & U64_MASK;
// |   r = (r - prod2) & M32;
// |   let settled;
// |   if (num > U64_MASK - prod1) {
// |     r = (r - 1n) & M32;
// |     settled = r < M32 - prod2;
// |   } else {
// |     settled = r <= M32 - prod2;
// |   }
// |   // The remainder went negative: add the divisor back until it carries.
// |   const divisorLow = words[0] | (words[1] << 32n);
// |   while (!settled) {
// |     num = (num + divisorLow) & U64_MASK;
// |     r = (r + divisorHi) & M32;
// |     if (num < divisorLow) {
// |       const before = r;
// |       r = (r + 1n) & M32;
// |       if (before < divisorHi) break;
// |     }
// |     settled = r < divisorHi;
// |   }
// |   b[at] = num & M32;
// |   b[at + 1] = num >> 32n;
// |   b[at + 2] = r;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Decimal::rescale` (ops/array.rs `rescale`): rounds half away from zero
// |  * when dropping digits; keeps the sign flag.
// |  * @param {RustDecimal} d @param {number} newScale
// |  */
// | function rescale(d, newScale) {
// |   if (d.places === newScale) return d;
// |   let m = d.magnitude;
// |   if (m === 0n) return new RustDecimal(0n, Math.min(newScale, MAX_SCALE), d.negative);
// |   if (d.places > newScale) {
// |     m /= POW10[d.places - newScale - 1];
// |     if (m !== 0n) {
// |       const digit = m % 10n;
// |       m /= 10n;
// |       if (digit >= 5n) m += 1n;
// |     }
// |     return new RustDecimal(m, newScale, d.negative);
// |   }
// |   let remaining = newScale - d.places;
// |   for (; remaining > 0 && m * 10n < U96; remaining--) m *= 10n;
// |   return new RustDecimal(m, newScale - remaining, d.negative);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** `Decimal::set_scale`: keeps the mantissa and the sign flag. @param {RustDecimal} d @param {number} scale */
// | function setScale(d, scale) {
// |   if (scale > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(scale));
// |   return new RustDecimal(d.magnitude, scale, d.negative);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `str.parse::<u32>()`: an optional '+', then at least one ASCII digit.
// |  * @param {string} s
// |  * @returns {number | null}
// |  */
// | function parseU32(s) {
// |   const digits = s.startsWith('+') ? s.slice(1) : s;
// |   if (!/^[0-9]+$/.test(digits)) return null;
// |   const value = Number(digits);
// |   return value <= 4294967295 ? value : null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `from_scientific` and `from_scientific_lossy` (decimal.rs).
// |  * @param {string} value @param {boolean} lossy
// |  * @returns {RustDecimal}
// |  */
// | function fromScientific(value, lossy) {
// |   const split = value.search(/[eE]/);
// |   if (split < 0) return err(DecimalError.ErrorString('Failed to parse'));
// |   let ret = RustDecimal.fromStr(value.slice(0, split));
// |   const exp = value.slice(split + 1);
// |   const currentScale = ret.places;
// |   const negativeExp = exp.startsWith('-');
// |   const e = parseU32(negativeExp ? exp.slice(1) : exp);
// |   if (e === null) return err(DecimalError.ErrorString('Failed to parse'));
// |   if (negativeExp) {
// |     if (e > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(e));
// |     if (lossy && currentScale + e > MAX_SCALE) {
// |       return setScale(rescale(ret, MAX_SCALE - e), MAX_SCALE);
// |     }
// |     return setScale(ret, currentScale + e);
// |   }
// |   if (e <= currentScale) return setScale(ret, currentScale - e);
// |   if (e > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(e));
// |   let rest = e;
// |   while (rest > 0) {
// |     const step = Math.min(rest, 19);
// |     rest -= step;
// |     const r = mul(ret, new RustDecimal(POW10[step], 0, false));
// |     if (!(r instanceof RustDecimal)) err(DecimalError.ExceedsMaximumPossibleValue);
// |     ret = /** @type {RustDecimal} */ (r);
// |   }
// |   return ret.normalize();
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const UTF8 = new TextEncoder();
// meta-language:translated JavaScript lexical_declaration items=1 sha256=17c7203f94adbe768523c7b70daa2208b8a37cefcfff925559491affdafaadf1
// | const DOT = 0x2e;
pub const DOT: f64 = 46f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=ed9d163509979d0e7a2227e9ee194cff2190e926b67524a95950c8503206994c
// | const UNDERSCORE = 0x5f;
pub const UNDERSCORE: f64 = 95f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b04f022f08eb64d49266a1b875558663310e18ac7479d61ce48290f0e3b9de4d
// | const MINUS = 0x2d;
pub const MINUS: f64 = 45f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=e6647f10d3bdf58683a17572e441f3d09483c1c37f93aff88d8a49f9900db30f
// | const PLUS = 0x2b;
pub const PLUS: f64 = 43f64;

// meta-language:translated JavaScript function_declaration items=1 sha256=eb0fb3dd538ac7673e35527149bf17d0675176f7630ad85c02d4bd3dd1496f32
// | /** @param {number} b */
// | function isDigit(b) {
// |   return b >= 0x30 && b <= 0x39;
// | }
pub fn is_digit(b: f64) -> bool {
    ((b >= (48f64)) && (b <= (57f64)))
}

// meta-language:carried JavaScript function_declaration (type)
// | /** @param {string} message */
// | function parseError(message) {
// |   return DecimalError.ErrorString(`Invalid decimal: ${message}`);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** `tail_invalid_digit` (str.rs). @param {number} b */
// | function tailInvalidDigit(b) {
// |   if (b === DOT) return parseError('two decimal points');
// |   if (b === UNDERSCORE) return parseError('must start lead with a number');
// |   return parseError('unknown character');
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * `handle_data` (str.rs).
// |  * @param {bigint} data @param {number} scale @param {boolean} negative @param {boolean} hasDigits
// |  */
// | function handleData(data, scale, negative, hasDigits) {
// |   if (!hasDigits) return parseError('no digits found');
// |   return fromParts(data, negative, scale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `maybe_round` (str.rs): rounds half up on `bytes[at]`, then checks that
// |  * the rest of the text is only digits, underscores and at most one point.
// |  * @param {bigint} data @param {Uint8Array} bytes @param {number} at
// |  * @param {number} scale @param {boolean} point @param {boolean} negative
// |  */
// | function maybeRound(data, bytes, at, scale, point, negative) {
// |   const next = bytes[at];
// |   let digit;
// |   if (isDigit(next)) digit = next - 0x30;
// |   else if (next === UNDERSCORE || (next === DOT && !point)) digit = 0;
// |   else return tailInvalidDigit(next);
// |   if (digit >= 5) {
// |     data += 1n;
// |     if (data >= U96) {
// |       if (scale === 0) return parseError('overflow from mantissa after rounding');
// |       data = (data + 4n) / 10n;
// |       scale -= 1;
// |     }
// |   }
// |   let seenDot = point || next === DOT;
// |   for (let i = at + 1; i < bytes.length; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b) || b === UNDERSCORE) continue;
// |     if (b === DOT && !seenDot) seenDot = true;
// |     else if (b === DOT) return parseError('two decimal points');
// |     else return tailInvalidDigit(b);
// |   }
// |   return handleData(data, scale, negative, true);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `handle_full_128` (str.rs): the digits past `WILL_OVERFLOW_U64`, from
// |  * `bytes[at]` on, accumulated up to 96 bits.
// |  * @param {bigint} data @param {Uint8Array} bytes @param {number} at
// |  * @param {number} scale @param {boolean} point @param {boolean} negative
// |  */
// | function handleFull128(data, bytes, at, scale, point, negative) {
// |   const n = bytes.length;
// |   for (let i = at; ; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b)) {
// |       const next = data * 10n + BigInt(b - 0x30);
// |       if (next >= U96) {
// |         if (!point) return parseError('overflow from too many digits');
// |         return maybeRound(data, bytes, i, scale, point, negative);
// |       }
// |       data = next;
// |       if (point) scale += 1;
// |       if (i + 1 >= n) return handleData(data, scale, negative, true);
// |       if (point && scale >= MAX_SCALE) {
// |         let j = i + 1;
// |         if (bytes[j] === UNDERSCORE) {
// |           while (j < n && bytes[j] === UNDERSCORE) j++;
// |           if (j >= n) return handleData(data, scale, negative, true);
// |         }
// |         return maybeRound(data, bytes, j, scale, point, negative);
// |       }
// |       continue;
// |     }
// |     if (b === DOT && !point) point = true;
// |     else if (b !== UNDERSCORE) return tailInvalidDigit(b);
// |     if (i + 1 >= n) return handleData(data, scale, negative, true);
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `parse_str_radix_10` (str.rs) without the scientific fallback: the
// |  * `dispatch_next` state machine on 64 bits, then `handle_full_128` once
// |  * the value may overflow a `u64` (only for texts of 18 bytes or more).
// |  * @param {string} s
// |  * @returns {RustDecimal | DecimalError}
// |  */
// | function parseStrRadix10(s) {
// |   const bytes = UTF8.encode(s);
// |   const n = bytes.length;
// |   if (n === 0) return parseError('empty');
// |   const big = n >= BYTES_TO_OVERFLOW_U64;
// |   let point = false;
// |   let negative = false;
// |   let hasDigits = false;
// |   let data = 0n;
// |   let scale = 0;
// |   for (let i = 0; ; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b)) {
// |       data = data * 10n + BigInt(b - 0x30);
// |       scale = point ? scale + 1 : 0;
// |       hasDigits = true;
// |       if (i + 1 >= n) return handleData(data, scale, negative, true);
// |       if (point && big && scale >= MAX_SCALE) return maybeRound(data, bytes, i + 1, scale, point, negative);
// |       if (big && data >= WILL_OVERFLOW_U64) return handleFull128(data, bytes, i + 1, scale, point, negative);
// |       continue;
// |     }
// |     if (b === DOT && !point) point = true;
// |     else if ((b === MINUS || b === PLUS) && i === 0) negative = b === MINUS;
// |     else if (!(b === UNDERSCORE && hasDigits)) return tailInvalidDigit(b);
// |     if (i + 1 >= n) return handleData(data, scale, negative, hasDigits);
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const F64_VIEW = new DataView(new ArrayBuffer(8));

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `from_f64` with `remove_excess_bits = true` and `base2_to_decimal`
// |  * (decimal.rs): the exact binary value is converted to a 96-bit mantissa
// |  * times a power of ten, then trimmed to at most 28 places and to the 53
// |  * significant bits of the float.
// |  * @param {number} n
// |  * @returns {RustDecimal | null}
// |  */
// | function fromF64(n) {
// |   if (!Number.isFinite(n)) return null;
// |   F64_VIEW.setFloat64(0, n);
// |   const raw = F64_VIEW.getBigUint64(0);
// |   const positive = raw >> 63n === 0n;
// |   const biasedExponent = Number((raw >> 52n) & 0x7ffn);
// |   let bits = raw & 0xfffffffffffffn;
// |   if (biasedExponent === 0 && bits === 0n) return positive ? RustDecimal.ZERO : new RustDecimal(0n, 0, true);
// |   let exponent2 = biasedExponent - 1023;
// |   if (biasedExponent === 0) exponent2 += 1;
// |   else bits |= 1n << 52n;
// |   exponent2 -= 52;
// |
// |   let exponent5 = -exponent2;
// |   let exponent10 = exponent2;
// |   // An odd mantissa is multiplied by 5 unless that overflows; otherwise halve.
// |   for (; exponent5 > 0; exponent5--) {
// |     if ((bits & 1n) === 0n || bits * 5n >= U96) {
// |       exponent10 += 1;
// |       bits >>= 1n;
// |     } else {
// |       bits *= 5n;
// |     }
// |   }
// |   for (; exponent5 < 0; exponent5++) {
// |     if (bits < 1n << 95n) {
// |       exponent10 -= 1;
// |       bits <<= 1n;
// |     } else if (exponent10 * 2 > -exponent5) {
// |       return null;
// |     } else {
// |       bits /= 5n;
// |     }
// |   }
// |   for (; exponent10 > 0; exponent10--) if ((bits *= 10n) >= U96) return null;
// |   // Drop digits, rounding half up, to at most 28 places (a zero mantissa
// |   // resets the exponent), then to the float's 53 bits.
// |   while (exponent10 < -MAX_SCALE || (exponent10 < 0 && bits >= 1n << 52n)) {
// |     const digit = bits % 10n;
// |     bits /= 10n;
// |     exponent10 += 1;
// |     if (bits === 0n) exponent10 = 0;
// |     else if (digit >= 5n) bits += 1n;
// |   }
// |   for (; exponent10 < 0 && bits % 10n === 0n; exponent10++) bits /= 10n;
// |   return new RustDecimal(bits, -exponent10, !positive);
// | }
