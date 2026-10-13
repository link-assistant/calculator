// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=59a261a64cd7fdd9a3a9ae9b2144cf28f3be40e840fddb74f8802485450218cd bytes=1965

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Display of values (port of `src/types/value/display.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from '../rational.js';
use crate::types::rational::Rational;
// meta-language:translated JavaScript import_statement items=1 sha256=21afef223d33789ee49943bff97554385079bec9413fb96394ef8852b71f38e3
// | import { formatDuration } from './duration.js';
use crate::types::value::duration::formatDuration;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::to_display_string`: converts the value to a display string.
// |  * @param {import('./index.js').Value} self
// |  * @returns {string}
// |  */
// | export function toDisplayString(self) {
// |   const kind = self.kind;
// |   switch (kind.tag) {
// |     case 'Number': {
// |       const nStr = kind._0.normalize().toString();
// |       return self.unit.tag === 'None' ? nStr : `${nStr} ${self.unit}`;
// |     }
// |     case 'Rational': {
// |       const rStr = kind._0.toDisplayString();
// |       return self.unit.tag === 'None' ? rStr : `${rStr} ${self.unit}`;
// |     }
// |     case 'Percent':
// |       return `${kind._0.mul(Rational.fromInteger(100)).toDisplayString()}%`;
// |     case 'Ratio':
// |       switch (kind.format.tag) {
// |         case 'Fraction':
// |           return kind.value.toFractionString();
// |         case 'Reciprocal': {
// |           const ratio = Rational.one().checkedDiv(kind.value);
// |           return `1/${ratio === null ? 'undefined' : ratio.toDisplayString()}`;
// |         }
// |         default:
// |           return `${kind.value.toDisplayString()}x`;
// |       }
// |     case 'Uncertainty': {
// |       const suffix = self.unit.tag === 'None' ? '' : ` ${self.unit}`;
// |       return `${kind.value.toDisplayString()} ± ${kind.uncertainty.toDisplayString()}${suffix}`;
// |     }
// |     case 'DateTime':
// |       return kind._0.toString();
// |     case 'Duration':
// |       return formatDuration(kind.seconds);
// |     case 'Boolean':
// |       return String(kind._0);
// |     case 'Comparison':
// |       return `${kind.left} ${kind.relation} ${kind.right}`;
// |     case 'EquationSolution':
// |       return `${kind.variable} = ${kind.value.toDisplayString()}`;
// |     case 'EquationSolutions':
// |       return kind.values
// |         .map((/** @type {Rational} */ value) => `${kind.variable} = ${value.toDisplayString()}`)
// |         .join(' or ');
// |     default:
// |       return `${kind.variable} = ${kind.expression}`;
// |   }
// | }
