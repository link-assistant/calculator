// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=78e452da3f383ca86b295d8898d1216ee5e44e376b9ea70e7f0834715714f3bd bytes=30985

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Lexer for tokenizing calculator input.
// |  *
// |  * The input is an array of one-code-point strings (`Vec<char>`), so token
// |  * `start`/`end` and the "at position N" of the errors are char indexes, as
// |  * in Rust.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import * as arithmeticWords from './arithmetic_words.js';
// meta-language:translated JavaScript import_statement items=1 sha256=ad41c3b947a8989a16616db7163faaff3eb2d67d9f72c9103f56205735fac111
// | import { isMathFunction } from './math_functions.js';
use crate::grammar::math_functions::isMathFunction;
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as localeNumbers from './locale_numbers.js';
// meta-language:translated JavaScript import_statement items=1 sha256=56c1478162f05e40ea70a94d1cda8081d36a0940d9b7b6496f6357122e0f51fa
// | import { DateTimeGrammar } from './datetime_grammar.js';
use crate::grammar::datetime_grammar::DateTimeGrammar;
// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=ce67731667957b72fc0dc16b353642398c1e6444c9bbd86c52b8a0595324a9b0
// | import { DateTime } from '../types/datetime.js';
use crate::types::datetime::DateTime;
// meta-language:translated JavaScript import_statement items=1 sha256=f27af09684b1ee0316fcdae1d69b10239b29621e43246d267e4ed0d0ddcfb9f1
// | import { err, isOk } from '../rust/core.js';
use crate::rust::core_::{err, isOk};
// meta-language:translated JavaScript import_statement items=1 sha256=ed0c33f0945575ce66bd32d6f90adaee2df0fac7a1ee6cba0c39326bfa9a4520
// | import { debugStruct, debugTuple } from '../rust/fmt.js';
use crate::rust::fmt::{debugStruct, debugTuple};
// meta-language:translated JavaScript import_statement items=1 sha256=1f994514e6b42c54cfa40ed474d5de9deeb93c1f56514fa7c2495d92b657e787
// | import { debugStr, isAlphabetic, isAlphanumeric, isAsciiDigit, isWhitespace } from '../rust/str.js';
use crate::rust::str_::{debugStr, isAlphabetic, isAlphanumeric, isAsciiDigit, isWhitespace};

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Checks if a character is a Unicode combining mark (General Category M).
// |  *
// |  * This includes:
// |  * - Mn (Mark, Nonspacing) — e.g., Devanagari virama ्, Arabic fathah  َ
// |  * - Mc (Mark, Spacing Combining) — e.g., Devanagari dependent vowels ा ि ी
// |  * - Me (Mark, Enclosing) — rare, e.g., combining enclosing circle ⃝
// |  *
// |  * These characters are integral parts of words in scripts like Devanagari (Hindi),
// |  * Arabic, and Thai, but are not classified as `is_alphabetic()` in Rust.
// |  * @param {string} ch
// |  * @returns {boolean}
// |  */
// | function isUnicodeMark(ch) {
// |   // Unicode General Category "M" (Mark) covers Mn, Mc, and Me.
// |   // We check the ranges for Devanagari (U+0900-U+097F marks), Arabic (U+0610-U+065F),
// |   // and other common combining mark blocks.
// |   // Using a broad approach: if it's not alphanumeric, not whitespace, not ASCII,
// |   // and not a punctuation/symbol, it's likely a combining mark.
// |   // More precisely, we check the Unicode categories directly.
// |   const category = unicodeGeneralCategory(ch);
// |   return category === GeneralCategory.Mn || category === GeneralCategory.Mc || category === GeneralCategory.Me;
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | /**
// |  * Minimal Unicode General Category detection for combining marks.
// |  *
// |  * Only distinguishes Mark categories (Mn, Mc, Me) from everything else (Other).
// |  * This avoids pulling in a full Unicode data crate for a focused need.
// |  */
// | class GeneralCategory {
// |   /** @param {string} tag */
// |   constructor(tag) {
// |     this.tag = tag;
// |     Object.freeze(this);
// |   }
// |
// |   /** Nonspacing Mark (Mn) */
// |   static Mn = new GeneralCategory('Mn');
// |
// |   /** Spacing Combining Mark (Mc) */
// |   static Mc = new GeneralCategory('Mc');
// |
// |   /** Enclosing Mark (Me) */
// |   static Me = new GeneralCategory('Me');
// |
// |   /** Any other category */
// |   static Other = new GeneralCategory('Other');
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * Characters that continue a word: alphanumerics, `_` and combining marks.
// |  * @param {string} ch
// |  * @returns {boolean}
// |  */
// | export function isWordChar(ch) {
// |   return isAlphanumeric(ch) || ch === '_' || isUnicodeMark(ch);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} ch
// |  * @returns {string | null}
// |  */
// | function superscriptValue(ch) {
// |   switch (ch) {
// |     case '⁰': return '0';
// |     case '¹': return '1';
// |     case '²': return '2';
// |     case '³': return '3';
// |     case '⁴': return '4';
// |     case '⁵': return '5';
// |     case '⁶': return '6';
// |     case '⁷': return '7';
// |     case '⁸': return '8';
// |     case '⁹': return '9';
// |     case '⁺': return '+';
// |     case '⁻': return '-';
// |     default: return null;
// |   }
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=e4f0ccf7151a29de7649877c382a11e7fa85eeca01a8f32114947898dea43206
// | /**
// |  * `lo..=hi` contains `cp`.
// |  * @param {number} cp
// |  * @param {number} lo
// |  * @param {number} hi
// |  */
// | function inRange(cp, lo, hi) {
// |   return cp >= lo && cp <= hi;
// | }
pub fn in_range(cp: f64, lo: f64, hi: f64) -> bool {
    ((cp >= lo) && (cp <= hi))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Returns the Unicode General Category for combining mark detection.
// |  *
// |  * Covers the most common combining mark ranges needed for multilingual input:
// |  * Devanagari, Arabic, Bengali, Gurmukhi, Gujarati, Tamil, Telugu, Kannada,
// |  * Malayalam, Thai, and other Indic/Southeast Asian scripts.
// |  * @param {string} ch
// |  * @returns {GeneralCategory}
// |  */
// | function unicodeGeneralCategory(ch) {
// |   const cp = /** @type {number} */ (ch.codePointAt(0));
// |   // Devanagari (U+0900–U+097F)
// |   if (inRange(cp, 0x0900, 0x0902)
// |     || cp === 0x093A
// |     || cp === 0x093C
// |     || inRange(cp, 0x0941, 0x0948)
// |     || cp === 0x094D
// |     || inRange(cp, 0x0951, 0x0957)
// |     || inRange(cp, 0x0962, 0x0963)) {
// |     return GeneralCategory.Mn;
// |   }
// |   if (cp === 0x0903 || cp === 0x093B || inRange(cp, 0x093E, 0x0940) || inRange(cp, 0x0949, 0x094C)
// |     || inRange(cp, 0x094E, 0x094F) || inRange(cp, 0x0982, 0x0983)) {
// |     return GeneralCategory.Mc;
// |   }
// |
// |   // Arabic combining marks (U+0610–U+065F, U+06D6–U+06ED, U+08D3–U+08FF)
// |   if (inRange(cp, 0x0610, 0x061A)
// |     || inRange(cp, 0x064B, 0x065F)
// |     || cp === 0x0670
// |     || inRange(cp, 0x06D6, 0x06DC)
// |     || inRange(cp, 0x06DF, 0x06E4)
// |     || inRange(cp, 0x06E7, 0x06E8)
// |     || inRange(cp, 0x06EA, 0x06ED)
// |     || inRange(cp, 0x08D3, 0x08FF)) {
// |     return GeneralCategory.Mn;
// |   }
// |
// |   // Bengali (U+0980–U+09FF)
// |   if (cp === 0x09BC || inRange(cp, 0x09C1, 0x09C4) || cp === 0x09CD || inRange(cp, 0x09E2, 0x09E3)) {
// |     return GeneralCategory.Mn;
// |   }
// |   if (inRange(cp, 0x09BE, 0x09C0) || inRange(cp, 0x09CB, 0x09CC) || cp === 0x09D7) {
// |     return GeneralCategory.Mc;
// |   }
// |
// |   // Gurmukhi (U+0A00–U+0A7F)
// |   if (inRange(cp, 0x0A01, 0x0A02)
// |     || cp === 0x0A3C
// |     || inRange(cp, 0x0A41, 0x0A42)
// |     || inRange(cp, 0x0A47, 0x0A48)
// |     || inRange(cp, 0x0A4B, 0x0A4D)
// |     || cp === 0x0A51
// |     || inRange(cp, 0x0A70, 0x0A71)
// |     || cp === 0x0A75) {
// |     return GeneralCategory.Mn;
// |   }
// |   if (cp === 0x0A03 || inRange(cp, 0x0A3E, 0x0A40) || cp === 0x0A83) {
// |     return GeneralCategory.Mc;
// |   }
// |
// |   // General combining marks (U+0300–U+036F: Combining Diacritical Marks)
// |   if (inRange(cp, 0x0300, 0x036F)) {
// |     return GeneralCategory.Mn;
// |   }
// |
// |   // Combining Diacritical Marks Extended (U+1AB0–U+1AFF)
// |   if (inRange(cp, 0x1AB0, 0x1ACE)) {
// |     return GeneralCategory.Mn;
// |   }
// |
// |   // Combining Diacritical Marks Supplement (U+1DC0–U+1DFF)
// |   if (inRange(cp, 0x1DC0, 0x1DFF)) {
// |     return GeneralCategory.Mn;
// |   }
// |
// |   // Combining Half Marks (U+FE20–U+FE2F)
// |   if (inRange(cp, 0xFE20, 0xFE2F)) {
// |     return GeneralCategory.Mn;
// |   }
// |
// |   // Enclosing marks (U+20DD–U+20E0, U+20E2–U+20E4)
// |   if (inRange(cp, 0x20DD, 0x20E0) || inRange(cp, 0x20E2, 0x20E4)) {
// |     return GeneralCategory.Me;
// |   }
// |
// |   return GeneralCategory.Other;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Token kinds in the calculator grammar. */
// | export class TokenKind {
// |   /** @param {string} tag @param {{ _0: string }} [fields] */
// |   constructor(tag, fields) {
// |     /** @type {string} */
// |     this.tag = tag;
// |     if (fields !== undefined) {
// |       /** The payload of the tuple variants. @type {string} */
// |       this._0 = fields._0;
// |     }
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * The derived `Debug` (`{:?}`): `Eof`, `Number("42")`.
// |    * @returns {string}
// |    */
// |   fmtDebug() {
// |     return Object.hasOwn(this, '_0') ? debugTuple(this.tag, [debugStr(this._0)]) : this.tag;
// |   }
// |
// |   /**
// |    * A number (integer or decimal).
// |    * @param {string} value
// |    * @returns {TokenKind}
// |    */
// |   static Number(value) { return new TokenKind('Number', { _0: value }); }
// |
// |   /**
// |    * A Unicode superscript exponent (for example, `³` or `⁻³`).
// |    * @param {string} value
// |    * @returns {TokenKind}
// |    */
// |   static Superscript(value) { return new TokenKind('Superscript', { _0: value }); }
// |
// |   /**
// |    * A numeric date literal (e.g. `2026-01-22`, `15/10/2025`, `15.10.2025`).
// |    *
// |    * Recognized as a single token so dates do not get split into separate
// |    * numbers and `-`/`/` operators by the arithmetic grammar.
// |    * @param {string} value
// |    * @returns {TokenKind}
// |    */
// |   static DateLiteral(value) { return new TokenKind('DateLiteral', { _0: value }); }
// |
// |   /**
// |    * An identifier (variable name, currency code, etc.).
// |    * @param {string} value
// |    * @returns {TokenKind}
// |    */
// |   static Identifier(value) { return new TokenKind('Identifier', { _0: value }); }
// |
// |   /** The plus operator. */
// |   static Plus = new TokenKind('Plus');
// |
// |   /** The minus operator. */
// |   static Minus = new TokenKind('Minus');
// |
// |   /** The multiplication operator. */
// |   static Star = new TokenKind('Star');
// |
// |   /** The question mark placeholder unknown for equations. */
// |   static Question = new TokenKind('Question');
// |
// |   /** The division operator. */
// |   static Slash = new TokenKind('Slash');
// |
// |   /** The power/exponent operator. */
// |   static Caret = new TokenKind('Caret');
// |
// |   /** The percent operator (e.g., `3%` means `0.03`). */
// |   static Percent = new TokenKind('Percent');
// |
// |   /** A word meaning percent (distinct from modulo operator words). */
// |   static PercentWord = new TokenKind('PercentWord');
// |
// |   /** A percentage written before its amount. */
// |   static PercentPrefix = new TokenKind('PercentPrefix');
// |
// |   /** A part of a whole, with the whole on the left (CJK phrasing). */
// |   static Portion = new TokenKind('Portion');
// |
// |   /** A leading percentage increase applied to a following base. */
// |   static IncreaseFrom = new TokenKind('IncreaseFrom');
// |
// |   /** A leading percentage decrease applied to a following base. */
// |   static DecreaseFrom = new TokenKind('DecreaseFrom');
// |
// |   /** Left parenthesis. */
// |   static LeftParen = new TokenKind('LeftParen');
// |
// |   /** Right parenthesis. */
// |   static RightParen = new TokenKind('RightParen');
// |
// |   /** A colon (for time). */
// |   static Colon = new TokenKind('Colon');
// |
// |   /** A value with an absolute uncertainty (`±`). */
// |   static PlusMinus = new TokenKind('PlusMinus');
// |
// |   /** A comma. */
// |   static Comma = new TokenKind('Comma');
// |
// |   /** The "at" keyword for temporal context. */
// |   static At = new TokenKind('At');
// |
// |   /** The "as" keyword for unit conversion (e.g., "741 KB as MB"). */
// |   static As = new TokenKind('As');
// |
// |   /** The "in" keyword for unit conversion (e.g., "19 TON in USD"). */
// |   static In = new TokenKind('In');
// |
// |   /** The "to" keyword for unit conversion (e.g., "19 TON to USD"). */
// |   static To = new TokenKind('To');
// |
// |   /** The "until" keyword for duration until a datetime. */
// |   static Until = new TokenKind('Until');
// |
// |   /** The "of" keyword for percent-of expressions (e.g., `8% of $50`). */
// |   static Of = new TokenKind('Of');
// |
// |   /** The "and" keyword for natural comparison forms. */
// |   static And = new TokenKind('And');
// |
// |   /** The "compare" keyword for natural comparison forms. */
// |   static Compare = new TokenKind('Compare');
// |
// |   /** The "vs" / "versus" keyword for natural comparison forms. */
// |   static Vs = new TokenKind('Vs');
// |
// |   /** The equals sign for equality checks (e.g., `1 + 1 = 2`). */
// |   static Equals = new TokenKind('Equals');
// |
// |   /** The explicit equality comparison operator (`==`). */
// |   static DoubleEquals = new TokenKind('DoubleEquals');
// |
// |   /** The not-equals comparison operator (`!=`). */
// |   static NotEqual = new TokenKind('NotEqual');
// |
// |   /** The less-than comparison operator (`<`). */
// |   static Less = new TokenKind('Less');
// |
// |   /** The less-than-or-equal comparison operator (`<=`). */
// |   static LessOrEqual = new TokenKind('LessOrEqual');
// |
// |   /** The greater-than comparison operator (`>`). */
// |   static Greater = new TokenKind('Greater');
// |
// |   /** The greater-than-or-equal comparison operator (`>=`). */
// |   static GreaterOrEqual = new TokenKind('GreaterOrEqual');
// |
// |   /** The exclamation mark for factorial postfix notation (e.g., `5!`). */
// |   static Bang = new TokenKind('Bang');
// |
// |   /** End of input. */
// |   static Eof = new TokenKind('Eof');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A token with its position in the input. */
// | export class Token {
// |   /** @param {{ kind: TokenKind, start: number, end: number, text: string }} fields */
// |   constructor({ kind, start, end, text }) {
// |     /** The kind of token. */
// |     this.kind = kind;
// |     /** Start position in the input. */
// |     this.start = start;
// |     /** End position in the input. */
// |     this.end = end;
// |     /** The original text of the token. */
// |     this.text = text;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * Creates a new token.
// |    * @param {TokenKind} kind
// |    * @param {number} start
// |    * @param {number} end
// |    * @param {string} text
// |    * @returns {Token}
// |    */
// |   static new(kind, start, end, text) {
// |     return new Token({ kind, start, end, text });
// |   }
// |
// |   /**
// |    * Checks if this token is a number.
// |    * @returns {boolean}
// |    */
// |   isNumber() {
// |     return this.kind.tag === 'Number';
// |   }
// |
// |   /**
// |    * Checks if this token is an identifier.
// |    * @returns {boolean}
// |    */
// |   isIdentifier() {
// |     return this.kind.tag === 'Identifier';
// |   }
// |
// |   /**
// |    * The derived `Debug`: `Token { kind: Eof, start: 5, end: 5, text: "" }`.
// |    * @returns {string}
// |    */
// |   fmtDebug() {
// |     return debugStruct('Token', [
// |       ['kind', this.kind.fmtDebug()],
// |       ['start', String(this.start)],
// |       ['end', String(this.end)],
// |       ['text', debugStr(this.text)],
// |     ]);
// |   }
// | }

