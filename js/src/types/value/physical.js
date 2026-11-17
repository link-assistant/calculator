/**
 * Dimensional arithmetic on physical units (issue #235; port of
 * `src/types/value/physical.rs`).
 *
 * Multiplication, division and integer powers combine unit products;
 * addition, subtraction, comparison and conversion require equal dimensions
 * and convert exactly through the unit database factors.
 *
 * Affine temperatures follow Numbat: `20 °C` is read as 293.15 K, and a value
 * displayed in `°C`/`°F` (the result of a conversion) only converts further;
 * any arithmetic on it is rejected.
 */

import { CalculatorError } from '../../error.js';
import { eq, err, expect } from '../../rust/core.js';
import { pow } from '../../rust/libm.js';
import { I128, intToF64, round } from '../../rust/num.js';
import { PhysicalUnit } from '../physical.js';
import { Rational } from '../rational.js';
import { DurationUnit, Unit } from '../unit.js';
import { Value } from './index.js';

/**
 * Whether a unit takes part in dimensional algebra.
 * @param {Unit} unit
 */
function isDimensional(unit) {
  return (
    unit.tag === 'Physical' || unit.tag === 'Mass' || unit.tag === 'Duration' || unit.tag === 'DataSize'
  );
}

/**
 * The affine unit of a value displayed in `°C`/`°F`, if any.
 * @param {Value} value
 * @returns {PhysicalUnit | null}
 */
function affineUnit(value) {
  if (value.unit.tag === 'Physical' && value.unit._0.hasAffine()) {
    return value.unit._0;
  }
  return null;
}

/**
 * @param {Value} value
 * @param {string} operation
 * @returns {CalculatorError}
 */
function affineError(value, operation) {
  return CalculatorError.domain(
    `cannot ${operation} ${value.toDisplayString()}: affine temperature units only convert; ` +
      `convert to K first (e.g. \`${value.toDisplayString()} in K\`)`,
  );
}

/**
 * `Value::physical_parts`: the numeric amount and unit product of a
 * dimensional value.
 * @param {Value} self
 * @returns {[Rational, PhysicalUnit] | null}
 */
function physicalParts(self) {
  if (!(self.kind.tag === 'Number' || self.kind.tag === 'Rational')) {
    return null;
  }
  const amount = self.toRational();
  if (amount === null) {
    return null;
  }
  const unit = PhysicalUnit.fromUnit(self.unit);
  return unit === null ? null : [amount, unit];
}

/**
 * `Value::physical_base`: the amount in coherent SI units and the dimension
 * of a value with a dimensional unit (`30 °` → `(0.5236, angle)`).
 * @param {Value} self
 * @returns {[Rational, import('../dimension.js').Dimension] | null}
 */
export function physicalBase(self) {
  if (!isDimensional(self.unit)) {
    return null;
  }
  const parts = physicalParts(self);
  if (parts === null) {
    return null;
  }
  const [amount, unit] = parts;
  return [toBase(amount, unit), unit.dimension()];
}

/**
 * `Value::from_physical`: builds a value from an amount and an unsimplified
 * unit product.
 * @param {Rational} amount
 * @param {PhysicalUnit} unit
 * @returns {Value}
 */
export function fromPhysical(amount, unit) {
  const [simplified, scale] = unit.simplify();
  return Value.rationalWithUnit(amount.mul(scale), simplified.intoUnit());
}

/**
 * `Value::reject_affine`: rejects arithmetic on a value displayed in an
 * affine unit. Throws `Err(CalculatorError)`.
 * @param {Value} self
 * @param {string} operation
 */
export function rejectAffine(self, operation) {
  if (affineUnit(self) !== null) {
    err(affineError(self, operation));
  }
}

/**
 * `Value::uses_physical_algebra`: whether an operation between the two values
 * uses dimensional algebra: one side is a physical unit, or both carry
 * dimensional units.
 * @param {Value} self
 * @param {Value} other
 */
function usesPhysicalAlgebra(self, other) {
  const physical = (/** @type {Value} */ value) => value.unit.tag === 'Physical';
  return (
    (physical(self) || physical(other) || (isDimensional(self.unit) && isDimensional(other.unit))) &&
    physicalParts(self) !== null &&
    physicalParts(other) !== null
  );
}

/**
 * `Value::physical_product`: multiplies or divides dimensional values
 * (`10 m × 10 m`, `5 V / 2 Ω`). `null` for `Ok(None)`; throws
 * `Err(CalculatorError)`.
 * @param {Value} self
 * @param {Value} other
 * @param {boolean} divide
 * @returns {Value | null}
 */
export function physicalProduct(self, other, divide) {
  const bothDurations = self.unit.tag === 'Duration' && other.unit.tag === 'Duration';
  if (bothDurations || !usesPhysicalAlgebra(self, other)) {
    // `1 / (2 kg)` is `0.5 kg^-1`.
    const reciprocal =
      divide &&
      self.unit.tag === 'None' &&
      isDimensional(other.unit) &&
      physicalParts(self) !== null &&
      physicalParts(other) !== null;
    if (!reciprocal) {
      return null;
    }
  }
  const operation = divide ? 'divide' : 'multiply';
  rejectAffine(self, operation);
  rejectAffine(other, operation);
  const [a, left] = expect(physicalParts(self), 'checked above');
  const [b, right] = expect(physicalParts(other), 'checked above');
  if (divide) {
    if (b.isZero()) {
      err(CalculatorError.DivisionByZero);
    }
    return fromPhysical(a.div(b), left.product(right.powi(-1)));
  }
  return fromPhysical(a.mul(b), left.product(right));
}

/**
 * `Value::physical_sum`: adds or subtracts dimensional values of equal
 * dimension, converting the right operand to the left operand's unit
 * (`1 ft + 1 in`). `null` for `None`; the `Some(Err)` case throws
 * `Err(CalculatorError)`.
 * @param {Value} self
 * @param {Value} other
 * @param {boolean} subtract
 * @returns {Value | null}
 */
export function physicalSum(self, other, subtract) {
  const operation = subtract ? 'subtract' : 'add';
  rejectAffine(self, operation);
  rejectAffine(other, operation);
  if (
    eq(self.unit, other.unit) ||
    self.unit.tag === 'None' ||
    other.unit.tag === 'None' ||
    (self.unit.tag === 'Mass' && other.unit.tag === 'Mass') ||
    !usesPhysicalAlgebra(self, other)
  ) {
    return null;
  }
  return physicalSumUnchecked(self, other, subtract);
}

/**
 * @param {Value} self
 * @param {Value} other
 * @param {boolean} subtract
 * @returns {Value}
 */
function physicalSumUnchecked(self, other, subtract) {
  const operation = subtract ? 'subtract' : 'add';
  const [a, left] = expect(physicalParts(self), 'checked by caller');
  const [b, right] = expect(physicalParts(other), 'checked by caller');
  if (!eq(left.dimension(), right.dimension())) {
    // An exact zero has every dimension (`4 m + 0 kg`).
    if (b.isZero()) {
      return self;
    }
    if (a.isZero() && !subtract) {
      return other;
    }
    err(CalculatorError.unitMismatch(operation, self.unit.displayName(), other.unit.displayName()));
  }
  const bInLeft = b.mul(right.factor()).div(left.factor());
  const amount = subtract ? a.sub(bInLeft) : a.add(bInLeft);
  return Value.rationalWithUnit(amount, self.unit);
}

/**
 * `Value::metres_in_time_context`: reads a value in metres as minutes next to
 * a duration or a date (`1h + 5m`, `now - 5m`): `m` is metre unless the time
 * context is explicit. `null` when neither side is a bare metre in such a
 * context.
 * @param {Value} self
 * @param {Value} other
 * @returns {[Value, Value] | null}
 */
export function metresInTimeContext(self, other) {
  const isTime = (/** @type {Value} */ value) =>
    value.unit.tag === 'Duration' || value.kind.tag === 'DateTime' || value.kind.tag === 'Duration';
  const asMinutes = (/** @type {Value} */ value) => {
    if (value.unit.tag === 'Physical' && isMetre(value.unit._0)) {
      return new Value({ kind: value.kind, unit: Unit.Duration(DurationUnit.Minutes) });
    }
    return null;
  };
  if (isTime(other)) {
    const left = asMinutes(self);
    return left === null ? null : [left, other];
  }
  if (isTime(self)) {
    const right = asMinutes(other);
    return right === null ? null : [self, right];
  }
  return null;
}

/**
 * `Value::physical_power`: raises a dimensional value to an integer power
 * (`(2 m)^2` → `4 m^2`). `null` for `None`; the `Some(Err)` case throws
 * `Err(CalculatorError)`.
 * @param {Value} self
 * @param {number} power an `i32`
 * @returns {Value | null}
 */
export function physicalPower(self, power) {
  if (!isDimensional(self.unit)) {
    return null;
  }
  const parts = physicalParts(self);
  if (parts === null) {
    return null;
  }
  const [amount, unit] = parts;
  rejectAffine(self, 'raise to a power');
  if (power < 0 && amount.isZero()) {
    err(CalculatorError.DivisionByZero);
  }
  return fromPhysical(amount.powI32(power), unit.powi(power));
}

/**
 * `Value::physical_root`: takes the `n`th root of a dimensional value
 * (`sqrt(4 m^2)` → `2 m`).
 *
 * Fails with a non-integer dimension error when a unit power is not a
 * multiple of `n` (`sqrt(2 m)`). `null` for `None`; the `Some(Err)` case
 * throws `Err(CalculatorError)`.
 * @param {Value} self
 * @param {number} n an `i32`
 * @returns {Value | null}
 */
export function physicalRoot(self, n) {
  if (!isDimensional(self.unit)) {
    return null;
  }
  const parts = physicalParts(self);
  if (parts === null) {
    return null;
  }
  const [amount, unit] = parts;
  return physicalRootUnchecked(self, amount, unit, n);
}

/**
 * @param {Value} self
 * @param {Rational} amount
 * @param {PhysicalUnit} unit
 * @param {number} n
 * @returns {Value}
 */
function physicalRootUnchecked(self, amount, unit, n) {
  rejectAffine(self, 'take a root of');
  const [simplified, scale] = unit.simplify();
  amount = amount.mul(scale);
  const rootUnit = simplified.root(n);
  if (rootUnit === null) {
    err(CalculatorError.domain(`root of ${self.toDisplayString()} has a non-integer dimension`));
  }
  if (amount.isNegative() && n % 2 === 0) {
    err(CalculatorError.domain('even root of a negative quantity'));
  }
  let root = exactRoot(amount, n);
  if (root === null) {
    const magnitude = pow(amount.abs().toF64(), 1.0 / n);
    const signed = amount.isNegative() ? -magnitude : magnitude;
    root = Rational.fromF64(signed);
  }
  return Value.rationalWithUnit(root, rootUnit.intoUnit());
}

/**
 * `Value::physical_eq`: whether two dimensional values of equal dimension
 * denote the same quantity (`2 m == 200 cm`).
 * @param {Value} self
 * @param {Value} other
 * @returns {boolean | null}
 */
export function physicalEq(self, other) {
  if (eq(self.unit, other.unit) || !usesPhysicalAlgebra(self, other)) {
    return null;
  }
  const leftParts = physicalParts(self);
  if (leftParts === null) {
    return null;
  }
  const rightParts = physicalParts(other);
  if (rightParts === null) {
    return null;
  }
  const [a, left] = leftParts;
  const [b, right] = rightParts;
  if (!eq(left.dimension(), right.dimension())) {
    return false;
  }
  return toBase(a, left).equals(toBase(b, right));
}

/**
 * `Value::physical_conversion`: converts a value to a physical target unit
 * (`1 ft to cm`, `25 °C in °F`).
 *
 * A value in metres converted to a duration reads `m` as minutes: `5m`
 * is a length, but `5m in seconds` asks for a duration. `null` for `None`;
 * the `Some(Err)` case throws `Err(CalculatorError)`.
 * @param {Value} self
 * @param {Unit} target
 * @returns {Value | null}
 */
export function physicalConversion(self, target) {
  const targetIsPhysical = target.tag === 'Physical';
  const sourceIsPhysical = self.unit.tag === 'Physical';
  const crossesLegacy = isDimensional(self.unit) && isDimensional(target) && self.unit.tag !== target.tag;
  if (!(targetIsPhysical || sourceIsPhysical || crossesLegacy)) {
    return null;
  }
  const targetUnit = PhysicalUnit.fromUnit(target);
  if (targetUnit === null) {
    return null;
  }
  const parts = physicalParts(self);
  if (parts === null) {
    return null;
  }
  const amount = parts[0];
  let source = parts[1];
  if (self.unit.tag === 'None') {
    return Value.rationalWithUnit(amount, target);
  }
  if (isMetre(source) && target.tag === 'Duration') {
    source = expect(PhysicalUnit.parse('min'), 'minute is a database unit');
  }
  if (!eq(source.dimension(), targetUnit.dimension())) {
    err(
      CalculatorError.InvalidOperation(
        `Cannot convert ${self.unit.displayName()} to ${target.displayName()}: dimensions ${source.dimension()} and ${targetUnit.dimension()} differ`,
      ),
    );
  }
  const base = toBase(amount, source);
  let converted = base.div(targetUnit.factor());
  const offset = targetUnit.affineOffset();
  if (offset !== null) {
    converted = converted.sub(offset);
  }
  return Value.rationalWithUnit(converted, target);
}

/**
 * Normalizes a quantity in an affine unit to kelvin (`20 °C` → `293.15 K`).
 *
 * Other units are returned unchanged.
 * @param {Rational} amount
 * @param {Unit} unit
 * @returns {[Rational, Unit]}
 */
export function normalizeAffine(amount, unit) {
  if (unit.tag !== 'Physical') {
    return [amount, unit];
  }
  const physical = unit._0;
  if (physical.affineOffset() === null) {
    return [amount, unit];
  }
  const base = toBase(amount, physical);
  const kelvin = expect(PhysicalUnit.parse('K'), 'kelvin is a database unit');
  return [base, Unit.Physical(kelvin)];
}

/**
 * The amount in coherent SI units, applying an affine offset.
 * @param {Rational} amount
 * @param {PhysicalUnit} unit
 * @returns {Rational}
 */
function toBase(amount, unit) {
  const offset = unit.affineOffset();
  const shifted = offset === null ? amount : amount.add(offset);
  return shifted.mul(unit.factor());
}

/** @param {PhysicalUnit} unit */
function isMetre(unit) {
  const terms = unit.terms();
  return terms.length === 1 && terms[0].symbol === 'm' && terms[0].power === 1;
}

/**
 * The exact `n`th root of a rational whose numerator and denominator are
 * perfect powers (`sqrt(9/4)` → `3/2`).
 * @param {Rational} value
 * @param {number} n
 * @returns {Rational | null}
 */
function exactRoot(value, n) {
  const rootOf = (/** @type {bigint} */ x) => {
    const xAbs = x < 0n ? -x : x;
    const magnitude = intToF64(xAbs);
    const guess = I128.fromF64(round(pow(magnitude, 1.0 / n)));
    if (n < 0) {
      return null;
    }
    for (let candidate = guess - 1n; candidate <= guess + 1n; candidate += 1n) {
      if (candidate >= 0n && I128.checkedPow(candidate, n) === xAbs) {
        return x < 0n ? -candidate : candidate;
      }
    }
    return null;
  };
  const numer = rootOf(value.numer());
  if (numer === null) {
    return null;
  }
  const denom = rootOf(value.denom());
  if (denom === null) {
    return null;
  }
  return Rational.new(numer, denom);
}
