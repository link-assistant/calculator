use crate::error::CalculatorError;
use crate::grammar::TokenKind;
use crate::types::{ComparisonOp, Expression};

use super::TokenParser;

impl TokenParser<'_> {
    pub fn parse_expression(&mut self) -> Result<Expression, CalculatorError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expression, CalculatorError> {
        if self.check_compare() {
            self.advance(); // consume "compare"
            let left = self.parse_additive()?;
            self.expect(&TokenKind::And)?;
            let right = self.parse_additive()?;
            return Ok(Expression::comparison(left, ComparisonOp::Compare, right));
        }

        let left = self.parse_additive()?;

        if self.check_vs() {
            self.advance(); // consume "vs"
            let right = self.parse_additive()?;
            return Ok(Expression::comparison(left, ComparisonOp::Compare, right));
        }

        if self.check(&TokenKind::Equals) {
            self.advance(); // consume '=' or '=='
            let right = self.parse_additive()?;
            return Ok(Expression::equality(left, right));
        }

        if let Some(op) = self.match_ordering_op() {
            let right = self.parse_additive()?;
            return Ok(Expression::comparison(left, op, right));
        }

        Ok(left)
    }

    fn match_ordering_op(&mut self) -> Option<ComparisonOp> {
        let op = match self.current_kind()? {
            TokenKind::DoubleEquals => ComparisonOp::Equal,
            TokenKind::Less => ComparisonOp::Less,
            TokenKind::LessOrEqual => ComparisonOp::LessOrEqual,
            TokenKind::Greater => ComparisonOp::Greater,
            TokenKind::GreaterOrEqual => ComparisonOp::GreaterOrEqual,
            TokenKind::NotEqual => ComparisonOp::NotEqual,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn check_compare(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Compare))
    }

    fn check_vs(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Vs))
    }
}
