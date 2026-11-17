/**
 * Natural integral notation (port of `src/grammar/token_parser/integral.rs`).
 */

import { CalculatorError } from '../../error.js';
import { err } from '../../rust/core.js';
import { debugOption } from '../../rust/fmt.js';
import { isAsciiAlphabetic } from '../../rust/str.js';
import { BinaryOp, Expression } from '../../types/index.js';
import { TokenKind } from '../lexer.js';
import { isMathFunction } from '../math_functions.js';
import { TokenParser } from '../token_parser.js';

/**
 * Parses natural integral notation: "integrate <expr> d<var>".
 *
 * Examples include `integrate sin(x)/x dx` and `integrate x^2 dx`.
 * @param {TokenParser} self
 * @returns {Expression}
 */
export function parseNaturalIntegral(self) {
  const startPos = self.pos;
  /** @type {number | null} */
  let integrandEndPos = null;
  /** @type {string | null} */
  let varName = null;

  // Scan forward to find differential notation such as `dx`, `dy`, or `dt`.
  for (let scanPos = self.pos; scanPos < self.tokens.length; scanPos++) {
    const kind = self.tokens[scanPos].kind;
    if (kind.tag === 'Identifier') {
      const idLower = kind._0.toLowerCase();
      // `id_lower.len() == 2` counts UTF-8 bytes; starting with ASCII 'd'
      // and having an ASCII second char means two code units here.
      const chars = [...idLower];
      if (chars[0] === 'd' && chars.length === 2 && isAsciiAlphabetic(chars[1])) {
        integrandEndPos = scanPos;
        varName = chars[1];
        break;
      }
    }
  }

  if (integrandEndPos === null || varName === null) {
    return err(CalculatorError.parse(
      'Invalid integration syntax. Expected: integrate <expression> d<var> (e.g., integrate sin(x)/x dx)',
    ));
  }

  self.pos = startPos;
  const integrand = self.parseIntegrandUntil(integrandEndPos);

  self.pos = integrandEndPos;
  self.advance();

  return Expression.indefiniteIntegral(integrand, varName);
}

/**
 * Parse an integrand expression up to (but not including) `untilPos`.
 * @param {TokenParser} self
 * @param {number} untilPos
 * @returns {Expression}
 */
export function parseIntegrandUntil(self, untilPos) {
  const result = self.parseIntegrandExpression(untilPos);
  if (self.pos > untilPos) {
    self.pos = untilPos;
  }
  return result;
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandExpression(self, boundary) {
  return self.parseIntegrandAdditive(boundary);
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandAdditive(self, boundary) {
  let left = self.parseIntegrandMultiplicative(boundary);

  while (self.pos < boundary) {
    const op = self.matchAdditiveOp();
    if (op === null) {
      break;
    }
    if (self.pos >= boundary) {
      self.pos -= 1;
      break;
    }
    const right = self.parseIntegrandMultiplicative(boundary);
    left = Expression.binary(left, op, right);
  }

  return left;
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandMultiplicative(self, boundary) {
  let left = self.parseIntegrandPower(boundary);

  while (self.pos < boundary) {
    const op = self.matchMultiplicativeOp();
    if (op !== null) {
      if (self.pos >= boundary) {
        self.pos -= 1;
        break;
      }
      const right = self.parseIntegrandPower(boundary);
      left = Expression.binary(left, op, right);
    } else if (self.implicitMultiplicationStarts(left)) {
      const right = self.parseIntegrandPower(boundary);
      left = Expression.binary(left, BinaryOp.Multiply, right);
    } else {
      break;
    }
  }

  return left;
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandPower(self, boundary) {
  let left = self.parseIntegrandUnary(boundary);

  if (self.pos < boundary && self.check(TokenKind.Caret)) {
    self.advance();
    const right = self.parseIntegrandPower(boundary);
    left = Expression.power(left, right);
  } else if (self.pos < boundary) {
    const kind = self.currentKind();
    if (kind !== null && kind.tag === 'Superscript') {
      const exponent = self.numberGrammar.parseNumber(kind._0);
      self.advance();
      left = Expression.power(left, Expression.number(exponent));
    }
  }

  return left;
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandUnary(self, boundary) {
  if (self.pos < boundary && self.check(TokenKind.Minus)) {
    self.advance();
    const expr = self.parseIntegrandUnary(boundary);
    return Expression.negate(expr);
  }

  return self.parseIntegrandPrimary(boundary);
}

/** @param {TokenParser} self @param {number} boundary @returns {Expression} */
export function parseIntegrandPrimary(self, boundary) {
  if (self.pos >= boundary) {
    return err(CalculatorError.parse('Unexpected end of integrand'));
  }

  if (self.check(TokenKind.LeftParen)) {
    self.advance();
    const expr = self.parseExpression();
    self.expect(TokenKind.RightParen);
    return Expression.group(expr);
  }

  const kind = self.currentKind();
  if (kind !== null && kind.tag === 'Number') {
    const number = kind._0;
    self.advance();
    const value = self.numberGrammar.parseNumber(number);
    return Expression.number(value);
  }

  if (kind !== null && kind.tag === 'Identifier') {
    const id = kind._0;
    self.advance();

    if (self.pos < boundary && self.check(TokenKind.LeftParen)) {
      if (TokenParser.isSingleLetterVariable(id) && !isMathFunction(id)) {
        return Expression.variable(id);
      }
      return self.parseFunctionCall(id);
    }

    if (TokenParser.isMathConstant(id)) {
      return Expression.functionCall(id, []);
    }
    if (TokenParser.isUnaryMathFunction(id) && self.pos < boundary) {
      const argument = self.parseIntegrandUnary(boundary);
      return Expression.functionCall(id, [argument]);
    }
    if (isMathFunction(id)) {
      return Expression.functionCall(id, []);
    }
    return Expression.variable(id);
  }

  return err(CalculatorError.parse(
    `Unexpected token in integrand: ${debugOption(self.current(), (t) => t.fmtDebug())}`,
  ));
}
