// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c07156800042f9c32d8b331e305e19733ca9ea708f77ecb76331ade4ef0c3727 bytes=22587

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

/// JavaScript's Number operations that Rust's f64 does not share.
pub mod ml_number {
    /// ECMAScript Number::toString: the shortest decimal that reads back as
    /// the value, the even one on a tie, laid out as JavaScript prints it.
    pub fn js_number(x: f64) -> String {
        if x.is_nan() {
            return String::from("NaN");
        }
        if x.is_infinite() {
            return String::from(if x > 0.0 { "Infinity" } else { "-Infinity" });
        }
        if x == 0.0 {
            return String::from("0");
        }
        let (mut digits, n) = decimal(&format!("{:e}", x.abs()));
        // Rust's shortest digits round a tie away from zero; ECMAScript takes the even neighbour.
        let (exact, _) = decimal(&format!("{:.800e}", x.abs()));
        let rest = &exact[digits.len().min(exact.len())..];
        if digits.len() > 1 && rest.starts_with('5') && rest[1..].bytes().all(|b| b == b'0') {
            let last = digits.as_bytes()[digits.len() - 1] - b'0';
            let down = exact[..digits.len()].to_string();
            let other = if digits == down { increment(&down) } else { Some(down) };
            if let Some(other) = other {
                let back = format!("{}.{}e{}", &other[..1], &other[1..], n - 1);
                if last % 2 == 1 && back.parse::<f64>() == Ok(x.abs()) {
                    digits = other.trim_end_matches('0').to_string();
                }
            }
        }
        let k = digits.len() as i32;
        let body = if k <= n && n <= 21 {
            format!("{digits}{}", "0".repeat((n - k) as usize))
        } else if 0 < n && n <= 21 {
            format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
        } else if -6 < n && n <= 0 {
            format!("0.{}{digits}", "0".repeat((-n) as usize))
        } else {
            let sign = if n - 1 < 0 { '-' } else { '+' };
            let power = (n - 1).abs();
            if k == 1 {
                format!("{digits}e{sign}{power}")
            } else {
                format!("{}.{}e{sign}{power}", &digits[..1], &digits[1..])
            }
        };
        if x < 0.0 {
            format!("-{body}")
        } else {
            body
        }
    }

    /// What console.log prints: -0 as "-0", where String(-0) is "0".
    pub fn js_console(x: f64) -> String {
        if x == 0.0 && x.is_sign_negative() {
            String::from("-0")
        } else {
            js_number(x)
        }
    }

    /// SameValue, as Object.is and assert.strictEqual compare: NaN equals
    /// NaN, and 0 differs from -0.
    pub fn same_value(a: f64, b: f64) -> bool {
        if a.is_nan() || b.is_nan() {
            a.is_nan() && b.is_nan()
        } else {
            a.to_bits() == b.to_bits()
        }
    }

    /// A decimal digit string plus one, or None when it gains a digit.
    fn increment(digits: &str) -> Option<String> {
        let mut bytes = digits.as_bytes().to_vec();
        for index in (0..bytes.len()).rev() {
            if bytes[index] == b'9' {
                bytes[index] = b'0';
            } else {
                bytes[index] += 1;
                return String::from_utf8(bytes).ok();
            }
        }
        None
    }

    /// The significant digits without trailing zeros, and the exponent n with
    /// value 0.digits × 10^n, of Rust's {:e} text.
    fn decimal(text: &str) -> (String, i32) {
        let (mantissa, exponent) = text.split_once('e').expect("exponent");
        let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
        let digits = digits.trim_end_matches('0').to_string();
        let digits = if digits.is_empty() { String::from("0") } else { digits };
        (digits, exponent.parse::<i32>().expect("exponent") + 1)
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | //! Polynomial-equation helpers for the expression evaluator.

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=e0430524213e7e0aba35ba83d0c013edef8a31223066a098c64d5c30727092b5
// | import { clone, eq, err } from '../rust/core.js';
use crate::rust::core_::{clone, eq, err};
// meta-language:translated JavaScript import_statement items=1 sha256=0216d025dda9d77919588f0e98e28937e00896ada0f84e0eb7d6ad96c91ecf16
// | import { I128, U128, U32 } from '../rust/num.js';
use crate::rust::num::{I128, U128, U32};
// meta-language:translated JavaScript import_statement items=1 sha256=6a58e9adb3b15ffddb375f0e6d3ccd15d27e918ab78e0d0474c5b280ad9f79ac
// | import { eqIgnoreAsciiCase } from '../rust/str.js';
use crate::rust::str_::eqIgnoreAsciiCase;
// meta-language:translated JavaScript import_statement items=1 sha256=da5833820f84db3f2f11ceaf4f23afda2567135bae29426b6f9d440067c23195
// | import { Rational, Unit, Value } from '../types/index.js';
use crate::types::index::{Rational, Unit, Value};

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../types/index.js').Expression} Expression */

// meta-language:translated JavaScript lexical_declaration items=1 sha256=937498ef900841a6dcaa5a4ea11f116dd6a8382181f29fe7a753cd01cea43218
// | const MAX_POLYNOMIAL_DEGREE = 32;
pub const MAX_POLYNOMIAL_DEGREE: f64 = 32f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5dcc23ef90ea4c93948a4617685014fe1472333a7c94f7f1ed8292653ab700c
// | const ROOT_SEARCH_BOUND = 100n;
pub static ROOT_SEARCH_BOUND: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(100));
// meta-language:translated JavaScript lexical_declaration items=1 sha256=446f896352efc8d7b65f1d035afe7fd5992d820a062f88c09f6283fdea1ef2cc
// | const MAX_FACTORABLE_COEFFICIENT_ABS = 1_000_000_000n;
pub static MAX_FACTORABLE_COEFFICIENT_ABS: std::sync::LazyLock<crate::ml::Big> = std::sync::LazyLock::new(|| crate::ml::Big::from_i128(1000000000));

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `BTreeMap<u32, Rational>` iteration: the entries sorted by degree.
// |  * @param {Map<number, Rational>} terms
// |  * @returns {[number, Rational][]}
// |  */
// | function sortedTerms(terms) {
// |   return [...terms].sort(([a], [b]) => a - b);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `BTreeSet<Rational>`: a `Map` keyed by the reduced fraction.
// |  * @param {Map<string, Rational>} set
// |  * @param {Rational} value
// |  */
// | function insertRational(set, value) {
// |   set.set(`${value.numerBigint()}/${value.denomBigint()}`, value);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `BTreeSet<Rational>` iteration: the values in ascending order.
// |  * @param {Map<string, Rational>} set
// |  * @returns {Rational[]}
// |  */
// | function sortedRationals(set) {
// |   return [...set.values()].sort((a, b) => a.cmp(b));
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | class PolynomialForm {
// |   /** @param {{ variable: string | null, terms: Map<number, Rational> }} fields */
// |   constructor({ variable, terms }) {
// |     this.variable = variable;
// |     this.terms = terms;
// |   }
// |
// |   /** @param {Rational} value */
// |   static constant(value) {
// |     /** @type {Map<number, Rational>} */
// |     const terms = new Map();
// |     if (!value.isZero()) {
// |       terms.set(0, value);
// |     }
// |     return new PolynomialForm({
// |       variable: null,
// |       terms,
// |     });
// |   }
// |
// |   /** @param {string} name */
// |   static variable(name) {
// |     /** @type {Map<number, Rational>} */
// |     const terms = new Map();
// |     terms.set(1, Rational.one());
// |     return new PolynomialForm({
// |       variable: name,
// |       terms,
// |     });
// |   }
// |
// |   /**
// |    * @param {Expression} expr
// |    * @returns {PolynomialForm}
// |    */
// |   static fromExpression(expr) {
// |     switch (expr.tag) {
// |       case 'Number': {
// |         if (!eq(expr.unit, Unit.None)) {
// |           return err(PolynomialForm.unsupportedEquation());
// |         }
// |         return PolynomialForm.constant(Rational.fromDecimal(expr.value));
// |       }
// |       case 'Variable':
// |         return PolynomialForm.variable(expr._0);
// |       case 'Binary': {
// |         const op = expr.op;
// |         const relative = !expr.left.isPercentage() && expr.right.isPercentage();
// |         const left = PolynomialForm.fromExpression(expr.left);
// |         let right = PolynomialForm.fromExpression(expr.right);
// |         right =
// |           relative && (op.tag === 'Add' || op.tag === 'Subtract')
// |             ? clone(left).multiply(right)
// |             : right;
// |         switch (op.tag) {
// |           case 'Add':
// |             return left.add(right);
// |           case 'Subtract':
// |             return left.subtract(right);
// |           case 'Multiply':
// |             return left.multiply(right);
// |           case 'Divide':
// |             return left.divide(right);
// |           default:
// |             // BinaryOp::Modulo
// |             return err(PolynomialForm.unsupportedEquation());
// |         }
// |       }
// |       case 'FunctionCall': {
// |         const { name, args } = expr;
// |         if (eqIgnoreAsciiCase(name, 'percent') && args.length === 1) {
// |           return PolynomialForm.fromExpression(args[0]).divide(
// |             PolynomialForm.constant(Rational.fromInteger(100)),
// |           );
// |         }
// |         if (
// |           (eqIgnoreAsciiCase(name, 'as_percent') || eqIgnoreAsciiCase(name, 'percent_of')) &&
// |           args.length === 1
// |         ) {
// |           return PolynomialForm.fromExpression(args[0]);
// |         }
// |         return err(PolynomialForm.unsupportedEquation());
// |       }
// |       case 'Negate':
// |         return PolynomialForm.fromExpression(expr._0).negate();
// |       case 'Group':
// |         return PolynomialForm.fromExpression(expr._0);
// |       case 'Power': {
// |         const base = PolynomialForm.fromExpression(expr.base);
// |         const exponent = PolynomialForm.fromExpression(
// |           expr.exponent,
// |         ).asNonNegativeIntegerExponent();
// |         return base.pow(exponent);
// |       }
// |       default:
// |         // DateTime, Now, Today, Until, AtTime, IndefiniteIntegral,
// |         // UnitConversion, Equality, Comparison
// |         return err(PolynomialForm.unsupportedEquation());
// |     }
// |   }
// |
// |   /** @param {PolynomialForm} other @returns {PolynomialForm} */
// |   add(other) {
// |     const variable = mergeVariableNames(this.variable, other.variable);
// |     const result = new PolynomialForm({
// |       variable,
// |       terms: this.terms,
// |     });
// |
// |     for (const [degree, coefficient] of sortedTerms(other.terms)) {
// |       result.addTerm(degree, coefficient);
// |     }
// |
// |     return result;
// |   }
// |
// |   /** @param {PolynomialForm} other @returns {PolynomialForm} */
// |   subtract(other) {
// |     return this.add(other.negate());
// |   }
// |
// |   /** @param {PolynomialForm} other @returns {PolynomialForm} */
// |   multiply(other) {
// |     const variable = mergeVariableNames(this.variable, other.variable);
// |     const result = new PolynomialForm({
// |       variable,
// |       terms: new Map(),
// |     });
// |
// |     for (const [leftDegree, leftCoefficient] of sortedTerms(this.terms)) {
// |       for (const [rightDegree, rightCoefficient] of sortedTerms(other.terms)) {
// |         const degree = U32.checkedAdd(BigInt(leftDegree), BigInt(rightDegree));
// |         if (degree === null || degree > BigInt(MAX_POLYNOMIAL_DEGREE)) {
// |           return err(PolynomialForm.unsupportedEquation());
// |         }
// |         result.addTerm(Number(degree), leftCoefficient.mul(rightCoefficient));
// |       }
// |     }
// |
// |     return result;
// |   }
// |
// |   /** @param {PolynomialForm} other @returns {PolynomialForm} */
// |   divide(other) {
// |     if (other.hasVariableTerms()) {
// |       return err(
// |         CalculatorError.InvalidOperation('polynomial equation has a variable denominator'),
// |       );
// |     }
// |
// |     const denominator = other.coefficient(0);
// |     if (denominator.isZero()) {
// |       return err(CalculatorError.DivisionByZero);
// |     }
// |
// |     return this.divideByRational(denominator);
// |   }
// |
// |   /** @param {number} exponent @returns {PolynomialForm} */
// |   pow(exponent) {
// |     if (exponent === 0) {
// |       return PolynomialForm.constant(Rational.one());
// |     }
// |
// |     let result = PolynomialForm.constant(Rational.one());
// |     for (let i = 0; i < exponent; i += 1) {
// |       result = result.multiply(clone(this));
// |     }
// |
// |     return result;
// |   }
// |
// |   /** @returns {PolynomialForm} */
// |   negate() {
// |     return this.scale(Rational.fromInteger(-1));
// |   }
// |
// |   /** @param {Rational} scale @returns {PolynomialForm} */
// |   scale(scale) {
// |     for (const [degree, coefficient] of this.terms) {
// |       this.terms.set(degree, coefficient.mul(scale));
// |     }
// |     this.removeZeroTerms();
// |     return this;
// |   }
// |
// |   /** @param {Rational} denominator @returns {PolynomialForm} */
// |   divideByRational(denominator) {
// |     if (denominator.isZero()) {
// |       return err(CalculatorError.DivisionByZero);
// |     }
// |
// |     for (const [degree, coefficient] of this.terms) {
// |       this.terms.set(degree, coefficient.div(denominator));
// |     }
// |     this.removeZeroTerms();
// |     return this;
// |   }
// |
// |   /** @param {number} degree @param {Rational} coefficient */
// |   addTerm(degree, coefficient) {
// |     if (coefficient.isZero()) {
// |       return;
// |     }
// |
// |     const newCoefficient = this.coefficient(degree).add(coefficient);
// |     if (newCoefficient.isZero()) {
// |       this.terms.delete(degree);
// |     } else {
// |       this.terms.set(degree, newCoefficient);
// |     }
// |   }
// |
// |   removeZeroTerms() {
// |     for (const [degree, coefficient] of [...this.terms]) {
// |       if (coefficient.isZero()) {
// |         this.terms.delete(degree);
// |       }
// |     }
// |     if (!this.hasVariableTerms()) {
// |       this.variable = null;
// |     }
// |   }
// |
// |   hasVariableTerms() {
// |     return [...this.terms.keys()].some((degree) => degree > 0);
// |   }
// |
// |   /** @param {number} degree @returns {Rational} */
// |   coefficient(degree) {
// |     return this.terms.get(degree) ?? Rational.zero();
// |   }
// |
// |   /** @returns {number} */
// |   degree() {
// |     return this.terms.size === 0 ? 0 : Math.max(...this.terms.keys());
// |   }
// |
// |   /** @returns {number} */
// |   asNonNegativeIntegerExponent() {
// |     if (this.hasVariableTerms()) {
// |       return err(PolynomialForm.unsupportedEquation());
// |     }
// |
// |     const exponent = this.coefficient(0);
// |     if (!exponent.isInteger()) {
// |       return err(PolynomialForm.unsupportedEquation());
// |     }
// |
// |     const numer = exponent.numer();
// |     if (numer < 0n || numer > BigInt(MAX_POLYNOMIAL_DEGREE)) {
// |       return err(PolynomialForm.unsupportedEquation());
// |     }
// |
// |     const result = U32.tryFrom(numer);
// |     if (result === null) {
// |       return err(PolynomialForm.unsupportedEquation());
// |     }
// |     return Number(result);
// |   }
// |
// |   /** @param {Rational} value @returns {Rational} */
// |   evaluateAt(value) {
// |     let result = Rational.zero();
// |
// |     for (let degree = this.degree(); degree >= 0; degree -= 1) {
// |       result = result.mul(value).add(this.coefficient(degree));
// |     }
// |
// |     return result;
// |   }
// |
// |   /** @returns {string} */
// |   format() {
// |     const variable = this.variable;
// |     if (variable === null) {
// |       return this.coefficient(0).toDisplayString();
// |     }
// |
// |     /** @type {[boolean, string][]} */
// |     const parts = sortedTerms(this.terms)
// |       .reverse()
// |       .map(([degree, coefficient]) => {
// |         const coefficientAbs = coefficient.abs();
// |         return [
// |           coefficient.isNegative(),
// |           PolynomialForm.formatTermBody(coefficientAbs, degree, variable),
// |         ];
// |       });
// |
// |     return joinSignedParts(parts);
// |   }
// |
// |   /** @param {Rational} coefficientAbs @param {number} degree @param {string} variable */
// |   static formatTermBody(coefficientAbs, degree, variable) {
// |     if (degree === 0) {
// |       return coefficientAbs.toDisplayString();
// |     }
// |
// |     const variablePower = formatVariablePower(variable, degree);
// |     if (coefficientAbs.equals(Rational.one())) {
// |       return variablePower;
// |     } else {
// |       return `${coefficientAbs.toDisplayString()}*${variablePower}`;
// |     }
// |   }
// |
// |   static unsupportedEquation() {
// |     return CalculatorError.InvalidOperation(
// |       'only polynomial equations with one variable, unitless numbers, nonnegative integer powers, and real rational roots are supported',
// |     );
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | export class PolynomialEquationSolution {
// |   /**
// |    * @param {{ variable: string, polynomial: PolynomialForm, degree: number, roots: Rational[],
// |    *   originalLeft: string, originalRight: string }} fields
// |    */
// |   constructor({ variable, polynomial, degree, roots, originalLeft, originalRight }) {
// |     this.variable = variable;
// |     this.polynomial = polynomial;
// |     this.degree = degree;
// |     this.roots = roots;
// |     this.originalLeft = originalLeft;
// |     this.originalRight = originalRight;
// |   }
// |
// |   /** @returns {Value} */
// |   toValue() {
// |     if (this.roots.length === 1) {
// |       return Value.equationSolution(this.variable, this.roots[0]);
// |     } else {
// |       return Value.equationSolutions(this.variable, [...this.roots]);
// |     }
// |   }
// |
// |   /** @returns {string[]} */
// |   derivationSteps() {
// |     const steps = [
// |       `Original equation: ${this.originalLeft} = ${this.originalRight}`,
// |       `Move all terms to the left: ${this.polynomial.format()} = 0`,
// |       `Polynomial form: ${this.polynomial.format()} = 0`,
// |       `Polynomial degree: ${this.degree}`,
// |       `Choose target variable: ${this.variable}`,
// |       'Find real rational roots: test exact rational candidates and keep the values that make the polynomial equal 0',
// |     ];
// |
// |     for (const root of this.roots) {
// |       steps.push(
// |         `Verify root ${root.toDisplayString()}: substituting ${this.variable} = ${root.toDisplayString()} gives 0`,
// |       );
// |     }
// |
// |     steps.push(`Factor roots: ${formatRootFactors(this.variable, this.roots)}`);
// |     steps.push(`Solutions: ${formatSolutions(this.variable, this.roots)}`);
// |
// |     return steps;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {Expression} left
// |  * @param {Expression} right
// |  * @returns {PolynomialEquationSolution}
// |  */
// | export function solve(left, right) {
// |   const leftForm = PolynomialForm.fromExpression(left);
// |   const rightForm = PolynomialForm.fromExpression(right);
// |   const polynomial = leftForm.subtract(rightForm);
// |   const variable = polynomial.variable;
// |   if (variable === null) {
// |     return err(CalculatorError.InvalidOperation('polynomial equation has no variable'));
// |   }
// |   const degree = polynomial.degree();
// |
// |   if (degree <= 1) {
// |     return err(CalculatorError.InvalidOperation('polynomial equation is linear'));
// |   }
// |
// |   const roots = findRealRationalRoots(polynomial);
// |   if (roots.length === 0) {
// |     return err(
// |       CalculatorError.InvalidOperation(
// |         'polynomial equation has no supported real rational solutions',
// |       ),
// |     );
// |   }
// |
// |   return new PolynomialEquationSolution({
// |     variable,
// |     polynomial,
// |     degree,
// |     roots,
// |     originalLeft: left.toString(),
// |     originalRight: right.toString(),
// |   });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string | null} left
// |  * @param {string | null} right
// |  * @returns {string | null}
// |  */
// | function mergeVariableNames(left, right) {
// |   if (left !== null && right !== null && left !== right) {
// |     return err(
// |       CalculatorError.InvalidOperation(
// |         'polynomial equations with multiple variables are not supported',
// |       ),
// |     );
// |   }
// |   return left ?? right;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {PolynomialForm} polynomial @returns {Rational[]} */
// | function findRealRationalRoots(polynomial) {
// |   const binomial = binomialPowerRoots(polynomial);
// |   if (binomial !== null) {
// |     return binomial;
// |   }
// |
// |   /** @type {Map<string, Rational>} */
// |   const roots = new Map();
// |
// |   if (polynomial.degree() === 2) {
// |     for (const root of quadraticRoots(polynomial)) {
// |       insertRational(roots, root);
// |     }
// |   }
// |
// |   for (const candidate of rationalRootCandidates(polynomial)) {
// |     if (polynomial.evaluateAt(candidate).isZero()) {
// |       insertRational(roots, candidate);
// |     }
// |   }
// |
// |   return sortedRationals(roots);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {PolynomialForm} polynomial @returns {Rational[] | null} */
// | function binomialPowerRoots(polynomial) {
// |   const degree = polynomial.degree();
// |   if (degree < 2) {
// |     return null;
// |   }
// |
// |   const hasOnlyHighAndConstantTerms = [...polynomial.terms.keys()].every(
// |     (termDegree) => termDegree === 0 || termDegree === degree,
// |   );
// |   if (!hasOnlyHighAndConstantTerms) {
// |     return null;
// |   }
// |
// |   const leading = polynomial.coefficient(degree);
// |   if (leading.isZero()) {
// |     return null;
// |   }
// |
// |   const target = polynomial.coefficient(0).neg().div(leading);
// |   if (target.isZero()) {
// |     return [Rational.zero()];
// |   }
// |
// |   if (target.isNegative() && degree % 2 === 0) {
// |     return [];
// |   }
// |
// |   const root = rationalNthRoot(target, degree);
// |   if (root === null) {
// |     return null;
// |   }
// |   if (degree % 2 === 0) {
// |     const negativeRoot = root.neg();
// |     return [negativeRoot, root];
// |   } else {
// |     return [root];
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {PolynomialForm} polynomial @returns {Rational[]} */
// | function quadraticRoots(polynomial) {
// |   const a = polynomial.coefficient(2);
// |   const b = polynomial.coefficient(1);
// |   const c = polynomial.coefficient(0);
// |   if (a.isZero()) {
// |     return [];
// |   }
// |
// |   const four = Rational.fromInteger(4);
// |   const two = Rational.fromInteger(2);
// |   const discriminant = b.mul(b).sub(four.mul(a).mul(c));
// |   const squareRoot = rationalSquareRoot(discriminant);
// |   if (squareRoot === null) {
// |     return [];
// |   }
// |
// |   const denominator = two.mul(a);
// |   const first = b.neg().sub(squareRoot).div(denominator);
// |   const second = b.neg().add(squareRoot).div(denominator);
// |
// |   switch (first.cmp(second)) {
// |     case 0:
// |       return [first];
// |     case -1:
// |       return [first, second];
// |     default:
// |       return [second, first];
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {Rational} value @returns {Rational | null} */
// | function rationalSquareRoot(value) {
// |   return rationalNthRoot(value, 2);
// | }

