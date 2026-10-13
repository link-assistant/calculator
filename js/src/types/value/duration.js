/**
 * Duration helpers for values (port of `src/types/value/duration.rs`).
 */

import { CalculatorError } from '../../error.js';
import { err, ok, unreachable } from '../../rust/core.js';
import { I32, I64 } from '../../rust/num.js';
import { DateTime } from '../datetime.js';
import { Rational } from '../rational.js';
import { DurationUnit, Unit } from '../unit.js';
import { Value } from './index.js';

/**
 * Interprets an unadorned four-digit integer as January 1 of that year.
 * @param {Value} value
 * @returns {DateTime | null}
 */
export function bareYearDatetime(value) {
  if (value.unit.tag !== 'None') {
    return null;
  }

  /** @type {number} */
  let year;
  const kind = value.kind;
  if (kind.tag === 'Rational' && kind._0.isInteger()) {
    const parsed = I32.tryFrom(kind._0.numer());
    if (parsed === null) {
      return null;
    }
    year = Number(parsed);
  } else if (kind.tag === 'Number') {
    const parsed = ok(() => I32.parse(kind._0.toString()));
    if (parsed === null) {
      return null;
    }
    year = Number(parsed);
  } else {
    return null;
  }

  return year >= 1000 && year <= 9999 ? DateTime.fromYear(year) : null;
}

/**
 * Formats a duration in seconds to a human-readable string.
 * @param {bigint} totalSeconds an `i64`
 * @returns {string}
 */
export function formatDuration(totalSeconds) {
  const isNegative = totalSeconds < 0n;
  totalSeconds = isNegative ? -totalSeconds : totalSeconds;

  const days = totalSeconds / 86400n;
  const hours = (totalSeconds % 86400n) / 3600n;
  const minutes = (totalSeconds % 3600n) / 60n;
  const seconds = totalSeconds % 60n;

  /** @type {string[]} */
  const parts = [];

  if (days > 0n) {
    parts.push(`${days} day${days === 1n ? '' : 's'}`);
  }
  if (hours > 0n) {
    parts.push(`${hours} hour${hours === 1n ? '' : 's'}`);
  }
  if (minutes > 0n) {
    parts.push(`${minutes} minute${minutes === 1n ? '' : 's'}`);
  }
  if (seconds > 0n || parts.length === 0) {
    parts.push(`${seconds} second${seconds === 1n ? '' : 's'}`);
  }

  const result = parts.join(', ');
  return isNegative ? `-${result}` : result;
}

/**
 * Divides compatible duration-unit values as a unitless ratio.
 * Throws `Err(CalculatorError)`.
 * @param {Value} left
 * @param {Value} right
 * @returns {Value | null}
 */
export function divideDurationUnits(left, right) {
  if (left.unit.tag !== 'Duration' || right.unit.tag !== 'Duration') {
    return null;
  }
  const leftUnit = left.unit._0;
  const rightUnit = right.unit._0;

  const leftAmount = left.toRational();
  if (leftAmount === null) {
    err(CalculatorError.InvalidOperation('duration division requires numeric values'));
  }
  const rightAmount = right.toRational();
  if (rightAmount === null) {
    err(CalculatorError.InvalidOperation('duration division requires numeric values'));
  }

  const leftSeconds = leftAmount.mul(durationUnitSeconds(leftUnit));
  const rightSeconds = rightAmount.mul(durationUnitSeconds(rightUnit));
  if (rightSeconds.isZero()) {
    err(CalculatorError.DivisionByZero);
  }

  return Value.rational(leftSeconds.div(rightSeconds));
}

/**
 * Converts a raw duration in seconds into a numeric amount in `unit`.
 * @param {bigint} seconds
 * @param {DurationUnit} unit
 * @returns {Rational}
 */
export function durationSecondsToUnit(seconds, unit) {
  return Rational.fromInteger(seconds).div(durationUnitSeconds(unit));
}

/**
 * Converts a raw duration in seconds into its numeric day count.
 * @param {bigint} seconds
 * @returns {Rational}
 */
export function durationSecondsToDays(seconds) {
  return durationSecondsToUnit(seconds, DurationUnit.Days);
}

/**
 * Throws `Err(CalculatorError)`.
 * @param {bigint} seconds
 * @param {Value} divisor
 * @returns {Value}
 */
export function divideRawDuration(seconds, divisor) {
  const divisorAmount = divisor.toRational();
  if (divisorAmount === null) {
    err(CalculatorError.InvalidOperation('duration division requires a numeric divisor'));
  }
  if (divisorAmount.isZero()) {
    err(CalculatorError.DivisionByZero);
  }

  switch (divisor.unit.tag) {
    case 'None':
      return Value.rational(durationSecondsToDays(seconds).div(divisorAmount));
    case 'Duration': {
      const divisorSeconds = divisorAmount.mul(durationUnitSeconds(divisor.unit._0));
      if (divisorSeconds.isZero()) {
        err(CalculatorError.DivisionByZero);
      }
      return Value.rational(Rational.fromInteger(seconds).div(divisorSeconds));
    }
    case 'Currency':
      return Value.rational(durationSecondsToDays(seconds).div(divisorAmount));
    default:
      return err(
        CalculatorError.InvalidOperation(`Cannot divide duration by ${divisor.unit.displayName()}`),
      );
  }
}

/**
 * Throws `Err(CalculatorError)`.
 * @param {bigint} seconds
 * @param {Unit} targetUnit
 * @returns {Value}
 */
export function convertRawDuration(seconds, targetUnit) {
  switch (targetUnit.tag) {
    case 'None':
      return Value.rational(durationSecondsToDays(seconds));
    case 'Duration':
      return Value.rationalWithUnit(
        durationSecondsToUnit(seconds, targetUnit._0),
        Unit.Duration(targetUnit._0),
      );
    default:
      return err(
        CalculatorError.InvalidOperation(`Cannot convert duration to ${targetUnit.displayName()}`),
      );
  }
}

/**
 * Throws `Err(CalculatorError)`.
 * @param {Value} value
 * @param {DurationUnit} unit
 * @returns {Value}
 */
export function applyDurationUnit(value, unit) {
  const amount = value.toRational();
  if (amount === null) {
    err(CalculatorError.InvalidOperation('duration unit conversion requires a numeric value'));
  }
  return Value.rationalWithUnit(amount, Unit.Duration(unit));
}

/**
 * Applies a signed duration to a `DateTime`, using calendar arithmetic for
 * months and years and second-based arithmetic for all other units.
 *
 * `amount` is positive for addition and negative for subtraction.
 * @param {DateTime} dt
 * @param {DurationUnit} unit
 * @param {number} amount
 * @returns {DateTime}
 */
export function addCalendarMonthsOrDuration(dt, unit, amount) {
  switch (unit.tag) {
    case 'Months':
      return dt.addCalendarMonths(Number(I32.fromF64(amount)));
    case 'Years':
      return dt.addCalendarMonths(Number(I32.fromF64(amount * 12.0)));
    default: {
      const seconds = I64.fromF64(unit.toSecs(Math.abs(amount)));
      if (amount >= 0.0) {
        return dt.addDuration(seconds);
      }
      return dt.addDuration(-seconds);
    }
  }
}

/**
 * @param {DurationUnit} unit
 * @returns {Rational}
 */
export function durationUnitSeconds(unit) {
  switch (unit.tag) {
    case 'Milliseconds':
      return Rational.new(1, 1000);
    case 'Seconds':
      return Rational.one();
    case 'Minutes':
      return Rational.fromInteger(60);
    case 'Hours':
      return Rational.fromInteger(3600);
    case 'Days':
      return Rational.fromInteger(86_400);
    case 'Weeks':
      return Rational.fromInteger(604_800);
    case 'Months':
      return Rational.fromInteger(2_592_000);
    case 'Years':
      return Rational.fromInteger(31_536_000);
    default:
      return unreachable();
  }
}
