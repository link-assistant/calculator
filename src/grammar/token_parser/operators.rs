use crate::grammar::TokenKind;
use crate::types::BinaryOp;

use super::TokenParser;

impl TokenParser<'_> {
    pub(super) fn percent_starts_binary_expression(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Number(_) | TokenKind::Identifier(_) | TokenKind::LeftParen)
        )
    }

    pub(super) fn match_additive_op(&mut self) -> Option<BinaryOp> {
        if self.check(&TokenKind::Plus) {
            self.advance();
            Some(BinaryOp::Add)
        } else if self.check(&TokenKind::Minus) {
            self.advance();
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    pub(super) fn match_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.check(&TokenKind::Star) {
            self.advance();
            Some(BinaryOp::Multiply)
        } else if self.check(&TokenKind::Slash) {
            self.advance();
            Some(BinaryOp::Divide)
        } else if self.check(&TokenKind::Percent) && self.percent_starts_binary_expression() {
            self.advance();
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }
}
