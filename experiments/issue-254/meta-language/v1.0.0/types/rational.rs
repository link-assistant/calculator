// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=954f41d0583d258da3fa3e21e6082f5a0f2819250a8d1dbda100b275cdbeec18 bytes=20136

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Rational number type for exact fractional arithmetic (port of
// |  * `src/types/rational.rs`).
// |  *
// |  * This module provides a `Rational` type that represents exact fractions
// |  * using arbitrary-precision `BigInt` numerator/denominator pairs. This allows
// |  * for perfect precision in calculations like `(1/3)*3 = 1` and exact
// |  * representation of large numbers like `10^100`.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { Err, err, ok, panic, tryResult } from '../rust/core.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { I128, U128, intToF64 } from '../rust/num.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import {
// |   Ratio,
// |   bigintAbs,
// |   bigintDiv,
// |   bigintFromStr,
// |   bigintPow,
// |   bigintRem,
// |   bigintToF64,
// |   bigintToI128,
// |   bigintToI64,
// |   bigintToU128,
// | } from '../rust/ratio.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { splitOnce, trim, trimEndMatches, trimStartMatches } from '../rust/str.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { Decimal } from './decimal.js';

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * A rational number represented as a fraction (numerator/denominator)
// |  * with arbitrary-precision integers.
// |  *
// |  * This type maintains exact precision by storing the numerator and denominator
// |  * separately and only performing the division when displaying or converting
// |  * to a decimal approximation. Using `BigInt` allows representing numbers
// |  * of any size exactly, such as `10^100`.
// |  */
// | export class Rational {
// |   /** @param {{ inner: Ratio }} fields */
// |   constructor({ inner }) {
// |     /** The underlying rational number with arbitrary-precision integers. */
// |     this.inner = inner;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `Ord`.
// |    * @param {Rational} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     return this.inner.cmp(other.inner);
// |   }
// |
// |   /**
// |    * `PartialOrd`: always ordered.
// |    * @param {Rational} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   partialCmp(other) {
// |     return this.cmp(other);
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {unknown} other
// |    */
// |   equals(other) {
// |     return other instanceof Rational && this.inner.equals(other.inner);
// |   }
// |
// |   /** Derived `Debug`. */
// |   fmtDebug() {
// |     return `Rational { inner: ${this.inner.fmtDebug()} }`;
// |   }
// |
// |   /** Derived `Serialize`: `{"inner": ["numer", "denom"]}`. */
// |   toJSON() {
// |     return { inner: serializeRatio(this.inner) };
// |   }
// |
// |   /**
// |    * Derived `Deserialize`. Throws `new Err(string)`.
// |    * @param {{ inner: [string, string] }} value
// |    */
// |   static fromJSON(value) {
// |     return new Rational({ inner: deserializeRatio(value.inner) });
// |   }
// |
// |   /**
// |    * Creates a new rational number from an i128 integer.
// |    * @param {number | bigint} n
// |    */
// |   static fromInteger(n) {
// |     return new Rational({ inner: Ratio.fromInteger(BigInt(n)) });
// |   }
// |
// |   /**
// |    * Creates a new rational number from a `BigInt`.
// |    * @param {bigint} n
// |    */
// |   static fromBigint(n) {
// |     return new Rational({ inner: Ratio.fromInteger(n) });
// |   }
// |
// |   /**
// |    * Creates a new rational number from i128 numerator and denominator.
// |    *
// |    * The fraction is automatically reduced to lowest terms.
// |    *
// |    * Panics if denominator is zero.
// |    * @param {number | bigint} numer
// |    * @param {number | bigint} denom
// |    */
// |   static new(numer, denom) {
// |     if (BigInt(denom) === 0n) {
// |       panic('Denominator cannot be zero');
// |     }
// |     return new Rational({ inner: Ratio.new(BigInt(numer), BigInt(denom)) });
// |   }
// |
// |   /**
// |    * Creates a new rational number from `BigInt` numerator and denominator.
// |    *
// |    * Panics if denominator is zero.
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    */
// |   static newBigint(numer, denom) {
// |     if (denom === 0n) {
// |       panic('Denominator cannot be zero');
// |     }
// |     return new Rational({ inner: Ratio.new(numer, denom) });
// |   }
// |
// |   /**
// |    * Creates a rational number from a Decimal value (also `From<Decimal>`).
// |    *
// |    * This converts the decimal to a fraction by using powers of 10.
// |    * @param {Decimal} d
// |    */
// |   static fromDecimal(d) {
// |     const s = d.toString();
// |
// |     // `s` is ASCII, so UTF-16 indexes are byte offsets.
// |     const dotPos = s.indexOf('.');
// |     if (dotPos !== -1) {
// |       const sign = s.startsWith('-') ? -1n : 1n;
// |       const sAbs = trimStartMatches(s, '-');
// |       const dotPosAbs = s.startsWith('-') ? dotPos - 1 : dotPos;
// |
// |       const intPart = sAbs.slice(0, dotPosAbs);
// |       const fracPart = sAbs.slice(dotPosAbs + 1);
// |
// |       const decimalPlaces = fracPart.length;
// |       const denom = bigintPow(10n, decimalPlaces);
// |
// |       const intVal = intPart === '' ? 0n : (ok(() => bigintFromStr(intPart)) ?? 0n);
// |       const fracVal = fracPart === '' ? 0n : (ok(() => bigintFromStr(fracPart)) ?? 0n);
// |
// |       const numer = sign * (intVal * denom + fracVal);
// |       return Rational.newBigint(numer, denom);
// |     }
// |     const n = ok(() => bigintFromStr(s)) ?? 0n;
// |     return Rational.fromBigint(n);
// |   }
// |
// |   /**
// |    * Creates a rational number from an f64.
// |    *
// |    * This uses a rational approximation via continued fraction expansion.
// |    * @param {number} value
// |    */
// |   static fromF64(value) {
// |     if (Number.isNaN(value) || !Number.isFinite(value)) {
// |       return Rational.fromInteger(0);
// |     }
// |
// |     const [numer, denom] = Rational.f64ToRationalContinuedFraction(value, 1_000_000_000n);
// |     return Rational.new(numer, denom);
// |   }
// |
// |   /**
// |    * Converts an f64 to a rational using continued fraction expansion.
// |    * @internal
// |    * @param {number} value
// |    * @param {bigint} maxDenom
// |    * @returns {[bigint, bigint]}
// |    */
// |   static f64ToRationalContinuedFraction(value, maxDenom) {
// |     if (value === 0) {
// |       return [0n, 1n];
// |     }
// |
// |     const sign = value < 0 ? -1n : 1n;
// |     value = Math.abs(value);
// |
// |     let hPrev = 0n;
// |     let kPrev = 1n;
// |     let hCurr = 1n;
// |     let kCurr = 0n;
// |
// |     let x = value;
// |
// |     for (let i = 0; i < 50; i++) {
// |       const a = I128.fromF64(Math.floor(x));
// |
// |       const hNext = I128.saturatingAdd(I128.saturatingMul(a, hCurr), hPrev);
// |       const kNext = I128.saturatingAdd(I128.saturatingMul(a, kCurr), kPrev);
// |
// |       if (kNext > maxDenom) {
// |         break;
// |       }
// |
// |       hPrev = hCurr;
// |       kPrev = kCurr;
// |       hCurr = hNext;
// |       kCurr = kNext;
// |
// |       const frac = x - intToF64(a);
// |       if (Math.abs(frac) < 1e-15) {
// |         break;
// |       }
// |
// |       x = 1.0 / frac;
// |     }
// |
// |     return [sign * hCurr, kCurr];
// |   }
// |
// |   /** Returns the numerator as a `BigInt` reference. @returns {bigint} */
// |   numerBigint() {
// |     return this.inner.numer();
// |   }
// |
// |   /** Returns the denominator as a `BigInt` reference. @returns {bigint} */
// |   denomBigint() {
// |     return this.inner.denom();
// |   }
// |
// |   /**
// |    * Returns the numerator, truncated to i128.
// |    * For numbers exceeding i128 range, this saturates.
// |    * @returns {bigint}
// |    */
// |   numer() {
// |     return bigintToI128(this.inner.numer()) ?? (this.inner.numer() < 0n ? I128.MIN : I128.MAX);
// |   }
// |
// |   /**
// |    * Returns the denominator, truncated to i128.
// |    * For numbers exceeding i128 range, this saturates.
// |    * @returns {bigint}
// |    */
// |   denom() {
// |     return bigintToI128(this.inner.denom()) ?? I128.MAX;
// |   }
// |
// |   /** Returns true if this is an integer (denominator is 1). */
// |   isInteger() {
// |     return this.inner.isInteger();
// |   }
// |
// |   /** Returns true if this is zero. */
// |   isZero() {
// |     return this.inner.isZero();
// |   }
// |
// |   /** Returns true if this is negative. */
// |   isNegative() {
// |     return this.inner.isNegative();
// |   }
// |
// |   /** Returns the absolute value. */
// |   abs() {
// |     return new Rational({ inner: this.inner.abs() });
// |   }
// |
// |   /** Converts to f64 (may lose precision for large or very precise numbers). */
// |   toF64() {
// |     const n = bigintToF64(this.inner.numer());
// |     const d = bigintToF64(this.inner.denom());
// |     return n / d;
// |   }
// |
// |   /** Converts to a Decimal (may lose precision for very large numbers). */
// |   toDecimal() {
// |     if (this.isInteger()) {
// |       const n = bigintToI64(this.inner.numer());
// |       if (n !== null) {
// |         return Decimal.new(n);
// |       }
// |     }
// |     return Decimal.fromF64(this.toF64());
// |   }
// |
// |   /** Returns zero. */
// |   static zero() {
// |     return new Rational({ inner: Ratio.zero() });
// |   }
// |
// |   /** Returns one. */
// |   static one() {
// |     return new Rational({ inner: Ratio.one() });
// |   }
// |
// |   /**
// |    * Raises this rational to an integer power (exact computation).
// |    *
// |    * For negative exponents, computes the reciprocal raised to the positive power.
// |    * @param {number} exp an i32
// |    */
// |   powI32(exp) {
// |     if (exp === 0) {
// |       return Rational.one();
// |     }
// |     if (exp > 0) {
// |       return new Rational({ inner: this.inner.pow(exp) });
// |     }
// |     // Negative exponent: (a/b)^(-n) = (b/a)^n
// |     const recip = this.inner.recip();
// |     return new Rational({ inner: recip.pow(-exp) });
// |   }
// |
// |   /**
// |    * Checked division that returns None on division by zero.
// |    * @param {Rational} other
// |    * @returns {Rational | null}
// |    */
// |   checkedDiv(other) {
// |     if (other.isZero()) {
// |       return null;
// |     }
// |     return new Rational({ inner: this.inner.div(other.inner) });
// |   }
// |
// |   /**
// |    * Returns the signed remainder after division, truncated toward zero.
// |    * @param {Rational} other
// |    * @returns {Rational | null}
// |    */
// |   remainder(other) {
// |     if (other.isZero()) {
// |       return null;
// |     }
// |
// |     const quotientRatio = this.inner.div(other.inner);
// |     const quotient = bigintDiv(quotientRatio.numer(), quotientRatio.denom());
// |     const remainder = this.inner.sub(other.inner.mul(Ratio.fromInteger(quotient)));
// |
// |     return new Rational({ inner: remainder });
// |   }
// |
// |   /**
// |    * Converts the rational to a display string.
// |    *
// |    * If the rational is an integer, returns the exact integer string
// |    * (using arbitrary precision, so `10^100` displays all 101 digits).
// |    * Otherwise, returns the decimal representation.
// |    * @returns {string}
// |    */
// |   toDisplayString() {
// |     if (this.isInteger()) {
// |       return this.inner.numer().toString();
// |     }
// |     const approx = this.toDecimal();
// |     if (approx.isZero() && !this.isZero()) {
// |       // Too small for the 28-digit decimal (e.g. `10^-40`): show it
// |       // in scientific notation instead of rounding it to 0.
// |       return this.toScientificString(15);
// |     }
// |     return approx.normalize().toString();
// |   }
// |
// |   /**
// |    * Formats a non-zero rational as `m.mmme-k` with up to `digits`
// |    * significant digits, computed exactly (truncated, trailing zeros removed).
// |    * @internal
// |    * @param {number} digits
// |    * @returns {string}
// |    */
// |   toScientificString(digits) {
// |     const numer = bigintAbs(this.inner.numer());
// |     const denom = bigintAbs(this.inner.denom());
// |     const ten = 10n;
// |     // Find the exponent e with 10^e <= |self| < 10^(e+1).
// |     let exponent = numer.toString().length - denom.toString().length;
// |     /**
// |      * @param {number} e
// |      * @returns {[bigint, bigint]}
// |      */
// |     const scaled = (e) => {
// |       if (e >= 0) {
// |         return [numer, denom * bigintPow(ten, Math.abs(e))];
// |       }
// |       return [numer * bigintPow(ten, Math.abs(e)), denom];
// |     };
// |     {
// |       const [n, d] = scaled(exponent);
// |       if (n < d) {
// |         exponent -= 1;
// |       }
// |     }
// |     const [n, d] = scaled(exponent - digits + 1);
// |     const mantissa = bigintDiv(n, d).toString();
// |     const intPart = mantissa.slice(0, 1);
// |     const fracPart = trimEndMatches(mantissa.slice(1), '0');
// |     const sign = this.isNegative() ? '-' : '';
// |     if (fracPart === '') {
// |       return `${sign}${intPart}e${exponent}`;
// |     }
// |     return `${sign}${intPart}.${fracPart}e${exponent}`;
// |   }
// |
// |   /**
// |    * Returns a fractional representation (e.g., "1/3").
// |    * @returns {string}
// |    */
// |   toFractionString() {
// |     if (this.isInteger()) {
// |       return this.inner.numer().toString();
// |     }
// |     return `${this.inner.numer()}/${this.inner.denom()}`;
// |   }
// |
// |   /**
// |    * Detects if the decimal representation is a repeating decimal
// |    * and returns the notation (e.g., "0.3̅" or "0.(3)").
// |    *
// |    * Returns None if not a repeating decimal, if detection failed,
// |    * or if the numbers are too large for the detection algorithm.
// |    * @returns {RepeatingDecimal | null}
// |    */
// |   toRepeatingDecimalNotation() {
// |     if (this.isInteger()) {
// |       return null;
// |     }
// |
// |     const numerAbs = bigintAbs(this.inner.numer());
// |     const denomAbs = bigintAbs(this.inner.denom());
// |     const isNegative = this.isNegative();
// |
// |     const numerU128 = bigintToU128(numerAbs);
// |     if (numerU128 === null) {
// |       return null;
// |     }
// |     const denomU128 = bigintToU128(denomAbs);
// |     if (denomU128 === null) {
// |       return null;
// |     }
// |
// |     return detectRepeatingDecimal(numerU128, denomU128, isNegative);
// |   }
// |
// |   /** `Default`: zero. */
// |   static default() {
// |     return Rational.zero();
// |   }
// |
// |   /** `Display`. */
// |   toString() {
// |     return this.toDisplayString();
// |   }
// |
// |   /**
// |    * `FromStr`. Throws `new Err(string)`.
// |    * @param {string} s
// |    * @returns {Rational}
// |    */
// |   static fromStr(s) {
// |     // Try parsing as a fraction first (e.g., "1/3")
// |     const parts = splitOnce(s, '/');
// |     if (parts !== null) {
// |       const [numerStr, denomStr] = parts;
// |       const numer = mapErr(() => I128.parse(trim(numerStr)));
// |       const denom = mapErr(() => I128.parse(trim(denomStr)));
// |       if (denom === 0n) {
// |         return err('Denominator cannot be zero');
// |       }
// |       return Rational.new(numer, denom);
// |     }
// |
// |     // Try parsing as a decimal
// |     const d = mapErr(() => Decimal.fromStr(s));
// |     return Rational.fromDecimal(d);
// |   }
// |
// |   /**
// |    * `From<i64>`.
// |    * @param {number | bigint} value
// |    */
// |   static fromI64(value) {
// |     return Rational.fromInteger(value);
// |   }
// |
// |   /**
// |    * `From<i32>`.
// |    * @param {number} value
// |    */
// |   static fromI32(value) {
// |     return Rational.fromInteger(value);
// |   }
// |
// |   /**
// |    * `Add`.
// |    * @param {Rational} other
// |    */
// |   add(other) {
// |     return new Rational({ inner: this.inner.add(other.inner) });
// |   }
// |
// |   /**
// |    * `Sub`.
// |    * @param {Rational} other
// |    */
// |   sub(other) {
// |     return new Rational({ inner: this.inner.sub(other.inner) });
// |   }
// |
// |   /**
// |    * `Mul`.
// |    * @param {Rational} other
// |    */
// |   mul(other) {
// |     return new Rational({ inner: this.inner.mul(other.inner) });
// |   }
// |
// |   /**
// |    * `Div`; panics on division by zero.
// |    * @param {Rational} other
// |    */
// |   div(other) {
// |     return new Rational({ inner: this.inner.div(other.inner) });
// |   }
// |
// |   /** `Neg`. */
// |   neg() {
// |     return new Rational({ inner: this.inner.neg() });
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serialize_ratio`: `(numer.to_string(), denom.to_string())`.
// |  * @param {Ratio} ratio
// |  * @returns {[string, string]}
// |  */
// | function serializeRatio(ratio) {
// |   return [ratio.numer().toString(), ratio.denom().toString()];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `deserialize_ratio`. Throws `new Err(string)`.
// |  * @param {[string, string]} tuple
// |  * @returns {Ratio}
// |  */
// | function deserializeRatio([numerStr, denomStr]) {
// |   const n = tryResult(() => bigintFromStr(numerStr));
// |   if (!n.ok) {
// |     return err(`invalid numerator: ${n.error}`);
// |   }
// |   const d = tryResult(() => bigintFromStr(denomStr));
// |   if (!d.ok) {
// |     return err(`invalid denominator: ${d.error}`);
// |   }
// |   return Ratio.new(/** @type {bigint} */ (n.value), /** @type {bigint} */ (d.value));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `.map_err(|e| format!("{e}"))?`.
// |  * @template T
// |  * @param {() => T} fn
// |  * @returns {T}
// |  */
// | function mapErr(fn) {
// |   try {
// |     return fn();
// |   } catch (e) {
// |     if (e instanceof Err) {
// |       return err(String(e.error));
// |     }
// |     throw e;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Represents a repeating decimal with its components. */
// | export class RepeatingDecimal {
// |   /**
// |    * @param {{ isNegative: boolean, integerPart: string, nonRepeating: string, repeating: string }} fields
// |    */
// |   constructor({ isNegative, integerPart, nonRepeating, repeating }) {
// |     /** Whether the number is negative. */
// |     this.isNegative = isNegative;
// |     /** The integer part. */
// |     this.integerPart = integerPart;
// |     /** The non-repeating part after the decimal point. */
// |     this.nonRepeating = nonRepeating;
// |     /** The repeating part (repetend). */
// |     this.repeating = repeating;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * Formats using vinculum notation (overline): 0.3̅
// |    * @returns {string}
// |    */
// |   toVinculumNotation() {
// |     const sign = this.isNegative ? '-' : '';
// |
// |     if (this.repeating === '') {
// |       // Not actually repeating
// |       if (this.nonRepeating === '') {
// |         return `${sign}${this.integerPart}`;
// |       }
// |       return `${sign}${this.integerPart}.${this.nonRepeating}`;
// |     }
// |     // Add combining overline character (U+0305) after each repeating digit
// |     let repetendWithOverline = '';
// |     for (const c of this.repeating) {
// |       repetendWithOverline += `${c}̅`;
// |     }
// |
// |     if (this.nonRepeating === '') {
// |       return `${sign}${this.integerPart}.${repetendWithOverline}`;
// |     }
// |     return `${sign}${this.integerPart}.${this.nonRepeating}${repetendWithOverline}`;
// |   }
// |
// |   /**
// |    * Formats using parenthesis notation: 0.(3)
// |    * @returns {string}
// |    */
// |   toParenthesisNotation() {
// |     const sign = this.isNegative ? '-' : '';
// |
// |     if (this.repeating === '') {
// |       if (this.nonRepeating === '') {
// |         return `${sign}${this.integerPart}`;
// |       }
// |       return `${sign}${this.integerPart}.${this.nonRepeating}`;
// |     } else if (this.nonRepeating === '') {
// |       return `${sign}${this.integerPart}.(${this.repeating})`;
// |     }
// |     return `${sign}${this.integerPart}.${this.nonRepeating}(${this.repeating})`;
// |   }
// |
// |   /**
// |    * Formats using ellipsis notation: 0.333...
// |    * @returns {string}
// |    */
// |   toEllipsisNotation() {
// |     const sign = this.isNegative ? '-' : '';
// |
// |     if (this.repeating === '') {
// |       if (this.nonRepeating === '') {
// |         return `${sign}${this.integerPart}`;
// |       }
// |       return `${sign}${this.integerPart}.${this.nonRepeating}`;
// |     }
// |     // Show a few repetitions then ellipsis (the repetend is ASCII digits,
// |     // so `.length` is its byte length)
// |     const repetitions = this.repeating.repeat(
// |       Math.min(3, Math.trunc(10 / Math.max(this.repeating.length, 1))),
// |     );
// |     if (this.nonRepeating === '') {
// |       return `${sign}${this.integerPart}.${repetitions}...`;
// |     }
// |     return `${sign}${this.integerPart}.${this.nonRepeating}${repetitions}...`;
// |   }
// |
// |   /**
// |    * Formats using LaTeX: 0.\overline{3}
// |    * @returns {string}
// |    */
// |   toLatex() {
// |     const sign = this.isNegative ? '-' : '';
// |
// |     if (this.repeating === '') {
// |       if (this.nonRepeating === '') {
// |         return `${sign}${this.integerPart}`;
// |       }
// |       return `${sign}${this.integerPart}.${this.nonRepeating}`;
// |     } else if (this.nonRepeating === '') {
// |       return `${sign}${this.integerPart}.\\overline{${this.repeating}}`;
// |     }
// |     return `${sign}${this.integerPart}.${this.nonRepeating}\\overline{${this.repeating}}`;
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Detects repeating pattern in a fraction's decimal expansion.
// |  * @param {bigint} numerator a u128
// |  * @param {bigint} denominator a u128
// |  * @param {boolean} isNegative
// |  * @returns {RepeatingDecimal | null}
// |  */
// | function detectRepeatingDecimal(numerator, denominator, isNegative) {
// |   const MAX_DIGITS = 1000;
// |
// |   if (denominator === 0n) {
// |     return null;
// |   }
// |
// |   const integerPart = bigintDiv(numerator, denominator);
// |   let remainder = bigintRem(numerator, denominator);
// |
// |   if (remainder === 0n) {
// |     // No decimal part
// |     return new RepeatingDecimal({
// |       isNegative,
// |       integerPart: integerPart.toString(),
// |       nonRepeating: '',
// |       repeating: '',
// |     });
// |   }
// |
// |   /** @type {string[]} */
// |   const digits = [];
// |   /** @type {Map<string, number>} */
// |   const remainderPositions = new Map();
// |   /** @type {number | null} */
// |   let repeatStart = null;
// |
// |   // Perform long division, tracking remainders
// |
// |   while (remainder !== 0n && digits.length < MAX_DIGITS) {
// |     const pos = remainderPositions.get(remainder.toString());
// |     if (pos !== undefined) {
// |       repeatStart = pos;
// |       break;
// |     }
// |
// |     remainderPositions.set(remainder.toString(), digits.length);
// |
// |     // `remainder * 10` can overflow u128 for denominators above
// |     // `u128::MAX / 10`; the release build wraps.
// |     remainder = U128.wrappingMul(remainder, 10n);
// |     const digit = remainder / denominator;
// |     digits.push(String(digit));
// |     remainder %= denominator;
// |   }
// |
// |   if (repeatStart !== null) {
// |     const nonRepeating = digits.slice(0, repeatStart).join('');
// |     const repeating = digits.slice(repeatStart).join('');
// |
// |     return new RepeatingDecimal({
// |       isNegative,
// |       integerPart: integerPart.toString(),
// |       nonRepeating,
// |       repeating,
// |     });
// |   } else if (remainder === 0n) {
// |     // Terminating decimal
// |     return new RepeatingDecimal({
// |       isNegative,
// |       integerPart: integerPart.toString(),
// |       nonRepeating: digits.join(''),
// |       repeating: '',
// |     });
// |   }
// |   // Too long, couldn't detect pattern
// |   return null;
// | }
