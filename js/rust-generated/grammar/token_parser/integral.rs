// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=18addd93e2c35beda7a1d9177f2d0d653aeaa247602042ddcf47fa606cb3d638 bytes=6243

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Natural integral notation (port of `src/grammar/token_parser/integral.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from '../../rust/core.js';
use crate::rust::core_::err;
// meta-language:translated JavaScript import_statement items=1 sha256=caf77155a4ce4882db3067d389e46524f0824953f36fc6ea13a6f53f55fa79a1
// | import { debugOption } from '../../rust/fmt.js';
use crate::rust::fmt::debugOption;
// meta-language:translated JavaScript import_statement items=1 sha256=d53b63172cc2677bd863daa7ddae3d14e886d66f495223e66ef8fa0e9af9780b
// | import { isAsciiAlphabetic } from '../../rust/str.js';
use crate::rust::str_::isAsciiAlphabetic;
// meta-language:translated JavaScript import_statement items=1 sha256=0cf39381b387e171be19cbfdbeb594ed9680e5bb1c46abbd51c78ef024972882
// | import { BinaryOp, Expression } from '../../types/index.js';
use crate::types::index::{BinaryOp, Expression};
// meta-language:translated JavaScript import_statement items=1 sha256=4a5e8864a64b03fb15b813ce53f3d1bea23430f02cb95a5fb83e97950cdd4770
// | import { TokenKind } from '../lexer.js';
use crate::grammar::lexer::TokenKind;
// meta-language:translated JavaScript import_statement items=1 sha256=ad41c3b947a8989a16616db7163faaff3eb2d67d9f72c9103f56205735fac111
// | import { isMathFunction } from '../math_functions.js';
use crate::grammar::math_functions::isMathFunction;
// meta-language:translated JavaScript import_statement items=1 sha256=83f70f18110fd2f11614aa0bd8190a56a8975c29c0731241f144c7b70c946e64
// | import { TokenParser } from '../token_parser.js';
use crate::grammar::token_parser::TokenParser;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses natural integral notation: "integrate <expr> d<var>".
// |  *
// |  * Examples include `integrate sin(x)/x dx` and `integrate x^2 dx`.
// |  * @param {TokenParser} self
// |  * @returns {Expression}
// |  */
// | export function parseNaturalIntegral(self) {
// |   const startPos = self.pos;
// |   /** @type {number | null} */
// |   let integrandEndPos = null;
// |   /** @type {string | null} */
// |   let varName = null;
// |
// |   // Scan forward to find differential notation such as `dx`, `dy`, or `dt`.
// |   for (let scanPos = self.pos; scanPos < self.tokens.length; scanPos++) {
// |     const kind = self.tokens[scanPos].kind;
// |     if (kind.tag === 'Identifier') {
// |       const idLower = kind._0.toLowerCase();
// |       // `id_lower.len() == 2` counts UTF-8 bytes; starting with ASCII 'd'
// |       // and having an ASCII second char means two code units here.
// |       const chars = [...idLower];
// |       if (chars[0] === 'd' && chars.length === 2 && isAsciiAlphabetic(chars[1])) {
// |         integrandEndPos = scanPos;
// |         varName = chars[1];
// |         break;
// |       }
// |     }
// |   }
// |
// |   if (integrandEndPos === null || varName === null) {
// |     return err(CalculatorError.parse(
// |       'Invalid integration syntax. Expected: integrate <expression> d<var> (e.g., integrate sin(x)/x dx)',
// |     ));
// |   }
// |
// |   self.pos = startPos;
// |   const integrand = self.parseIntegrandUntil(integrandEndPos);
// |
// |   self.pos = integrandEndPos;
// |   self.advance();
// |
// |   return Expression.indefiniteIntegral(integrand, varName);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parse an integrand expression up to (but not including) `untilPos`.
// |  * @param {TokenParser} self
// |  * @param {number} untilPos
// |  * @returns {Expression}
// |  */
// | export function parseIntegrandUntil(self, untilPos) {
// |   const result = self.parseIntegrandExpression(untilPos);
// |   if (self.pos > untilPos) {
// |     self.pos = untilPos;
// |   }
// |   return result;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandExpression(self, boundary) {
// |   return self.parseIntegrandAdditive(boundary);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandAdditive(self, boundary) {
// |   let left = self.parseIntegrandMultiplicative(boundary);
// |
// |   while (self.pos < boundary) {
// |     const op = self.matchAdditiveOp();
// |     if (op === null) {
// |       break;
// |     }
// |     if (self.pos >= boundary) {
// |       self.pos -= 1;
// |       break;
// |     }
// |     const right = self.parseIntegrandMultiplicative(boundary);
// |     left = Expression.binary(left, op, right);
// |   }
// |
// |   return left;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandMultiplicative(self, boundary) {
// |   let left = self.parseIntegrandPower(boundary);
// |
// |   while (self.pos < boundary) {
// |     const op = self.matchMultiplicativeOp();
// |     if (op !== null) {
// |       if (self.pos >= boundary) {
// |         self.pos -= 1;
// |         break;
// |       }
// |       const right = self.parseIntegrandPower(boundary);
// |       left = Expression.binary(left, op, right);
// |     } else if (self.implicitMultiplicationStarts(left)) {
// |       const right = self.parseIntegrandPower(boundary);
// |       left = Expression.binary(left, BinaryOp.Multiply, right);
// |     } else {
// |       break;
// |     }
// |   }
// |
// |   return left;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandPower(self, boundary) {
// |   let left = self.parseIntegrandUnary(boundary);
// |
// |   if (self.pos < boundary && self.check(TokenKind.Caret)) {
// |     self.advance();
// |     const right = self.parseIntegrandPower(boundary);
// |     left = Expression.power(left, right);
// |   } else if (self.pos < boundary) {
// |     const kind = self.currentKind();
// |     if (kind !== null && kind.tag === 'Superscript') {
// |       const exponent = self.numberGrammar.parseNumber(kind._0);
// |       self.advance();
// |       left = Expression.power(left, Expression.number(exponent));
// |     }
// |   }
// |
// |   return left;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandUnary(self, boundary) {
// |   if (self.pos < boundary && self.check(TokenKind.Minus)) {
// |     self.advance();
// |     const expr = self.parseIntegrandUnary(boundary);
// |     return Expression.negate(expr);
// |   }
// |
// |   return self.parseIntegrandPrimary(boundary);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self @param {number} boundary @returns {Expression} */
// | export function parseIntegrandPrimary(self, boundary) {
// |   if (self.pos >= boundary) {
// |     return err(CalculatorError.parse('Unexpected end of integrand'));
// |   }
// |
// |   if (self.check(TokenKind.LeftParen)) {
// |     self.advance();
// |     const expr = self.parseExpression();
// |     self.expect(TokenKind.RightParen);
// |     return Expression.group(expr);
// |   }
// |
// |   const kind = self.currentKind();
// |   if (kind !== null && kind.tag === 'Number') {
// |     const number = kind._0;
// |     self.advance();
// |     const value = self.numberGrammar.parseNumber(number);
// |     return Expression.number(value);
// |   }
// |
// |   if (kind !== null && kind.tag === 'Identifier') {
// |     const id = kind._0;
// |     self.advance();
// |
// |     if (self.pos < boundary && self.check(TokenKind.LeftParen)) {
// |       if (TokenParser.isSingleLetterVariable(id) && !isMathFunction(id)) {
// |         return Expression.variable(id);
// |       }
// |       return self.parseFunctionCall(id);
// |     }
// |
// |     if (TokenParser.isMathConstant(id)) {
// |       return Expression.functionCall(id, []);
// |     }
// |     if (TokenParser.isUnaryMathFunction(id) && self.pos < boundary) {
// |       const argument = self.parseIntegrandUnary(boundary);
// |       return Expression.functionCall(id, [argument]);
// |     }
// |     if (isMathFunction(id)) {
// |       return Expression.functionCall(id, []);
// |     }
// |     return Expression.variable(id);
// |   }
// |
// |   return err(CalculatorError.parse(
// |     `Unexpected token in integrand: ${debugOption(self.current(), (t) => t.fmtDebug())}`,
// |   ));
// | }
