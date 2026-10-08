//! Percentage syntax lowered into exact ratio functions and ordinary arithmetic.
use super::TokenParser;
use crate::error::CalculatorError;
use crate::grammar::TokenKind;
use crate::types::{BinaryOp, Decimal, Expression};

impl TokenParser<'_> {
    pub(super) fn has_proportion_question(&self) -> bool {
        self.tokens[self.pos..].windows(3).any(|tokens| {
            matches!(&tokens[0].kind, TokenKind::Comma)
                && matches!(&tokens[1].kind, TokenKind::Identifier(word) if word.eq_ignore_ascii_case("what"))
                && matches!(&tokens[2].kind, TokenKind::Identifier(word) if word.eq_ignore_ascii_case("is"))
        })
    }

    pub(super) fn ratio_keyword(word: &str) -> bool {
        matches!(
            word.to_ascii_lowercase().as_str(),
            "is" | "what" | "on" | "off" | "from"
        )
    }

    pub(super) fn ratio_word(&self, word: &str) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Identifier(id)) if id.eq_ignore_ascii_case(word))
    }

    pub(super) fn expect_ratio_word(&mut self, word: &str) -> Result<(), CalculatorError> {
        if !self.ratio_word(word) {
            return Err(CalculatorError::parse(format!(
                "Expected '{word}' in ratio expression"
            )));
        }
        self.advance();
        Ok(())
    }

    pub(super) fn parse_percent_postfix(
        &mut self,
        mut expression: Expression,
    ) -> Result<Expression, CalculatorError> {
        while (self.check(&TokenKind::Percent) || self.check(&TokenKind::PercentWord))
            && !self.percent_starts_binary_expression()
        {
            self.advance();
            expression = Expression::function_call("percent", vec![expression]);
        }
        if matches!(&expression, Expression::FunctionCall { name, .. } if name == "percent")
            && (self.check(&TokenKind::Of) || self.ratio_word("on") || self.ratio_word("off"))
        {
            let op = if self.ratio_word("on") {
                BinaryOp::Add
            } else if self.ratio_word("off") {
                BinaryOp::Subtract
            } else {
                BinaryOp::Multiply
            };
            // Leave inverse queries to parse_ratio_suffix: `20 is 10% of what`.
            if matches!(self.peek_kind(), Some(TokenKind::Identifier(word)) if word.eq_ignore_ascii_case("what"))
            {
                return Ok(expression);
            }
            self.advance();
            let base = self.parse_power()?;
            let result = if op == BinaryOp::Multiply {
                Expression::binary(expression, op, base)
            } else {
                Expression::binary(base, op, expression)
            };
            return Ok(Expression::function_call("percent_of", vec![result]));
        }
        Ok(expression)
    }

    pub(super) fn combine_ratio_product(
        &self,
        left: Expression,
        op: BinaryOp,
        right: Expression,
    ) -> Expression {
        // Soulver's spaced trailing ratio form: `20/200 %` means `(20/200) as %`.
        // Adjacent `20/200%` keeps standard division by a percentage.
        if op == BinaryOp::Divide && self.pos >= 2 {
            let percent = &self.tokens[self.pos - 1];
            let previous = &self.tokens[self.pos - 2];
            if matches!(percent.kind, TokenKind::Percent) && percent.start > previous.end {
                if let Expression::FunctionCall { name, args } = &right {
                    if name == "percent" && args.len() == 1 {
                        return Expression::function_call(
                            "as_percent",
                            vec![Expression::binary(left, op, args[0].clone())],
                        );
                    }
                }
            }
        }
        Expression::binary(left, op, right)
    }

    pub(super) fn parse_ratio_suffix(
        &mut self,
        left: &Expression,
    ) -> Result<Option<Expression>, CalculatorError> {
        let start = self.pos;
        // `3 in 20 is what %` and `50 to 75 is what %` precede unit conversion.
        if self.check(&TokenKind::In) || self.check(&TokenKind::To) {
            let change = self.check(&TokenKind::To);
            self.advance();
            if let Ok(right) = self.parse_multiplicative() {
                if self.ratio_word("is") || self.check(&TokenKind::As) {
                    self.advance();
                    if self.ratio_word("what") {
                        self.advance();
                    }
                    if let Some(format) = self.consume_ratio_format() {
                        let ratio = Expression::binary(
                            if change { right.clone() } else { left.clone() },
                            BinaryOp::Divide,
                            if change { left.clone() } else { right },
                        );
                        let ratio = if change && format == "as_percent" {
                            Expression::binary(
                                ratio,
                                BinaryOp::Subtract,
                                Expression::number(Decimal::new(1)),
                            )
                        } else {
                            ratio
                        };
                        return Ok(Some(Expression::function_call(format, vec![ratio])));
                    }
                }
            }
            self.pos = start;
        }
        if self.check_as() || self.check_to() {
            self.advance();
            if self.ratio_word("a") {
                self.advance();
            }
            if let Some(format) = self.consume_ratio_format() {
                return Ok(Some(self.finish_ratio_format(left.clone(), format)?));
            }
            self.pos = start;
        }
        if self.ratio_word("is") {
            self.advance();
            if self.ratio_word("what") {
                self.advance();
                let format = self
                    .consume_ratio_format()
                    .ok_or_else(|| CalculatorError::parse("Expected a percentage or multiplier"))?;
                return Ok(Some(self.finish_ratio_format(left.clone(), format)?));
            }
            let amount = self.parse_multiplicative()?;
            // Proportions: `20% is 500, what is 750` and `if 20 is 30%, what is 60%`.
            if self.check(&TokenKind::Comma) && (left.is_percentage() || amount.is_percentage()) {
                self.advance();
                self.expect_ratio_word("what")?;
                self.expect_ratio_word("is")?;
                let target = self.parse_additive()?;
                let result = Expression::binary(
                    Expression::binary(left.clone(), BinaryOp::Multiply, target),
                    BinaryOp::Divide,
                    amount,
                );
                return Ok(Some(if left.is_percentage() {
                    Expression::function_call("as_percent", vec![result])
                } else {
                    result
                }));
            }
            // `5% is 1 in what` formats odds without changing the numeric value.
            if self.check(&TokenKind::In) {
                self.advance();
                self.expect_ratio_word("what")?;
                if amount != Expression::number(Decimal::new(1)) {
                    return Err(CalculatorError::parse("Expected 'is 1 in what'"));
                }
                return Ok(Some(Expression::function_call(
                    "as_reciprocal",
                    vec![left.clone()],
                )));
            }
            let factor = if self.check(&TokenKind::Of) {
                amount
            } else if self.ratio_word("on") || self.ratio_word("off") {
                let op = if self.ratio_word("on") {
                    BinaryOp::Add
                } else {
                    BinaryOp::Subtract
                };
                // Multiplication clears percent display, ensuring point arithmetic in the factor.
                let scalar = Expression::binary(
                    amount,
                    BinaryOp::Multiply,
                    Expression::number(Decimal::new(1)),
                );
                Expression::binary(Expression::number(Decimal::new(1)), op, scalar)
            } else {
                return Err(CalculatorError::parse("Expected 'of', 'on', or 'off'"));
            };
            self.advance();
            self.expect_ratio_word("what")?;
            return Ok(Some(Expression::binary(
                left.clone(),
                BinaryOp::Divide,
                factor,
            )));
        }
        Ok(None)
    }

    fn consume_ratio_format(&mut self) -> Option<&'static str> {
        let format = if self.check(&TokenKind::Percent) || self.check(&TokenKind::PercentWord) {
            "as_percent"
        } else if self.ratio_word("fraction") {
            "as_fraction"
        } else if self.ratio_word("x")
            || self.ratio_word("multiplier")
            || self.ratio_word("multiple")
        {
            "as_multiplier"
        } else if self.ratio_word("dec") || self.ratio_word("decimal") {
            "as_decimal"
        } else {
            return None;
        };
        self.advance();
        Some(format)
    }

    fn finish_ratio_format(
        &mut self,
        left: Expression,
        format: &str,
    ) -> Result<Expression, CalculatorError> {
        let ratio = if self.check(&TokenKind::Of) || self.ratio_word("on") || self.ratio_word("off")
        {
            let off = self.ratio_word("off");
            let change = off || self.ratio_word("on");
            self.advance();
            let base = self.parse_multiplicative()?;
            let ratio = Expression::binary(left, BinaryOp::Divide, base);
            if change {
                let one = Expression::number(Decimal::new(1));
                if off {
                    Expression::binary(one, BinaryOp::Subtract, ratio)
                } else {
                    Expression::binary(ratio, BinaryOp::Subtract, one)
                }
            } else {
                ratio
            }
        } else {
            left
        };
        Ok(Expression::function_call(format, vec![ratio]))
    }
}
