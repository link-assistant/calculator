/**
 * Binary operator matching (port of `src/grammar/token_parser/operators.rs`).
 */

import { BinaryOp } from '../../types/index.js';
import { TokenKind } from '../lexer.js';
import { TokenParser } from '../token_parser.js';

/** @param {TokenParser} self */
export function percentStartsBinaryExpression(self) {
  if (self.check(TokenKind.PercentWord)) {
    return false;
  }
  const current = self.current();
  if (current !== null && current.text !== '%') {
    return true;
  }
  const peek = self.peekKind();
  if (peek !== null && peek.tag === 'Identifier' && TokenParser.ratioKeyword(peek._0)) {
    return false;
  }
  if (peek !== null && peek.tag === 'Number' && self.pos > 0) {
    const percent = self.tokens[self.pos];
    if (percent.start === self.tokens[self.pos - 1].end
      && self.tokens[self.pos + 1].start > percent.end) {
      return false;
    }
  }
  return peek !== null
    && (peek.tag === 'Number' || peek.tag === 'Identifier' || peek.tag === 'LeftParen');
}

/**
 * @param {TokenParser} self
 * @returns {BinaryOp | null}
 */
export function matchAdditiveOp(self) {
  if (self.check(TokenKind.Plus)) {
    self.advance();
    return BinaryOp.Add;
  } else if (self.check(TokenKind.Minus)) {
    self.advance();
    return BinaryOp.Subtract;
  }
  return null;
}

/**
 * @param {TokenParser} self
 * @returns {BinaryOp | null}
 */
export function matchMultiplicativeOp(self) {
  if (self.check(TokenKind.Star)) {
    self.advance();
    return BinaryOp.Multiply;
  } else if (self.check(TokenKind.Slash)) {
    self.advance();
    return BinaryOp.Divide;
  } else if (self.check(TokenKind.Percent) && self.percentStartsBinaryExpression()) {
    self.advance();
    return BinaryOp.Modulo;
  }
  return null;
}
