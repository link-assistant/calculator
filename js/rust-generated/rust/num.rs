// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=174f00ba8d6d41e23c563ece5b6cd1f87c9654f5e76de0193a180827a5197591 bytes=10196

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

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
// |  * Rust number semantics: `f64` methods whose JavaScript counterparts differ,
// |  * saturating `as` casts, fixed-width integers with `checked_*` arithmetic,
// |  * and `str::parse` for floats and integers.
// |  *
// |  * Transcendental functions (`sin`, `ln`, `powf`, ...) and `mul_add` live in
// |  * `rust/libm.js`, a port of the `libm` crate, so every engine returns the same
// |  * bits. See js/PORTING.md.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from './core.js';
use crate::rust::core_::err;

// meta-language:translated JavaScript export_statement items=1 sha256=81e96126cdc5156f01df250ce76dfa1b08f21a9e1b578a2b07750da0b54e119b
// | /** `f64::MIN_POSITIVE`. */
// | export const F64_MIN_POSITIVE = 2.2250738585072014e-308;
pub const F64_MIN_POSITIVE: f64 = 2.2250738585072014e-308f64;
// meta-language:translated JavaScript export_statement items=1 sha256=6e54bd4deeeaaf41350000cf8aebf264d06f0c0540263f2f49ad726e61a745c6
// | /** `f64::consts::PI`. */
// | export const PI = Math.PI;
pub const PI: f64 = 3.141592653589793f64;
// meta-language:translated JavaScript export_statement items=1 sha256=b7fe8303bf19e7be7a334164d5818514ede9ff85f3478f2e19f06b3ad9910400
// | /** `f64::consts::E`. */
// | export const E = Math.E;
pub const E: f64 = 2.718281828459045f64;

// meta-language:translated JavaScript export_statement items=1 sha256=606867b92bfc8b3d7e1a0371dc038db9adb3797aca6f7143152c102b83df17e1
// | /**
// |  * `f64::round`: half away from zero (`Math.round` rounds half up).
// |  * @param {number} x
// |  */
// | export function round(x) {
// |   const truncated = Math.trunc(x);
// |   if (Math.abs(x - truncated) >= 0.5) {
// |     return truncated + Math.sign(x);
// |   }
// |   return truncated;
// | }
pub fn round(x: f64) -> f64 {
    {
        let truncated = x.trunc();
        if ((((x - truncated)).abs()) >= (0.5f64)) {
            (truncated + crate::ml_math::sign(x))
        } else {
            truncated
        }
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `f64::signum`: 1.0 for +0.0 and positive values, -1.0 for -0.0 and
// |  * negative values, NaN for NaN (`Math.sign` returns zeros for zeros).
// |  * @param {number} x
// |  */
// | export function signum(x) {
// |   if (Number.isNaN(x)) {
// |     return Number.NaN;
// |   }
// |   return x > 0 || Object.is(x, 0) ? 1 : -1;
// | }

// meta-language:translated JavaScript export_statement items=1 sha256=00e6bf251ddf9cfbf8fd574d15b17aaebfb300ae3507fb016230c214a1a8fa87
// | /**
// |  * `f64::max`: NaN is ignored (`Math.max` propagates it).
// |  * @param {number} a
// |  * @param {number} b
// |  */
// | export function fmax(a, b) {
// |   if (Number.isNaN(a)) {
// |     return b;
// |   }
// |   if (Number.isNaN(b)) {
// |     return a;
// |   }
// |   return a < b ? b : a;
// | }
pub fn fmax(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        b
    } else {
        if b.is_nan() {
            a
        } else {
            if (a < b) {
                b
            } else {
                a
            }
        }
    }
}

// meta-language:translated JavaScript export_statement items=1 sha256=e1cfb94f0e11c7fffd06f29dc48d3a142c4c87b895f7f1dc3a2565059589cecc
// | /**
// |  * `f64::min`: NaN is ignored (`Math.min` propagates it).
// |  * @param {number} a
// |  * @param {number} b
// |  */
// | export function fmin(a, b) {
// |   if (Number.isNaN(a)) {
// |     return b;
// |   }
// |   if (Number.isNaN(b)) {
// |     return a;
// |   }
// |   return b < a ? b : a;
// | }
pub fn fmin(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        b
    } else {
        if b.is_nan() {
            a
        } else {
            if (b < a) {
                b
            } else {
                a
            }
        }
    }
}

// meta-language:translated JavaScript export_statement items=1 sha256=3e8fafd4404bb22352333fc711e38e81ccce2028f6c05f610402b919fee3d3d3
// | /**
// |  * `f64::to_degrees`.
// |  * @param {number} x
// |  */
// | export function toDegrees(x) {
// |   // The f64 nearest to Rust's `PIS_IN_180` constant, 57.2957795130823208767981548141051703.
// |   return x * 57.29577951308232;
// | }
pub fn to_degrees(x: f64) -> f64 {
    (x * 57.29577951308232f64)
}

// meta-language:translated JavaScript export_statement items=1 sha256=521f49f075c162cd5aed0d3405424c78e0087c016a265cbae72d50a6a4d84a90
// | /**
// |  * `f64::to_radians`.
// |  * @param {number} x
// |  */
// | export function toRadians(x) {
// |   return x * (Math.PI / 180);
// | }
pub fn to_radians(x: f64) -> f64 {
    (x * (3.141592653589793f64 / 180f64))
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `rem_euclid` for integers (numbers or bigints): the non-negative remainder.
// |  * @template {number | bigint} T
// |  * @param {T} a
// |  * @param {T} b
// |  * @returns {T}
// |  */
// | export function remEuclid(a, b) {
// |   const r = /** @type {any} */ (a) % /** @type {any} */ (b);
// |   if (r < 0) {
// |     return /** @type {T} */ (b < 0 ? r - /** @type {any} */ (b) : r + /** @type {any} */ (b));
// |   }
// |   return /** @type {T} */ (r === 0 ? (typeof r === 'bigint' ? 0n : 0) : r);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * A Rust primitive integer type. Values are bigints; `checked_*` return null
// |  * on overflow, and `as` casts saturate (from floats) or wrap (from integers).
// |  */
// | export class IntType {
// |   /**
// |    * @param {string} name
// |    * @param {number} bits
// |    * @param {boolean} signed
// |    */
// |   constructor(name, bits, signed) {
// |     this.name = name;
// |     this.bits = bits;
// |     this.signed = signed;
// |     this.MIN = signed ? -(1n << BigInt(bits - 1)) : 0n;
// |     this.MAX = signed ? (1n << BigInt(bits - 1)) - 1n : (1n << BigInt(bits)) - 1n;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * The value if it fits, otherwise null (`T::try_from(v).ok()`).
// |    * @param {number | bigint} value an integer
// |    * @returns {bigint | null}
// |    */
// |   tryFrom(value) {
// |     const v = BigInt(value);
// |     return v < this.MIN || v > this.MAX ? null : v;
// |   }
// |
// |   /**
// |    * `f64 as T`: truncates toward zero, saturates, NaN -> 0.
// |    * @param {number} x
// |    */
// |   fromF64(x) {
// |     if (Number.isNaN(x)) {
// |       return 0n;
// |     }
// |     if (x === Infinity) {
// |       return this.MAX;
// |     }
// |     if (x === -Infinity) {
// |       return this.MIN;
// |     }
// |     const v = BigInt(Math.trunc(x));
// |     if (v < this.MIN) {
// |       return this.MIN;
// |     }
// |     return v > this.MAX ? this.MAX : v;
// |   }
// |
// |   /**
// |    * `integer as T`: keeps the low `bits` bits (two's complement).
// |    * @param {number | bigint} value
// |    */
// |   wrap(value) {
// |     const v = BigInt(value);
// |     return this.signed ? BigInt.asIntN(this.bits, v) : BigInt.asUintN(this.bits, v);
// |   }
// |
// |   /** @param {bigint} v */
// |   checked(v) {
// |     return v < this.MIN || v > this.MAX ? null : v;
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   checkedAdd(a, b) {
// |     return this.checked(a + b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   checkedSub(a, b) {
// |     return this.checked(a - b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   checkedMul(a, b) {
// |     return this.checked(a * b);
// |   }
// |
// |   /** `checked_div`: None on division by zero and on `MIN / -1`. @param {bigint} a @param {bigint} b */
// |   checkedDiv(a, b) {
// |     return b === 0n ? null : this.checked(a / b);
// |   }
// |
// |   /** `checked_rem`: None on division by zero and on `MIN % -1`. @param {bigint} a @param {bigint} b */
// |   checkedRem(a, b) {
// |     if (b === 0n || this.checked(a / b) === null) {
// |       return null;
// |     }
// |     return a % b;
// |   }
// |
// |   /** @param {bigint} a */
// |   checkedAbs(a) {
// |     return this.checked(a < 0n ? -a : a);
// |   }
// |
// |   /** @param {bigint} a */
// |   checkedNeg(a) {
// |     return this.checked(-a);
// |   }
// |
// |   /**
// |    * `checked_pow(exp)`; `exp` is a `u32`.
// |    * @param {bigint} base
// |    * @param {number} exp
// |    * @returns {bigint | null}
// |    */
// |   checkedPow(base, exp) {
// |     let result = 1n;
// |     for (let index = 0; index < exp; index += 1) {
// |       result *= base;
// |       if (this.checked(result) === null) {
// |         return null;
// |       }
// |     }
// |     return result;
// |   }
// |
// |   /** @param {bigint} v */
// |   saturate(v) {
// |     if (v < this.MIN) {
// |       return this.MIN;
// |     }
// |     return v > this.MAX ? this.MAX : v;
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   saturatingAdd(a, b) {
// |     return this.saturate(a + b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   saturatingSub(a, b) {
// |     return this.saturate(a - b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   saturatingMul(a, b) {
// |     return this.saturate(a * b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   wrappingAdd(a, b) {
// |     return this.wrap(a + b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   wrappingSub(a, b) {
// |     return this.wrap(a - b);
// |   }
// |
// |   /** @param {bigint} a @param {bigint} b */
// |   wrappingMul(a, b) {
// |     return this.wrap(a * b);
// |   }
// |
// |   /**
// |    * `s.parse::<T>()` (radix 10). Throws `Err(ParseIntError)`.
// |    * @param {string} s
// |    */
// |   parse(s) {
// |     return this.fromStrRadix(s, 10);
// |   }
// |
// |   /**
// |    * `T::from_str_radix(s, radix)`: an optional `+` (or `-` for signed
// |    * types), then ASCII digits; no whitespace or separators.
// |    * @param {string} s
// |    * @param {number} radix
// |    */
// |   fromStrRadix(s, radix) {
// |     if (s === '') {
// |       return err(new ParseIntError('Empty'));
// |     }
// |     if (s === '+' || s === '-') {
// |       return err(new ParseIntError('InvalidDigit'));
// |     }
// |     let digits = s;
// |     let positive = true;
// |     if (s[0] === '+') {
// |       digits = s.slice(1);
// |     } else if (s[0] === '-' && this.signed) {
// |       digits = s.slice(1);
// |       positive = false;
// |     }
// |     const big = BigInt(radix);
// |     let result = 0n;
// |     for (const c of digits) {
// |       const code = c.charCodeAt(0);
// |       let digit = 99;
// |       if (c.length === 1 && code >= 48 && code <= 57) {
// |         digit = code - 48;
// |       } else if (c.length === 1 && code >= 97 && code <= 122) {
// |         digit = code - 87;
// |       } else if (c.length === 1 && code >= 65 && code <= 90) {
// |         digit = code - 55;
// |       }
// |       if (digit >= radix) {
// |         return err(new ParseIntError('InvalidDigit'));
// |       }
// |       result = positive ? result * big + BigInt(digit) : result * big - BigInt(digit);
// |       if (result > this.MAX) {
// |         return err(new ParseIntError('PosOverflow'));
// |       }
// |       if (result < this.MIN) {
// |         return err(new ParseIntError('NegOverflow'));
// |       }
// |     }
// |     return result;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | export const I8 = new IntType('i8', 8, true);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const I16 = new IntType('i16', 16, true);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const I32 = new IntType('i32', 32, true);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const I64 = new IntType('i64', 64, true);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const I128 = new IntType('i128', 128, true);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const U8 = new IntType('u8', 8, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const U16 = new IntType('u16', 16, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const U32 = new IntType('u32', 32, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const U64 = new IntType('u64', 64, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const U128 = new IntType('u128', 128, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | /** `usize` of the 64-bit targets the goldens are recorded on. */
// | export const USIZE = new IntType('usize', 64, false);
// meta-language:carried JavaScript export_statement (unsupported)
// | export const ISIZE = new IntType('isize', 64, true);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const INT_ERROR_MESSAGES = {
// |   Empty: 'cannot parse integer from empty string',
// |   InvalidDigit: 'invalid digit found in string',
// |   PosOverflow: 'number too large to fit in target type',
// |   NegOverflow: 'number too small to fit in target type',
// |   Zero: 'number would be zero for non-zero type',
// | };

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `core::num::ParseIntError`. */
// | export class ParseIntError {
// |   /** @param {'Empty' | 'InvalidDigit' | 'PosOverflow' | 'NegOverflow' | 'Zero'} kind */
// |   constructor(kind) {
// |     this.kind = kind;
// |     Object.freeze(this);
// |   }
// |
// |   toString() {
// |     return INT_ERROR_MESSAGES[this.kind];
// |   }
// |
// |   fmtDebug() {
// |     return `ParseIntError { kind: ${this.kind} }`;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `core::num::ParseFloatError`. */
// | export class ParseFloatError {
// |   /** @param {'Empty' | 'Invalid'} kind */
// |   constructor(kind) {
// |     this.kind = kind;
// |     Object.freeze(this);
// |   }
// |
// |   toString() {
// |     return this.kind === 'Empty' ? 'cannot parse float from empty string' : 'invalid float literal';
// |   }
// |
// |   fmtDebug() {
// |     return `ParseFloatError { kind: ${this.kind} }`;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const FLOAT_LITERAL = /^[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const FLOAT_SPECIAL = /^([+-]?)(inf|infinity|nan)$/i;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `s.parse::<f64>()`: Rust's grammar (no whitespace, no `_`, no hex,
// |  * case-insensitive `inf`/`infinity`/`nan`). Throws `Err(ParseFloatError)`.
// |  * @param {string} s
// |  * @returns {number}
// |  */
// | export function parseF64(s) {
// |   if (s === '') {
// |     return err(new ParseFloatError('Empty'));
// |   }
// |   if (FLOAT_LITERAL.test(s)) {
// |     return Number(s);
// |   }
// |   const special = FLOAT_SPECIAL.exec(s);
// |   if (special === null) {
// |     return err(new ParseFloatError('Invalid'));
// |   }
// |   if (special[2].toLowerCase() === 'nan') {
// |     return Number.NaN;
// |   }
// |   return special[1] === '-' ? -Infinity : Infinity;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `integer as f64` for bigints: rounds to nearest, ties to even, as Rust.
// |  * @param {bigint} value
// |  */
// | export function intToF64(value) {
// |   return Number(value);
// | }
