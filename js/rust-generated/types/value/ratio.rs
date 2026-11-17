// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a5f880c18703ad689925f879d258e5a406b17c401186b421ee753e3777d3cc8a bytes=4363

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Exact percentage arithmetic and dimensionless ratio presentation (port of
// |  * `src/types/value/ratio.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9cc4096c32b6299ca3bf9f404f1c7f0ffda8eb044bcd93641b168fe72dea24c6
// | import { eq, err, expect } from '../../rust/core.js';
use crate::rust::core_::{eq, err, expect};
// meta-language:translated JavaScript import_statement items=1 sha256=7d876b93ab16bdb22806a41f56ba9e12b2b3713500573429ade82cef99675d19
// | import { toAsciiLowercase } from '../../rust/str.js';
use crate::rust::str_::toAsciiLowercase;
// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from '../rational.js';
use crate::types::rational::Rational;
// meta-language:translated JavaScript import_statement items=1 sha256=4a34e46992370c42b7a43e2a0a14b26893d1efc9e061cb8ea770b18223976640
// | import { Unit } from '../unit.js';
use crate::types::unit::Unit;
// meta-language:translated JavaScript import_statement items=1 sha256=7de9c865599f633c8b51ec0d046b6c834ef331106bf58027dd8e22e11685211e
// | import { RatioFormat, ValueKind } from './kind.js';
use crate::types::value::kind::{RatioFormat, ValueKind};
// meta-language:translated JavaScript import_statement items=1 sha256=6227932a669ff41f119a2566a3f97623e5311d86e6e144a0c641e05447aa9418
// | import { Value } from './index.js';
use crate::types::value::index::Value;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::percent`: creates a percentage from its exact fraction (`1/10`
// |  * displays `10%`).
// |  * @param {Rational} fraction
// |  * @returns {Value}
// |  */
// | export function percent(fraction) {
// |   return new Value({ kind: ValueKind.Percent(fraction), unit: Unit.None });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::ratio`.
// |  * @param {Rational} value
// |  * @param {RatioFormat} format
// |  * @returns {Value}
// |  */
// | export function ratio(value, format) {
// |   return new Value({ kind: ValueKind.Ratio({ value, format }), unit: Unit.None });
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `Value::is_percent`.
// |  * @param {Value} self
// |  */
// | export function isPercent(self) {
// |   return self.kind.tag === 'Percent';
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `Value::has_ratio_display`.
// |  * @param {Value} self
// |  */
// | export function hasRatioDisplay(self) {
// |   return self.kind.tag === 'Percent' || self.kind.tag === 'Ratio';
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::scalar`.
// |  * @param {Value} self
// |  * @returns {Value}
// |  */
// | export function scalar(self) {
// |   if (hasRatioDisplay(self)) {
// |     return Value.rationalWithUnit(expect(self.toRational(), 'numeric ratio'), self.unit);
// |   }
// |   return self;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::additive_ratio`: `null` for `None`; the `Some(Err)` case throws
// |  * `Err(CalculatorError)`.
// |  * @param {Value} self
// |  * @param {Value} other
// |  * @param {boolean} subtract
// |  * @returns {Value | null}
// |  */
// | export function additiveRatio(self, other, subtract) {
// |   if (!hasRatioDisplay(self) && !hasRatioDisplay(other)) {
// |     return null;
// |   }
// |   const left = self.toRational();
// |   if (left === null) {
// |     err(CalculatorError.InvalidOperation('percentage arithmetic requires numeric operands'));
// |   }
// |   let right = other.toRational();
// |   if (right === null) {
// |     err(CalculatorError.InvalidOperation('percentage arithmetic requires numeric operands'));
// |   }
// |   right = subtract ? right.neg() : right;
// |   if (isPercent(self)) {
// |     if (self.unit.tag !== 'None' || other.unit.tag !== 'None') {
// |       err(CalculatorError.InvalidOperation('cannot add a unit to a percentage'));
// |     }
// |     return percent(left.add(right));
// |   } else if (isPercent(other)) {
// |     return Value.rationalWithUnit(left.mul(Rational.one().add(right)), self.unit);
// |   } else if (eq(self.unit, other.unit)) {
// |     return Value.rationalWithUnit(left.add(right), self.unit);
// |   }
// |   return err(CalculatorError.InvalidOperation('ratio arithmetic requires compatible units'));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Evaluates ratio constructors introduced by the percentage grammar.
// |  * Returning `None` (`null`) lets ordinary mathematical functions use their
// |  * evaluator; the `Some(Err)` case throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {Value[]} args
// |  * @returns {Value | null}
// |  */
// | export function evaluateRatioFunction(name, args) {
// |   name = toAsciiLowercase(name);
// |   if (
// |     !(
// |       name === 'percent' ||
// |       name === 'as_percent' ||
// |       name === 'as_fraction' ||
// |       name === 'as_reciprocal' ||
// |       name === 'as_multiplier' ||
// |       name === 'as_decimal' ||
// |       name === 'percent_of'
// |     )
// |   ) {
// |     return null;
// |   }
// |   if (args.length !== 1) {
// |     err(CalculatorError.invalidArgs(name, 'expected one numeric argument'));
// |   }
// |   const value = args[0];
// |   if (name === 'percent_of') {
// |     return value;
// |   }
// |   if (value.unit.tag !== 'None') {
// |     err(CalculatorError.InvalidOperation('ratio conversion requires a unitless number'));
// |   }
// |   const fraction = value.toRational();
// |   if (fraction === null) {
// |     err(CalculatorError.invalidArgs(name, 'expected numeric argument'));
// |   }
// |   switch (name) {
// |     case 'percent':
// |       return percent(fraction.div(Rational.fromInteger(100)));
// |     case 'as_percent':
// |       return percent(fraction);
// |     case 'as_fraction':
// |       return ratio(fraction, RatioFormat.Fraction);
// |     case 'as_multiplier':
// |       return ratio(fraction, RatioFormat.Multiplier);
// |     case 'as_reciprocal':
// |       if (fraction.isZero()) {
// |         err(CalculatorError.DivisionByZero);
// |       }
// |       return ratio(fraction, RatioFormat.Reciprocal);
// |     default:
// |       return Value.rational(fraction);
// |   }
// | }
