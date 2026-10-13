// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7d0d22eae8d195cf4f8a94468e92a7ee3e8d66898be02edd39f62b9e19d0c650 bytes=5032

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Decimal number type for precise calculations (port of
// |  * `src/types/decimal.rs`): a thin wrapper around `rust_decimal::Decimal`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=bb51b70ae0621f3ca98a0c5abb2fe382cec7758a64464b8b96268199dd285521
// | import { RustDecimal } from '../rust/decimal.js';
use crate::rust::decimal::RustDecimal;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * A decimal number with arbitrary precision: `pub struct Decimal(RustDecimal)`.
// |  * Instances are frozen (`Copy` in Rust).
// |  */
// | export class Decimal {
// |   /** @param {RustDecimal} inner */
// |   constructor(inner) {
// |     /** @type {RustDecimal} */
// |     this._0 = inner;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * Creates a new Decimal from an integer (`From<i64>`).
// |    * @param {number | bigint} value
// |    */
// |   static new(value) {
// |     return new Decimal(RustDecimal.fromI64(value));
// |   }
// |
// |   /**
// |    * Parses conventional decimal scientific notation such as `1.5e-6`.
// |    * Throws `new Err(DecimalError)`.
// |    * @param {string} value
// |    */
// |   static fromScientific(value) {
// |     return new Decimal(RustDecimal.fromScientific(value));
// |   }
// |
// |   /**
// |    * Creates a new Decimal from a float (may lose precision); `null` if the
// |    * conversion fails.
// |    * @param {number} value
// |    * @returns {Decimal | null}
// |    */
// |   static tryFromF64(value) {
// |     const inner = RustDecimal.tryFromF64(value);
// |     return inner === null ? null : new Decimal(inner);
// |   }
// |
// |   /**
// |    * Creates a new Decimal from a float, defaulting to zero on failure.
// |    * @param {number} value
// |    */
// |   static fromF64(value) {
// |     return Decimal.tryFromF64(value) ?? Decimal.zero();
// |   }
// |
// |   /** Returns zero. */
// |   static zero() {
// |     return new Decimal(RustDecimal.ZERO);
// |   }
// |
// |   /** Returns one. */
// |   static one() {
// |     return new Decimal(RustDecimal.ONE);
// |   }
// |
// |   /** `Default`: zero. */
// |   static default() {
// |     return Decimal.zero();
// |   }
// |
// |   /**
// |    * `FromStr`. Throws `new Err(DecimalError)`.
// |    * @param {string} s
// |    */
// |   static fromStr(s) {
// |     return new Decimal(RustDecimal.fromStr(s));
// |   }
// |
// |   /**
// |    * `From<i64>`.
// |    * @param {number | bigint} value
// |    */
// |   static fromI64(value) {
// |     return Decimal.new(value);
// |   }
// |
// |   /**
// |    * `From<i32>`.
// |    * @param {number} value
// |    */
// |   static fromI32(value) {
// |     return new Decimal(RustDecimal.fromI32(value));
// |   }
// |
// |   /** Checks if the value is zero. */
// |   isZero() {
// |     return this._0.isZero();
// |   }
// |
// |   /** Checks if the value is negative (`-0` is not). */
// |   isNegative() {
// |     return this._0.isSignNegative() && !this.isZero();
// |   }
// |
// |   /** Returns the absolute value. */
// |   abs() {
// |     return new Decimal(this._0.abs());
// |   }
// |
// |   /** Converts to f64 (may lose precision); `0` if the conversion fails. */
// |   toF64() {
// |     return this._0.toF64() ?? 0;
// |   }
// |
// |   /**
// |    * Rounds to the specified number of decimal places (banker's rounding).
// |    * @param {number} dp
// |    */
// |   round(dp) {
// |     return new Decimal(this._0.roundDp(dp));
// |   }
// |
// |   /** Normalizes the decimal (removes trailing zeros). */
// |   normalize() {
// |     return new Decimal(this._0.normalize());
// |   }
// |
// |   /**
// |    * Checked multiplication that returns `null` on overflow.
// |    * @param {Decimal} other
// |    * @returns {Decimal | null}
// |    */
// |   checkedMul(other) {
// |     return wrap(this._0.checkedMul(other._0));
// |   }
// |
// |   /**
// |    * Checked division that returns `null` on division by zero (or overflow).
// |    * @param {Decimal} other
// |    * @returns {Decimal | null}
// |    */
// |   checkedDiv(other) {
// |     if (other.isZero()) {
// |       return null;
// |     }
// |     return wrap(this._0.checkedDiv(other._0));
// |   }
// |
// |   /**
// |    * Checked remainder, returning `null` when the divisor is zero.
// |    * @param {Decimal} other
// |    * @returns {Decimal | null}
// |    */
// |   checkedRem(other) {
// |     return wrap(this._0.checkedRem(other._0));
// |   }
// |
// |   /** `Display`: the normalized value. */
// |   toString() {
// |     return this._0.normalize().toString();
// |   }
// |
// |   /**
// |    * `Add`; panics on overflow.
// |    * @param {Decimal} other
// |    */
// |   add(other) {
// |     return new Decimal(this._0.add(other._0));
// |   }
// |
// |   /**
// |    * `Sub`; panics on overflow.
// |    * @param {Decimal} other
// |    */
// |   sub(other) {
// |     return new Decimal(this._0.sub(other._0));
// |   }
// |
// |   /**
// |    * `Mul`; panics on overflow.
// |    * @param {Decimal} other
// |    */
// |   mul(other) {
// |     return new Decimal(this._0.mul(other._0));
// |   }
// |
// |   /**
// |    * `Div`; panics on division by zero or overflow.
// |    * @param {Decimal} other
// |    */
// |   div(other) {
// |     return new Decimal(this._0.div(other._0));
// |   }
// |
// |   /** `Neg`. */
// |   neg() {
// |     return new Decimal(this._0.neg());
// |   }
// |
// |   /**
// |    * Derived `Ord` (numeric).
// |    * @param {Decimal} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     return this._0.cmp(other._0);
// |   }
// |
// |   /**
// |    * Derived `PartialEq` (numeric: `1.0 == 1`).
// |    * @param {unknown} other
// |    */
// |   equals(other) {
// |     return other instanceof Decimal && this._0.equals(other._0);
// |   }
// |
// |   /** Derived `Debug`: `Decimal(1.50)` (the inner value is not normalized). */
// |   fmtDebug() {
// |     return `Decimal(${this._0.fmtDebug()})`;
// |   }
// |
// |   /** Derived `Serialize`: the inner value's string form. */
// |   toJSON() {
// |     return this._0.toJSON();
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Option::map(Self)`.
// |  * @param {RustDecimal | null} inner
// |  * @returns {Decimal | null}
// |  */
// | function wrap(inner) {
// |   return inner === null ? null : new Decimal(inner);
// | }
