// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=612be1143f1afbfe37569076c28dbff89aeae615bfa99c62e7e1519707453a91 bytes=1745

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Binary operator matching (port of `src/grammar/token_parser/operators.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=9e48e3e3252ae075397d88bcb087aaa4203778978632c7c3495bdaf0d22c8ffe
// | import { BinaryOp } from '../../types/index.js';
use crate::types::index::BinaryOp;
// meta-language:translated JavaScript import_statement items=1 sha256=4a5e8864a64b03fb15b813ce53f3d1bea23430f02cb95a5fb83e97950cdd4770
// | import { TokenKind } from '../lexer.js';
use crate::grammar::lexer::TokenKind;
// meta-language:translated JavaScript import_statement items=1 sha256=83f70f18110fd2f11614aa0bd8190a56a8975c29c0731241f144c7b70c946e64
// | import { TokenParser } from '../token_parser.js';
use crate::grammar::token_parser::TokenParser;

// meta-language:carried JavaScript export_statement (unsupported)
// | /** @param {TokenParser} self */
// | export function percentStartsBinaryExpression(self) {
// |   if (self.check(TokenKind.PercentWord)) {
// |     return false;
// |   }
// |   const current = self.current();
// |   if (current !== null && current.text !== '%') {
// |     return true;
// |   }
// |   const peek = self.peekKind();
// |   if (peek !== null && peek.tag === 'Identifier' && TokenParser.ratioKeyword(peek._0)) {
// |     return false;
// |   }
// |   if (peek !== null && peek.tag === 'Number' && self.pos > 0) {
// |     const percent = self.tokens[self.pos];
// |     if (percent.start === self.tokens[self.pos - 1].end
// |       && self.tokens[self.pos + 1].start > percent.end) {
// |       return false;
// |     }
// |   }
// |   return peek !== null
// |     && (peek.tag === 'Number' || peek.tag === 'Identifier' || peek.tag === 'LeftParen');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {TokenParser} self
// |  * @returns {BinaryOp | null}
// |  */
// | export function matchAdditiveOp(self) {
// |   if (self.check(TokenKind.Plus)) {
// |     self.advance();
// |     return BinaryOp.Add;
// |   } else if (self.check(TokenKind.Minus)) {
// |     self.advance();
// |     return BinaryOp.Subtract;
// |   }
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {TokenParser} self
// |  * @returns {BinaryOp | null}
// |  */
// | export function matchMultiplicativeOp(self) {
// |   if (self.check(TokenKind.Star)) {
// |     self.advance();
// |     return BinaryOp.Multiply;
// |   } else if (self.check(TokenKind.Slash)) {
// |     self.advance();
// |     return BinaryOp.Divide;
// |   } else if (self.check(TokenKind.Percent) && self.percentStartsBinaryExpression()) {
// |     self.advance();
// |     return BinaryOp.Modulo;
// |   }
// |   return null;
// | }
