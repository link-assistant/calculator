/**
 * Comparison-level parsing (port of `src/grammar/token_parser/comparison.rs`).
 */

import { BinaryOp, ComparisonOp, DurationUnit, Expression, Unit } from '../../types/index.js';
import { eqIgnoreAsciiCase } from '../../rust/str.js';
import { TokenKind } from '../lexer.js';

/** @typedef {import('../token_parser.js').TokenParser} TokenParser */

/**
 * Throws `Err(CalculatorError)`.
 * @param {TokenParser} self
 * @returns {Expression}
 */
export function parseExpression(self) {
  if (self.ratioWord('if') && self.hasProportionQuestion()) {
    self.advance();
  }
  if (self.ratioWord('what') && self.peekIdentifierIs('is')) {
    self.advance();
    self.advance();
  }
  return self.parseComparison();
}

/**
 * @param {TokenParser} self
 * @returns {Expression}
 */
export function parseComparison(self) {
  const daySpan = self.tryParseDaySpan();
  if (daySpan !== null) {
    return daySpan;
  }

  if (self.checkCompare()) {
    self.advance(); // consume "compare"
    const left = self.parseAdditive();
    self.expect(TokenKind.And);
    const right = self.parseAdditive();
    return Expression.comparison(left, ComparisonOp.Compare, right);
  }

  let left = self.parseAdditive();
  if (self.check(TokenKind.PlusMinus)) {
    self.advance();
    const uncertainty = self.parseAdditive();
    left = Expression.functionCall('uncertainty', [left, uncertainty]);
  }

  if (self.checkVs()) {
    self.advance(); // consume "vs"
    const right = self.parseAdditive();
    return Expression.comparison(left, ComparisonOp.Compare, right);
  }

  if (self.check(TokenKind.Equals)) {
    self.advance(); // consume '=' or '=='
    if (self.isAtEnd()) {
      return left;
    }
    const right = self.parseAdditive();
    return Expression.equality(left, right);
  }

  const op = self.matchOrderingOp();
  if (op !== null) {
    const right = self.parseAdditive();
    return Expression.comparison(left, op, right);
  }

  return left;
}

/**
 * Parses natural day-span queries:
 * - `days between <datetime> and <datetime>`
 * - `days to <datetime>` (the target datetime minus now)
 * @param {TokenParser} self
 * @returns {Expression | null}
 */
export function tryParseDaySpan(self) {
  const kind = self.currentKind();
  if (kind === null || kind.tag !== 'Identifier') {
    return null;
  }
  if (!eqIgnoreAsciiCase(kind._0, 'days')) {
    return null;
  }

  const isBetween = self.peekIdentifierIs('between');
  const isTo = self.peekKind()?.tag === 'To';
  if (!isBetween && !isTo) {
    return null;
  }

  self.advance(); // consume "days"
  self.advance(); // consume "between" or "to"

  const left = self.parseAdditive();
  let difference;
  if (isBetween) {
    self.expect(TokenKind.And);
    const right = self.parseAdditive();
    difference = Expression.binary(left, BinaryOp.Subtract, right);
  } else {
    difference = Expression.binary(left, BinaryOp.Subtract, Expression.Now());
  }

  return Expression.unitConversion(difference, Unit.Duration(DurationUnit.Days));
}

/** @type {Record<string, ComparisonOp>} */
const ORDERING_OPS = {
  DoubleEquals: ComparisonOp.Equal,
  Less: ComparisonOp.Less,
  LessOrEqual: ComparisonOp.LessOrEqual,
  Greater: ComparisonOp.Greater,
  GreaterOrEqual: ComparisonOp.GreaterOrEqual,
  NotEqual: ComparisonOp.NotEqual,
};

/**
 * @param {TokenParser} self
 * @returns {ComparisonOp | null}
 */
export function matchOrderingOp(self) {
  const kind = self.currentKind();
  if (kind === null || !Object.hasOwn(ORDERING_OPS, kind.tag)) {
    return null;
  }
  self.advance();
  return ORDERING_OPS[kind.tag];
}

/** @param {TokenParser} self */
export function checkCompare(self) {
  return self.currentKind()?.tag === 'Compare';
}

/** @param {TokenParser} self */
export function checkVs(self) {
  return self.currentKind()?.tag === 'Vs';
}
