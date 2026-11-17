// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=94baea8fa94c2c29896caac2ffb3d97b0392efb1886179c9d5276ba6445daa2e bytes=7150

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Duration helpers for values (port of `src/types/value/duration.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=b5be1ffd70c72a07a837297791d3424259ea07746c3d48b8e2faa22f728c49db
// | import { err, ok, unreachable } from '../../rust/core.js';
use crate::rust::core_::{err, ok, unreachable};
// meta-language:translated JavaScript import_statement items=1 sha256=6627c5cdac057c9a14a01cfe95c160f0147e12a670abba809401fffb730c438c
// | import { I32, I64 } from '../../rust/num.js';
use crate::rust::num::{I32, I64};
// meta-language:translated JavaScript import_statement items=1 sha256=ce67731667957b72fc0dc16b353642398c1e6444c9bbd86c52b8a0595324a9b0
// | import { DateTime } from '../datetime.js';
use crate::types::datetime::DateTime;
// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from '../rational.js';
use crate::types::rational::Rational;
// meta-language:translated JavaScript import_statement items=1 sha256=0b44f06c1611ca180475d1d99fd24ad3aa0f067385f27ec2b12518c44d886bab
// | import { DurationUnit, Unit } from '../unit.js';
use crate::types::unit::{DurationUnit, Unit};
// meta-language:translated JavaScript import_statement items=1 sha256=6227932a669ff41f119a2566a3f97623e5311d86e6e144a0c641e05447aa9418
// | import { Value } from './index.js';
use crate::types::value::index::Value;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Interprets an unadorned four-digit integer as January 1 of that year.
// |  * @param {Value} value
// |  * @returns {DateTime | null}
// |  */
// | export function bareYearDatetime(value) {
// |   if (value.unit.tag !== 'None') {
// |     return null;
// |   }
// |
// |   /** @type {number} */
// |   let year;
// |   const kind = value.kind;
// |   if (kind.tag === 'Rational' && kind._0.isInteger()) {
// |     const parsed = I32.tryFrom(kind._0.numer());
// |     if (parsed === null) {
// |       return null;
// |     }
// |     year = Number(parsed);
// |   } else if (kind.tag === 'Number') {
// |     const parsed = ok(() => I32.parse(kind._0.toString()));
// |     if (parsed === null) {
// |       return null;
// |     }
// |     year = Number(parsed);
// |   } else {
// |     return null;
// |   }
// |
// |   return year >= 1000 && year <= 9999 ? DateTime.fromYear(year) : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Formats a duration in seconds to a human-readable string.
// |  * @param {bigint} totalSeconds an `i64`
// |  * @returns {string}
// |  */
// | export function formatDuration(totalSeconds) {
// |   const isNegative = totalSeconds < 0n;
// |   totalSeconds = isNegative ? -totalSeconds : totalSeconds;
// |
// |   const days = totalSeconds / 86400n;
// |   const hours = (totalSeconds % 86400n) / 3600n;
// |   const minutes = (totalSeconds % 3600n) / 60n;
// |   const seconds = totalSeconds % 60n;
// |
// |   /** @type {string[]} */
// |   const parts = [];
// |
// |   if (days > 0n) {
// |     parts.push(`${days} day${days === 1n ? '' : 's'}`);
// |   }
// |   if (hours > 0n) {
// |     parts.push(`${hours} hour${hours === 1n ? '' : 's'}`);
// |   }
// |   if (minutes > 0n) {
// |     parts.push(`${minutes} minute${minutes === 1n ? '' : 's'}`);
// |   }
// |   if (seconds > 0n || parts.length === 0) {
// |     parts.push(`${seconds} second${seconds === 1n ? '' : 's'}`);
// |   }
// |
// |   const result = parts.join(', ');
// |   return isNegative ? `-${result}` : result;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Divides compatible duration-unit values as a unitless ratio.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {Value} left
// |  * @param {Value} right
// |  * @returns {Value | null}
// |  */
// | export function divideDurationUnits(left, right) {
// |   if (left.unit.tag !== 'Duration' || right.unit.tag !== 'Duration') {
// |     return null;
// |   }
// |   const leftUnit = left.unit._0;
// |   const rightUnit = right.unit._0;
// |
// |   const leftAmount = left.toRational();
// |   if (leftAmount === null) {
// |     err(CalculatorError.InvalidOperation('duration division requires numeric values'));
// |   }
// |   const rightAmount = right.toRational();
// |   if (rightAmount === null) {
// |     err(CalculatorError.InvalidOperation('duration division requires numeric values'));
// |   }
// |
// |   const leftSeconds = leftAmount.mul(durationUnitSeconds(leftUnit));
// |   const rightSeconds = rightAmount.mul(durationUnitSeconds(rightUnit));
// |   if (rightSeconds.isZero()) {
// |     err(CalculatorError.DivisionByZero);
// |   }
// |
// |   return Value.rational(leftSeconds.div(rightSeconds));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Converts a raw duration in seconds into a numeric amount in `unit`.
// |  * @param {bigint} seconds
// |  * @param {DurationUnit} unit
// |  * @returns {Rational}
// |  */
// | export function durationSecondsToUnit(seconds, unit) {
// |   return Rational.fromInteger(seconds).div(durationUnitSeconds(unit));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Converts a raw duration in seconds into its numeric day count.
// |  * @param {bigint} seconds
// |  * @returns {Rational}
// |  */
// | export function durationSecondsToDays(seconds) {
// |   return durationSecondsToUnit(seconds, DurationUnit.Days);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Throws `Err(CalculatorError)`.
// |  * @param {bigint} seconds
// |  * @param {Value} divisor
// |  * @returns {Value}
// |  */
// | export function divideRawDuration(seconds, divisor) {
// |   const divisorAmount = divisor.toRational();
// |   if (divisorAmount === null) {
// |     err(CalculatorError.InvalidOperation('duration division requires a numeric divisor'));
// |   }
// |   if (divisorAmount.isZero()) {
// |     err(CalculatorError.DivisionByZero);
// |   }
// |
// |   switch (divisor.unit.tag) {
// |     case 'None':
// |       return Value.rational(durationSecondsToDays(seconds).div(divisorAmount));
// |     case 'Duration': {
// |       const divisorSeconds = divisorAmount.mul(durationUnitSeconds(divisor.unit._0));
// |       if (divisorSeconds.isZero()) {
// |         err(CalculatorError.DivisionByZero);
// |       }
// |       return Value.rational(Rational.fromInteger(seconds).div(divisorSeconds));
// |     }
// |     case 'Currency':
// |       return Value.rational(durationSecondsToDays(seconds).div(divisorAmount));
// |     default:
// |       return err(
// |         CalculatorError.InvalidOperation(`Cannot divide duration by ${divisor.unit.displayName()}`),
// |       );
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Throws `Err(CalculatorError)`.
// |  * @param {bigint} seconds
// |  * @param {Unit} targetUnit
// |  * @returns {Value}
// |  */
// | export function convertRawDuration(seconds, targetUnit) {
// |   switch (targetUnit.tag) {
// |     case 'None':
// |       return Value.rational(durationSecondsToDays(seconds));
// |     case 'Duration':
// |       return Value.rationalWithUnit(
// |         durationSecondsToUnit(seconds, targetUnit._0),
// |         Unit.Duration(targetUnit._0),
// |       );
// |     default:
// |       return err(
// |         CalculatorError.InvalidOperation(`Cannot convert duration to ${targetUnit.displayName()}`),
// |       );
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Throws `Err(CalculatorError)`.
// |  * @param {Value} value
// |  * @param {DurationUnit} unit
// |  * @returns {Value}
// |  */
// | export function applyDurationUnit(value, unit) {
// |   const amount = value.toRational();
// |   if (amount === null) {
// |     err(CalculatorError.InvalidOperation('duration unit conversion requires a numeric value'));
// |   }
// |   return Value.rationalWithUnit(amount, Unit.Duration(unit));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Applies a signed duration to a `DateTime`, using calendar arithmetic for
// |  * months and years and second-based arithmetic for all other units.
// |  *
// |  * `amount` is positive for addition and negative for subtraction.
// |  * @param {DateTime} dt
// |  * @param {DurationUnit} unit
// |  * @param {number} amount
// |  * @returns {DateTime}
// |  */
// | export function addCalendarMonthsOrDuration(dt, unit, amount) {
// |   switch (unit.tag) {
// |     case 'Months':
// |       return dt.addCalendarMonths(Number(I32.fromF64(amount)));
// |     case 'Years':
// |       return dt.addCalendarMonths(Number(I32.fromF64(amount * 12.0)));
// |     default: {
// |       const seconds = I64.fromF64(unit.toSecs(Math.abs(amount)));
// |       if (amount >= 0.0) {
// |         return dt.addDuration(seconds);
// |       }
// |       return dt.addDuration(-seconds);
// |     }
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {DurationUnit} unit
// |  * @returns {Rational}
// |  */
// | export function durationUnitSeconds(unit) {
// |   switch (unit.tag) {
// |     case 'Milliseconds':
// |       return Rational.new(1, 1000);
// |     case 'Seconds':
// |       return Rational.one();
// |     case 'Minutes':
// |       return Rational.fromInteger(60);
// |     case 'Hours':
// |       return Rational.fromInteger(3600);
// |     case 'Days':
// |       return Rational.fromInteger(86_400);
// |     case 'Weeks':
// |       return Rational.fromInteger(604_800);
// |     case 'Months':
// |       return Rational.fromInteger(2_592_000);
// |     case 'Years':
// |       return Rational.fromInteger(31_536_000);
// |     default:
// |       return unreachable();
// |   }
// | }
