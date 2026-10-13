/**
 * Display of values (port of `src/types/value/display.rs`).
 */

import { Rational } from '../rational.js';
import { formatDuration } from './duration.js';

/**
 * `Value::to_display_string`: converts the value to a display string.
 * @param {import('./index.js').Value} self
 * @returns {string}
 */
export function toDisplayString(self) {
  const kind = self.kind;
  switch (kind.tag) {
    case 'Number': {
      const nStr = kind._0.normalize().toString();
      return self.unit.tag === 'None' ? nStr : `${nStr} ${self.unit}`;
    }
    case 'Rational': {
      const rStr = kind._0.toDisplayString();
      return self.unit.tag === 'None' ? rStr : `${rStr} ${self.unit}`;
    }
    case 'Percent':
      return `${kind._0.mul(Rational.fromInteger(100)).toDisplayString()}%`;
    case 'Ratio':
      switch (kind.format.tag) {
        case 'Fraction':
          return kind.value.toFractionString();
        case 'Reciprocal': {
          const ratio = Rational.one().checkedDiv(kind.value);
          return `1/${ratio === null ? 'undefined' : ratio.toDisplayString()}`;
        }
        default:
          return `${kind.value.toDisplayString()}x`;
      }
    case 'Uncertainty': {
      const suffix = self.unit.tag === 'None' ? '' : ` ${self.unit}`;
      return `${kind.value.toDisplayString()} ± ${kind.uncertainty.toDisplayString()}${suffix}`;
    }
    case 'DateTime':
      return kind._0.toString();
    case 'Duration':
      return formatDuration(kind.seconds);
    case 'Boolean':
      return String(kind._0);
    case 'Comparison':
      return `${kind.left} ${kind.relation} ${kind.right}`;
    case 'EquationSolution':
      return `${kind.variable} = ${kind.value.toDisplayString()}`;
    case 'EquationSolutions':
      return kind.values
        .map((/** @type {Rational} */ value) => `${kind.variable} = ${value.toDisplayString()}`)
        .join(' or ');
    default:
      return `${kind.variable} = ${kind.expression}`;
  }
}
