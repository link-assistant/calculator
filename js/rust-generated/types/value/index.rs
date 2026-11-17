// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=210529ca563ec7da7a36605d051f9b5e6892ce5622005c2b8e0cc416bfdc1cca bytes=28764

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Value type representing typed values with units (port of
// |  * `src/types/value/mod.rs`).
// |  *
// |  * The `impl Value` blocks of the submodules (`display`, `formatting`,
// |  * `physical`, `ratio`, `uncertainty`) are ported as functions taking `self`
// |  * first; `Value` exposes them as thin wrapper methods.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9cc4096c32b6299ca3bf9f404f1c7f0ffda8eb044bcd93641b168fe72dea24c6
// | import { eq, err, expect } from '../../rust/core.js';
use crate::rust::core_::{eq, err, expect};
// meta-language:translated JavaScript import_statement items=1 sha256=ce67731667957b72fc0dc16b353642398c1e6444c9bbd86c52b8a0595324a9b0
// | import { DateTime } from '../datetime.js';
use crate::types::datetime::DateTime;
// meta-language:translated JavaScript import_statement items=1 sha256=4778b9bcc4095459b258bbc82f2717cd263f17c1934003cdd9016c641b612dcf
// | import { Decimal } from '../decimal.js';
use crate::types::decimal::Decimal;
// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from '../rational.js';
use crate::types::rational::Rational;
// meta-language:translated JavaScript import_statement items=1 sha256=4a34e46992370c42b7a43e2a0a14b26893d1efc9e061cb8ea770b18223976640
// | import { Unit } from '../unit.js';
use crate::types::unit::Unit;
// meta-language:translated JavaScript import_statement items=1 sha256=dc84a5fc884ae5651240aea1131c37ec12099ffc227932c48dc71f3d49f03497
// | import { toDisplayString } from './display.js';
use crate::types::value::display::toDisplayString;
// meta-language:translated JavaScript import_statement items=1 sha256=b89626b200d3f6b38a9e7b7cc1248664df3164b8c49aa48759b5dae7c81bd19b
// | import {
// |   addCalendarMonthsOrDuration,
// |   applyDurationUnit,
// |   bareYearDatetime,
// |   convertRawDuration,
// |   divideDurationUnits,
// |   divideRawDuration,
// | } from './duration.js';
use crate::types::value::duration::{addCalendarMonthsOrDuration, applyDurationUnit, bareYearDatetime, convertRawDuration, divideDurationUnits, divideRawDuration};
// meta-language:translated JavaScript import_statement items=1 sha256=b55ee219c39ce01b3c9c7f395daaea5827cb24f5440428ea88978d13d68c3883
// | import { asNumber, asRational, isNumber, toFractionString, toRational, typeName } from './formatting.js';
use crate::types::value::formatting::{asNumber, asRational, isNumber, toFractionString, toRational, typeName};
// meta-language:translated JavaScript import_statement items=1 sha256=7de9c865599f633c8b51ec0d046b6c834ef331106bf58027dd8e22e11685211e
// | import { RatioFormat, ValueKind } from './kind.js';
use crate::types::value::kind::{RatioFormat, ValueKind};
// meta-language:translated JavaScript import_statement items=1 sha256=32ae71dd1f2486353560f408482728fc3ad92870b7afeb0c46ddc36f197b5a66
// | import {
// |   fromPhysical,
// |   metresInTimeContext,
// |   normalizeAffine,
// |   physicalBase,
// |   physicalConversion,
// |   physicalEq,
// |   physicalPower,
// |   physicalProduct,
// |   physicalRoot,
// |   physicalSum,
// |   rejectAffine,
// | } from './physical.js';
use crate::types::value::physical::{fromPhysical, metresInTimeContext, normalizeAffine, physicalBase, physicalConversion, physicalEq, physicalPower, physicalProduct, physicalRoot, physicalSum, rejectAffine};
// meta-language:translated JavaScript import_statement items=1 sha256=d3be1a8c5ad398652b149d4c445f93938f262e1725184bc4bfa2680c275ba413
// | import {
// |   additiveRatio,
// |   evaluateRatioFunction,
// |   hasRatioDisplay,
// |   isPercent,
// |   percent,
// |   ratio,
// |   scalar,
// | } from './ratio.js';
use crate::types::value::ratio::{additiveRatio, evaluateRatioFunction, hasRatioDisplay, isPercent, percent, ratio, scalar};
// meta-language:translated JavaScript import_statement items=1 sha256=8b4add7ee66e3786d76f6995ddffe9bfaebb5ab822932df984ac16b98af6b6a0
// | import { negateUncertainty, withUncertainty } from './uncertainty.js';
use crate::types::value::uncertainty::{negateUncertainty, withUncertainty};

// meta-language:carried JavaScript export_statement (unsupported)
// | export { RatioFormat, ValueKind, normalizeAffine, evaluateRatioFunction };

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../currency.js').CurrencyDatabase} CurrencyDatabase */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Converts `amount` between two currencies, using a historical rate when
// |  * `date` is given. Throws `Err(CalculatorError)`.
// |  * @param {CurrencyDatabase} db
// |  * @param {number} amount
// |  * @param {string} from
// |  * @param {string} to
// |  * @param {DateTime | null} date
// |  * @returns {number}
// |  */
// | function convertCurrency(db, amount, from, to, date) {
// |   return date !== null ? db.convertAtDate(amount, from, to, date) : db.convert(amount, from, to);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The unit chosen for a product: the left unit unless it is `None`.
// |  * @param {Value} self
// |  * @param {Value} other
// |  */
// | function productUnit(self, other) {
// |   return eq(self.unit, Unit.None) ? other.unit : self.unit;
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * Whether the pair is `(None, Custom)` or `(Custom, None)`.
// |  * @param {Unit} u1
// |  * @param {Unit} u2
// |  */
// | function noneCustomPair(u1, u2) {
// |   return (u1.tag === 'None' && u2.tag === 'Custom') || (u1.tag === 'Custom' && u2.tag === 'None');
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The numeric value as a decimal, or throws `InvalidOperation(message)`.
// |  * @param {Value} self
// |  * @param {string} message
// |  * @returns {Decimal}
// |  */
// | function requireDecimal(self, message) {
// |   const value = self.asDecimal();
// |   if (value === null) {
// |     err(CalculatorError.InvalidOperation(message));
// |   }
// |   return value;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A typed value with an optional unit. */
// | export class Value {
// |   /** @param {{ kind: ValueKind, unit: Unit }} fields */
// |   constructor({ kind, unit }) {
// |     /** The kind of value. */
// |     this.kind = kind;
// |     /** The unit of measurement. */
// |     this.unit = unit;
// |     Object.freeze(this);
// |   }
// |
// |   /** Creates a numeric value without a unit. @param {Decimal} n */
// |   static number(n) {
// |     return new Value({ kind: ValueKind.Number(n), unit: Unit.None });
// |   }
// |
// |   /** Creates a numeric value with a unit. @param {Decimal} n @param {Unit} unit */
// |   static numberWithUnit(n, unit) {
// |     return new Value({ kind: ValueKind.Number(n), unit });
// |   }
// |
// |   /** Creates a rational value without a unit. @param {Rational} r */
// |   static rational(r) {
// |     return new Value({ kind: ValueKind.Rational(r), unit: Unit.None });
// |   }
// |
// |   /** Creates a rational value with a unit. @param {Rational} r @param {Unit} unit */
// |   static rationalWithUnit(r, unit) {
// |     return new Value({ kind: ValueKind.Rational(r), unit });
// |   }
// |
// |   /** Creates a rational value from an integer (`i64`). @param {number | bigint} n */
// |   static fromInteger(n) {
// |     return Value.rational(Rational.fromInteger(BigInt(n)));
// |   }
// |
// |   /** Creates a rational value from an integer with a unit. @param {number | bigint} n @param {Unit} unit */
// |   static fromIntegerWithUnit(n, unit) {
// |     return Value.rationalWithUnit(Rational.fromInteger(BigInt(n)), unit);
// |   }
// |
// |   /** Creates a currency value. @param {Decimal} amount @param {string} currencyCode */
// |   static currency(amount, currencyCode) {
// |     return new Value({ kind: ValueKind.Number(amount), unit: Unit.currency(currencyCode) });
// |   }
// |
// |   /** Creates a datetime value. @param {DateTime} dt */
// |   static datetime(dt) {
// |     return new Value({ kind: ValueKind.DateTime(dt), unit: Unit.None });
// |   }
// |
// |   /** Creates a duration value. @param {bigint} seconds an `i64` */
// |   static duration(seconds) {
// |     return new Value({ kind: ValueKind.Duration({ seconds: BigInt(seconds) }), unit: Unit.None });
// |   }
// |
// |   /** Creates a boolean value. @param {boolean} b */
// |   static boolean(b) {
// |     return new Value({ kind: ValueKind.Boolean(b), unit: Unit.None });
// |   }
// |
// |   /**
// |    * Creates a generic comparison result value.
// |    * @param {string} left @param {string} relation @param {string} right
// |    */
// |   static comparisonResult(left, relation, right) {
// |     return new Value({ kind: ValueKind.Comparison({ left, relation, right }), unit: Unit.None });
// |   }
// |
// |   /** Creates a solved equation value. @param {string} variable @param {Rational} value */
// |   static equationSolution(variable, value) {
// |     return new Value({ kind: ValueKind.EquationSolution({ variable, value }), unit: Unit.None });
// |   }
// |
// |   /**
// |    * Creates a solved equation value with multiple exact solutions.
// |    * @param {string} variable @param {Rational[]} values
// |    */
// |   static equationSolutions(variable, values) {
// |     return new Value({ kind: ValueKind.EquationSolutions({ variable, values }), unit: Unit.None });
// |   }
// |
// |   /** Creates a solved symbolic equation value. @param {string} variable @param {string} expression */
// |   static symbolicEquationSolution(variable, expression) {
// |     return new Value({
// |       kind: ValueKind.SymbolicEquationSolution({ variable, expression }),
// |       unit: Unit.None,
// |     });
// |   }
// |
// |   /**
// |    * Adds two values. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @param {CurrencyDatabase} currencyDb @returns {Value}
// |    */
// |   add(other, currencyDb) {
// |     return this.addAtDate(other, currencyDb, null);
// |   }
// |
// |   /**
// |    * Adds two values with optional date context for historical currency
// |    * conversion. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @param {CurrencyDatabase} currencyDb @param {DateTime | null} date
// |    * @returns {Value}
// |    */
// |   addAtDate(other, currencyDb, date) {
// |     const ratioResult = additiveRatio(this, other, false);
// |     if (ratioResult !== null) {
// |       return ratioResult;
// |     }
// |     const metres = metresInTimeContext(this, other);
// |     if (metres !== null) {
// |       return metres[0].addAtDate(metres[1], currencyDb, date);
// |     }
// |     const physical = physicalSum(this, other, false);
// |     if (physical !== null) {
// |       return physical;
// |     }
// |     const a = this.kind;
// |     const b = other.kind;
// |     if (a.tag === 'Rational' && b.tag === 'Rational') {
// |       return this.addRationals(a._0, b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Number') {
// |       return this.addNumbers(a._0, b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'Rational' && b.tag === 'Number') {
// |       return this.addRationals(a._0, Rational.fromDecimal(b._0), other, currencyDb, date);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Rational') {
// |       return this.addRationals(Rational.fromDecimal(a._0), b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'DateTime' && b.tag === 'Duration') {
// |       return Value.datetime(a._0.addDuration(b.seconds));
// |     }
// |     if (a.tag === 'Duration' && b.tag === 'DateTime') {
// |       // Duration + DateTime = DateTime (commutative)
// |       return Value.datetime(b._0.addDuration(a.seconds));
// |     }
// |     if (a.tag === 'Duration' && b.tag === 'Duration') {
// |       return Value.duration(/** @type {bigint} */ (a.seconds) + /** @type {bigint} */ (b.seconds));
// |     }
// |     // DateTime + number-with-duration-unit (e.g. "now + 10 days")
// |     if (a.tag === 'DateTime' && (b.tag === 'Rational' || b.tag === 'Number') && other.unit.tag === 'Duration') {
// |       return Value.datetime(addCalendarMonthsOrDuration(a._0, other.unit._0, b._0.toF64()));
// |     }
// |     // number-with-duration-unit + DateTime (commutative, e.g. "10 days + now")
// |     if ((a.tag === 'Rational' || a.tag === 'Number') && b.tag === 'DateTime' && this.unit.tag === 'Duration') {
// |       return Value.datetime(addCalendarMonthsOrDuration(b._0, this.unit._0, a._0.toF64()));
// |     }
// |     return err(CalculatorError.InvalidOperation(`Cannot add ${this.typeName()} and ${other.typeName()}`));
// |   }
// |
// |   /**
// |    * @param {Rational} a @param {Rational} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date @returns {Value}
// |    */
// |   addRationals(a, b, other, currencyDb, date) {
// |     return this.combineRationals(a, b, other, currencyDb, date, false);
// |   }
// |
// |   /**
// |    * @param {Decimal} a @param {Decimal} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date @returns {Value}
// |    */
// |   addNumbers(a, b, other, currencyDb, date) {
// |     return this.combineNumbers(a, b, other, currencyDb, date, false);
// |   }
// |
// |   /**
// |    * Shared body of `add_rationals` / `subtract_rationals` (the match arms are
// |    * identical apart from the operator and the operation name).
// |    * @param {Rational} a @param {Rational} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date
// |    * @param {boolean} subtract @returns {Value}
// |    */
// |   combineRationals(a, b, other, currencyDb, date, subtract) {
// |     const operation = subtract ? 'subtract' : 'add';
// |     const apply = () => (subtract ? a.sub(b) : a.add(b));
// |     const u1 = this.unit;
// |     const u2 = other.unit;
// |     if (u1.tag === 'None' && u2.tag === 'None') {
// |       return Value.rational(apply());
// |     }
// |     if (noneCustomPair(u1, u2)) {
// |       return err(CalculatorError.unitMismatch(operation, u1.displayName(), u2.displayName()));
// |     }
// |     if (u2.tag === 'None' || u1.tag === 'None') {
// |       return Value.rationalWithUnit(apply(), u2.tag === 'None' ? u1 : u2);
// |     }
// |     if (u1.tag === 'Currency' && u2.tag === 'Currency') {
// |       // Same currency
// |       if (u1._0 === u2._0) {
// |         return Value.rationalWithUnit(apply(), u1);
// |       }
// |       // Different currencies - need conversion (uses Decimal for approximation)
// |       const aDec = a.toDecimal();
// |       const bDec = b.toDecimal();
// |       const converted = Decimal.fromF64(convertCurrency(currencyDb, bDec.toF64(), u2._0, u1._0, date));
// |       return Value.currency(subtract ? aDec.sub(converted) : aDec.add(converted), u1._0);
// |     }
// |     // Mass +/- different mass unit (convert to first unit's type)
// |     if (u1.tag === 'Mass' && u2.tag === 'Mass' && !eq(u1._0, u2._0)) {
// |       const aVal = a.toF64();
// |       const bConverted = u2._0.convert(b.toF64(), u1._0);
// |       const result = Decimal.fromF64(subtract ? aVal - bConverted : aVal + bConverted);
// |       return Value.numberWithUnit(result, Unit.Mass(u1._0));
// |     }
// |     if (eq(u1, u2)) {
// |       return Value.rationalWithUnit(apply(), u1);
// |     }
// |     return err(CalculatorError.unitMismatch(operation, u1.displayName(), u2.displayName()));
// |   }
// |
// |   /**
// |    * Shared body of `add_numbers` / `subtract_numbers`.
// |    * @param {Decimal} a @param {Decimal} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date
// |    * @param {boolean} subtract @returns {Value}
// |    */
// |   combineNumbers(a, b, other, currencyDb, date, subtract) {
// |     const operation = subtract ? 'subtract' : 'add';
// |     const apply = (/** @type {Decimal} */ x, /** @type {Decimal} */ y) => (subtract ? x.sub(y) : x.add(y));
// |     const u1 = this.unit;
// |     const u2 = other.unit;
// |     if (u1.tag === 'None' && u2.tag === 'None') {
// |       return Value.number(apply(a, b));
// |     }
// |     if (noneCustomPair(u1, u2)) {
// |       return err(CalculatorError.unitMismatch(operation, u1.displayName(), u2.displayName()));
// |     }
// |     if (u2.tag === 'None' || u1.tag === 'None') {
// |       return Value.numberWithUnit(apply(a, b), u2.tag === 'None' ? u1 : u2);
// |     }
// |     if (u1.tag === 'Currency' && u2.tag === 'Currency') {
// |       if (u1._0 === u2._0) {
// |         return Value.currency(apply(a, b), u1._0);
// |       }
// |       // Convert c2 to c1, using historical rate if date is provided
// |       const converted = Decimal.fromF64(convertCurrency(currencyDb, b.toF64(), u2._0, u1._0, date));
// |       return Value.currency(apply(a, converted), u1._0);
// |     }
// |     if (eq(u1, u2)) {
// |       return Value.numberWithUnit(apply(a, b), u1);
// |     }
// |     return err(CalculatorError.unitMismatch(operation, u1.displayName(), u2.displayName()));
// |   }
// |
// |   /**
// |    * Subtracts two values. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @param {CurrencyDatabase} currencyDb @returns {Value}
// |    */
// |   subtract(other, currencyDb) {
// |     return this.subtractAtDate(other, currencyDb, null);
// |   }
// |
// |   /**
// |    * Subtracts two values with optional date context for historical currency
// |    * conversion. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @param {CurrencyDatabase} currencyDb @param {DateTime | null} date
// |    * @returns {Value}
// |    */
// |   subtractAtDate(other, currencyDb, date) {
// |     if (this.kind.tag === 'DateTime') {
// |       const year = bareYearDatetime(other);
// |       if (year !== null) {
// |         return Value.duration(this.kind._0.signedSubtractSeconds(year));
// |       }
// |     }
// |     if (other.kind.tag === 'DateTime') {
// |       const year = bareYearDatetime(this);
// |       if (year !== null) {
// |         return Value.duration(year.signedSubtractSeconds(other.kind._0));
// |       }
// |     }
// |
// |     const ratioResult = additiveRatio(this, other, true);
// |     if (ratioResult !== null) {
// |       return ratioResult;
// |     }
// |     const metres = metresInTimeContext(this, other);
// |     if (metres !== null) {
// |       return metres[0].subtractAtDate(metres[1], currencyDb, date);
// |     }
// |     const physical = physicalSum(this, other, true);
// |     if (physical !== null) {
// |       return physical;
// |     }
// |     const a = this.kind;
// |     const b = other.kind;
// |     if (a.tag === 'Rational' && b.tag === 'Rational') {
// |       return this.subtractRationals(a._0, b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Number') {
// |       return this.subtractNumbers(a._0, b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'Rational' && b.tag === 'Number') {
// |       return this.subtractRationals(a._0, Rational.fromDecimal(b._0), other, currencyDb, date);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Rational') {
// |       return this.subtractRationals(Rational.fromDecimal(a._0), b._0, other, currencyDb, date);
// |     }
// |     if (a.tag === 'DateTime' && b.tag === 'DateTime') {
// |       // Signed difference (dt1 - dt2): a negative result (dt1 earlier than
// |       // dt2) is preserved instead of collapsing to zero.
// |       return Value.duration(a._0.signedSubtractSeconds(b._0));
// |     }
// |     if (a.tag === 'DateTime' && b.tag === 'Duration') {
// |       return Value.datetime(a._0.addDuration(-(/** @type {bigint} */ (b.seconds))));
// |     }
// |     if (a.tag === 'Duration' && b.tag === 'Duration') {
// |       return Value.duration(/** @type {bigint} */ (a.seconds) - /** @type {bigint} */ (b.seconds));
// |     }
// |     // DateTime - number-with-duration-unit (e.g. "now - 10 days")
// |     if (a.tag === 'DateTime' && (b.tag === 'Rational' || b.tag === 'Number') && other.unit.tag === 'Duration') {
// |       return Value.datetime(addCalendarMonthsOrDuration(a._0, other.unit._0, -b._0.toF64()));
// |     }
// |     return err(
// |       CalculatorError.InvalidOperation(`Cannot subtract ${other.typeName()} from ${this.typeName()}`),
// |     );
// |   }
// |
// |   /**
// |    * @param {Rational} a @param {Rational} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date @returns {Value}
// |    */
// |   subtractRationals(a, b, other, currencyDb, date) {
// |     return this.combineRationals(a, b, other, currencyDb, date, true);
// |   }
// |
// |   /**
// |    * @param {Decimal} a @param {Decimal} b @param {Value} other
// |    * @param {CurrencyDatabase} currencyDb @param {DateTime | null} date @returns {Value}
// |    */
// |   subtractNumbers(a, b, other, currencyDb, date) {
// |     return this.combineNumbers(a, b, other, currencyDb, date, true);
// |   }
// |
// |   /**
// |    * Multiplies two values. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @returns {Value}
// |    */
// |   multiply(other) {
// |     if (this.hasRatioDisplay() || other.hasRatioDisplay()) {
// |       const result = this.scalar().multiply(other.scalar());
// |       if (this.isPercent() && other.isPercent()) {
// |         return Value.percent(expect(result.toRational(), 'numeric product'));
// |       }
// |       return result;
// |     }
// |     const physical = physicalProduct(this, other, false);
// |     if (physical !== null) {
// |       return physical;
// |     }
// |     if (
// |       this.unit.tag !== 'None' &&
// |       other.unit.tag !== 'None' &&
// |       (this.unit.isOpaqueCompound() || other.unit.isOpaqueCompound())
// |     ) {
// |       return err(CalculatorError.domain('compound unit multiplication requires a unitless factor'));
// |     }
// |     const a = this.kind;
// |     const b = other.kind;
// |     const unit = productUnit(this, other);
// |     if (a.tag === 'Rational' && b.tag === 'Rational') {
// |       return Value.rationalWithUnit(a._0.mul(b._0), unit);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Number') {
// |       return Value.numberWithUnit(a._0.mul(b._0), unit);
// |     }
// |     if (a.tag === 'Rational' && b.tag === 'Number') {
// |       return Value.rationalWithUnit(a._0.mul(Rational.fromDecimal(b._0)), unit);
// |     }
// |     if (a.tag === 'Number' && b.tag === 'Rational') {
// |       return Value.rationalWithUnit(Rational.fromDecimal(a._0).mul(b._0), unit);
// |     }
// |     return err(
// |       CalculatorError.InvalidOperation(`Cannot multiply ${this.typeName()} and ${other.typeName()}`),
// |     );
// |   }
// |
// |   /**
// |    * Divides two values. Throws `Err(CalculatorError)`.
// |    * @param {Value} other @returns {Value}
// |    */
// |   divide(other) {
// |     if (this.hasRatioDisplay() || other.hasRatioDisplay()) {
// |       return this.scalar().divide(other.scalar());
// |     }
// |     const physical = physicalProduct(this, other, true);
// |     if (physical !== null) {
// |       return physical;
// |     }
// |     if (
// |       other.unit.tag !== 'None' &&
// |       !eq(this.unit, other.unit) &&
// |       (this.unit.isOpaqueCompound() || other.unit.isOpaqueCompound())
// |     ) {
// |       return err(
// |         CalculatorError.domain('compound unit division requires matching units or a unitless divisor'),
// |       );
// |     }
// |     const a = this.kind;
// |     const b = other.kind;
// |     const numeric = (/** @type {ValueKind} */ k) => k.tag === 'Rational' || k.tag === 'Number';
// |     if (numeric(a) && numeric(b)) {
// |       if (b._0.isZero()) {
// |         return err(CalculatorError.DivisionByZero);
// |       }
// |       const durations = divideDurationUnits(this, other);
// |       if (durations !== null) {
// |         return durations;
// |       }
// |       // Handle unit division
// |       const unit = Value.divisionResultUnit(this.unit, other.unit);
// |       if (a.tag === 'Number' && b.tag === 'Number') {
// |         const result = a._0.checkedDiv(b._0);
// |         if (result === null) {
// |           return err(CalculatorError.Overflow);
// |         }
// |         return Value.numberWithUnit(result, unit);
// |       }
// |       const left = a.tag === 'Rational' ? a._0 : Rational.fromDecimal(a._0);
// |       const right = b.tag === 'Rational' ? b._0 : Rational.fromDecimal(b._0);
// |       return Value.rationalWithUnit(left.div(right), unit);
// |     }
// |     if (a.tag === 'Duration' && numeric(b)) {
// |       return divideRawDuration(a.seconds, other);
// |     }
// |     return err(CalculatorError.InvalidOperation(`Cannot divide ${this.typeName()} by ${other.typeName()}`));
// |   }
// |
// |   /** @param {Unit} left @param {Unit} right @returns {Unit} */
// |   static divisionResultUnit(left, right) {
// |     if (left.tag === 'Currency' && right.tag === 'Currency' && left._0 === right._0) {
// |       return Unit.None;
// |     }
// |     if (left.tag === 'Duration' && right.tag === 'Currency') {
// |       return Unit.None;
// |     }
// |     if (right.tag === 'None') {
// |       return left;
// |     }
// |     if (left.tag === 'None') {
// |       return Unit.None;
// |     }
// |     if (eq(left, right)) {
// |       return Unit.None;
// |     }
// |     return left;
// |   }
// |
// |   /**
// |    * Computes the signed remainder of two unitless numbers. Throws
// |    * `Err(CalculatorError)`.
// |    * @param {Value} other @returns {Value}
// |    */
// |   modulo(other) {
// |     if (this.unit.tag !== 'None' || other.unit.tag !== 'None') {
// |       return err(
// |         CalculatorError.InvalidOperation(
// |           `Cannot apply modulo to ${this.toDisplayString()} and ${other.toDisplayString()}; modulo requires unitless numbers`,
// |         ),
// |       );
// |     }
// |     const left = this.toRational();
// |     if (left === null) {
// |       return err(CalculatorError.InvalidOperation('modulo operands must be numeric'));
// |     }
// |     const right = other.toRational();
// |     if (right === null) {
// |       return err(CalculatorError.InvalidOperation('modulo operands must be numeric'));
// |     }
// |     const result = left.remainder(right);
// |     if (result === null) {
// |       return err(CalculatorError.DivisionByZero);
// |     }
// |     return Value.rational(result);
// |   }
// |
// |   /**
// |    * Converts this value to the given unit. Throws `Err(CalculatorError)`.
// |    * @param {Unit} targetUnit @param {CurrencyDatabase} currencyDb @returns {Value}
// |    */
// |   convertToUnit(targetUnit, currencyDb) {
// |     return this.convertToUnitAtDate(targetUnit, currencyDb, null);
// |   }
// |
// |   /**
// |    * Converts this value to the given unit, using a historical exchange rate
// |    * if `date` is provided. Throws `Err(CalculatorError)`.
// |    * @param {Unit} targetUnit @param {CurrencyDatabase} currencyDb @param {DateTime | null} date
// |    * @returns {Value}
// |    */
// |   convertToUnitAtDate(targetUnit, currencyDb, date) {
// |     if (this.kind.tag === 'Duration') {
// |       return convertRawDuration(this.kind.seconds, targetUnit);
// |     }
// |     if (targetUnit.tag !== 'None') {
// |       const physical = physicalConversion(this, targetUnit);
// |       if (physical !== null) {
// |         return physical;
// |       }
// |     }
// |     const from = this.unit;
// |     const to = targetUnit;
// |     if (to.tag === 'None') {
// |       const value = this.toRational();
// |       if (value === null) {
// |         return err(CalculatorError.InvalidOperation('number conversion requires a numeric value'));
// |       }
// |       return Value.rational(value);
// |     }
// |     // Data size to data size conversion
// |     if (from.tag === 'DataSize' && to.tag === 'DataSize') {
// |       const value = requireDecimal(this, 'data size conversion requires a numeric value');
// |       return Value.numberWithUnit(Decimal.fromF64(from._0.convert(value.toF64(), to._0)), Unit.DataSize(to._0));
// |     }
// |     // Currency to currency conversion
// |     if (from.tag === 'Currency' && to.tag === 'Currency') {
// |       const amount = requireDecimal(this, 'currency conversion requires a numeric value');
// |       const converted = convertCurrency(currencyDb, amount.toF64(), from._0, to._0, date);
// |       return Value.currency(Decimal.fromF64(converted), to._0);
// |     }
// |     // Mass to mass conversion
// |     if (from.tag === 'Mass' && to.tag === 'Mass') {
// |       const value = requireDecimal(this, 'mass conversion requires a numeric value');
// |       return Value.numberWithUnit(Decimal.fromF64(from._0.convert(value.toF64(), to._0)), Unit.Mass(to._0));
// |     }
// |     // Duration to duration conversion (e.g., "300000 ms in seconds")
// |     if (from.tag === 'Duration' && to.tag === 'Duration') {
// |       const value = requireDecimal(this, 'duration conversion requires a numeric value');
// |       const secs = from._0.toSecs(value.toF64());
// |       return Value.numberWithUnit(Decimal.fromF64(to._0.secsToUnit(secs)), Unit.Duration(to._0));
// |     }
// |     // Dimensionless value: just apply the target unit (e.g. "5 as MB", "5 as kg")
// |     if (from.tag === 'None' && (to.tag === 'DataSize' || to.tag === 'Mass')) {
// |       const value = requireDecimal(this, 'unit conversion requires a numeric value');
// |       return Value.numberWithUnit(value, targetUnit);
// |     }
// |     if (from.tag === 'None' && to.tag === 'Duration') {
// |       return applyDurationUnit(this, to._0);
// |     }
// |     // DateTime timezone conversion (e.g., "6 PM GMT as MSK")
// |     if (to.tag === 'Timezone') {
// |       const tzAbbrev = to._0;
// |       if (this.kind.tag === 'DateTime') {
// |         const targetOffset = DateTime.parseTzAbbreviation(tzAbbrev);
// |         if (targetOffset === null) {
// |           return err(CalculatorError.parse(`Unknown timezone: ${tzAbbrev}`));
// |         }
// |         return Value.datetime(this.kind._0.withTimezoneOffset(targetOffset, tzAbbrev));
// |       }
// |       return err(
// |         CalculatorError.InvalidOperation(
// |           `Cannot convert ${this.toDisplayString()} to timezone ${tzAbbrev}; only DateTime values can be converted to a timezone`,
// |         ),
// |       );
// |     }
// |     return err(CalculatorError.InvalidOperation(`Cannot convert ${from.displayName()} to ${to.displayName()}`));
// |   }
// |
// |   /** Negates the value. @returns {Value} */
// |   negate() {
// |     const kind = this.kind;
// |     switch (kind.tag) {
// |       case 'Number':
// |         return Value.numberWithUnit(kind._0.neg(), this.unit);
// |       case 'Rational':
// |         return Value.rationalWithUnit(kind._0.neg(), this.unit);
// |       case 'Percent':
// |         return Value.percent(kind._0.neg());
// |       case 'Ratio':
// |         return Value.ratio(kind.value.neg(), kind.format);
// |       case 'Duration':
// |         return Value.duration(-(/** @type {bigint} */ (kind.seconds)));
// |       case 'Uncertainty':
// |         return negateUncertainty(this);
// |       default:
// |         return this;
// |     }
// |   }
// |
// |   // --- display.rs / formatting.rs ---
// |
// |   /** @returns {string} */
// |   toDisplayString() {
// |     return toDisplayString(this);
// |   }
// |
// |   /** @returns {string} */
// |   typeName() {
// |     return typeName(this);
// |   }
// |
// |   /** @returns {boolean} */
// |   isNumber() {
// |     return isNumber(this);
// |   }
// |
// |   /** @returns {Decimal | null} */
// |   asNumber() {
// |     return asNumber(this);
// |   }
// |
// |   /** Alias for `asNumber`. @returns {Decimal | null} */
// |   asDecimal() {
// |     return asNumber(this);
// |   }
// |
// |   /** @returns {Rational | null} */
// |   asRational() {
// |     return asRational(this);
// |   }
// |
// |   /** @returns {Rational | null} */
// |   toRational() {
// |     return toRational(this);
// |   }
// |
// |   /** @returns {string | null} */
// |   toFractionString() {
// |     return toFractionString(this);
// |   }
// |
// |   // --- ratio.rs ---
// |
// |   /** @param {Rational} fraction @returns {Value} */
// |   static percent(fraction) {
// |     return percent(fraction);
// |   }
// |
// |   /** @param {Rational} value @param {RatioFormat} format @returns {Value} */
// |   static ratio(value, format) {
// |     return ratio(value, format);
// |   }
// |
// |   /** @returns {boolean} */
// |   isPercent() {
// |     return isPercent(this);
// |   }
// |
// |   /** @returns {boolean} */
// |   hasRatioDisplay() {
// |     return hasRatioDisplay(this);
// |   }
// |
// |   /** @returns {Value} */
// |   scalar() {
// |     return scalar(this);
// |   }
// |
// |   // --- uncertainty.rs ---
// |
// |   /** Throws `Err(CalculatorError)`. @param {Value} error @returns {Value} */
// |   withUncertainty(error) {
// |     return withUncertainty(this, error);
// |   }
// |
// |   // --- physical.rs ---
// |
// |   /** @returns {[Rational, import('../dimension.js').Dimension] | null} */
// |   physicalBase() {
// |     return physicalBase(this);
// |   }
// |
// |   /** @param {Rational} amount @param {import('../physical.js').PhysicalUnit} unit @returns {Value} */
// |   static fromPhysical(amount, unit) {
// |     return fromPhysical(amount, unit);
// |   }
// |
// |   /** Throws `Err(CalculatorError)`. @param {string} operation */
// |   rejectAffine(operation) {
// |     rejectAffine(this, operation);
// |   }
// |
// |   /** `null` for `None`; throws for `Some(Err)`. @param {number} power @returns {Value | null} */
// |   physicalPower(power) {
// |     return physicalPower(this, power);
// |   }
// |
// |   /** `null` for `None`; throws for `Some(Err)`. @param {number} n @returns {Value | null} */
// |   physicalRoot(n) {
// |     return physicalRoot(this, n);
// |   }
// |
// |   /** `impl Display`. */
// |   toString() {
// |     return this.toDisplayString();
// |   }
// |
// |   /**
// |    * `impl PartialEq`.
// |    * @param {Value} other
// |    * @returns {boolean}
// |    */
// |   equals(other) {
// |     if (this.hasRatioDisplay() || other.hasRatioDisplay()) {
// |       const a = this.toRational();
// |       const b = other.toRational();
// |       return eq(this.unit, other.unit) && a !== null && b !== null && a.equals(b);
// |     }
// |     const physical = physicalEq(this, other);
// |     if (physical !== null) {
// |       return physical;
// |     }
// |     const a = this.kind;
// |     const b = other.kind;
// |     if ((a.tag === 'Number' && b.tag === 'Number') || (a.tag === 'Rational' && b.tag === 'Rational')) {
// |       return a._0.equals(b._0) && eq(this.unit, other.unit);
// |     }
// |     // Cross-compare: convert to f64 for approximate equality
// |     if ((a.tag === 'Number' && b.tag === 'Rational') || (a.tag === 'Rational' && b.tag === 'Number')) {
// |       return Math.abs(a._0.toF64() - b._0.toF64()) < 1e-10 && eq(this.unit, other.unit);
// |     }
// |     return eq(a, b) && eq(this.unit, other.unit);
// |   }
// |
// |   /** serde: `{ kind, unit }`. */
// |   toJSON() {
// |     return { kind: this.kind, unit: this.unit };
// |   }
// | }
