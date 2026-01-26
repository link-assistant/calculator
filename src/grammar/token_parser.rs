//! Token-based expression parser.

use crate::error::CalculatorError;
use crate::grammar::{is_math_function, DateTimeGrammar, NumberGrammar, Token, TokenKind};
use crate::types::{BinaryOp, Expression, Unit};

/// Internal token-based parser.
pub struct TokenParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    number_grammar: &'a NumberGrammar,
    #[allow(dead_code)]
    original_input: &'a str,
}

impl<'a> TokenParser<'a> {
    pub const fn new(
        tokens: &'a [Token],
        number_grammar: &'a NumberGrammar,
        original_input: &'a str,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            number_grammar,
            original_input,
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, CalculatorError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_multiplicative()?;

        while let Some(op) = self.match_additive_op() {
            let right = self.parse_multiplicative()?;
            left = Expression::binary(left, op, right);
        }

        // Check for "at" keyword
        if self.check_at() {
            self.advance(); // consume "at"
            let time = self.parse_primary()?;
            left = Expression::at_time(left, time);
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_power()?;

        while let Some(op) = self.match_multiplicative_op() {
            let right = self.parse_power()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_unary()?;

        // Power is right-associative: 2^3^4 = 2^(3^4)
        if self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?; // Right-associative recursion
            left = Expression::power(left, right);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, CalculatorError> {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::negate(expr));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, CalculatorError> {
        // Parenthesized expression
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::group(expr));
        }

        // Number with optional unit
        if let Some(TokenKind::Number(n)) = self.current_kind() {
            let num_str = n.clone();
            self.advance();

            // Check for power operator after number (without unit)
            // This is handled in parse_power, so we just need to handle units here

            // Check for unit (identifier following number that is not a function)
            let unit = if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                // Don't treat function names as units
                if !is_math_function(id) && !self.peek_is_left_paren() {
                    let unit = self
                        .number_grammar
                        .parse_unit(id)
                        .unwrap_or_else(|_| Unit::Custom(id.clone()));
                    self.advance();
                    unit
                } else {
                    Unit::None
                }
            } else {
                Unit::None
            };

            let value = self.number_grammar.parse_number(&num_str)?;
            return Ok(Expression::number_with_unit(value, unit));
        }

        // Standalone identifier (could be a function call, unit, variable, or datetime part)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();
            self.advance();

            // Check if this is a function call (identifier followed by left paren)
            if self.check(&TokenKind::LeftParen) {
                return self.parse_function_call(&id);
            }

            // Check for natural integration syntax: "integrate <expr> d<var>"
            if id.to_lowercase() == "integrate" {
                return self.parse_natural_integral();
            }

            // If it looks like a datetime start (month name), try to parse more
            if DateTimeGrammar::looks_like_datetime(&id) {
                return self.try_parse_datetime_from_tokens(&id);
            }

            // Check if this is a math constant (pi, e)
            if is_math_function(&id) {
                // It's a constant like pi() or e() used without parens
                // Treat it as a zero-argument function call
                return Ok(Expression::function_call(id, vec![]));
            }

            // Allow single-letter identifiers as variables (for use in integrate, etc.)
            // Variables will be validated at evaluation time
            if id.len() == 1 && id.chars().next().unwrap().is_ascii_alphabetic() {
                return Ok(Expression::variable(id));
            }

            // Otherwise it's probably just an identifier/unit (which is an error in expression context)
            return Err(CalculatorError::parse(format!(
                "Unexpected identifier: {id}"
            )));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token: {:?}",
            self.current()
        )))
    }

    fn parse_function_call(&mut self, name: &str) -> Result<Expression, CalculatorError> {
        // We're positioned at the left paren
        self.expect(&TokenKind::LeftParen)?;

        let mut args = Vec::new();

        // Check for empty argument list
        if !self.check(&TokenKind::RightParen) {
            // Parse first argument
            args.push(self.parse_expression()?);

            // Parse remaining arguments
            while self.check(&TokenKind::Comma) {
                self.advance(); // consume comma
                args.push(self.parse_expression()?);
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(Expression::function_call(name, args))
    }

    fn try_parse_datetime_from_tokens(
        &mut self,
        first: &str,
    ) -> Result<Expression, CalculatorError> {
        // Collect tokens that might be part of a datetime
        let mut parts = vec![first.to_string()];

        // Look for patterns like: Jan 22, 2026 or Jan 27, 8:59am UTC
        // Collect: numbers, identifiers, colons, commas
        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Number(n)) => {
                    parts.push(n.clone());
                    self.advance();
                }
                Some(TokenKind::Identifier(id)) => {
                    parts.push(id.clone());
                    self.advance();
                }
                Some(TokenKind::Comma) => {
                    parts.push(",".to_string());
                    self.advance();
                }
                Some(TokenKind::Colon) => {
                    parts.push(":".to_string());
                    self.advance();
                }
                _ => break,
            }
        }

        let datetime_str = parts.join(" ").replace(" , ", ", ").replace(" : ", ":");

        match crate::types::DateTime::parse(&datetime_str) {
            Ok(dt) => Ok(Expression::DateTime(dt)),
            Err(e) => Err(e),
        }
    }

    /// Parses natural integral notation: "integrate <expr> d<var>"
    /// Examples:
    /// - integrate sin(x)/x dx
    /// - integrate x^2 dx
    fn parse_natural_integral(&mut self) -> Result<Expression, CalculatorError> {
        // We've already consumed "integrate", now we need to find the integrand and d<var>
        // Strategy: collect tokens until we find "d<var>" pattern (identifier starting with 'd')

        let start_pos = self.pos;
        let mut integrand_end_pos = None;
        let mut var_name = None;

        // Scan forward to find the d<var> pattern
        let mut scan_pos = self.pos;
        while scan_pos < self.tokens.len() {
            if let TokenKind::Identifier(id) = &self.tokens[scan_pos].kind {
                // Check if this is a differential notation like "dx", "dy", "dt"
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

        // If we didn't find d<var>, return an error with helpful message
        let (Some(end_pos), Some(var)) = (integrand_end_pos, var_name) else {
            return Err(CalculatorError::parse(
                "Invalid integration syntax. Expected: integrate <expression> d<var> (e.g., integrate sin(x)/x dx)"
            ));
        };

        // Reset position and parse the integrand expression
        // We need a sub-parser that only parses up to the d<var> token
        self.pos = start_pos;

        // Parse the integrand by parsing an expression and stopping at the d<var>
        let integrand = self.parse_integrand_until(end_pos)?;

        // Now consume the d<var> token
        self.pos = end_pos;
        self.advance();

        Ok(Expression::indefinite_integral(integrand, var))
    }

    /// Parse an integrand expression up to (but not including) the position `until_pos`.
    fn parse_integrand_until(&mut self, until_pos: usize) -> Result<Expression, CalculatorError> {
        // Save the tokens after until_pos temporarily
        let original_len = self.tokens.len();

        // We need to be careful - parse_expression will consume tokens
        // We'll parse and then check we didn't go past until_pos
        let result = self.parse_integrand_expression(until_pos)?;

        // Verify we stopped at the right place
        if self.pos > until_pos {
            self.pos = until_pos;
        }

        let _ = original_len; // Suppress unused warning
        Ok(result)
    }

    /// Parse integrand with awareness of the boundary.
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
                    // Put the operator back
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
                    // Put the operator back
                    self.pos -= 1;
                    break;
                }
                let right = self.parse_integrand_power(boundary)?;
                left = Expression::binary(left, op, right);
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

        // Parenthesized expression
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::group(expr));
        }

        // Number
        if let Some(TokenKind::Number(n)) = self.current_kind() {
            let num_str = n.clone();
            self.advance();
            let value = self.number_grammar.parse_number(&num_str)?;
            return Ok(Expression::number(value));
        }

        // Identifier (function call or variable)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();
            self.advance();

            // Check if this is a function call
            if self.pos < boundary && self.check(&TokenKind::LeftParen) {
                return self.parse_function_call(&id);
            }

            // Check if this is a math constant
            if is_math_function(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }

            // Single-letter identifier is a variable
            if id.len() == 1 && id.chars().next().unwrap().is_ascii_alphabetic() {
                return Ok(Expression::variable(id));
            }

            // Multi-letter identifier could be an implicit variable in integration context
            return Ok(Expression::variable(id));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token in integrand: {:?}",
            self.current()
        )))
    }

    fn match_additive_op(&mut self) -> Option<BinaryOp> {
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

    fn match_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.check(&TokenKind::Star) {
            self.advance();
            Some(BinaryOp::Multiply)
        } else if self.check(&TokenKind::Slash) {
            self.advance();
            Some(BinaryOp::Divide)
        } else {
            None
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current_kind()
            .is_some_and(|k| std::mem::discriminant(k) == std::mem::discriminant(kind))
    }

    fn check_at(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::At))
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current().map(|t| &t.kind)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    fn peek_is_left_paren(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::LeftParen))
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Eof) | None)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), CalculatorError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(CalculatorError::unexpected_token(
                &format!("{:?}", self.current_kind()),
                &format!("{kind:?}"),
                self.pos,
            ))
        }
    }
}
