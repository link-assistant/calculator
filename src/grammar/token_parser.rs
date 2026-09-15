//! Token-based expression parser.
mod comparison;
mod integral;
mod operators;
mod units;

use crate::error::CalculatorError;
use crate::grammar::{is_math_function, DateTimeGrammar, NumberGrammar, Token, TokenKind};
use crate::types::{BinaryOp, Decimal, Expression, Unit};

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

    pub fn parse_complete_expression(&mut self) -> Result<Expression, CalculatorError> {
        let expr = self.parse_expression()?;

        if self.is_at_end() {
            return Ok(expr);
        }

        let token = self.current().expect("non-EOF token should exist");
        Err(CalculatorError::parse(format!(
            "Unexpected trailing input '{}' at position {}",
            token.text, token.start
        )))
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

            // Check for "at" keyword after unit conversion (e.g. "22822 RUB in INR at Apr 11, 2026")
            if self.check_at() {
                self.advance(); // consume "at"
                let time = self.parse_primary()?;
                left = Expression::at_time(left, time);
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_power()?;

        loop {
            if let Some(op) = self.match_multiplicative_op() {
                let right = self.parse_power()?;
                left = Expression::binary(left, op, right);
            } else if self.implicit_multiplication_starts(&left) {
                let right = self.parse_power()?;
                left = Expression::binary(left, BinaryOp::Multiply, right);
            } else {
                break;
            }
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
        } else if let Some(TokenKind::Superscript(exponent)) = self.current_kind() {
            let exponent = self.number_grammar.parse_number(exponent)?;
            self.advance();
            left = Expression::power(left, Expression::number(exponent));
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
        // With optional "of <rhs>": expr% of rhs → (expr / 100) * rhs
        if matches!(self.current_kind(), Some(TokenKind::Percent))
            && !self.percent_starts_binary_expression()
        {
            self.advance();
            let percent_expr = Expression::binary(
                expr,
                BinaryOp::Divide,
                Expression::number(Decimal::new(100)),
            );
            if matches!(self.current_kind(), Some(TokenKind::Of)) {
                self.advance(); // consume "of"
                let rhs = self.parse_primary()?;
                return Ok(Expression::binary(percent_expr, BinaryOp::Multiply, rhs));
            }
            return Ok(percent_expr);
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

        // Placeholder unknowns for single-variable equations.
        if self.check(&TokenKind::Question) {
            self.advance();
            return Ok(Expression::variable("?"));
        }
        if self.check(&TokenKind::Star) {
            self.advance();
            return Ok(Expression::variable("*"));
        }

        // Numeric date literal (e.g. 2026-01-22, 15/10/2025, 15.10.2025).
        // The lexer already validated that this parses as a real calendar date.
        if let Some(TokenKind::DateLiteral(s)) = self.current_kind() {
            let s = s.clone();
            self.advance();

            // A time may follow the date literal, as in "2026-08-08 22:35" or
            // "15.10.2025 22:35 UTC". Try to absorb it, falling back to the
            // bare date when the combined form does not parse.
            if matches!(
                self.current_kind(),
                Some(TokenKind::Number(_) | TokenKind::Identifier(_))
            ) {
                let after_literal = self.pos;
                if let Ok(dt) = self.try_parse_datetime_from_tokens(&s) {
                    return Ok(dt);
                }
                self.pos = after_literal;
            }

            return crate::types::DateTime::parse(&s).map(Expression::DateTime);
        }

        // Number with optional unit
        if let Some(TokenKind::Number(n)) = self.current_kind() {
            let num_str = n.clone();
            let number_end = self.current().map_or(0, |token| token.end);
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

            // Russian shorthand: "11 по мск" means "11:00 by MSK".
            if matches!(self.current_kind(), Some(TokenKind::At)) {
                let save_before_po = self.pos;
                if let Ok(dt) = self.try_parse_russian_time_by_timezone(&num_str) {
                    return Ok(dt);
                }
                self.pos = save_before_po;
            }

            // If followed by an identifier that looks like a month name (e.g., "17 February 2027"
            // or "17 февраля 2027"), try to parse the whole as a datetime expression.
            if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                let is_ordinal_suffix =
                    matches!(id.to_lowercase().as_str(), "st" | "nd" | "rd" | "th");
                if DateTimeGrammar::looks_like_datetime(id) || is_ordinal_suffix {
                    let save_before_dt = self.pos;
                    if let Ok(dt) = self.try_parse_datetime_from_tokens(&num_str) {
                        return Ok(dt);
                    }
                    // Datetime parse failed — restore position and fall through to plain number
                    self.pos = save_before_dt;
                }
            }

            let mut value = self.number_grammar.parse_number(&num_str)?;
            if let Some(multiplier) = self.consume_adjacent_si_suffix(number_end) {
                value = value * multiplier;
            }

            // Check for unit (identifier following number that is not a function)
            let (unit, alternative_units) = if let Some(TokenKind::Identifier(id)) =
                self.current_kind()
            {
                // An unknown single-letter identifier is a conventional
                // coefficient (`2x` or `2 x`). Established short units
                // (`2h`) and multi-letter custom units retain their meaning.
                let coefficient_variable =
                    Self::is_single_letter_variable(id) && !self.identifier_is_known_unit(id);
                if !is_math_function(id) && !self.peek_is_left_paren() && !coefficient_variable {
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

            if let Some(function) = self.natural_root_function(&id) {
                self.advance(); // consume "square" / "cube"
                self.advance(); // consume "root"
                self.advance(); // consume "of"
                let argument = self.parse_unary()?;
                return Ok(Expression::function_call(function, vec![argument]));
            }

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

            // Resolve "today" at evaluation time so long-lived calculator
            // instances and configured local timezones use the current date.
            if id.eq_ignore_ascii_case("today") {
                self.advance();
                return Ok(Expression::Today);
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

            // Check if this is a function call (identifier followed by left paren).
            // `x(x - 3)` is conventional implicit polynomial multiplication.
            if self.check(&TokenKind::LeftParen) {
                if Self::is_single_letter_variable(&id) && !is_math_function(&id) {
                    return Ok(Expression::variable(id));
                }
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

            if Self::is_math_constant(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }
            if Self::is_unary_math_function(&id) && !self.is_at_end() {
                let argument = self.parse_unary()?;
                return Ok(Expression::function_call(id, vec![argument]));
            }
            if is_math_function(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }

            // Allow single-letter identifiers as variables (for use in integrate, etc.)
            // Variables will be validated at evaluation time
            if Self::is_single_letter_variable(&id) {
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

    fn consume_adjacent_si_suffix(&mut self, number_end: usize) -> Option<Decimal> {
        let suffix = self.current().and_then(|token| {
            if token.start != number_end {
                return None;
            }

            let TokenKind::Identifier(id) = &token.kind else {
                return None;
            };

            Some(id.clone())
        })?;
        let multiplier = NumberGrammar::si_suffix_multiplier(&suffix)?;

        // Preserve established adjacent unit abbreviations like `2h in minutes`.
        // They become SI suffixes only in suffix-before-unit forms like `5h USD`.
        if self.identifier_is_known_unit(&suffix) && !self.peek_identifier_is_known_unit() {
            return None;
        }

        self.advance();
        Some(multiplier)
    }

    fn identifier_is_known_unit(&self, id: &str) -> bool {
        matches!(
            self.number_grammar.parse_unit_with_alternatives(id),
            Ok((unit, _)) if !matches!(unit, Unit::Custom(_) | Unit::None)
        )
    }

    fn peek_identifier_is_known_unit(&self) -> bool {
        match self.peek_kind() {
            Some(TokenKind::Identifier(id)) if !is_math_function(id) => {
                self.identifier_is_known_unit(id)
            }
            _ => false,
        }
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
                Some(TokenKind::DateLiteral(date)) => {
                    parts.push(date.clone());
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
                Some(TokenKind::Of) => {
                    parts.push("of".to_string());
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
                Some(TokenKind::DateLiteral(date)) => {
                    parts.push(date.clone());
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
                Some(TokenKind::At) => {
                    let save_pos = self.pos;
                    self.advance();
                    if let Some(TokenKind::Identifier(tz_id)) = self.current_kind() {
                        if crate::types::DateTime::parse_tz_abbreviation(tz_id).is_some() {
                            parts.push(tz_id.clone());
                            self.advance();
                            continue;
                        }
                    }
                    self.pos = save_pos;
                    break;
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

    /// Tries to parse Russian "N по <TZ>" as "N:00 <TZ>".
    fn try_parse_russian_time_by_timezone(
        &mut self,
        hour_str: &str,
    ) -> Result<Expression, CalculatorError> {
        self.advance(); // consume "по"

        let Some(TokenKind::Identifier(tz_id)) = self.current_kind() else {
            return Err(CalculatorError::parse("Expected timezone after по"));
        };
        let tz = tz_id.clone();
        if crate::types::DateTime::parse_tz_abbreviation(&tz).is_none() {
            return Err(CalculatorError::parse(format!("Unknown timezone: {tz}")));
        }
        self.advance();

        let datetime_str = format!("{hour_str}:00 {tz}");
        crate::types::DateTime::parse(&datetime_str).map(Expression::DateTime)
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
                Some(TokenKind::Of) => {
                    token_positions.push(self.pos);
                    parts.push("of".to_string());
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

    fn implicit_multiplication_starts(&self, left: &Expression) -> bool {
        let left_can_multiply = matches!(
            left,
            Expression::Number {
                unit: Unit::None,
                ..
            } | Expression::Group(_)
                | Expression::FunctionCall { .. }
                | Expression::Variable(_)
                | Expression::Power { .. }
                | Expression::Negate(_)
        );
        if !left_can_multiply {
            return false;
        }

        match self.current_kind() {
            Some(TokenKind::LeftParen) => true,
            Some(TokenKind::Identifier(id)) => {
                is_math_function(id) || Self::is_single_letter_variable(id)
            }
            Some(TokenKind::Number(_)) => {
                matches!(left, Expression::Group(_) | Expression::FunctionCall { .. })
            }
            _ => false,
        }
    }

    fn natural_root_function(&self, degree: &str) -> Option<&'static str> {
        let function = if degree.eq_ignore_ascii_case("square") {
            "sqrt"
        } else if degree.eq_ignore_ascii_case("cube") {
            "cbrt"
        } else {
            return None;
        };
        let has_root = matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(TokenKind::Identifier(root)) if root.eq_ignore_ascii_case("root")
        );
        let has_of = matches!(
            self.tokens.get(self.pos + 2).map(|token| &token.kind),
            Some(TokenKind::Of)
        );

        (has_root && has_of).then_some(function)
    }

    fn is_single_letter_variable(id: &str) -> bool {
        let mut chars = id.chars();
        chars
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
            && chars.next().is_none()
    }

    fn is_math_constant(id: &str) -> bool {
        matches!(id.to_ascii_lowercase().as_str(), "pi" | "e")
    }

    fn is_unary_math_function(id: &str) -> bool {
        matches!(
            id.to_ascii_lowercase().as_str(),
            "sin"
                | "cos"
                | "tan"
                | "asin"
                | "acos"
                | "atan"
                | "sinh"
                | "cosh"
                | "tanh"
                | "exp"
                | "ln"
                | "log"
                | "log2"
                | "log10"
                | "sqrt"
                | "cbrt"
                | "abs"
                | "floor"
                | "ceil"
                | "round"
                | "trunc"
                | "sign"
                | "signum"
                | "factorial"
                | "deg"
                | "degrees"
                | "rad"
                | "radians"
        )
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
