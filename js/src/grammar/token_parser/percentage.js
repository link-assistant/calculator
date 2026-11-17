/**
 * Percentage syntax lowered into exact ratio functions and ordinary
 * arithmetic (port of `src/grammar/token_parser/percentage.rs`).
 */

import { CalculatorError } from '../../error.js';
import { eq, err, tryResult } from '../../rust/core.js';
import { eqIgnoreAsciiCase, toAsciiLowercase } from '../../rust/str.js';
import { BinaryOp, Decimal, Expression } from '../../types/index.js';
import { TokenKind } from '../lexer.js';
import { divideIntoQuantity } from './quantities.js';

/** @typedef {import('../token_parser.js').TokenParser} TokenParser */

/** @param {TokenParser} self */
export function hasProportionQuestion(self) {
  const tokens = self.tokens;
  for (let i = self.pos; i + 2 < tokens.length; i++) {
    const [a, b, c] = [tokens[i].kind, tokens[i + 1].kind, tokens[i + 2].kind];
    if (a.tag === 'Comma'
      && b.tag === 'Identifier' && eqIgnoreAsciiCase(b._0, 'what')
      && c.tag === 'Identifier' && eqIgnoreAsciiCase(c._0, 'is')) {
      return true;
    }
  }
  return false;
}

/** @param {string} word */
export function ratioKeyword(word) {
  const lower = toAsciiLowercase(word);
  return lower === 'is' || lower === 'what' || lower === 'on' || lower === 'off' || lower === 'from';
}

/** @param {TokenParser} self @param {string} word */
export function ratioWord(self, word) {
  const kind = self.currentKind();
  return kind !== null && kind.tag === 'Identifier' && eqIgnoreAsciiCase(kind._0, word);
}

/** Throws `Err(CalculatorError)`. @param {TokenParser} self @param {string} word */
export function expectRatioWord(self, word) {
  if (!self.ratioWord(word)) {
    err(CalculatorError.parse(`Expected '${word}' in ratio expression`));
  }
  self.advance();
}

/**
 * @param {TokenParser} self
 * @param {Expression} expression
 * @returns {Expression}
 */
export function parsePercentPostfix(self, expression) {
  while ((self.check(TokenKind.Percent) || self.check(TokenKind.PercentWord))
    && !self.percentStartsBinaryExpression()) {
    self.advance();
    expression = Expression.functionCall('percent', [expression]);
  }
  if (expression.tag === 'FunctionCall' && expression.name === 'percent'
    && (self.check(TokenKind.Of) || self.ratioWord('on') || self.ratioWord('off'))) {
    let op;
    if (self.ratioWord('on')) {
      op = BinaryOp.Add;
    } else if (self.ratioWord('off')) {
      op = BinaryOp.Subtract;
    } else {
      op = BinaryOp.Multiply;
    }
    // Leave inverse queries to parse_ratio_suffix: `20 is 10% of what`.
    if (self.peekIdentifierIs('what')) {
      return expression;
    }
    self.advance();
    const base = self.parsePower();
    const result = op === BinaryOp.Multiply
      ? Expression.binary(expression, op, base)
      : Expression.binary(base, op, expression);
    return Expression.functionCall('percent_of', [result]);
  }
  return expression;
}

/**
 * @param {TokenParser} self
 * @param {Expression} left
 * @param {BinaryOp} op
 * @param {Expression} right
 * @returns {Expression}
 */
export function combineRatioProduct(self, left, op, right) {
  // Soulver's spaced trailing ratio form: `20/200 %` means `(20/200) as %`.
  // Adjacent `20/200%` keeps standard division by a percentage.
  if (op === BinaryOp.Divide && self.pos >= 2) {
    const percent = self.tokens[self.pos - 1];
    const previous = self.tokens[self.pos - 2];
    if (percent.kind.tag === 'Percent' && percent.start > previous.end) {
      if (right.tag === 'FunctionCall' && right.name === 'percent' && right.args.length === 1) {
        return Expression.functionCall(
          'as_percent',
          [Expression.binary(left, op, right.args[0])],
        );
      }
    }
  }
  if (op === BinaryOp.Divide) {
    return divideIntoQuantity(left, right);
  }
  return Expression.binary(left, op, right);
}

/**
 * Throws `Err(CalculatorError)`.
 * @param {TokenParser} self
 * @param {Expression} left
 * @returns {Expression | null}
 */
export function parseRatioSuffix(self, left) {
  const start = self.pos;
  // `3 in 20 is what %` and `50 to 75 is what %` precede unit conversion.
  if (self.check(TokenKind.In) || self.check(TokenKind.To)) {
    const change = self.check(TokenKind.To);
    self.advance();
    const parsed = tryResult(() => self.parseMultiplicative());
    if (parsed.ok) {
      const right = parsed.value;
      if (self.ratioWord('is') || self.check(TokenKind.As)) {
        self.advance();
        if (self.ratioWord('what')) {
          self.advance();
        }
        const format = self.consumeRatioFormat();
        if (format !== null) {
          let ratio = Expression.binary(
            change ? right : left,
            BinaryOp.Divide,
            change ? left : right,
          );
          if (change && format === 'as_percent') {
            ratio = Expression.binary(
              ratio,
              BinaryOp.Subtract,
              Expression.number(Decimal.new(1)),
            );
          }
          return Expression.functionCall(format, [ratio]);
        }
      }
    }
    self.pos = start;
  }
  if (self.checkAs() || self.checkTo()) {
    self.advance();
    if (self.ratioWord('a')) {
      self.advance();
    }
    const format = self.consumeRatioFormat();
    if (format !== null) {
      return self.finishRatioFormat(left, format);
    }
    self.pos = start;
  }
  if (self.ratioWord('is')) {
    self.advance();
    if (self.ratioWord('what')) {
      self.advance();
      const format = self.consumeRatioFormat()
        ?? err(CalculatorError.parse('Expected a percentage or multiplier'));
      return self.finishRatioFormat(left, format);
    }
    const amount = self.parseMultiplicative();
    // Proportions: `20% is 500, what is 750` and `if 20 is 30%, what is 60%`.
    if (self.check(TokenKind.Comma) && (left.isPercentage() || amount.isPercentage())) {
      self.advance();
      self.expectRatioWord('what');
      self.expectRatioWord('is');
      const target = self.parseAdditive();
      const result = Expression.binary(
        Expression.binary(left, BinaryOp.Multiply, target),
        BinaryOp.Divide,
        amount,
      );
      return left.isPercentage()
        ? Expression.functionCall('as_percent', [result])
        : result;
    }
    // `5% is 1 in what` formats odds without changing the numeric value.
    if (self.check(TokenKind.In)) {
      self.advance();
      self.expectRatioWord('what');
      if (!eq(amount, Expression.number(Decimal.new(1)))) {
        return err(CalculatorError.parse("Expected 'is 1 in what'"));
      }
      return Expression.functionCall('as_reciprocal', [left]);
    }
    let factor;
    if (self.check(TokenKind.Of)) {
      factor = amount;
    } else if (self.ratioWord('on') || self.ratioWord('off')) {
      const op = self.ratioWord('on') ? BinaryOp.Add : BinaryOp.Subtract;
      // Multiplication clears percent display, ensuring point arithmetic in the factor.
      const scalar = Expression.binary(
        amount,
        BinaryOp.Multiply,
        Expression.number(Decimal.new(1)),
      );
      factor = Expression.binary(Expression.number(Decimal.new(1)), op, scalar);
    } else {
      return err(CalculatorError.parse("Expected 'of', 'on', or 'off'"));
    }
    self.advance();
    self.expectRatioWord('what');
    return Expression.binary(left, BinaryOp.Divide, factor);
  }
  return null;
}

/**
 * @param {TokenParser} self
 * @returns {string | null}
 */
export function consumeRatioFormat(self) {
  let format;
  if (self.check(TokenKind.Percent) || self.check(TokenKind.PercentWord)) {
    format = 'as_percent';
  } else if (self.ratioWord('fraction')) {
    format = 'as_fraction';
  } else if (self.ratioWord('x') || self.ratioWord('multiplier') || self.ratioWord('multiple')) {
    format = 'as_multiplier';
  } else if (self.ratioWord('dec') || self.ratioWord('decimal')) {
    format = 'as_decimal';
  } else {
    return null;
  }
  self.advance();
  return format;
}

/**
 * Throws `Err(CalculatorError)`.
 * @param {TokenParser} self
 * @param {Expression} left
 * @param {string} format
 * @returns {Expression}
 */
export function finishRatioFormat(self, left, format) {
  let ratio;
  if (self.check(TokenKind.Of) || self.ratioWord('on') || self.ratioWord('off')) {
    const off = self.ratioWord('off');
    const change = off || self.ratioWord('on');
    self.advance();
    const base = self.parseMultiplicative();
    ratio = Expression.binary(left, BinaryOp.Divide, base);
    if (change) {
      const one = Expression.number(Decimal.new(1));
      ratio = off
        ? Expression.binary(one, BinaryOp.Subtract, ratio)
        : Expression.binary(ratio, BinaryOp.Subtract, one);
    }
  } else {
    ratio = left;
  }
  return Expression.functionCall(format, [ratio]);
}
