/**
 * Lexer for tokenizing calculator input.
 *
 * The input is an array of one-code-point strings (`Vec<char>`), so token
 * `start`/`end` and the "at position N" of the errors are char indexes, as
 * in Rust.
 */

import * as arithmeticWords from './arithmetic_words.js';
import { isMathFunction } from './math_functions.js';
import * as localeNumbers from './locale_numbers.js';
import { DateTimeGrammar } from './datetime_grammar.js';
import { CalculatorError } from '../error.js';
import { DateTime } from '../types/datetime.js';
import { err, isOk } from '../rust/core.js';
import { debugStruct, debugTuple } from '../rust/fmt.js';
import { debugStr, isAlphabetic, isAlphanumeric, isAsciiDigit, isWhitespace } from '../rust/str.js';

/**
 * Checks if a character is a Unicode combining mark (General Category M).
 *
 * This includes:
 * - Mn (Mark, Nonspacing) — e.g., Devanagari virama ्, Arabic fathah  َ
 * - Mc (Mark, Spacing Combining) — e.g., Devanagari dependent vowels ा ि ी
 * - Me (Mark, Enclosing) — rare, e.g., combining enclosing circle ⃝
 *
 * These characters are integral parts of words in scripts like Devanagari (Hindi),
 * Arabic, and Thai, but are not classified as `is_alphabetic()` in Rust.
 * @param {string} ch
 * @returns {boolean}
 */
function isUnicodeMark(ch) {
  // Unicode General Category "M" (Mark) covers Mn, Mc, and Me.
  // We check the ranges for Devanagari (U+0900-U+097F marks), Arabic (U+0610-U+065F),
  // and other common combining mark blocks.
  // Using a broad approach: if it's not alphanumeric, not whitespace, not ASCII,
  // and not a punctuation/symbol, it's likely a combining mark.
  // More precisely, we check the Unicode categories directly.
  const category = unicodeGeneralCategory(ch);
  return category === GeneralCategory.Mn || category === GeneralCategory.Mc || category === GeneralCategory.Me;
}

/**
 * Minimal Unicode General Category detection for combining marks.
 *
 * Only distinguishes Mark categories (Mn, Mc, Me) from everything else (Other).
 * This avoids pulling in a full Unicode data crate for a focused need.
 */
class GeneralCategory {
  /** @param {string} tag */
  constructor(tag) {
    this.tag = tag;
    Object.freeze(this);
  }

  /** Nonspacing Mark (Mn) */
  static Mn = new GeneralCategory('Mn');

  /** Spacing Combining Mark (Mc) */
  static Mc = new GeneralCategory('Mc');

  /** Enclosing Mark (Me) */
  static Me = new GeneralCategory('Me');

  /** Any other category */
  static Other = new GeneralCategory('Other');
}

/**
 * Characters that continue a word: alphanumerics, `_` and combining marks.
 * @param {string} ch
 * @returns {boolean}
 */
export function isWordChar(ch) {
  return isAlphanumeric(ch) || ch === '_' || isUnicodeMark(ch);
}

/**
 * @param {string} ch
 * @returns {string | null}
 */
function superscriptValue(ch) {
  switch (ch) {
    case '⁰': return '0';
    case '¹': return '1';
    case '²': return '2';
    case '³': return '3';
    case '⁴': return '4';
    case '⁵': return '5';
    case '⁶': return '6';
    case '⁷': return '7';
    case '⁸': return '8';
    case '⁹': return '9';
    case '⁺': return '+';
    case '⁻': return '-';
    default: return null;
  }
}

/**
 * `lo..=hi` contains `cp`.
 * @param {number} cp
 * @param {number} lo
 * @param {number} hi
 */
function inRange(cp, lo, hi) {
  return cp >= lo && cp <= hi;
}

/**
 * Returns the Unicode General Category for combining mark detection.
 *
 * Covers the most common combining mark ranges needed for multilingual input:
 * Devanagari, Arabic, Bengali, Gurmukhi, Gujarati, Tamil, Telugu, Kannada,
 * Malayalam, Thai, and other Indic/Southeast Asian scripts.
 * @param {string} ch
 * @returns {GeneralCategory}
 */
function unicodeGeneralCategory(ch) {
  const cp = /** @type {number} */ (ch.codePointAt(0));
  // Devanagari (U+0900–U+097F)
  if (inRange(cp, 0x0900, 0x0902)
    || cp === 0x093A
    || cp === 0x093C
    || inRange(cp, 0x0941, 0x0948)
    || cp === 0x094D
    || inRange(cp, 0x0951, 0x0957)
    || inRange(cp, 0x0962, 0x0963)) {
    return GeneralCategory.Mn;
  }
  if (cp === 0x0903 || cp === 0x093B || inRange(cp, 0x093E, 0x0940) || inRange(cp, 0x0949, 0x094C)
    || inRange(cp, 0x094E, 0x094F) || inRange(cp, 0x0982, 0x0983)) {
    return GeneralCategory.Mc;
  }

  // Arabic combining marks (U+0610–U+065F, U+06D6–U+06ED, U+08D3–U+08FF)
  if (inRange(cp, 0x0610, 0x061A)
    || inRange(cp, 0x064B, 0x065F)
    || cp === 0x0670
    || inRange(cp, 0x06D6, 0x06DC)
    || inRange(cp, 0x06DF, 0x06E4)
    || inRange(cp, 0x06E7, 0x06E8)
    || inRange(cp, 0x06EA, 0x06ED)
    || inRange(cp, 0x08D3, 0x08FF)) {
    return GeneralCategory.Mn;
  }

  // Bengali (U+0980–U+09FF)
  if (cp === 0x09BC || inRange(cp, 0x09C1, 0x09C4) || cp === 0x09CD || inRange(cp, 0x09E2, 0x09E3)) {
    return GeneralCategory.Mn;
  }
  if (inRange(cp, 0x09BE, 0x09C0) || inRange(cp, 0x09CB, 0x09CC) || cp === 0x09D7) {
    return GeneralCategory.Mc;
  }

  // Gurmukhi (U+0A00–U+0A7F)
  if (inRange(cp, 0x0A01, 0x0A02)
    || cp === 0x0A3C
    || inRange(cp, 0x0A41, 0x0A42)
    || inRange(cp, 0x0A47, 0x0A48)
    || inRange(cp, 0x0A4B, 0x0A4D)
    || cp === 0x0A51
    || inRange(cp, 0x0A70, 0x0A71)
    || cp === 0x0A75) {
    return GeneralCategory.Mn;
  }
  if (cp === 0x0A03 || inRange(cp, 0x0A3E, 0x0A40) || cp === 0x0A83) {
    return GeneralCategory.Mc;
  }

  // General combining marks (U+0300–U+036F: Combining Diacritical Marks)
  if (inRange(cp, 0x0300, 0x036F)) {
    return GeneralCategory.Mn;
  }

  // Combining Diacritical Marks Extended (U+1AB0–U+1AFF)
  if (inRange(cp, 0x1AB0, 0x1ACE)) {
    return GeneralCategory.Mn;
  }

  // Combining Diacritical Marks Supplement (U+1DC0–U+1DFF)
  if (inRange(cp, 0x1DC0, 0x1DFF)) {
    return GeneralCategory.Mn;
  }

  // Combining Half Marks (U+FE20–U+FE2F)
  if (inRange(cp, 0xFE20, 0xFE2F)) {
    return GeneralCategory.Mn;
  }

  // Enclosing marks (U+20DD–U+20E0, U+20E2–U+20E4)
  if (inRange(cp, 0x20DD, 0x20E0) || inRange(cp, 0x20E2, 0x20E4)) {
    return GeneralCategory.Me;
  }

  return GeneralCategory.Other;
}

/** Token kinds in the calculator grammar. */
export class TokenKind {
  /** @param {string} tag @param {{ _0: string }} [fields] */
  constructor(tag, fields) {
    /** @type {string} */
    this.tag = tag;
    if (fields !== undefined) {
      /** The payload of the tuple variants. @type {string} */
      this._0 = fields._0;
    }
    Object.freeze(this);
  }

  /**
   * The derived `Debug` (`{:?}`): `Eof`, `Number("42")`.
   * @returns {string}
   */
  fmtDebug() {
    return Object.hasOwn(this, '_0') ? debugTuple(this.tag, [debugStr(this._0)]) : this.tag;
  }

  /**
   * A number (integer or decimal).
   * @param {string} value
   * @returns {TokenKind}
   */
  static Number(value) { return new TokenKind('Number', { _0: value }); }

  /**
   * A Unicode superscript exponent (for example, `³` or `⁻³`).
   * @param {string} value
   * @returns {TokenKind}
   */
  static Superscript(value) { return new TokenKind('Superscript', { _0: value }); }

  /**
   * A numeric date literal (e.g. `2026-01-22`, `15/10/2025`, `15.10.2025`).
   *
   * Recognized as a single token so dates do not get split into separate
   * numbers and `-`/`/` operators by the arithmetic grammar.
   * @param {string} value
   * @returns {TokenKind}
   */
  static DateLiteral(value) { return new TokenKind('DateLiteral', { _0: value }); }

  /**
   * An identifier (variable name, currency code, etc.).
   * @param {string} value
   * @returns {TokenKind}
   */
  static Identifier(value) { return new TokenKind('Identifier', { _0: value }); }

  /** The plus operator. */
  static Plus = new TokenKind('Plus');

  /** The minus operator. */
  static Minus = new TokenKind('Minus');

  /** The multiplication operator. */
  static Star = new TokenKind('Star');

  /** The question mark placeholder unknown for equations. */
  static Question = new TokenKind('Question');

  /** The division operator. */
  static Slash = new TokenKind('Slash');

  /** The power/exponent operator. */
  static Caret = new TokenKind('Caret');

  /** The percent operator (e.g., `3%` means `0.03`). */
  static Percent = new TokenKind('Percent');

  /** A word meaning percent (distinct from modulo operator words). */
  static PercentWord = new TokenKind('PercentWord');

  /** A percentage written before its amount. */
  static PercentPrefix = new TokenKind('PercentPrefix');

  /** A part of a whole, with the whole on the left (CJK phrasing). */
  static Portion = new TokenKind('Portion');

  /** A leading percentage increase applied to a following base. */
  static IncreaseFrom = new TokenKind('IncreaseFrom');

  /** A leading percentage decrease applied to a following base. */
  static DecreaseFrom = new TokenKind('DecreaseFrom');

  /** Left parenthesis. */
  static LeftParen = new TokenKind('LeftParen');

  /** Right parenthesis. */
  static RightParen = new TokenKind('RightParen');

  /** A colon (for time). */
  static Colon = new TokenKind('Colon');

  /** A value with an absolute uncertainty (`±`). */
  static PlusMinus = new TokenKind('PlusMinus');

  /** A comma. */
  static Comma = new TokenKind('Comma');

  /** The "at" keyword for temporal context. */
  static At = new TokenKind('At');

  /** The "as" keyword for unit conversion (e.g., "741 KB as MB"). */
  static As = new TokenKind('As');

  /** The "in" keyword for unit conversion (e.g., "19 TON in USD"). */
  static In = new TokenKind('In');

  /** The "to" keyword for unit conversion (e.g., "19 TON to USD"). */
  static To = new TokenKind('To');

  /** The "until" keyword for duration until a datetime. */
  static Until = new TokenKind('Until');

  /** The "of" keyword for percent-of expressions (e.g., `8% of $50`). */
  static Of = new TokenKind('Of');

  /** The "and" keyword for natural comparison forms. */
  static And = new TokenKind('And');

  /** The "compare" keyword for natural comparison forms. */
  static Compare = new TokenKind('Compare');

  /** The "vs" / "versus" keyword for natural comparison forms. */
  static Vs = new TokenKind('Vs');

  /** The equals sign for equality checks (e.g., `1 + 1 = 2`). */
  static Equals = new TokenKind('Equals');

  /** The explicit equality comparison operator (`==`). */
  static DoubleEquals = new TokenKind('DoubleEquals');

  /** The not-equals comparison operator (`!=`). */
  static NotEqual = new TokenKind('NotEqual');

  /** The less-than comparison operator (`<`). */
  static Less = new TokenKind('Less');

  /** The less-than-or-equal comparison operator (`<=`). */
  static LessOrEqual = new TokenKind('LessOrEqual');

  /** The greater-than comparison operator (`>`). */
  static Greater = new TokenKind('Greater');

  /** The greater-than-or-equal comparison operator (`>=`). */
  static GreaterOrEqual = new TokenKind('GreaterOrEqual');

  /** The exclamation mark for factorial postfix notation (e.g., `5!`). */
  static Bang = new TokenKind('Bang');

  /** End of input. */
  static Eof = new TokenKind('Eof');
}

/** A token with its position in the input. */
export class Token {
  /** @param {{ kind: TokenKind, start: number, end: number, text: string }} fields */
  constructor({ kind, start, end, text }) {
    /** The kind of token. */
    this.kind = kind;
    /** Start position in the input. */
    this.start = start;
    /** End position in the input. */
    this.end = end;
    /** The original text of the token. */
    this.text = text;
    Object.freeze(this);
  }

  /**
   * Creates a new token.
   * @param {TokenKind} kind
   * @param {number} start
   * @param {number} end
   * @param {string} text
   * @returns {Token}
   */
  static new(kind, start, end, text) {
    return new Token({ kind, start, end, text });
  }

  /**
   * Checks if this token is a number.
   * @returns {boolean}
   */
  isNumber() {
    return this.kind.tag === 'Number';
  }

  /**
   * Checks if this token is an identifier.
   * @returns {boolean}
   */
  isIdentifier() {
    return this.kind.tag === 'Identifier';
  }

  /**
   * The derived `Debug`: `Token { kind: Eof, start: 5, end: 5, text: "" }`.
   * @returns {string}
   */
  fmtDebug() {
    return debugStruct('Token', [
      ['kind', this.kind.fmtDebug()],
      ['start', String(this.start)],
      ['end', String(this.end)],
      ['text', debugStr(this.text)],
    ]);
  }
}

/** Lexer for tokenizing calculator input. */
export class Lexer {
  /**
   * @param {{ input: string[], pos: number, previousTokenCanEndValue: boolean,
   *   previousIdentifier: string | null, parentheses: boolean[], decimalComma: boolean,
   *   pending: Token[] }} fields
   */
  constructor({ input, pos, previousTokenCanEndValue, previousIdentifier, parentheses, decimalComma, pending }) {
    this.input = input;
    this.pos = pos;
    this.previousTokenCanEndValue = previousTokenCanEndValue;
    this.previousIdentifier = previousIdentifier;
    this.parentheses = parentheses;
    this.decimalComma = decimalComma;
    /** Tokens already scanned from number or operator words. */
    this.pending = pending;
  }

  /**
   * Creates a new lexer for the given input.
   * @param {string} input
   * @returns {Lexer}
   */
  static new(input) {
    return new Lexer({
      input: [...input],
      pos: 0,
      previousTokenCanEndValue: false,
      previousIdentifier: null,
      parentheses: [],
      decimalComma: false,
      pending: [],
    });
  }

  /**
   * @param {string} input
   * @returns {Lexer}
   */
  static withDecimalCommas(input) {
    const lexer = Lexer.new(input);
    lexer.decimalComma = true;
    return lexer;
  }

  /**
   * Tokenizes the entire input; throws `Err(CalculatorError)`.
   * @returns {Token[]}
   */
  tokenize() {
    const tokens = [];

    while (true) {
      const token = this.nextToken();
      const isEof = token.kind.tag === 'Eof';
      tokens.push(token);
      if (isEof) {
        break;
      }
    }

    return tokens;
  }

  /**
   * Returns the next token; throws `Err(CalculatorError)`.
   * @returns {Token}
   */
  nextToken() {
    const token = this.pending.length > 0
      ? /** @type {Token} */ (this.pending.shift())
      : this.scanToken();

    switch (token.kind.tag) {
      case 'LeftParen': {
        const fn = this.previousIdentifier !== null && isMathFunction(this.previousIdentifier);
        this.parentheses.push(fn);
        break;
      }
      case 'RightParen':
        this.parentheses.pop();
        break;
      default:
    }
    this.previousIdentifier = token.kind.tag === 'Identifier' ? token.kind._0 : null;
    this.previousTokenCanEndValue = [
      'Number',
      'Percent',
      'PercentWord',
      'DateLiteral',
      'Identifier',
      'RightParen',
      'Superscript',
      'Bang',
    ].includes(token.kind.tag);

    return token;
  }

  /** @returns {Token} */
  scanToken() {
    this.skipWhitespace();

    if (this.isAtEnd()) {
      return Token.new(TokenKind.Eof, this.pos, this.pos, '');
    }

    const start = this.pos;
    const ch = this.current();

    // Single-character tokens
    /** @type {Token} */
    let token;
    switch (ch) {
      case '±':
        this.advance();
        token = Token.new(TokenKind.PlusMinus, start, this.pos, ch);
        break;
      case '+':
        this.advance();
        token = Token.new(TokenKind.Plus, start, this.pos, '+');
        break;
      case '-': case '−':
        this.advance();
        token = Token.new(TokenKind.Minus, start, this.pos, ch);
        break;
      case '*':
        this.advance();
        if (!this.isAtEnd()
          && this.current() === '*'
          && this.previousTokenCanEndValue
          && this.doubleStarHasRightOperand()) {
          this.advance();
          token = Token.new(TokenKind.Caret, start, this.pos, '**');
        } else {
          token = Token.new(TokenKind.Star, start, this.pos, '*');
        }
        break;
      case '×': case '·': case '⋅':
        this.advance();
        token = Token.new(TokenKind.Star, start, this.pos, ch);
        break;
      case '?':
        this.advance();
        token = Token.new(TokenKind.Question, start, this.pos, '?');
        break;
      case '/': case '÷':
        this.advance();
        token = Token.new(TokenKind.Slash, start, this.pos, ch);
        break;
      case '^':
        this.advance();
        token = Token.new(TokenKind.Caret, start, this.pos, '^');
        break;
      case '%':
        this.advance();
        token = Token.new(TokenKind.Percent, start, this.pos, '%');
        break;
      case '(':
        this.advance();
        token = Token.new(TokenKind.LeftParen, start, this.pos, '(');
        break;
      case ')':
        this.advance();
        token = Token.new(TokenKind.RightParen, start, this.pos, ')');
        break;
      case ':':
        this.advance();
        token = Token.new(TokenKind.Colon, start, this.pos, ':');
        break;
      case ',':
        this.advance();
        token = Token.new(TokenKind.Comma, start, this.pos, ',');
        break;
      case '=':
        this.advance();
        if (!this.isAtEnd() && this.current() === '=') {
          this.advance();
          token = Token.new(TokenKind.DoubleEquals, start, this.pos, '==');
        } else {
          token = Token.new(TokenKind.Equals, start, this.pos, '=');
        }
        break;
      case '!':
        this.advance();
        if (!this.isAtEnd() && this.current() === '=') {
          this.advance();
          token = Token.new(TokenKind.NotEqual, start, this.pos, '!=');
        } else {
          token = Token.new(TokenKind.Bang, start, this.pos, '!');
        }
        break;
      case '≠':
        this.advance();
        token = Token.new(TokenKind.NotEqual, start, this.pos, ch);
        break;
      case '≤':
        this.advance();
        token = Token.new(TokenKind.LessOrEqual, start, this.pos, ch);
        break;
      case '≥':
        this.advance();
        token = Token.new(TokenKind.GreaterOrEqual, start, this.pos, ch);
        break;
      case '<':
        this.advance();
        if (!this.isAtEnd() && this.current() === '=') {
          this.advance();
          token = Token.new(TokenKind.LessOrEqual, start, this.pos, '<=');
        } else {
          token = Token.new(TokenKind.Less, start, this.pos, '<');
        }
        break;
      case '>':
        this.advance();
        if (!this.isAtEnd() && this.current() === '=') {
          this.advance();
          token = Token.new(TokenKind.GreaterOrEqual, start, this.pos, '>=');
        } else {
          token = Token.new(TokenKind.Greater, start, this.pos, '>');
        }
        break;
      default:
        if (isAsciiDigit(ch)) {
          // Prefer a full numeric date literal (e.g. 2026-01-22, 15.10.2025)
          // over splitting it into separate numbers and operators.
          const date = this.tryScanDate();
          if (date !== null) {
            const [text, end] = date;
            token = Token.new(TokenKind.DateLiteral(text), start, end, text);
            this.pos = end;
          } else {
            token = this.scanNumber();
          }
        } else if (ch === '.') {
          token = this.scanNumber();
        } else if (superscriptValue(ch) !== null) {
          token = this.scanSuperscript();
        } else if (ch === '√' || ch === '∛') {
          this.advance();
          const name = ch === '√' ? 'sqrt' : 'cbrt';
          token = Token.new(TokenKind.Identifier(name), start, this.pos, ch);
        } else if (isAlphabetic(ch)) {
          token = this.scanWords() ?? this.scanIdentifier();
        } else if (['°', '℃', '℉', '\'', '′', '"', '″'].includes(ch)) {
          // Degree signs, `℃`/`℉`, and the foot/inch primes of `6'2"` are
          // unit symbols (issue #235); `°C` and `°F` stay one identifier.
          token = this.scanUnitSymbol();
        } else if (['$', '€', '£', '¥', '₽', '₹', '₩', '₿', '₫', '₸'].includes(ch)) {
          // Currency symbols used as prefix notation (e.g., $10, €5, £3)
          // These are recognized as single-character identifiers and mapped to ISO codes
          // by CurrencyDatabase::parse_currency().
          this.advance();
          const symbol = ch;
          token = Token.new(TokenKind.Identifier(symbol), start, this.pos, symbol);
        } else {
          return err(CalculatorError.parse(`Unexpected character '${ch}' at position ${start}`));
        }
    }

    return token;
  }

  /**
   * Number words and operator phrases from `data/words` (issue #222).
   * @returns {Token | null}
   */
  scanWords() {
    const tokens = arithmeticWords.scan(this.input, this.pos);
    if (tokens === null) {
      return null;
    }
    const mapped = tokens.map(([kind, start, end, text]) => Token.new(kind, start, end, text));
    if (mapped.length === 0) {
      return null;
    }
    const first = /** @type {Token} */ (mapped.shift());
    this.pending.push(...mapped);
    this.pos = this.pending.length > 0 ? this.pending[this.pending.length - 1].end : first.end;
    return first;
  }

  /** @returns {boolean} */
  doubleStarHasRightOperand() {
    const character = this.input.slice(this.pos + 1).find((c) => !isWhitespace(c));
    return character !== undefined
      && (isAsciiDigit(character)
        || isAlphabetic(character)
        || ['.', '(', '+', '-', '−', '√', '∛'].includes(character));
  }

  /**
   * Throws `Err(CalculatorError)`.
   * @returns {Token}
   */
  scanNumber() {
    const start = this.pos;
    if (this.current() === '.') {
      const next = this.peek();
      if (!(next !== null && isAsciiDigit(next))) {
        return err(CalculatorError.parse(`Unexpected character '.' at position ${start}`));
      }
    }
    const dateComponent = this.previousIdentifier !== null
      && DateTimeGrammar.looksLikeDatetime(this.previousIdentifier);
    const allowComma = !this.parentheses.includes(true) && !dateComponent;
    let [text, end] = localeNumbers.scan(this.input, start, allowComma, this.decimalComma);
    this.pos = end;

    // Scientific notation is consumed only when the exponent is complete,
    // keeping an adjacent constant such as `2e` available for implicit
    // multiplication rather than accepting a malformed number.
    if (this.current() === 'e' || this.current() === 'E') {
      const exponentMarker = this.pos;
      let exponentPos = exponentMarker + 1;
      if (this.input[exponentPos] === '+' || this.input[exponentPos] === '-') {
        exponentPos += 1;
      }
      if (this.input[exponentPos] !== undefined && isAsciiDigit(this.input[exponentPos])) {
        text += this.current();
        this.advance();
        if (this.current() === '+' || this.current() === '-') {
          text += this.current();
          this.advance();
        }
        while (!this.isAtEnd()) {
          const ch = this.current();
          const next = this.peek();
          if (isAsciiDigit(ch)) {
            text += ch;
            this.advance();
          } else if (ch === '_'
            && text.length > 0
            && isAsciiDigit(text[text.length - 1])
            && next !== null && isAsciiDigit(next)) {
            this.advance();
          } else {
            break;
          }
        }
      }
    }
    end = this.pos;

    const original = this.input.slice(start, this.pos).join('');

    return Token.new(TokenKind.Number(text), start, end, original);
  }

  /**
   * Scans a unit symbol that is not a word: `°`, `°C`, `°F`, `℃`, `℉`,
   * and the primes `'`/`′` (foot) and `"`/`″` (inch).
   * @returns {Token}
   */
  scanUnitSymbol() {
    const start = this.pos;
    const ch = this.current();
    this.advance();
    let text = ch;
    const next = this.peek();
    if (ch === '°'
      && (this.current() === 'C' || this.current() === 'F')
      && !(next !== null && isWordChar(next))) {
      text += this.current();
      this.advance();
    }
    return Token.new(TokenKind.Identifier(text), start, this.pos, text);
  }

  /** @returns {Token} */
  scanSuperscript() {
    const start = this.pos;
    let normalized = '';

    while (!this.isAtEnd()) {
      const value = superscriptValue(this.current());
      if (value === null) {
        break;
      }
      normalized += value;
      this.advance();
    }

    const original = this.input.slice(start, this.pos).join('');
    return Token.new(TokenKind.Superscript(normalized), start, this.pos, original);
  }

  /**
   * Attempts to scan a full numeric date literal starting at the current
   * position, returning the matched text and the end index on success.
   *
   * A date literal is three digit groups joined by a single, consistent
   * separator (`-`, `/`, or `.`), where exactly one of the outer groups is a
   * 4-digit year. The candidate is only accepted when it parses as a real
   * calendar date, so ordinary arithmetic such as `2026 - 1 - 22` (written
   * with spaces) or `1/3` is never swallowed.
   * @returns {[string, number] | null}
   */
  tryScanDate() {
    const len = this.input.length;

    // Scans a run of ASCII digits from `j`, returning the end index.
    const scanDigits = (/** @type {number} */ j) => {
      let end = j;
      while (end < len && isAsciiDigit(this.input[end])) {
        end += 1;
      }
      return end;
    };

    // First digit group.
    const start = this.pos;
    const e1 = scanDigits(start);
    const l1 = e1 - start;
    if (l1 === 0) {
      return null;
    }

    // First separator.
    if (e1 >= len) {
      return null;
    }
    const sep = this.input[e1];
    if (sep !== '-' && sep !== '/' && sep !== '.') {
      return null;
    }

    // Second digit group.
    const e2 = scanDigits(e1 + 1);
    const l2 = e2 - (e1 + 1);
    if (l2 === 0) {
      return null;
    }

    // Second separator must match the first.
    if (e2 >= len || this.input[e2] !== sep) {
      return null;
    }

    // Third digit group.
    const e3 = scanDigits(e2 + 1);
    const l3 = e3 - (e2 + 1);
    if (l3 === 0) {
      return null;
    }

    // Reject anything that continues into a 4th component or more digits
    // (e.g. version strings like `1.2.3.4` or `12.3456`).
    if (e3 < len && (this.input[e3] === sep || isAsciiDigit(this.input[e3]))) {
      return null;
    }

    // Require exactly one 4-digit year in an outer position.
    const yearFirst = l1 === 4 && l2 >= 1 && l2 <= 2 && l3 >= 1 && l3 <= 2;
    const yearLast = l3 === 4 && l1 >= 1 && l1 <= 2 && l2 >= 1 && l2 <= 2;
    if (!yearFirst && !yearLast) {
      return null;
    }

    const candidate = this.input.slice(start, e3).join('');
    if (isOk(() => DateTime.parse(candidate))) {
      return [candidate, e3];
    }
    return null;
  }

  /** @returns {Token} */
  scanIdentifier() {
    const start = this.pos;
    let text = '';

    while (!this.isAtEnd()) {
      const ch = this.current();
      // Accept alphanumeric, underscore, and Unicode combining marks (Mn/Mc/Me).
      // Combining marks are needed for scripts like Devanagari (Hindi) where
      // virama (्, U+094D) and dependent vowels (ा, ि, etc.) are part of words
      // but not classified as alphabetic.
      // Superscripts end a word so that `m²` is `m` squared.
      if ((isAlphanumeric(ch) || ch === '_' || isUnicodeMark(ch))
        && superscriptValue(ch) === null) {
        text += ch;
        this.advance();
      } else {
        break;
      }
    }

    // Abbreviations are commonly written with a trailing dot ("руб.", "кг.", "янв.").
    // Consume that dot when it terminates the word, i.e. when it is not followed by
    // another word character (so "янв.2026" or "3.5" are left untouched).
    const next = this.peek();
    if (text !== ''
      && this.current() === '.'
      && !(next !== null && (isAlphanumeric(next) || next === '_' || isUnicodeMark(next)))) {
      this.advance();
    }

    // Check for keywords (including multilingual equivalents)
    /** @type {TokenKind} */
    let kind;
    switch (text.toLowerCase()) {
      // Operator words ("plus", "multiplied by", "умножить на", …) come
      // from data/words/arithmetic-words.lino via `scan_words`; `mod(`
      // is left to us as the function name.
      case 'mod': case 'modulo': kind = TokenKind.Identifier('mod'); break;
      case 'π': kind = TokenKind.Identifier('pi'); break;
      case 'fact': kind = TokenKind.Identifier('factorial'); break;
      case 'arcsin': kind = TokenKind.Identifier('asin'); break;
      case 'arccos': kind = TokenKind.Identifier('acos'); break;
      case 'arctan': kind = TokenKind.Identifier('atan'); break;
      case 'at': kind = TokenKind.At; break;
      case 'as': kind = TokenKind.As; break;
      case 'in': kind = TokenKind.In; break;
      case 'to': kind = TokenKind.To; break;
      case 'until': kind = TokenKind.Until; break;
      case 'of': kind = TokenKind.Of; break;
      case 'and': kind = TokenKind.And; break;
      case 'compare': kind = TokenKind.Compare; break;
      case 'vs': case 'versus': kind = TokenKind.Vs; break;
      // Russian: "от" means "of/from" in percent-of expressions (e.g. "38% от 100к").
      case 'от': kind = TokenKind.Of; break;
      // Russian: "в" means "in/into" (e.g. "1000 рублей в долларах")
      case 'в': kind = TokenKind.In; break;
      // Russian: "по" means "by/according to" for timezone context (e.g. "11 по мск").
      case 'по': kind = TokenKind.At; break;
      // Russian: "на" means "at/on" for temporal context (e.g. "1000 рублей на 11 апреля 2026")
      case 'на': kind = TokenKind.At; break;
      // French: "en" means "in/into" (e.g. "1000 dollars en euros")
      // Note: German "in" is identical to English "in", no extra entry needed.
      case 'en': kind = TokenKind.In; break;
      // Chinese: conversion phrases (e.g. "1000美元换成欧元")
      case '换成': case '兑换成': case '转换为': case '兑成': case '转为': kind = TokenKind.To; break;
      // Hindi: "में" means "in" as postposition (e.g. "1000 डॉलर में यूरो")
      case 'में': kind = TokenKind.In; break;
      // Arabic: "إلى" means "to/into" (e.g. "1000 دولار إلى يورو")
      case 'إلى': kind = TokenKind.To; break;
      default: kind = TokenKind.Identifier(text);
    }

    return Token.new(kind, start, this.pos, text);
  }

  skipWhitespace() {
    while (!this.isAtEnd() && isWhitespace(this.current())) {
      this.advance();
    }
  }

  /** @returns {string} */
  current() {
    return this.input[this.pos] ?? '\0';
  }

  /** @returns {string | null} */
  peek() {
    return this.input[this.pos + 1] ?? null;
  }

  advance() {
    this.pos += 1;
  }

  /** @returns {boolean} */
  isAtEnd() {
    return this.pos >= this.input.length;
  }
}
