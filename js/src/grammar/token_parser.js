/**
 * Token-based expression parser.
 */

import { CalculatorError } from '../error.js';
import { err, ok, tryResult } from '../rust/core.js';
import { debugOption } from '../rust/fmt.js';
import { charCount, eqIgnoreAsciiCase, isAsciiAlphabetic, toAsciiLowercase } from '../rust/str.js';
import { BinaryOp, CurrencyDatabase, DateTime, Expression, Unit } from '../types/index.js';
import { DateTimeGrammar } from './datetime_grammar.js';
import { TokenKind } from './lexer.js';
/** @typedef {import('./lexer.js').Token} Token */
import { isMathFunction } from './math_functions.js';
import { NumberGrammar } from './number_grammar.js';
import * as comparison from './token_parser/comparison.js';
import * as datetime from './token_parser/datetime.js';
import { isOrdinalSuffix } from './token_parser/datetime.js';
import * as integral from './token_parser/integral.js';
import * as numbers from './token_parser/numbers.js';
import * as operators from './token_parser/operators.js';
import * as percentage from './token_parser/percentage.js';
import * as quantities from './token_parser/quantities.js';
import * as units from './token_parser/units.js';

/** @typedef {import('../types/decimal.js').Decimal} Decimal */
/** @typedef {import('../types/expression.js').ComparisonOp} ComparisonOp */

/** Internal token-based parser. */
export class TokenParser {
  /**
   * @param {{ tokens: Token[], pos: number, numberGrammar: NumberGrammar, originalInput: string }} fields
   */
  constructor({ tokens, pos, numberGrammar, originalInput }) {
    this.tokens = tokens;
    this.pos = pos;
    this.numberGrammar = numberGrammar;
    this.originalInput = originalInput;
  }

  /**
   * @param {Token[]} tokens
   * @param {NumberGrammar} numberGrammar
   * @param {string} originalInput
   * @returns {TokenParser}
   */
  static new(tokens, numberGrammar, originalInput) {
    return new TokenParser({
      tokens,
      pos: 0,
      numberGrammar,
      originalInput,
    });
  }

  /** Throws `Err(CalculatorError)`. @returns {Expression} */
  parseCompleteExpression() {
    const expr = this.parseExpression();

    if (this.isAtEnd()) {
      return expr;
    }

    const token = /** @type {Token} */ (this.current());
    return err(CalculatorError.parse(
      `Unexpected trailing input '${token.text}' at position ${token.start}`,
    ));
  }

  /** @returns {Expression} */
  parseAdditive() {
    let left = this.parseMultiplicative();

    let op;
    while ((op = this.matchAdditiveOp()) !== null) {
      const right = this.parseMultiplicative();
      left = Expression.binary(left, op, right);
    }

    // Check for "at" keyword
    if (this.checkAt()) {
      this.advance(); // consume "at"
      const time = this.parsePrimary();
      left = Expression.atTime(left, time);
    }

    const ratio = this.parseRatioSuffix(left);
    if (ratio !== null) {
      return ratio;
    }

    // Check for "as", "in", or "to" keyword (unit conversion, e.g. "741 KB as MB", "19 TON in USD")
    if (this.checkAs() || this.checkIn() || this.checkTo()) {
      this.advance(); // consume "as"/"in"/"to"
      const targetUnit = this.parseUnitForConversion();

      // Resolve unit ambiguity using conversion context.
      // If the source has an ambiguous unit (e.g., "ton" parsed as Mass with Currency
      // alternative) and the target unit category doesn't match the primary but matches
      // an alternative, swap to the alternative interpretation.
      left = TokenParser.resolveUnitAmbiguityForConversion(left, targetUnit);

      left = Expression.unitConversion(left, targetUnit);

      // Check for "at" keyword after unit conversion (e.g. "22822 RUB in INR at Apr 11, 2026")
      if (this.checkAt()) {
        this.advance(); // consume "at"
        const time = this.parsePrimary();
        left = Expression.atTime(left, time);
      }
    }

    return left;
  }

  /** @returns {Expression} */
  parseMultiplicative() {
    let left = this.parsePower();

    for (;;) {
      const op = this.matchMultiplicativeOp();
      if (op !== null) {
        const right = this.parsePower();
        left = this.combineRatioProduct(left, op, right);
      } else if (this.check(TokenKind.Of)
        && !(left.tag === 'Number' && left.unit.tag !== 'None')
        && !this.peekIdentifierIs('what')) {
        this.advance();
        const right = this.parsePower();
        left = Expression.functionCall(
          'percent_of',
          [Expression.binary(left, BinaryOp.Multiply, right)],
        );
      } else if (this.check(TokenKind.Portion)) {
        this.advance();
        const part = this.parsePower();
        left = Expression.functionCall(
          'percent_of',
          [Expression.binary(left, BinaryOp.Multiply, part)],
        );
      } else if (this.implicitMultiplicationStarts(left)) {
        const right = this.parsePower();
        left = Expression.binary(left, BinaryOp.Multiply, right);
      } else {
        break;
      }
    }

    return left;
  }

  /** @returns {Expression} */
  parsePower() {
    // Unary minus binds looser than `^`, as in standard notation and in
    // most calculators: -2^2 = -(2^2) = -4, while 2^-1 = 0.5 still works
    // because the exponent is parsed by this same rule.
    if (this.check(TokenKind.Minus)) {
      this.advance();
      const operand = this.parsePower();
      return Expression.negate(operand);
    }

    let left = this.parseUnary();

    // Power is right-associative: 2^3^4 = 2^(3^4)
    const kind = this.currentKind();
    if (this.check(TokenKind.Caret)) {
      this.advance();
      const right = this.parsePower(); // Right-associative recursion
      left = Expression.power(left, right);
    } else if (kind !== null && kind.tag === 'Superscript') {
      const exponent = this.numberGrammar.parseNumber(kind._0);
      this.advance();
      left = Expression.power(left, Expression.number(exponent));
    }

    return left;
  }

  /** @returns {Expression} */
  parseUnary() {
    if (this.check(TokenKind.Minus)) {
      this.advance();
      const expr = this.parseUnary();
      return Expression.negate(expr);
    }

    let expr = this.parsePrimary();
    expr = this.parsePercentPostfix(expr);

    // Handle postfix factorial operator: expr! → factorial(expr)
    if (this.currentKind()?.tag === 'Bang') {
      this.advance();
      return Expression.functionCall('factorial', [expr]);
    }

    return expr;
  }

  /** @returns {Expression} */
  parsePrimary() {
    if (this.check(TokenKind.PercentPrefix) || this.check(TokenKind.Percent)) {
      this.advance();
      const amount = this.parsePower();
      return Expression.functionCall('percent', [amount]);
    }
    if (this.check(TokenKind.IncreaseFrom) || this.check(TokenKind.DecreaseFrom)) {
      const op = this.check(TokenKind.IncreaseFrom)
        ? BinaryOp.Add
        : BinaryOp.Subtract;
      this.advance();
      const amount = this.parseMultiplicative();
      this.expectRatioWord('from');
      const base = this.parseMultiplicative();
      return Expression.binary(base, op, amount);
    }
    // Handle "until" keyword: "until <datetime>"
    if (this.checkUntil()) {
      this.advance(); // consume "until"
      // Try to parse the rest as a datetime directly by collecting tokens
      const savePos = this.pos;
      const until = tryResult(() => this.tryParseUntilTarget());
      if (until.ok) {
        return Expression.Until(until.value);
      }
      // Fallback: parse as normal expression
      this.pos = savePos;
      const target = this.parsePrimary();
      return Expression.Until(target);
    }

    // Parenthesized expression
    if (this.check(TokenKind.LeftParen)) {
      this.advance();
      const expr = this.parseExpression();
      this.expect(TokenKind.RightParen);
      return Expression.group(expr);
    }

    // Placeholder unknowns for single-variable equations.
    if (this.check(TokenKind.Question)) {
      this.advance();
      return Expression.variable('?');
    }
    if (this.check(TokenKind.Star)) {
      this.advance();
      return Expression.variable('*');
    }

    // Numeric date literal (e.g. 2026-01-22, 15/10/2025, 15.10.2025).
    // The lexer already validated that this parses as a real calendar date.
    let kind = this.currentKind();
    if (kind !== null && kind.tag === 'DateLiteral') {
      const s = kind._0;
      this.advance();

      // A time may follow the date literal, as in "2026-08-08 22:35" or
      // "15.10.2025 22:35 UTC". Try to absorb it, falling back to the
      // bare date when the combined form does not parse.
      const next = this.currentKind();
      if (next !== null && (next.tag === 'Number' || next.tag === 'Identifier')) {
        const afterLiteral = this.pos;
        const dt = ok(() => this.tryParseDatetimeFromTokens(s));
        if (dt !== null) {
          return dt;
        }
        this.pos = afterLiteral;
      }

      return Expression.DateTime(DateTime.parse(s));
    }

    // Number with optional unit
    if (kind !== null && kind.tag === 'Number') {
      const numStr = kind._0;
      const numberEnd = this.current()?.end ?? 0;
      const savePos = this.pos;
      this.advance();

      // If followed by a Colon, this might be a time like "11:59pm EST on Monday, January 26th"
      // Try collecting all remaining tokens as a datetime string first.
      if (this.currentKind()?.tag === 'Colon') {
        const dt = ok(() => this.tryParseTimeStartingWithNumber(numStr));
        if (dt !== null) {
          return dt;
        }
        // Datetime parse failed — restore position and fall through to plain number
        this.pos = savePos;
        this.advance(); // re-consume the number token
      }

      // If followed by AM/PM, this is a time like "6 PM", "6 PM GMT", "6 PM MSK"
      // Try to parse as datetime (with optional timezone) before treating as unit.
      kind = this.currentKind();
      if (kind !== null && kind.tag === 'Identifier') {
        const idLower = kind._0.toLowerCase();
        if (idLower === 'am' || idLower === 'pm') {
          const saveBeforeAmpm = this.pos;
          const dt = ok(() => this.tryParseTimeWithAmpm(numStr));
          if (dt !== null) {
            return dt;
          }
          // Datetime parse failed — restore to before AM/PM
          this.pos = saveBeforeAmpm;
        }
      }

      // Russian shorthand: "11 по мск" means "11:00 by MSK".
      if (this.currentKind()?.tag === 'At') {
        const saveBeforePo = this.pos;
        const dt = ok(() => this.tryParseRussianTimeByTimezone(numStr));
        if (dt !== null) {
          return dt;
        }
        this.pos = saveBeforePo;
      }

      // If followed by an identifier that looks like a month name (e.g., "17 February 2027"
      // or "17 февраля 2027"), try to parse the whole as a datetime expression.
      kind = this.currentKind();
      if (kind !== null && kind.tag === 'Identifier') {
        const id = kind._0;
        const isOrdinal = isOrdinalSuffix(id);
        if (DateTimeGrammar.looksLikeDatetime(id) || isOrdinal) {
          const saveBeforeDt = this.pos;
          const dt = ok(() => this.tryParseDatetimeFromTokens(numStr));
          if (dt !== null) {
            return dt;
          }
          // Datetime parse failed — restore position and fall through to plain number
          this.pos = saveBeforeDt;
        }
      }

      let value = this.numberGrammar.parseNumber(numStr);
      let multiplier = this.consumeAdjacentSiSuffix(numberEnd);
      if (multiplier !== null) {
        value = value.checkedMul(multiplier) ?? err(CalculatorError.Overflow);
      }

      multiplier = this.consumeWordScale();
      if (multiplier !== null) {
        value = value.checkedMul(multiplier) ?? err(CalculatorError.Overflow);
      }
      const [unit, alternativeUnits] = this.parseQuantityUnit(numberEnd) ?? [Unit.None, []];
      const quantity = alternativeUnits.length === 0
        ? Expression.numberWithUnit(value, unit)
        : Expression.numberWithUnitAlternatives(value, unit, alternativeUnits);
      return this.parseCompoundQuantity(quantity);
    }

    // Standalone identifier (could be a function call, unit, variable, or datetime part)
    if (kind !== null && kind.tag === 'Identifier') {
      const id = kind._0;

      const fn = this.naturalRootFunction(id);
      if (fn !== null) {
        this.advance(); // consume "square" / "cube"
        this.advance(); // consume "root"
        this.advance(); // consume "of"
        const argument = this.parseUnary();
        return Expression.functionCall(fn, [argument]);
      }

      // Check for "now" keyword
      if (id.toLowerCase() === 'now') {
        this.advance();
        // Check for optional timezone after "now" (e.g., "now UTC", "now EST")
        const next = this.currentKind();
        if (next !== null && next.tag === 'Identifier') {
          const tz = next._0;
          const tzLower = tz.toLowerCase();
          // Collect "now" + timezone as a combined datetime string
          if (ok(() => DateTime.parse(`now ${tzLower}`)) !== null) {
            const combined = `now ${tz}`;
            this.advance();
            const dt = ok(() => DateTime.parse(combined));
            return dt !== null ? Expression.DateTime(dt) : Expression.Now();
          }
        }
        return Expression.Now();
      }

      // Resolve "today" at evaluation time so long-lived calculator
      // instances and configured local timezones use the current date.
      if (eqIgnoreAsciiCase(id, 'today')) {
        this.advance();
        return Expression.Today();
      }

      // Check for prefix currency symbol notation (e.g., $10, €5, £3).
      if (charCount(id) === 1) {
        const ch = [...id][0];
        if (!isAsciiAlphabetic(ch)) {
          const currencyCode = CurrencyDatabase.parseCurrency(id);
          if (currencyCode !== null) {
            if (this.peekKind()?.tag === 'Number') {
              this.advance(); // consume currency symbol
              const current = this.currentKind();
              if (current !== null && current.tag === 'Number') {
                const numStr = current._0;
                this.advance();
                const value = this.numberGrammar.parseNumber(numStr);
                return Expression.numberWithUnit(
                  value,
                  Unit.currency(currencyCode),
                );
              }
            }
          }
        }
      }

      this.advance();

      // Check if this is a function call (identifier followed by left paren).
      // `x(x - 3)` is conventional implicit polynomial multiplication.
      if (this.check(TokenKind.LeftParen)) {
        if (TokenParser.isSingleLetterVariable(id) && !isMathFunction(id)) {
          return Expression.variable(id);
        }
        return this.parseFunctionCall(id);
      }

      // Check for natural integration syntax: "integrate <expr> d<var>"
      if (id.toLowerCase() === 'integrate') {
        return this.parseNaturalIntegral();
      }

      // If it looks like a datetime start (month name, "time", "current", etc.), try to parse more
      if (DateTimeGrammar.looksLikeDatetime(id)) {
        return this.tryParseDatetimeFromTokens(id);
      }

      if (TokenParser.isMathConstant(id)) {
        return Expression.functionCall(id, []);
      }
      if (TokenParser.isUnaryMathFunction(id) && !this.isAtEnd()) {
        const argument = this.parseUnary();
        return Expression.functionCall(id, [argument]);
      }
      if (isMathFunction(id)) {
        return Expression.functionCall(id, []);
      }

      // Allow single-letter identifiers as variables (for use in integrate, etc.)
      // Variables will be validated at evaluation time
      if (TokenParser.isSingleLetterVariable(id)) {
        return Expression.variable(id);
      }

      // Otherwise it's probably just an identifier/unit (which is an error in expression context)
      return err(CalculatorError.parse(`Unexpected identifier: ${id}`));
    }

    return err(CalculatorError.parse(
      `Unexpected token: ${debugOption(this.current(), (t) => t.fmtDebug())}`,
    ));
  }

  /**
   * @param {number} numberEnd
   * @returns {Decimal | null}
   */
  consumeAdjacentSiSuffix(numberEnd) {
    const token = this.current();
    if (token === null || token.start !== numberEnd || token.kind.tag !== 'Identifier') {
      return null;
    }
    const suffix = token.kind._0;
    const multiplier = NumberGrammar.siSuffixMultiplier(suffix);
    if (multiplier === null) {
      return null;
    }

    // Preserve established adjacent unit abbreviations like `2h in minutes`.
    // They become SI suffixes only in suffix-before-unit forms like `5h USD`.
    // Of the physical units only metre (`5m`) wins over its prefix (issue
    // #235), so `5T` and `5G` keep meaning tera and giga.
    const suffixIsUnit = suffix === 'm' || this.identifierIsLegacyUnit(suffix);
    if (suffixIsUnit && !this.peekIdentifierIsKnownUnit()) {
      return null;
    }

    this.advance();
    return multiplier;
  }

  /** @param {string} id */
  identifierIsKnownUnit(id) {
    const parsed = ok(() => this.numberGrammar.parseUnitWithAlternatives(id));
    return parsed !== null && parsed[0].tag !== 'Custom' && parsed[0].tag !== 'None';
  }

  /** @param {string} id */
  identifierIsLegacyUnit(id) {
    const parsed = ok(() => this.numberGrammar.parseUnitWithAlternatives(id));
    return parsed !== null
      && parsed[0].tag !== 'Custom' && parsed[0].tag !== 'None' && parsed[0].tag !== 'Physical';
  }

  peekIdentifierIsKnownUnit() {
    const kind = this.peekKind();
    if (kind !== null && kind.tag === 'Identifier' && !isMathFunction(kind._0)) {
      return this.identifierIsKnownUnit(kind._0);
    }
    return false;
  }

  /** @param {string} name @returns {Expression} */
  parseFunctionCall(name) {
    // We're positioned at the left paren
    this.expect(TokenKind.LeftParen);

    /** @type {Expression[]} */
    const args = [];

    // Check for empty argument list
    if (!this.check(TokenKind.RightParen)) {
      // Parse first argument
      args.push(this.parseExpression());

      // Parse remaining arguments
      while (this.check(TokenKind.Comma)) {
        this.advance(); // consume comma
        args.push(this.parseExpression());
      }
    }

    this.expect(TokenKind.RightParen);

    return Expression.functionCall(name, args);
  }

  /** See `token_parser/datetime.js`. @returns {Expression} */
  tryParseUntilTarget() {
    return datetime.tryParseUntilTarget(this);
  }

  /**
   * See `token_parser/datetime.js`.
   * @param {string} hourStr
   * @returns {Expression}
   */
  tryParseTimeStartingWithNumber(hourStr) {
    return datetime.tryParseTimeStartingWithNumber(this, hourStr);
  }

  /**
   * See `token_parser/datetime.js`.
   * @param {string} hourStr
   * @returns {Expression}
   */
  tryParseTimeWithAmpm(hourStr) {
    return datetime.tryParseTimeWithAmpm(this, hourStr);
  }

  /**
   * See `token_parser/datetime.js`.
   * @param {string} hourStr
   * @returns {Expression}
   */
  tryParseRussianTimeByTimezone(hourStr) {
    return datetime.tryParseRussianTimeByTimezone(this, hourStr);
  }

  /**
   * See `token_parser/datetime.js`.
   * @param {string} first
   * @returns {Expression}
   */
  tryParseDatetimeFromTokens(first) {
    return datetime.tryParseDatetimeFromTokens(this, first);
  }

  /** @param {TokenKind} kind */
  check(kind) {
    const current = this.currentKind();
    return current !== null && current.tag === kind.tag;
  }

  checkAt() {
    return this.currentKind()?.tag === 'At';
  }

  checkAs() {
    return this.currentKind()?.tag === 'As';
  }

  checkIn() {
    return this.currentKind()?.tag === 'In';
  }

  checkTo() {
    return this.currentKind()?.tag === 'To';
  }

  checkUntil() {
    return this.currentKind()?.tag === 'Until';
  }

  /** @returns {Token | null} */
  current() {
    return this.tokens[this.pos] ?? null;
  }

  /** @returns {TokenKind | null} */
  currentKind() {
    return this.current()?.kind ?? null;
  }

  /** @returns {TokenKind | null} */
  peekKind() {
    return this.tokens[this.pos + 1]?.kind ?? null;
  }

  /**
   * `matches!(self.peek_kind(), Some(TokenKind::Identifier(w)) if w.eq_ignore_ascii_case(word))`.
   * @param {string} word
   */
  peekIdentifierIs(word) {
    const kind = this.peekKind();
    return kind !== null && kind.tag === 'Identifier' && eqIgnoreAsciiCase(kind._0, word);
  }

  peekIsLeftParen() {
    return this.peekKind()?.tag === 'LeftParen';
  }

  /** @param {Expression} left */
  implicitMultiplicationStarts(left) {
    const leftCanMultiply = (left.tag === 'Number' && left.unit.tag === 'None')
      || left.tag === 'Group'
      || left.tag === 'FunctionCall'
      || left.tag === 'Variable'
      || left.tag === 'Power'
      || left.tag === 'Negate';
    if (!leftCanMultiply) {
      return false;
    }

    const kind = this.currentKind();
    switch (kind?.tag) {
      case 'LeftParen':
        return true;
      case 'Identifier':
        return isMathFunction(kind._0) || TokenParser.isSingleLetterVariable(kind._0);
      case 'Number':
        return left.tag === 'Group' || left.tag === 'FunctionCall';
      default:
        return false;
    }
  }

  /**
   * @param {string} degree
   * @returns {string | null}
   */
  naturalRootFunction(degree) {
    let fn;
    if (eqIgnoreAsciiCase(degree, 'square')) {
      fn = 'sqrt';
    } else if (eqIgnoreAsciiCase(degree, 'cube')) {
      fn = 'cbrt';
    } else {
      return null;
    }
    const root = this.tokens[this.pos + 1]?.kind;
    const hasRoot = root !== undefined && root.tag === 'Identifier' && eqIgnoreAsciiCase(root._0, 'root');
    const hasOf = this.tokens[this.pos + 2]?.kind.tag === 'Of';

    return hasRoot && hasOf ? fn : null;
  }

  /** @param {string} id */
  static isSingleLetterVariable(id) {
    const chars = [...id];
    return chars.length === 1 && isAsciiAlphabetic(chars[0]);
  }

  /** @param {string} id */
  static isMathConstant(id) {
    const lower = toAsciiLowercase(id);
    return lower === 'pi' || lower === 'e';
  }

  /** @param {string} id */
  static isUnaryMathFunction(id) {
    return UNARY_MATH_FUNCTIONS.has(toAsciiLowercase(id));
  }

  advance() {
    if (!this.isAtEnd()) {
      this.pos += 1;
    }
  }

  isAtEnd() {
    const kind = this.currentKind();
    return kind === null || kind.tag === 'Eof';
  }

  /** Throws `Err(CalculatorError)`. @param {TokenKind} kind */
  expect(kind) {
    if (this.check(kind)) {
      this.advance();
      return;
    }
    err(CalculatorError.unexpectedToken(
      debugOption(this.currentKind(), (k) => k.fmtDebug()),
      kind.fmtDebug(),
      this.pos,
    ));
  }

  // --- comparison.rs ---

  /** Throws `Err(CalculatorError)`. @returns {Expression} */
  parseExpression() {
    return comparison.parseExpression(this);
  }

  /** @returns {Expression} */
  parseComparison() {
    return comparison.parseComparison(this);
  }

  /** @returns {Expression | null} */
  tryParseDaySpan() {
    return comparison.tryParseDaySpan(this);
  }

  /** @returns {ComparisonOp | null} */
  matchOrderingOp() {
    return comparison.matchOrderingOp(this);
  }

  checkCompare() {
    return comparison.checkCompare(this);
  }

  checkVs() {
    return comparison.checkVs(this);
  }

  // --- integral.rs ---

  /** @returns {Expression} */
  parseNaturalIntegral() {
    return integral.parseNaturalIntegral(this);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandUntil(boundary) {
    return integral.parseIntegrandUntil(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandExpression(boundary) {
    return integral.parseIntegrandExpression(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandAdditive(boundary) {
    return integral.parseIntegrandAdditive(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandMultiplicative(boundary) {
    return integral.parseIntegrandMultiplicative(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandPower(boundary) {
    return integral.parseIntegrandPower(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandUnary(boundary) {
    return integral.parseIntegrandUnary(this, boundary);
  }

  /** @param {number} boundary @returns {Expression} */
  parseIntegrandPrimary(boundary) {
    return integral.parseIntegrandPrimary(this, boundary);
  }

  // --- numbers.rs ---

  /** @returns {Decimal | null} */
  consumeWordScale() {
    return numbers.consumeWordScale(this);
  }

  // --- operators.rs ---

  percentStartsBinaryExpression() {
    return operators.percentStartsBinaryExpression(this);
  }

  /** @returns {BinaryOp | null} */
  matchAdditiveOp() {
    return operators.matchAdditiveOp(this);
  }

  /** @returns {BinaryOp | null} */
  matchMultiplicativeOp() {
    return operators.matchMultiplicativeOp(this);
  }

  // --- percentage.rs ---

  hasProportionQuestion() {
    return percentage.hasProportionQuestion(this);
  }

  /** @param {string} word */
  static ratioKeyword(word) {
    return percentage.ratioKeyword(word);
  }

  /** @param {string} word */
  ratioWord(word) {
    return percentage.ratioWord(this, word);
  }

  /** @param {string} word */
  expectRatioWord(word) {
    percentage.expectRatioWord(this, word);
  }

  /** @param {Expression} expression @returns {Expression} */
  parsePercentPostfix(expression) {
    return percentage.parsePercentPostfix(this, expression);
  }

  /** @param {Expression} left @param {BinaryOp} op @param {Expression} right @returns {Expression} */
  combineRatioProduct(left, op, right) {
    return percentage.combineRatioProduct(this, left, op, right);
  }

  /** @param {Expression} left @returns {Expression | null} */
  parseRatioSuffix(left) {
    return percentage.parseRatioSuffix(this, left);
  }

  /** @returns {string | null} */
  consumeRatioFormat() {
    return percentage.consumeRatioFormat(this);
  }

  /** @param {Expression} left @param {string} format @returns {Expression} */
  finishRatioFormat(left, format) {
    return percentage.finishRatioFormat(this, left, format);
  }

  // --- quantities.rs ---

  /** @param {number} numberEnd @returns {[Unit, Unit[]] | null} */
  parseQuantityUnit(numberEnd) {
    return quantities.parseQuantityUnit(this, numberEnd);
  }

  /** @param {string} id */
  identifierIsDimensionalUnit(id) {
    return quantities.identifierIsDimensionalUnit(this, id);
  }

  /** @param {Expression} first @returns {Expression} */
  parseCompoundQuantity(first) {
    return quantities.parseCompoundQuantity(this, first);
  }

  /** @returns {Unit | null} */
  parsePhysicalConversionTarget() {
    return quantities.parsePhysicalConversionTarget(this);
  }

  conversionTargetEnds() {
    return quantities.conversionTargetEnds(this);
  }

  /** @returns {Unit | null} */
  inchBeforeConversion() {
    return quantities.inchBeforeConversion(this);
  }

  /** @returns {Array<[string, number]> | null} */
  parseUnitTerms() {
    return quantities.parseUnitTerms(this);
  }

  /** @returns {Array<[string, number]> | null} */
  parseUnitGroup() {
    return quantities.parseUnitGroup(this);
  }

  /** @returns {Array<[string, number]> | null} */
  parseUnitFactor() {
    return quantities.parseUnitFactor(this);
  }

  /** @returns {import('../types/physical.js').PhysicalUnit | null} */
  parseUnitName() {
    return quantities.parseUnitName(this);
  }

  /** @returns {number | null} */
  parseUnitPower() {
    return quantities.parseUnitPower(this);
  }

  /** @returns {number | null} */
  parseSignedInteger() {
    return quantities.parseSignedInteger(this);
  }

  // --- units.rs ---

  /** Throws `Err(CalculatorError)`. @returns {Unit} */
  parseUnitForConversion() {
    return units.parseUnitForConversion(this);
  }

  /** @param {Expression} expr @param {Unit} targetUnit @returns {Expression} */
  static resolveUnitAmbiguityForConversion(expr, targetUnit) {
    return units.resolveUnitAmbiguityForConversion(expr, targetUnit);
  }
}

const UNARY_MATH_FUNCTIONS = new Set([
  'sin', 'cos', 'tan', 'asin', 'acos', 'atan', 'sinh', 'cosh', 'tanh', 'exp', 'ln', 'log',
  'log2', 'log10', 'sqrt', 'cbrt', 'abs', 'floor', 'ceil', 'round', 'trunc', 'sign', 'signum',
  'factorial', 'deg', 'degrees', 'rad', 'radians',
]);
