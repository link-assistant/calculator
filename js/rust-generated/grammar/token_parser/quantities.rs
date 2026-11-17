// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c7260adcb538c3b109ad90ec7eb81ab18950e408e0511b6e5c48edfe56e919ce bytes=12292

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Units after a number and as conversion targets (issue #235); port of
// |  * `src/grammar/token_parser/quantities.rs`.
// |  *
// |  * A unit expression is a product of unit names with optional integer powers:
// |  * `m^2`, `m²`, `s^-1`, adjacent `s−1`, joined by spaces, `*`, `·`, `⋅`, `/`
// |  * or `per`, with a parenthesized denominator (`kg / (m^-2 s^2)`). Only
// |  * dimensional units (data/units, mass, durations, data sizes) take part, so
// |  * `5 USD * 2` and `10 m / 2 s` stay arithmetic. A quantity followed by more
// |  * quantities of the same dimension is their sum: `5 feet 10 inches`, `6'2"`
// |  * or `1 h 30 min`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=69180a140dff9ce2d56b9cc92618d313cb3bb456ec986d5cd4b54bdfdd9e0bd0
// | import { eq, ok } from '../../rust/core.js';
use crate::rust::core_::{eq, ok};
// meta-language:translated JavaScript import_statement items=1 sha256=f794ec57e30a3721cfa7a9110f6cb1529861e227164553059f1def65f32a079e
// | import { I32 } from '../../rust/num.js';
use crate::rust::num::I32;
// meta-language:translated JavaScript import_statement items=1 sha256=5278e4b1966dfab735b12a591eadbf3333e9db039eeb09c8872c37ca807e4a83
// | import { eqIgnoreAsciiCase, replace } from '../../rust/str.js';
use crate::rust::str_::{eqIgnoreAsciiCase, replace};
// meta-language:translated JavaScript import_statement items=1 sha256=4ff2fc0a19b99eafe090b9d9c3876202a211dba694668e77051f6328ad5d43eb
// | import { BinaryOp, Decimal, Expression, PhysicalUnit, Unit } from '../../types/index.js';
use crate::types::index::{BinaryOp, Decimal, Expression, PhysicalUnit, Unit};
// meta-language:translated JavaScript import_statement items=1 sha256=4a5e8864a64b03fb15b813ce53f3d1bea23430f02cb95a5fb83e97950cdd4770
// | import { TokenKind } from '../lexer.js';
use crate::grammar::lexer::TokenKind;
// meta-language:translated JavaScript import_statement items=1 sha256=ad41c3b947a8989a16616db7163faaff3eb2d67d9f72c9103f56205735fac111
// | import { isMathFunction } from '../math_functions.js';
use crate::grammar::math_functions::isMathFunction;
// meta-language:translated JavaScript import_statement items=1 sha256=83f70f18110fd2f11614aa0bd8190a56a8975c29c0731241f144c7b70c946e64
// | import { TokenParser } from '../token_parser.js';
use crate::grammar::token_parser::TokenParser;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** `(canonical symbol, power)` factors of a unit expression. @typedef {Array<[string, number]>} Terms */

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ac3cb8a0e45f9b793b55ca8efb5dfb5e63d20a82c5a38b3863b2c7b9b0716d80
// | /** Largest power accepted in a unit expression, bounding `m^1000000`. */
// | const MAX_UNIT_POWER = 1000;
pub const MAX_UNIT_POWER: f64 = 1000f64;

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `s.parse::<i32>().ok()`.
// |  * @param {string} s
// |  * @returns {number | null}
// |  */
// | function parseI32(s) {
// |   const value = ok(() => I32.parse(s));
// |   return value === null ? null : Number(value);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses the unit after a number: a dimensional unit expression, a
// |  * legacy unit with alternatives, or `null` when no unit follows.
// |  * @param {TokenParser} self
// |  * @param {number} numberEnd
// |  * @returns {[Unit, Unit[]] | null}
// |  */
// | export function parseQuantityUnit(self, numberEnd) {
// |   const inch = self.inchBeforeConversion();
// |   if (inch !== null) {
// |     return [inch, []];
// |   }
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Identifier') {
// |     return null;
// |   }
// |   const id = kind._0;
// |   // An unknown single-letter identifier is a conventional coefficient
// |   // (`2x` or `2 x`). Established short units (`2h`, `2m`) and
// |   // multi-letter custom units retain their meaning.
// |   const coefficientVariable = TokenParser.isSingleLetterVariable(id) && !self.identifierIsKnownUnit(id);
// |   const current = self.current();
// |   const ratioKeyword = TokenParser.ratioKeyword(id) && current !== null && current.start > numberEnd;
// |   // `min` and `sec` are units after a number unless called: `5 min`,
// |   // but `2 min(3, 4)` stays a function call.
// |   const fn = isMathFunction(id) && !self.identifierIsDimensionalUnit(id);
// |   if (fn || self.peekIsLeftParen() || coefficientVariable || ratioKeyword) {
// |     return null;
// |   }
// |
// |   const save = self.pos;
// |   const terms = self.parseUnitTerms();
// |   if (terms !== null) {
// |     const compound = self.pos > save + 1 || terms.length !== 1 || terms[0][1] !== 1;
// |     if (compound) {
// |       return [PhysicalUnit.fromTerms(terms).intoUnit(), []];
// |     }
// |   }
// |   self.pos = save;
// |
// |   const unit = ok(() => self.numberGrammar.parseUnitWithAlternatives(id))
// |     ?? /** @type {[Unit, Unit[]]} */ ([Unit.Custom(id), []]);
// |   self.advance();
// |   return unit;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Whether `id` names a dimensional unit (`min`, `sec`), so a function
// |  * name such as `pi` that is also a currency code stays a constant.
// |  * @param {TokenParser} self
// |  * @param {string} id
// |  */
// | export function identifierIsDimensionalUnit(self, id) {
// |   const parsed = ok(() => self.numberGrammar.parseUnitWithAlternatives(id));
// |   if (parsed === null) {
// |     return false;
// |   }
// |   const unit = PhysicalUnit.fromUnit(parsed[0]);
// |   return unit !== null && !unit.isEmpty();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Extends a quantity into a sum of quantities of the same dimension:
// |  * `5 feet 10 inches`, `1 meter 20 cm`, `6'2"`, `1 h 30 min`.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {TokenParser} self
// |  * @param {Expression} first
// |  * @returns {Expression}
// |  */
// | export function parseCompoundQuantity(self, first) {
// |   if (first.tag !== 'Number') {
// |     return first;
// |   }
// |   const firstUnit = PhysicalUnit.fromUnit(first.unit);
// |   if (firstUnit === null || firstUnit.isEmpty()) {
// |     return first;
// |   }
// |   const dimension = firstUnit.dimension();
// |   let sum = first;
// |   for (;;) {
// |     const save = self.pos;
// |     const kind = self.currentKind();
// |     if (kind === null || kind.tag !== 'Number') {
// |       break;
// |     }
// |     const text = kind._0;
// |     self.advance();
// |     const terms = self.parseUnitTerms();
// |     let partUnit = null;
// |     if (terms !== null && terms.length === 1 && terms[0][1] === 1) {
// |       const part = PhysicalUnit.fromTerms(terms);
// |       if (eq(part.dimension(), dimension)) {
// |         partUnit = part;
// |       }
// |     }
// |     if (partUnit === null) {
// |       self.pos = save;
// |       break;
// |     }
// |     const value = self.numberGrammar.parseNumber(text);
// |     const part = Expression.numberWithUnit(value, partUnit.intoUnit());
// |     sum = Expression.binary(sum, BinaryOp.Add, part);
// |   }
// |   return sum;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses a conversion target as a unit expression (`m/s`, `°F`,
// |  * `kg m^2 s^-2`); `null` leaves the target to the legacy unit names.
// |  * @param {TokenParser} self
// |  * @returns {Unit | null}
// |  */
// | export function parsePhysicalConversionTarget(self) {
// |   const save = self.pos;
// |   const terms = self.parseUnitTerms();
// |   if (terms !== null && self.conversionTargetEnds()) {
// |     return PhysicalUnit.fromTerms(terms).intoUnit();
// |   }
// |   self.pos = save;
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Whether a conversion target is complete: the input, a group or an
// |  * `at` clause ends.
// |  * @param {TokenParser} self
// |  */
// | export function conversionTargetEnds(self) {
// |   return self.isAtEnd()
// |     || self.currentKind()?.tag === 'RightParen'
// |     || self.checkAt();
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const INCH_ENDS = new Set(['Eof', 'In', 'As', 'To', 'RightParen']);

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `2in` or `2 in` followed by the end of input or a conversion keyword
// |  * is two inches (`in` is otherwise the conversion keyword).
// |  * @param {TokenParser} self
// |  * @returns {Unit | null}
// |  */
// | export function inchBeforeConversion(self) {
// |   if (!self.checkIn()) {
// |     return null;
// |   }
// |   const next = self.tokens[self.pos + 1];
// |   const ends = next === undefined || INCH_ENDS.has(next.kind.tag);
// |   if (!ends) {
// |     return null;
// |   }
// |   self.advance();
// |   return PhysicalUnit.parse('in')?.intoUnit() ?? null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses `factor ((space | * | / | per) factor)*` into merged terms.
// |  * @param {TokenParser} self
// |  * @returns {Terms | null}
// |  */
// | export function parseUnitTerms(self) {
// |   const terms = self.parseUnitFactor();
// |   if (terms === null) {
// |     return null;
// |   }
// |   for (;;) {
// |     const save = self.pos;
// |     const kind = self.currentKind();
// |     /** @type {boolean | null} */
// |     let divide = null;
// |     if (kind?.tag === 'Star') {
// |       divide = false;
// |     } else if (kind?.tag === 'Slash') {
// |       divide = true;
// |     } else if (kind !== null && kind.tag === 'Identifier' && eqIgnoreAsciiCase(kind._0, 'per')) {
// |       divide = true;
// |     }
// |     if (divide !== null) {
// |       self.advance();
// |     }
// |     const factor = divide === true && self.check(TokenKind.LeftParen)
// |       ? self.parseUnitGroup()
// |       : self.parseUnitFactor();
// |     if (factor === null) {
// |       self.pos = save;
// |       break;
// |     }
// |     const sign = divide === true ? -1 : 1;
// |     for (const [symbol, power] of factor) {
// |       terms.push([symbol, power * sign]);
// |     }
// |   }
// |   return mergeTerms(terms);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses `( unit expression )`.
// |  * @param {TokenParser} self
// |  * @returns {Terms | null}
// |  */
// | export function parseUnitGroup(self) {
// |   const save = self.pos;
// |   self.advance();
// |   const terms = self.parseUnitTerms();
// |   if (terms !== null && self.check(TokenKind.RightParen)) {
// |     self.advance();
// |     return terms;
// |   }
// |   self.pos = save;
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses a dimensional unit name with an optional integer power.
// |  * @param {TokenParser} self
// |  * @returns {Terms | null}
// |  */
// | export function parseUnitFactor(self) {
// |   const unit = self.parseUnitName();
// |   if (unit === null) {
// |     return null;
// |   }
// |   const power = self.parseUnitPower();
// |   if (power === null) {
// |     return null;
// |   }
// |   return unit.powi(power).terms().map((term) => /** @type {[string, number]} */ ([term.symbol, term.power]));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Resolves the dimensional unit at the current token, including
// |  * two-word names such as `light year`.
// |  * @param {TokenParser} self
// |  * @returns {PhysicalUnit | null}
// |  */
// | export function parseUnitName(self) {
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Identifier') {
// |     return null;
// |   }
// |   const id = kind._0;
// |   const next = self.peekKind();
// |   if (next !== null && next.tag === 'Identifier') {
// |     const unit = PhysicalUnit.parse(`${id}_${next._0}`);
// |     if (unit !== null) {
// |       self.pos += 2;
// |       return unit;
// |     }
// |   }
// |   if ((isMathFunction(id) && self.peekIsLeftParen()) || eqIgnoreAsciiCase(id, 'per')) {
// |     return null;
// |   }
// |   const parsed = ok(() => self.numberGrammar.parseUnitWithAlternatives(id));
// |   if (parsed === null) {
// |     return null;
// |   }
// |   const unit = PhysicalUnit.fromUnit(parsed[0]);
// |   if (unit === null || unit.isEmpty()) {
// |     return null;
// |   }
// |   self.advance();
// |   return unit;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses `^n`, `^-n`, `^(-n)`, a superscript, or an adjacent `−n`;
// |  * `1` when no power follows and `null` for an invalid power.
// |  * @param {TokenParser} self
// |  * @returns {number | null}
// |  */
// | export function parseUnitPower(self) {
// |   if (self.pos === 0) {
// |     return null;
// |   }
// |   const previous = self.tokens[self.pos - 1];
// |   if (previous === undefined) {
// |     return null;
// |   }
// |   const previousEnd = previous.end;
// |   const save = self.pos;
// |   const kind = self.currentKind();
// |   let power;
// |   if (kind?.tag === 'Caret') {
// |     self.advance();
// |     const parenthesized = self.check(TokenKind.LeftParen);
// |     if (parenthesized) {
// |       self.advance();
// |     }
// |     power = self.parseSignedInteger();
// |     if (parenthesized && power !== null) {
// |       if (!self.check(TokenKind.RightParen)) {
// |         return null;
// |       }
// |       self.advance();
// |     }
// |   } else if (kind !== null && kind.tag === 'Superscript') {
// |     power = parseI32(replace(kind._0, '+', ''));
// |     self.advance();
// |   } else if (kind?.tag === 'Minus' && self.current()?.start === previousEnd) {
// |     const minusEnd = /** @type {import('../lexer.js').Token} */ (self.current()).end;
// |     const after = self.tokens[self.pos + 1];
// |     const adjacent = after !== undefined && after.start === minusEnd && after.kind.tag === 'Number';
// |     if (adjacent) {
// |       power = self.parseSignedInteger();
// |     } else {
// |       return 1;
// |     }
// |   } else {
// |     return 1;
// |   }
// |   if (power !== null && power !== 0 && Math.abs(power) <= MAX_UNIT_POWER) {
// |     return power;
// |   }
// |   self.pos = save;
// |   // `2 kg^x` is not a unit power; leave `^` to arithmetic.
// |   return 1;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses an optionally negative integer literal.
// |  * @param {TokenParser} self
// |  * @returns {number | null}
// |  */
// | export function parseSignedInteger(self) {
// |   const negative = self.check(TokenKind.Minus);
// |   if (negative) {
// |     self.advance();
// |   }
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Number') {
// |     return null;
// |   }
// |   const power = parseI32(kind._0);
// |   if (power === null) {
// |     return null;
// |   }
// |   self.advance();
// |   // `-power` cannot overflow: the lexer never yields a signed number.
// |   return negative ? -power : power;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Adds the powers of equal symbols, keeping first-seen order.
// |  * @param {Terms} terms
// |  * @returns {Terms}
// |  */
// | function mergeTerms(terms) {
// |   /** @type {Terms} */
// |   const merged = [];
// |   for (const [symbol, power] of terms) {
// |     const existing = merged.find(([s]) => s === symbol);
// |     if (existing !== undefined) {
// |       existing[1] += power;
// |     } else {
// |       merged.push([symbol, power]);
// |     }
// |   }
// |   return merged.filter(([, power]) => power !== 0);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `a / b unit` reads as `(a / b) unit` when `b` carries a unit, so
// |  * `1/0.5 kg` is `2 kg`.
// |  * @param {Expression} left
// |  * @param {Expression} right
// |  * @returns {Expression}
// |  */
// | export function divideIntoQuantity(left, right) {
// |   if (right.tag === 'Number' && right.unit.tag !== 'None'
// |     && left.tag === 'Number' && left.unit.tag === 'None') {
// |     const quotient = Expression.binary(left, BinaryOp.Divide, Expression.number(right.value));
// |     const unitOne = Expression.numberWithUnitAlternatives(
// |       Decimal.new(1), right.unit, right.alternativeUnits,
// |     );
// |     return Expression.binary(quotient, BinaryOp.Multiply, unitOne);
// |   }
// |   return Expression.binary(left, BinaryOp.Divide, right);
// | }
