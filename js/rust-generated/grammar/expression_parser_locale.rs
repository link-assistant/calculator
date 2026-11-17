// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a7e45190b5dc629ece64c6bbc4c02f7a4eed0b4d5f729fdb9f734ac0e11a8705 bytes=1446

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Primary and alternative numeric interpretations, resolved during lexing
// |  * (port of `src/grammar/expression_parser_locale.rs`).
// |  */

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./expression_parser.js').ExpressionParser} ExpressionParser */
// | /** @typedef {import('../types/expression.js').Expression} Expression */

// meta-language:translated JavaScript import_statement items=1 sha256=c41ed540bafa7bc71f2a0b7ae82e23a650e3ae82f2c3c9bb7e2fa757ba3f2be3
// | import { tryResult } from '../rust/core.js';
use crate::rust::core_::tryResult;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses an expression string into an Expression AST.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {ExpressionParser} self
// |  * @param {string} input
// |  * @returns {Expression}
// |  */
// | export function parse(self, input) {
// |   return self.parseTokenized(input);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses the primary numeric convention and a decimal-comma alternative.
// |  *
// |  * Thousands grouping wins for a comma before three digits. The decimal
// |  * interpretation is still available to callers that display alternatives.
// |  * Function arguments use commas as separators in both interpretations.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {ExpressionParser} self
// |  * @param {string} input
// |  * @returns {Expression[]}
// |  */
// | export function parseInterpretations(self, input) {
// |   const primary = self.parseTokenized(input);
// |   const interpretations = [primary];
// |   if (input.includes(',')) {
// |     const alternative = tryResult(() => self.parseTokenizedDecimalCommas(input));
// |     if (alternative.ok && alternative.value.toLino() !== interpretations[0].toLino()) {
// |       interpretations.push(alternative.value);
// |     }
// |   }
// |   return interpretations;
// | }
