//! Token-based expression parser.

use crate::error::CalculatorError;
use crate::grammar::{is_math_function, DateTimeGrammar, NumberGrammar, Token, TokenKind};
use crate::types::{BinaryOp, DataSizeUnit, Decimal, Expression, MassUnit, Unit};

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
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, CalculatorError> {
        let left = self.parse_additive()?;

        if self.check(&TokenKind::Equals) {
            self.advance(); // consume '='
            let right = self.parse_additive()?;
            return Ok(Expression::equality(left, right));
        }

        Ok(left)
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

        // Check for "as", "in", or "to" keyword (unit conversion, e.g. "741 KB as MB", "19 TON in USD")
        if self.check_as() || self.check_in() || self.check_to() {
            self.advance(); // consume "as"/"in"/"to"
            let target_unit = self.parse_unit_for_conversion()?;

            // Resolve unit ambiguity using conversion context.
            // If the source has an ambiguous unit (e.g., "ton" parsed as Mass with Currency
            // alternative) and the target unit category doesn't match the primary but matches
            // an alternative, swap to the alternative interpretation.
            left = Self::resolve_unit_ambiguity_for_conversion(left, &target_unit);

            left = Expression::unit_conversion(left, target_unit);
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

        let expr = self.parse_primary()?;

        // Handle postfix percent operator: expr% → expr / 100
        if matches!(self.current_kind(), Some(TokenKind::Percent)) {
            self.advance();
            return Ok(Expression::binary(
                expr,
                BinaryOp::Divide,
                Expression::number(Decimal::new(100)),
            ));
        }

        // Handle postfix factorial operator: expr! → factorial(expr)
        if matches!(self.current_kind(), Some(TokenKind::Bang)) {
            self.advance();
            return Ok(Expression::function_call("factorial", vec![expr]));
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, CalculatorError> {
        // Handle "until" keyword: "until <datetime>"
        if self.check_until() {
            self.advance(); // consume "until"
                            // Try to parse the rest as a datetime directly by collecting tokens
            let save_pos = self.pos;
            if let Ok(target) = self.try_parse_until_target() {
                return Ok(Expression::Until(Box::new(target)));
            }
            // Fallback: parse as normal expression
            self.pos = save_pos;
            let target = self.parse_primary()?;
            return Ok(Expression::Until(Box::new(target)));
        }

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
            let save_pos = self.pos;
            self.advance();

            // If followed by a Colon, this might be a time like "11:59pm EST on Monday, January 26th"
            // Try collecting all remaining tokens as a datetime string first.
            if matches!(self.current_kind(), Some(TokenKind::Colon)) {
                if let Ok(dt) = self.try_parse_time_starting_with_number(&num_str) {
                    return Ok(dt);
                }
                // Datetime parse failed — restore position and fall through to plain number
                self.pos = save_pos;
                self.advance(); // re-consume the number token
            }

            // If followed by AM/PM, this is a time like "6 PM", "6 PM GMT", "6 PM MSK"
            // Try to parse as datetime (with optional timezone) before treating as unit.
            if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                let id_lower = id.to_lowercase();
                if id_lower == "am" || id_lower == "pm" {
                    let save_before_ampm = self.pos;
                    if let Ok(dt) = self.try_parse_time_with_ampm(&num_str) {
                        return Ok(dt);
                    }
                    // Datetime parse failed — restore to before AM/PM
                    self.pos = save_before_ampm;
                }
            }

            // If followed by an identifier that looks like a month name (e.g., "17 February 2027"
            // or "17 февраля 2027"), try to parse the whole as a datetime expression.
            if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                if DateTimeGrammar::looks_like_datetime(id) {
                    let save_before_dt = self.pos;
                    if let Ok(dt) = self.try_parse_datetime_from_tokens(&num_str) {
                        return Ok(dt);
                    }
                    // Datetime parse failed — restore position and fall through to plain number
                    self.pos = save_before_dt;
                }
            }

            // Check for unit (identifier following number that is not a function)
            let (unit, alternative_units) =
                if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                    // Don't treat function names as units
                    if !is_math_function(id) && !self.peek_is_left_paren() {
                        let (unit, alts) = self
                            .number_grammar
                            .parse_unit_with_alternatives(id)
                            .unwrap_or_else(|_| (Unit::Custom(id.clone()), Vec::new()));
                        self.advance();
                        (unit, alts)
                    } else {
                        (Unit::None, Vec::new())
                    }
                } else {
                    (Unit::None, Vec::new())
                };

            let value = self.number_grammar.parse_number(&num_str)?;
            if alternative_units.is_empty() {
                return Ok(Expression::number_with_unit(value, unit));
            }
            return Ok(Expression::number_with_unit_alternatives(
                value,
                unit,
                alternative_units,
            ));
        }

        // Standalone identifier (could be a function call, unit, variable, or datetime part)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();

            // Check for "now" keyword
            if id.to_lowercase() == "now" {
                self.advance();
                // Check for optional timezone after "now" (e.g., "now UTC", "now EST")
                if let Some(TokenKind::Identifier(tz)) = self.current_kind() {
                    let tz_lower = tz.to_lowercase();
                    // Collect "now" + timezone as a combined datetime string
                    if crate::types::DateTime::parse(&format!("now {tz_lower}")).is_ok() {
                        let combined = format!("now {}", tz);
                        self.advance();
                        match crate::types::DateTime::parse(&combined) {
                            Ok(dt) => return Ok(Expression::DateTime(dt)),
                            Err(_) => return Ok(Expression::Now),
                        }
                    }
                }
                return Ok(Expression::Now);
            }

            // Check for prefix currency symbol notation (e.g., $10, €5, £3).
            if id.chars().count() == 1 {
                let ch = id.chars().next().unwrap();
                if !ch.is_ascii_alphabetic() {
                    if let Some(currency_code) = crate::types::CurrencyDatabase::parse_currency(&id)
                    {
                        if let Some(TokenKind::Number(_)) = self.peek_kind() {
                            self.advance(); // consume currency symbol
                            if let Some(TokenKind::Number(n)) = self.current_kind() {
                                let num_str = n.clone();
                                self.advance();
                                let value = self.number_grammar.parse_number(&num_str)?;
                                return Ok(Expression::number_with_unit(
                                    value,
                                    Unit::currency(&currency_code),
                                ));
                            }
                        }
                    }
                }
            }

            self.advance();

            // Check if this is a function call (identifier followed by left paren)
            if self.check(&TokenKind::LeftParen) {
                return self.parse_function_call(&id);
            }

            // Check for natural integration syntax: "integrate <expr> d<var>"
            if id.to_lowercase() == "integrate" {
                return self.parse_natural_integral();
            }

            // If it looks like a datetime start (month name, "time", "current", etc.), try to parse more
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

    /// Tries to parse the remaining tokens after "until" as a datetime expression.
    /// Handles cases like "until 11:59pm EST January 26th" where the datetime
    /// starts with a number rather than a month name.
    fn try_parse_until_target(&mut self) -> Result<Expression, CalculatorError> {
        let mut parts: Vec<String> = Vec::new();

        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Number(n)) => {
                    parts.push(n.clone());
                    self.advance();
                }
                Some(TokenKind::Identifier(id)) => {
                    let id_lower = id.to_lowercase();
                    if matches!(id_lower.as_str(), "st" | "nd" | "rd" | "th") {
                        if let Some(last) = parts.last_mut() {
                            if last.chars().all(|c| c.is_ascii_digit()) {
                                last.push_str(id);
                                self.advance();
                                continue;
                            }
                        }
                    }
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

        if parts.is_empty() {
            return Err(CalculatorError::parse(
                "until requires a datetime expression",
            ));
        }

        let datetime_str = parts.join(" ").replace(" , ", ", ").replace(" : ", ":");
        match crate::types::DateTime::parse(&datetime_str) {
            Ok(dt) => Ok(Expression::DateTime(dt)),
            Err(e) => Err(e),
        }
    }

    /// Tries to parse a time/datetime expression that starts with a number followed by a colon,
    /// e.g. "11:59pm EST on Monday, January 26th".
    /// The `hour_str` is the number already consumed, and the current position is at the Colon.
    fn try_parse_time_starting_with_number(
        &mut self,
        hour_str: &str,
    ) -> Result<Expression, CalculatorError> {
        let mut parts = vec![hour_str.to_string()];

        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Number(n)) => {
                    parts.push(n.clone());
                    self.advance();
                }
                Some(TokenKind::Identifier(id)) => {
                    let id_lower = id.to_lowercase();
                    // Attach ordinal suffixes directly to preceding number
                    if matches!(id_lower.as_str(), "st" | "nd" | "rd" | "th") {
                        if let Some(last) = parts.last_mut() {
                            if last.chars().all(|c| c.is_ascii_digit()) {
                                last.push_str(id);
                                self.advance();
                                continue;
                            }
                        }
                    }
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

    /// Tries to parse "N AM/PM [TZ]" as a time expression.
    ///
    /// Called when the parser has consumed a number token and sees AM/PM next.
    /// Handles patterns like "6 PM", "6 PM GMT", "6 PM MSK".
    /// The timezone identifier is only consumed if it is a recognized timezone abbreviation.
    fn try_parse_time_with_ampm(&mut self, hour_str: &str) -> Result<Expression, CalculatorError> {
        // Consume the AM/PM identifier
        let ampm = if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id_str = id.clone();
            self.advance();
            id_str
        } else {
            return Err(CalculatorError::parse("Expected AM/PM"));
        };

        // Check if the next identifier is a recognized timezone abbreviation
        let mut datetime_str = format!("{hour_str}:00 {ampm}");
        if let Some(TokenKind::Identifier(tz_id)) = self.current_kind() {
            if crate::types::DateTime::parse_tz_abbreviation(tz_id).is_some() {
                datetime_str = format!("{hour_str}:00 {} {}", ampm, tz_id);
                self.advance(); // consume timezone token
            }
        }

        match crate::types::DateTime::parse(&datetime_str) {
            Ok(dt) => Ok(Expression::DateTime(dt)),
            Err(e) => Err(e),
        }
    }

    fn try_parse_datetime_from_tokens(
        &mut self,
        first: &str,
    ) -> Result<Expression, CalculatorError> {
        // Collect tokens that might be part of a datetime
        let mut parts = vec![first.to_string()];
        // Track token positions so we can backtrack to any prefix
        let mut token_positions = vec![self.pos]; // position before each token was consumed

        // Look for patterns like: Jan 22, 2026 or Jan 27, 8:59am UTC
        // Also handles: Monday, January 26th, 2026
        // Collect: numbers, identifiers, colons, commas
        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Number(n)) => {
                    token_positions.push(self.pos);
                    parts.push(n.clone());
                    self.advance();
                }
                Some(TokenKind::Identifier(id)) => {
                    // Handle ordinal suffixes: if previous part is a number and
                    // this is "st", "nd", "rd", or "th", attach without space
                    let id_lower = id.to_lowercase();
                    if matches!(id_lower.as_str(), "st" | "nd" | "rd" | "th") {
                        // Append to previous number part (ordinal suffix)
                        if let Some(last) = parts.last_mut() {
                            if last.chars().all(|c| c.is_ascii_digit()) {
                                token_positions.push(self.pos);
                                last.push_str(id);
                                self.advance();
                                continue;
                            }
                        }
                    }
                    token_positions.push(self.pos);
                    parts.push(id.clone());
                    self.advance();
                }
                Some(TokenKind::Comma) => {
                    token_positions.push(self.pos);
                    parts.push(",".to_string());
                    self.advance();
                }
                Some(TokenKind::Colon) => {
                    token_positions.push(self.pos);
                    parts.push(":".to_string());
                    self.advance();
                }
                _ => break,
            }
        }

        // Try the full collected string first, then progressively shorter prefixes.
        // This handles cases like "17 февраля 2027 - 6 months" where the datetime
        // is "17 февраля 2027" but greedily collecting too many tokens would fail.
        let end_pos = self.pos;
        for len in (1..=parts.len()).rev() {
            let candidate_parts = &parts[..len];
            let datetime_str = candidate_parts
                .join(" ")
                .replace(" , ", ", ")
                .replace(" : ", ":");
            if let Ok(dt) = crate::types::DateTime::parse(&datetime_str) {
                // Restore position to just after the tokens we actually consumed
                // token_positions[len - 1] is the position *before* consuming parts[len-1]
                // so the position after consuming parts[len-1] is:
                // - for the last element: end_pos (we already advanced past all)
                // - for shorter prefix: token_positions[len] (position before parts[len])
                if len < parts.len() {
                    self.pos = token_positions[len];
                } else {
                    self.pos = end_pos;
                }
                return Ok(Expression::DateTime(dt));
            }
        }

        // No prefix worked — restore original position and report error
        if parts.is_empty() {
            return Err(crate::error::CalculatorError::parse("empty datetime"));
        }
        // Return error with the full string for better error messages
        let datetime_str = parts.join(" ").replace(" , ", ", ").replace(" : ", ":");
        Err(crate::error::CalculatorError::InvalidDateTime(format!(
            "Could not parse '{datetime_str}' as a date or time"
        )))
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

    fn check_as(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::As))
    }

    fn check_in(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::In))
    }

    fn check_to(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::To))
    }

    fn check_until(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Until))
    }

    /// Parses a unit name after the `as`, `in`, or `to` keyword.
    ///
    /// Handles:
    /// - Data size units (e.g., `MB`, `KiB`, `mebibytes`)
    /// - Mass units (e.g., `kg`, `tons`, `pounds`)
    /// - Currency codes and natural language names (e.g., `USD`, `dollars`, `BTC`, `toncoin`)
    fn parse_unit_for_conversion(&mut self) -> Result<Unit, CalculatorError> {
        // The next token should be an identifier (the unit name)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let unit_str = id.clone();
            self.advance();

            // Try to parse the unit string (exact match)
            if let Some(data_size) = DataSizeUnit::parse(&unit_str) {
                return Ok(Unit::DataSize(data_size));
            }

            // Try mass unit (e.g., "kg", "tons", "pounds")
            if let Some(mass) = MassUnit::parse(&unit_str) {
                return Ok(Unit::Mass(mass));
            }

            // Try case-insensitive matching for data size
            let lower = unit_str.to_lowercase();
            if let Some(data_size) = DataSizeUnit::parse(&lower) {
                return Ok(Unit::DataSize(data_size));
            }

            // Try case-insensitive mass unit
            if let Some(mass) = MassUnit::parse(&lower) {
                return Ok(Unit::Mass(mass));
            }

            // Try timezone abbreviation (before currency, since currency catch-all matches any 2-5 letter code)
            if crate::types::DateTime::parse_tz_abbreviation(&unit_str).is_some() {
                return Ok(Unit::Timezone(unit_str.to_uppercase()));
            }

            // Try treating it as a currency code or natural language alias
            // (e.g., "USD", "EUR", "dollars", "euros", "BTC", "toncoin")
            if let Some(currency_code) = crate::types::CurrencyDatabase::parse_currency(&unit_str) {
                return Ok(Unit::currency(&currency_code));
            }

            // Return error with helpful message
            Err(CalculatorError::parse(format!(
                "Unknown unit '{unit_str}'. Supported conversions: \
                 data sizes (B, KB, MB, GB, KiB, MiB, GiB, ...), \
                 mass (g, kg, tons, lb, oz), \
                 currencies (USD, EUR, GBP, TON, BTC, ETH, ...) and natural language \
                 aliases (dollars, euros, bitcoin, toncoin, ...), \
                 timezones (UTC, GMT, EST, MSK, JST, ...)."
            )))
        } else {
            Err(CalculatorError::parse(
                "Expected a unit name after 'as'/'in'/'to' (e.g., 'MB', 'kg', 'USD', 'dollars')",
            ))
        }
    }

    /// Resolves unit ambiguity when a conversion target provides context.
    ///
    /// For example, in "19 ton in usd": "ton" is initially parsed as Mass(MetricTon)
    /// with Currency("TON") as an alternative. Since the target is Currency("USD"),
    /// and Mass→Currency conversion is not valid but Currency→Currency is, this method
    /// swaps the primary unit to the matching alternative.
    fn resolve_unit_ambiguity_for_conversion(expr: Expression, target_unit: &Unit) -> Expression {
        if let Expression::Number {
            value,
            ref unit,
            ref alternative_units,
        } = expr
        {
            if alternative_units.is_empty() {
                return expr;
            }

            // Check if the primary unit is already compatible with the target
            if unit.is_same_category(target_unit) {
                return expr;
            }

            // Look for an alternative that is compatible with the target
            for alt in alternative_units {
                if alt.is_same_category(target_unit) {
                    // Swap: make the compatible alternative the primary,
                    // and move the original primary to alternatives
                    let mut new_alternatives: Vec<Unit> = alternative_units
                        .iter()
                        .filter(|u| *u != alt)
                        .cloned()
                        .collect();
                    new_alternatives.push(unit.clone());
                    return Expression::number_with_unit_alternatives(
                        value,
                        alt.clone(),
                        new_alternatives,
                    );
                }
            }
        }
        expr
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
