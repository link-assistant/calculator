//! Grammar for parsing numbers with optional units.

use crate::error::CalculatorError;
use crate::types::{CurrencyDatabase, Decimal, Unit};

/// Grammar for parsing numbers with optional units.
#[derive(Debug, Default)]
pub struct NumberGrammar;

impl NumberGrammar {
    /// Creates a new number grammar.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Parses a number string into a Decimal.
    pub fn parse_number(&self, s: &str) -> Result<Decimal, CalculatorError> {
        let s = s.trim();

        // Handle negative numbers
        let (is_negative, s) = s
            .strip_prefix('-')
            .map_or((false, s), |stripped| (true, stripped.trim()));

        let decimal: Decimal = s
            .parse()
            .map_err(|_| CalculatorError::parse(format!("Invalid number: {s}")))?;

        Ok(if is_negative { -decimal } else { decimal })
    }

    /// Parses a number with an optional unit.
    pub fn parse_number_with_unit(
        &self,
        number_str: &str,
        unit_str: Option<&str>,
    ) -> Result<(Decimal, Unit), CalculatorError> {
        let number = self.parse_number(number_str)?;

        let unit = match unit_str {
            Some(u) => self.parse_unit(u)?,
            None => Unit::None,
        };

        Ok((number, unit))
    }

    /// Parses a unit string.
    pub fn parse_unit(&self, s: &str) -> Result<Unit, CalculatorError> {
        let s = s.trim();

        // Try to parse as currency
        if let Some(currency_code) = CurrencyDatabase::parse_currency(s) {
            return Ok(Unit::currency(&currency_code));
        }

        // Could add more unit types here (duration, length, etc.)

        // If nothing matches, treat as custom unit
        if s.is_empty() {
            Ok(Unit::None)
        } else {
            Ok(Unit::Custom(s.to_string()))
        }
    }

    /// Checks if a string looks like a number.
    #[must_use]
    pub fn looks_like_number(s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }

        let s = s.strip_prefix('-').unwrap_or(s);
        let s = s.strip_prefix('+').unwrap_or(s);

        // Must start with a digit or decimal point
        let first = s.chars().next();
        first.is_some_and(|c| c.is_ascii_digit() || c == '.')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let grammar = NumberGrammar::new();
        let n = grammar.parse_number("42").unwrap();
        assert_eq!(n.to_string(), "42");
    }

    #[test]
    fn test_parse_decimal() {
        let grammar = NumberGrammar::new();
        let n = grammar.parse_number("3.14").unwrap();
        assert!(n.to_string().starts_with("3.14"));
    }

    #[test]
    fn test_parse_negative() {
        let grammar = NumberGrammar::new();
        let n = grammar.parse_number("-5").unwrap();
        assert_eq!(n.to_string(), "-5");
    }

    #[test]
    fn test_parse_number_with_currency() {
        let grammar = NumberGrammar::new();
        let (n, unit) = grammar.parse_number_with_unit("100", Some("USD")).unwrap();
        assert_eq!(n.to_string(), "100");
        assert_eq!(unit, Unit::currency("USD"));
    }

    #[test]
    fn test_looks_like_number() {
        assert!(NumberGrammar::looks_like_number("42"));
        assert!(NumberGrammar::looks_like_number("-5"));
        assert!(NumberGrammar::looks_like_number("3.14"));
        assert!(!NumberGrammar::looks_like_number("abc"));
        assert!(!NumberGrammar::looks_like_number(""));
    }

    #[test]
    fn test_parse_currency_unit() {
        let grammar = NumberGrammar::new();
        let unit = grammar.parse_unit("EUR").unwrap();
        assert_eq!(unit, Unit::currency("EUR"));
    }
}
