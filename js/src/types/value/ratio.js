/**
 * Exact percentage arithmetic and dimensionless ratio presentation (port of
 * `src/types/value/ratio.rs`).
 */

import { CalculatorError } from '../../error.js';
import { eq, err, expect } from '../../rust/core.js';
import { toAsciiLowercase } from '../../rust/str.js';
import { Rational } from '../rational.js';
import { Unit } from '../unit.js';
import { RatioFormat, ValueKind } from './kind.js';
import { Value } from './index.js';

/**
 * `Value::percent`: creates a percentage from its exact fraction (`1/10`
 * displays `10%`).
 * @param {Rational} fraction
 * @returns {Value}
 */
export function percent(fraction) {
  return new Value({ kind: ValueKind.Percent(fraction), unit: Unit.None });
}

/**
 * `Value::ratio`.
 * @param {Rational} value
 * @param {RatioFormat} format
 * @returns {Value}
 */
export function ratio(value, format) {
  return new Value({ kind: ValueKind.Ratio({ value, format }), unit: Unit.None });
}

/**
 * `Value::is_percent`.
 * @param {Value} self
 */
export function isPercent(self) {
  return self.kind.tag === 'Percent';
}

/**
 * `Value::has_ratio_display`.
 * @param {Value} self
 */
export function hasRatioDisplay(self) {
  return self.kind.tag === 'Percent' || self.kind.tag === 'Ratio';
}

/**
 * `Value::scalar`.
 * @param {Value} self
 * @returns {Value}
 */
export function scalar(self) {
  if (hasRatioDisplay(self)) {
    return Value.rationalWithUnit(expect(self.toRational(), 'numeric ratio'), self.unit);
  }
  return self;
}

/**
 * `Value::additive_ratio`: `null` for `None`; the `Some(Err)` case throws
 * `Err(CalculatorError)`.
 * @param {Value} self
 * @param {Value} other
 * @param {boolean} subtract
 * @returns {Value | null}
 */
export function additiveRatio(self, other, subtract) {
  if (!hasRatioDisplay(self) && !hasRatioDisplay(other)) {
    return null;
  }
  const left = self.toRational();
  if (left === null) {
    err(CalculatorError.InvalidOperation('percentage arithmetic requires numeric operands'));
  }
  let right = other.toRational();
  if (right === null) {
    err(CalculatorError.InvalidOperation('percentage arithmetic requires numeric operands'));
  }
  right = subtract ? right.neg() : right;
  if (isPercent(self)) {
    if (self.unit.tag !== 'None' || other.unit.tag !== 'None') {
      err(CalculatorError.InvalidOperation('cannot add a unit to a percentage'));
    }
    return percent(left.add(right));
  } else if (isPercent(other)) {
    return Value.rationalWithUnit(left.mul(Rational.one().add(right)), self.unit);
  } else if (eq(self.unit, other.unit)) {
    return Value.rationalWithUnit(left.add(right), self.unit);
  }
  return err(CalculatorError.InvalidOperation('ratio arithmetic requires compatible units'));
}

/**
 * Evaluates ratio constructors introduced by the percentage grammar.
 * Returning `None` (`null`) lets ordinary mathematical functions use their
 * evaluator; the `Some(Err)` case throws `Err(CalculatorError)`.
 * @param {string} name
 * @param {Value[]} args
 * @returns {Value | null}
 */
export function evaluateRatioFunction(name, args) {
  name = toAsciiLowercase(name);
  if (
    !(
      name === 'percent' ||
      name === 'as_percent' ||
      name === 'as_fraction' ||
      name === 'as_reciprocal' ||
      name === 'as_multiplier' ||
      name === 'as_decimal' ||
      name === 'percent_of'
    )
  ) {
    return null;
  }
  if (args.length !== 1) {
    err(CalculatorError.invalidArgs(name, 'expected one numeric argument'));
  }
  const value = args[0];
  if (name === 'percent_of') {
    return value;
  }
  if (value.unit.tag !== 'None') {
    err(CalculatorError.InvalidOperation('ratio conversion requires a unitless number'));
  }
  const fraction = value.toRational();
  if (fraction === null) {
    err(CalculatorError.invalidArgs(name, 'expected numeric argument'));
  }
  switch (name) {
    case 'percent':
      return percent(fraction.div(Rational.fromInteger(100)));
    case 'as_percent':
      return percent(fraction);
    case 'as_fraction':
      return ratio(fraction, RatioFormat.Fraction);
    case 'as_multiplier':
      return ratio(fraction, RatioFormat.Multiplier);
    case 'as_reciprocal':
      if (fraction.isZero()) {
        err(CalculatorError.DivisionByZero);
      }
      return ratio(fraction, RatioFormat.Reciprocal);
    default:
      return Value.rational(fraction);
  }
}
