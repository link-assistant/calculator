use crate::error::CalculatorError;
use crate::grammar::{is_math_function, TokenKind};
use crate::types::{BinaryOp, Expression};

use super::TokenParser;

impl TokenParser<'_> {
    /// Parses natural integral notation: "integrate <expr> d<var>".
    ///
    /// Examples include `integrate sin(x)/x dx` and `integrate x^2 dx`.
    pub(super) fn parse_natural_integral(&mut self) -> Result<Expression, CalculatorError> {
        let start_pos = self.pos;
        let mut integrand_end_pos = None;
        let mut var_name = None;

        // Scan forward to find differential notation such as `dx`, `dy`, or `dt`.
        let mut scan_pos = self.pos;
        while scan_pos < self.tokens.len() {
            if let TokenKind::Identifier(id) = &self.tokens[scan_pos].kind {
                let id_lower = id.to_lowercase();
                if id_lower.starts_with('d') && id_lower.len() == 2 {
                    let var_char = id_lower.chars().nth(1).unwrap();
                    if var_char.is_ascii_alphabetic() {
                        integrand_end_pos = Some(scan_pos);
                        var_name = Some(var_char.to_string());
                        break;
                    }
                }
            }
            scan_pos += 1;
        }

        let (Some(end_pos), Some(var)) = (integrand_end_pos, var_name) else {
            return Err(CalculatorError::parse(
                "Invalid integration syntax. Expected: integrate <expression> d<var> (e.g., integrate sin(x)/x dx)",
            ));
        };

        self.pos = start_pos;
        let integrand = self.parse_integrand_until(end_pos)?;

        self.pos = end_pos;
        self.advance();

        Ok(Expression::indefinite_integral(integrand, var))
    }

    /// Parse an integrand expression up to (but not including) `until_pos`.
    fn parse_integrand_until(&mut self, until_pos: usize) -> Result<Expression, CalculatorError> {
        let result = self.parse_integrand_expression(until_pos)?;
        if self.pos > until_pos {
            self.pos = until_pos;
        }
        Ok(result)
    }

    fn parse_integrand_expression(
        &mut self,
        boundary: usize,
    ) -> Result<Expression, CalculatorError> {
        self.parse_integrand_additive(boundary)
    }

    fn parse_integrand_additive(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_multiplicative(boundary)?;

        while self.pos < boundary {
            if let Some(op) = self.match_additive_op() {
                if self.pos >= boundary {
                    self.pos -= 1;
                    break;
                }
                let right = self.parse_integrand_multiplicative(boundary)?;
                left = Expression::binary(left, op, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_integrand_multiplicative(
        &mut self,
        boundary: usize,
    ) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_power(boundary)?;

        while self.pos < boundary {
            if let Some(op) = self.match_multiplicative_op() {
                if self.pos >= boundary {
                    self.pos -= 1;
                    break;
                }
                let right = self.parse_integrand_power(boundary)?;
                left = Expression::binary(left, op, right);
            } else if self.implicit_multiplication_starts(&left) {
                let right = self.parse_integrand_power(boundary)?;
                left = Expression::binary(left, BinaryOp::Multiply, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_integrand_power(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_unary(boundary)?;

        if self.pos < boundary && self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_integrand_power(boundary)?;
            left = Expression::power(left, right);
        } else if self.pos < boundary {
            if let Some(TokenKind::Superscript(exponent)) = self.current_kind() {
                let exponent = self.number_grammar.parse_number(exponent)?;
                self.advance();
                left = Expression::power(left, Expression::number(exponent));
            }
        }

        Ok(left)
    }

    fn parse_integrand_unary(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        if self.pos < boundary && self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_integrand_unary(boundary)?;
            return Ok(Expression::negate(expr));
        }

        self.parse_integrand_primary(boundary)
    }

    fn parse_integrand_primary(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        if self.pos >= boundary {
            return Err(CalculatorError::parse("Unexpected end of integrand"));
        }

        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::group(expr));
        }

        if let Some(TokenKind::Number(number)) = self.current_kind() {
            let number = number.clone();
            self.advance();
            let value = self.number_grammar.parse_number(&number)?;
            return Ok(Expression::number(value));
        }

        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();
            self.advance();

            if self.pos < boundary && self.check(&TokenKind::LeftParen) {
                if Self::is_single_letter_variable(&id) && !is_math_function(&id) {
                    return Ok(Expression::variable(id));
                }
                return self.parse_function_call(&id);
            }

            if Self::is_math_constant(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }
            if Self::is_unary_math_function(&id) && self.pos < boundary {
                let argument = self.parse_integrand_unary(boundary)?;
                return Ok(Expression::function_call(id, vec![argument]));
            }
            if is_math_function(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }
            if Self::is_single_letter_variable(&id) {
                return Ok(Expression::variable(id));
            }

            return Ok(Expression::variable(id));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token in integrand: {:?}",
            self.current()
        )))
    }
}
