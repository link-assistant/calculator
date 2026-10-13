/**
 * Primary and alternative numeric interpretations, resolved during lexing
 * (port of `src/grammar/expression_parser_locale.rs`).
 */

/** @typedef {import('./expression_parser.js').ExpressionParser} ExpressionParser */
/** @typedef {import('../types/expression.js').Expression} Expression */

import { tryResult } from '../rust/core.js';

/**
 * Parses an expression string into an Expression AST.
 * Throws `Err(CalculatorError)`.
 * @param {ExpressionParser} self
 * @param {string} input
 * @returns {Expression}
 */
export function parse(self, input) {
  return self.parseTokenized(input);
}

/**
 * Parses the primary numeric convention and a decimal-comma alternative.
 *
 * Thousands grouping wins for a comma before three digits. The decimal
 * interpretation is still available to callers that display alternatives.
 * Function arguments use commas as separators in both interpretations.
 * Throws `Err(CalculatorError)`.
 * @param {ExpressionParser} self
 * @param {string} input
 * @returns {Expression[]}
 */
export function parseInterpretations(self, input) {
  const primary = self.parseTokenized(input);
  const interpretations = [primary];
  if (input.includes(',')) {
    const alternative = tryResult(() => self.parseTokenizedDecimalCommas(input));
    if (alternative.ok && alternative.value.toLino() !== interpretations[0].toLino()) {
      interpretations.push(alternative.value);
    }
  }
  return interpretations;
}
