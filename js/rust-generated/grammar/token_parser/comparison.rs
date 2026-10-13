// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=70c7312adb96fa94f08c55ccc1e25f50ff0dafe45ce51624b78f50a67c6ce066 bytes=3815

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Comparison-level parsing (port of `src/grammar/token_parser/comparison.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=7f52f3fda394ec23ea9a49fa214946d3769021376b040d97e7da6e155f2b7ffe
// | import { BinaryOp, ComparisonOp, DurationUnit, Expression, Unit } from '../../types/index.js';
use crate::types::index::{BinaryOp, ComparisonOp, DurationUnit, Expression, Unit};
// meta-language:translated JavaScript import_statement items=1 sha256=6a58e9adb3b15ffddb375f0e6d3ccd15d27e918ab78e0d0474c5b280ad9f79ac
// | import { eqIgnoreAsciiCase } from '../../rust/str.js';
use crate::rust::str_::eqIgnoreAsciiCase;
// meta-language:translated JavaScript import_statement items=1 sha256=4a5e8864a64b03fb15b813ce53f3d1bea23430f02cb95a5fb83e97950cdd4770
// | import { TokenKind } from '../lexer.js';
use crate::grammar::lexer::TokenKind;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../token_parser.js').TokenParser} TokenParser */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Throws `Err(CalculatorError)`.
// |  * @param {TokenParser} self
// |  * @returns {Expression}
// |  */
// | export function parseExpression(self) {
// |   if (self.ratioWord('if') && self.hasProportionQuestion()) {
// |     self.advance();
// |   }
// |   if (self.ratioWord('what') && self.peekIdentifierIs('is')) {
// |     self.advance();
// |     self.advance();
// |   }
// |   return self.parseComparison();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {TokenParser} self
// |  * @returns {Expression}
// |  */
// | export function parseComparison(self) {
// |   const daySpan = self.tryParseDaySpan();
// |   if (daySpan !== null) {
// |     return daySpan;
// |   }
// |
// |   if (self.checkCompare()) {
// |     self.advance(); // consume "compare"
// |     const left = self.parseAdditive();
// |     self.expect(TokenKind.And);
// |     const right = self.parseAdditive();
// |     return Expression.comparison(left, ComparisonOp.Compare, right);
// |   }
// |
// |   let left = self.parseAdditive();
// |   if (self.check(TokenKind.PlusMinus)) {
// |     self.advance();
// |     const uncertainty = self.parseAdditive();
// |     left = Expression.functionCall('uncertainty', [left, uncertainty]);
// |   }
// |
// |   if (self.checkVs()) {
// |     self.advance(); // consume "vs"
// |     const right = self.parseAdditive();
// |     return Expression.comparison(left, ComparisonOp.Compare, right);
// |   }
// |
// |   if (self.check(TokenKind.Equals)) {
// |     self.advance(); // consume '=' or '=='
// |     if (self.isAtEnd()) {
// |       return left;
// |     }
// |     const right = self.parseAdditive();
// |     return Expression.equality(left, right);
// |   }
// |
// |   const op = self.matchOrderingOp();
// |   if (op !== null) {
// |     const right = self.parseAdditive();
// |     return Expression.comparison(left, op, right);
// |   }
// |
// |   return left;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses natural day-span queries:
// |  * - `days between <datetime> and <datetime>`
// |  * - `days to <datetime>` (the target datetime minus now)
// |  * @param {TokenParser} self
// |  * @returns {Expression | null}
// |  */
// | export function tryParseDaySpan(self) {
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Identifier') {
// |     return null;
// |   }
// |   if (!eqIgnoreAsciiCase(kind._0, 'days')) {
// |     return null;
// |   }
// |
// |   const isBetween = self.peekIdentifierIs('between');
// |   const isTo = self.peekKind()?.tag === 'To';
// |   if (!isBetween && !isTo) {
// |     return null;
// |   }
// |
// |   self.advance(); // consume "days"
// |   self.advance(); // consume "between" or "to"
// |
// |   const left = self.parseAdditive();
// |   let difference;
// |   if (isBetween) {
// |     self.expect(TokenKind.And);
// |     const right = self.parseAdditive();
// |     difference = Expression.binary(left, BinaryOp.Subtract, right);
// |   } else {
// |     difference = Expression.binary(left, BinaryOp.Subtract, Expression.Now());
// |   }
// |
// |   return Expression.unitConversion(difference, Unit.Duration(DurationUnit.Days));
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Record<string, ComparisonOp>} */
// | const ORDERING_OPS = {
// |   DoubleEquals: ComparisonOp.Equal,
// |   Less: ComparisonOp.Less,
// |   LessOrEqual: ComparisonOp.LessOrEqual,
// |   Greater: ComparisonOp.Greater,
// |   GreaterOrEqual: ComparisonOp.GreaterOrEqual,
// |   NotEqual: ComparisonOp.NotEqual,
// | };

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {TokenParser} self
// |  * @returns {ComparisonOp | null}
// |  */
// | export function matchOrderingOp(self) {
// |   const kind = self.currentKind();
// |   if (kind === null || !Object.hasOwn(ORDERING_OPS, kind.tag)) {
// |     return null;
// |   }
// |   self.advance();
// |   return ORDERING_OPS[kind.tag];
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self */
// | export function checkCompare(self) {
// |   return self.currentKind()?.tag === 'Compare';
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self */
// | export function checkVs(self) {
// |   return self.currentKind()?.tag === 'Vs';
// | }
