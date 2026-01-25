//! Expression parser that combines all grammars.

use crate::error::CalculatorError;
use crate::grammar::{DateTimeGrammar, Lexer, NumberGrammar, Token, TokenKind};
use crate::types::{BinaryOp, CurrencyDatabase, Expression, Unit, Value};

/// Parser for calculator expressions.
#[derive(Debug, Default)]
pub struct ExpressionParser {
    number_grammar: NumberGrammar,
    datetime_grammar: DateTimeGrammar,
    currency_db: CurrencyDatabase,
}

impl ExpressionParser {
    /// Creates a new expression parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            number_grammar: NumberGrammar::new(),
            datetime_grammar: DateTimeGrammar::new(),
            currency_db: CurrencyDatabase::new(),
        }
    }

    /// Parses and evaluates an expression, returning the result, steps, and lino representation.
    pub fn parse_and_evaluate(
        &self,
        input: &str,
    ) -> Result<(Value, Vec<String>, String), CalculatorError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CalculatorError::EmptyInput);
        }

        // Try datetime subtraction pattern first: "(datetime) - (datetime)"
        if let Some(result) = self.try_parse_datetime_subtraction(input) {
            return Ok(result);
        }

        // Parse the expression
        let expr = self.parse(input)?;

        // Generate links notation representation
        let lino = expr.to_lino();

        // Evaluate with step tracking
        let (value, steps) = self.evaluate_with_steps(&expr)?;

        Ok((value, steps, lino))
    }

    /// Tries to parse a datetime subtraction expression like "(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)".
    fn try_parse_datetime_subtraction(&self, input: &str) -> Option<(Value, Vec<String>, String)> {
        // Look for pattern: (datetime) - (datetime)
        let input = input.trim();

        // Check if it starts with '(' and contains '-'
        if !input.starts_with('(') || !input.contains('-') {
            return None;
        }

        // Try to find the matching closing paren for the first datetime
        let mut paren_depth = 0;
        let mut first_end = None;

        for (i, ch) in input.char_indices() {
            match ch {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        first_end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let first_end = first_end?;

        // Extract first datetime (without parens)
        let first_dt_str = &input[1..first_end];

        // Find the minus sign
        let rest = input[first_end + 1..].trim();
        if !rest.starts_with('-') {
            return None;
        }

        let second_part = rest[1..].trim();
        if !second_part.starts_with('(') || !second_part.ends_with(')') {
            return None;
        }

        // Extract second datetime (without parens)
        let second_dt_str = &second_part[1..second_part.len() - 1];

        // Try to parse both as datetimes
        let Ok(dt1) = self.datetime_grammar.parse(first_dt_str) else {
            return None;
        };

        let Ok(dt2) = self.datetime_grammar.parse(second_dt_str) else {
            return None;
        };

        // Calculate the difference
        let diff = dt1.subtract(&dt2);
        #[allow(clippy::cast_possible_wrap)]
        let seconds = diff.as_secs() as i64;

        let value = Value::duration(seconds);

        let steps = vec![
            format!("Parse first datetime: {dt1}"),
            format!("Parse second datetime: {dt2}"),
            format!("Calculate difference: {dt1} - {dt2}"),
            format!("Result: {}", value.to_display_string()),
        ];

        let lino = format!("((({}) - ({})))", first_dt_str.trim(), second_dt_str.trim());

        Some((value, steps, lino))
    }

    /// Parses an expression string into an Expression AST.
    pub fn parse(&self, input: &str) -> Result<Expression, CalculatorError> {
        // Pre-check for advanced math expressions before tokenizing
        // This catches cases where special characters (^, =, >) would cause parse errors
        if let Some(keyword) = detect_advanced_math_keyword(input) {
            return Err(CalculatorError::unsupported_math(keyword, input));
        }

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = TokenParser::new(&tokens, &self.number_grammar, input);
        parser.parse_expression()
    }

    /// Evaluates an expression.
    pub fn evaluate(&self, expr: &Expression) -> Result<Value, CalculatorError> {
        self.evaluate_expr(expr)
    }

    /// Evaluates an expression with step-by-step tracking.
    fn evaluate_with_steps(
        &self,
        expr: &Expression,
    ) -> Result<(Value, Vec<String>), CalculatorError> {
        let mut steps = Vec::new();

        steps.push(format!("Input expression: {expr}"));

        let result = self.evaluate_expr_with_steps(expr, &mut steps)?;

        steps.push(format!("Final result: {}", result.to_display_string()));

        Ok((result, steps))
    }

    fn evaluate_expr(&self, expr: &Expression) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit } => Ok(Value::number_with_unit(*value, unit.clone())),
            Expression::DateTime(dt) => Ok(Value::datetime(dt.clone())),
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.apply_binary_op(&left_val, *op, &right_val)
            }
            Expression::Negate(inner) => {
                let val = self.evaluate_expr(inner)?;
                Ok(val.negate())
            }
            Expression::Group(inner) => self.evaluate_expr(inner),
            Expression::AtTime { value, time } => {
                // For now, just evaluate the value
                // In the future, this would use the time for currency conversion
                let _time_val = self.evaluate_expr(time)?;
                self.evaluate_expr(value)
            }
        }
    }

    fn evaluate_expr_with_steps(
        &self,
        expr: &Expression,
        steps: &mut Vec<String>,
    ) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit } => {
                let val = Value::number_with_unit(*value, unit.clone());
                steps.push(format!("Literal value: {}", val.to_display_string()));
                Ok(val)
            }
            Expression::DateTime(dt) => {
                steps.push(format!("DateTime value: {dt}"));
                Ok(Value::datetime(dt.clone()))
            }
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expr_with_steps(left, steps)?;
                let right_val = self.evaluate_expr_with_steps(right, steps)?;

                steps.push(format!(
                    "Compute: {} {} {}",
                    left_val.to_display_string(),
                    op,
                    right_val.to_display_string()
                ));

                let result = self.apply_binary_op(&left_val, *op, &right_val)?;
                steps.push(format!("= {}", result.to_display_string()));

                Ok(result)
            }
            Expression::Negate(inner) => {
                let val = self.evaluate_expr_with_steps(inner, steps)?;
                let result = val.negate();
                steps.push(format!("Negate: -{val} = {result}"));
                Ok(result)
            }
            Expression::Group(inner) => {
                steps.push("Evaluate grouped expression:".to_string());
                self.evaluate_expr_with_steps(inner, steps)
            }
            Expression::AtTime { value, time } => {
                let time_val = self.evaluate_expr_with_steps(time, steps)?;
                steps.push(format!("At time: {}", time_val.to_display_string()));
                self.evaluate_expr_with_steps(value, steps)
            }
        }
    }

    fn apply_binary_op(
        &self,
        left: &Value,
        op: BinaryOp,
        right: &Value,
    ) -> Result<Value, CalculatorError> {
        match op {
            BinaryOp::Add => left.add(right, &self.currency_db),
            BinaryOp::Subtract => left.subtract(right, &self.currency_db),
            BinaryOp::Multiply => left.multiply(right),
            BinaryOp::Divide => left.divide(right),
        }
    }
}

/// Keywords that indicate advanced mathematical expressions.
/// These are not supported by this calculator but can be handled by Wolfram Alpha.
const ADVANCED_MATH_KEYWORDS: &[&str] = &[
    "integrate",
    "integral",
    "differentiate",
    "derivative",
    "diff",
    "solve",
    "limit",
    "lim",
    "sum",
    "product",
    "series",
    "taylor",
    "fourier",
    "laplace",
    "simplify",
    "expand",
    "factor",
    "roots",
    "zeros",
    "plot",
    "graph",
    "sin",
    "cos",
    "tan",
    "log",
    "ln",
    "exp",
    "sqrt",
    "abs",
    "mod",
    "gcd",
    "lcm",
    "prime",
    "factorial",
    "permutation",
    "combination",
    "matrix",
    "determinant",
    "eigenvalue",
    "eigenvector",
];

/// Checks if an identifier is an advanced math keyword.
fn is_advanced_math_keyword(id: &str) -> bool {
    ADVANCED_MATH_KEYWORDS
        .iter()
        .any(|&keyword| id.to_lowercase() == keyword)
}

/// Detects advanced math keywords in the input string.
/// Returns the first keyword found, or None if no advanced math keyword is detected.
fn detect_advanced_math_keyword(input: &str) -> Option<&'static str> {
    let input_lower = input.to_lowercase();

    for &keyword in ADVANCED_MATH_KEYWORDS {
        // Check if the keyword appears as a word (not part of another word)
        if let Some(pos) = input_lower.find(keyword) {
            // Check that it's a word boundary (start of string or preceded by non-alphanumeric)
            let is_word_start = pos == 0
                || !input_lower[..pos]
                    .chars()
                    .last()
                    .is_some_and(char::is_alphanumeric);

            // Check that it's followed by a word boundary (end of string, space, or punctuation)
            let end_pos = pos + keyword.len();
            let is_word_end = end_pos >= input_lower.len()
                || !input_lower[end_pos..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric);

            if is_word_start && is_word_end {
                return Some(keyword);
            }
        }
    }

    None
}

/// Internal token-based parser.
struct TokenParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    number_grammar: &'a NumberGrammar,
    original_input: &'a str,
}

impl<'a> TokenParser<'a> {
    const fn new(
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

    fn parse_expression(&mut self) -> Result<Expression, CalculatorError> {
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
        let mut left = self.parse_unary()?;

        while let Some(op) = self.match_multiplicative_op() {
            let right = self.parse_unary()?;
            left = Expression::binary(left, op, right);
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

            // Check for unit (identifier following number)
            let unit = if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                let unit = self
                    .number_grammar
                    .parse_unit(id)
                    .unwrap_or_else(|_| Unit::Custom(id.clone()));
                self.advance();
                unit
            } else {
                Unit::None
            };

            let value = self.number_grammar.parse_number(&num_str)?;
            return Ok(Expression::number_with_unit(value, unit));
        }

        // Standalone identifier (could be a unit or datetime part)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            // This might be a datetime - try to collect more tokens
            let id = id.clone();
            self.advance();

            // If it looks like a datetime start (month name), try to parse more
            if DateTimeGrammar::looks_like_datetime(&id) {
                return self.try_parse_datetime_from_tokens(&id);
            }

            // Check if this looks like an advanced math expression
            if is_advanced_math_keyword(&id) {
                return Err(CalculatorError::unsupported_math(&id, self.original_input));
            }

            // Otherwise it's probably just an identifier/unit
            return Err(CalculatorError::parse(format!(
                "Unexpected identifier: {id}"
            )));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token: {:?}",
            self.current()
        )))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Decimal;

    #[test]
    fn test_parse_simple_number() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("42").unwrap();
        assert!(matches!(expr, Expression::Number { .. }));
    }

    #[test]
    fn test_parse_addition() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("2 + 3").unwrap();
        assert!(matches!(expr, Expression::Binary { .. }));
    }

    #[test]
    fn test_parse_currency() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("100 USD").unwrap();
        if let Expression::Number { value, unit } = expr {
            assert_eq!(value, Decimal::new(100));
            assert_eq!(unit, Unit::currency("USD"));
        } else {
            panic!("Expected Number expression");
        }
    }

    #[test]
    fn test_parse_currency_subtraction() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("84 USD - 34 EUR").unwrap();
        assert!(matches!(expr, Expression::Binary { .. }));
    }

    #[test]
    fn test_evaluate_simple() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2 + 3").unwrap();
        assert_eq!(value.to_display_string(), "5");
    }

    #[test]
    fn test_evaluate_multiplication() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("4 * 5").unwrap();
        assert_eq!(value.to_display_string(), "20");
    }

    #[test]
    fn test_evaluate_precedence() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2 + 3 * 4").unwrap();
        assert_eq!(value.to_display_string(), "14"); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_evaluate_parentheses() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("(2 + 3) * 4").unwrap();
        assert_eq!(value.to_display_string(), "20"); // (2 + 3) * 4 = 20
    }

    #[test]
    fn test_evaluate_negation() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("-5 + 3").unwrap();
        assert_eq!(value.to_display_string(), "-2");
    }

    #[test]
    fn test_datetime_subtraction() {
        let parser = ExpressionParser::new();
        let result = parser.parse_and_evaluate("(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
        assert!(result.is_ok());
        let (value, _, _) = result.unwrap();
        // Should be approximately 1 day, 20 hours, 8 minutes
        assert!(value.to_display_string().contains("day"));
    }

    #[test]
    fn test_lino_representation() {
        let parser = ExpressionParser::new();
        let (_, _, lino) = parser.parse_and_evaluate("84 USD - 34 EUR").unwrap();
        assert!(lino.contains("84 USD"));
        assert!(lino.contains("34 EUR"));
    }
}
