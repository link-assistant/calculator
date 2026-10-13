/**
 * Measurements with an absolute uncertainty (port of
 * `src/types/value/uncertainty.rs`).
 */

import { CalculatorError } from '../../error.js';
import { eq, err, unreachable } from '../../rust/core.js';
import { ValueKind } from './kind.js';
import { Value } from './index.js';

/**
 * `Value::with_uncertainty`: creates a measurement with a non-negative
 * absolute error in the same unit. Throws `Err(CalculatorError)`.
 * @param {Value} self
 * @param {Value} error
 * @returns {Value}
 */
export function withUncertainty(self, error) {
  if (!eq(self.unit, error.unit)) {
    err(CalculatorError.InvalidOperation('uncertainty and central value must have matching units'));
  }
  const value = self.toRational();
  if (value === null) {
    err(CalculatorError.domain('central value must be numeric'));
  }
  const uncertainty = error.toRational();
  if (uncertainty === null) {
    err(CalculatorError.domain('uncertainty must be numeric'));
  }
  if (uncertainty.isNegative()) {
    err(CalculatorError.domain('uncertainty must be non-negative'));
  }
  return new Value({ kind: ValueKind.Uncertainty({ value, uncertainty }), unit: self.unit });
}

/**
 * `Value::negate_uncertainty`.
 * @param {Value} self
 * @returns {Value}
 */
export function negateUncertainty(self) {
  const kind = self.kind;
  if (kind.tag !== 'Uncertainty') {
    unreachable();
  }
  return new Value({
    kind: ValueKind.Uncertainty({ value: kind.value.neg(), uncertainty: kind.uncertainty }),
    unit: self.unit,
  });
}
