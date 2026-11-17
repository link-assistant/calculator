// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8d5e81a202c88b794aa62f1f1c1a556704ca49c55c1a0ccc14bf890f20499d4e bytes=2699

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Numeric access for calculator values (port of
// |  * `src/types/value/formatting.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from '../rational.js';
use crate::types::rational::Rational;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::type_name`: returns the type name for error messages.
// |  * @param {import('./index.js').Value} self
// |  * @returns {string}
// |  */
// | export function typeName(self) {
// |   switch (self.kind.tag) {
// |     case 'Number':
// |       return 'number';
// |     case 'Rational':
// |     case 'Ratio':
// |       return 'number';
// |     case 'Percent':
// |       return 'percent';
// |     case 'Uncertainty':
// |       return 'uncertainty';
// |     case 'DateTime':
// |       return 'datetime';
// |     case 'Duration':
// |       return 'duration';
// |     case 'Boolean':
// |       return 'boolean';
// |     case 'Comparison':
// |       return 'comparison result';
// |     default:
// |       return 'equation solution';
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::is_number`: returns true if this is a number (either Decimal or
// |  * Rational).
// |  * @param {import('./index.js').Value} self
// |  * @returns {boolean}
// |  */
// | export function isNumber(self) {
// |   const tag = self.kind.tag;
// |   return tag === 'Number' || tag === 'Rational' || tag === 'Percent' || tag === 'Ratio';
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The `Rational(r) | Percent(r) | Ratio { value: r, .. }` binding, or `null`.
// |  * @param {import('./index.js').Value} self
// |  * @returns {Rational | null}
// |  */
// | function ratioLike(self) {
// |   const kind = self.kind;
// |   switch (kind.tag) {
// |     case 'Rational':
// |     case 'Percent':
// |       return kind._0;
// |     case 'Ratio':
// |       return kind.value;
// |     default:
// |       return null;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::as_number`: returns the decimal value if this is a number.
// |  * @param {import('./index.js').Value} self
// |  * @returns {import('../decimal.js').Decimal | null}
// |  */
// | export function asNumber(self) {
// |   if (self.kind.tag === 'Number') {
// |     return self.kind._0;
// |   }
// |   const r = ratioLike(self);
// |   return r === null ? null : r.toDecimal();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::as_rational`: returns the rational value if this is a Rational.
// |  * @param {import('./index.js').Value} self
// |  * @returns {Rational | null}
// |  */
// | export function asRational(self) {
// |   return ratioLike(self);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::to_rational`: converts this value to a Rational if numeric (clones
// |  * Rational, converts Decimal).
// |  * @param {import('./index.js').Value} self
// |  * @returns {Rational | null}
// |  */
// | export function toRational(self) {
// |   if (self.kind.tag === 'Number') {
// |     return Rational.fromDecimal(self.kind._0);
// |   }
// |   return ratioLike(self);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::to_fraction_string`: returns the fraction string representation if
// |  * this is a Rational.
// |  * @param {import('./index.js').Value} self
// |  * @returns {string | null}
// |  */
// | export function toFractionString(self) {
// |   const r = ratioLike(self);
// |   return r === null ? null : r.toFractionString();
// | }
