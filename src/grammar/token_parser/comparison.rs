use crate::error::CalculatorError;
use crate::grammar::TokenKind;
use crate::types::{BinaryOp, ComparisonOp, DurationUnit, Expression, Unit};

use super::TokenParser;

impl TokenParser<'_> {
    pub fn parse_expression(&mut self) -> Result<Expression, CalculatorError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expression, CalculatorError> {
        if let Some(day_span) = self.try_parse_day_span()? {
            return Ok(day_span);
        }

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

    /// Parses natural day-span queries:
    /// - `days between <datetime> and <datetime>`
    /// - `days to <datetime>` (the target datetime minus now)
    fn try_parse_day_span(&mut self) -> Result<Option<Expression>, CalculatorError> {
        let Some(TokenKind::Identifier(unit)) = self.current_kind() else {
            return Ok(None);
        };
        if !unit.eq_ignore_ascii_case("days") {
            return Ok(None);
        }

        let is_between = matches!(
            self.peek_kind(),
            Some(TokenKind::Identifier(keyword)) if keyword.eq_ignore_ascii_case("between")
        );
        let is_to = matches!(self.peek_kind(), Some(TokenKind::To));
        if !is_between && !is_to {
            return Ok(None);
        }

        self.advance(); // consume "days"
        self.advance(); // consume "between" or "to"

        let left = self.parse_additive()?;
        let difference = if is_between {
            self.expect(&TokenKind::And)?;
            let right = self.parse_additive()?;
            Expression::binary(left, BinaryOp::Subtract, right)
        } else {
            Expression::binary(left, BinaryOp::Subtract, Expression::Now)
        };

        Ok(Some(Expression::unit_conversion(
            difference,
            Unit::Duration(DurationUnit::Days),
        )))
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
