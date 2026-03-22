//! Grammar for parsing numbers with optional units.

use crate::error::CalculatorError;
use crate::crypto_api;
use crate::types::{CurrencyDatabase, DataSizeUnit, Decimal, MassUnit, Unit};

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
        let (unit, _alternatives) = self.parse_unit_with_alternatives(s)?;
        Ok(unit)
    }

    /// Parses a unit string, returning the primary unit and any alternative interpretations.
    ///
    /// Some identifiers are ambiguous — for example, "ton" can mean either a metric ton
    /// (mass unit, 1000 kg) or Toncoin (TON cryptocurrency). This method returns all
    /// valid interpretations so the caller can surface them as alternatives to the user.
    pub fn parse_unit_with_alternatives(
        &self,
        s: &str,
    ) -> Result<(Unit, Vec<Unit>), CalculatorError> {
        let s = s.trim();
        let mut alternatives = Vec::new();

        // Try to parse as data size unit first (before currency, to avoid conflicts)
        if let Some(data_size) = DataSizeUnit::parse(s) {
            return Ok((Unit::DataSize(data_size), alternatives));
        }

        // Try to parse as mass unit (before currency, to avoid "t" being treated as currency)
        if let Some(mass) = MassUnit::parse(s) {
            let primary = Unit::Mass(mass);

            // Check if this identifier also matches a well-known cryptocurrency or fiat code.
            // Only flag ambiguity for known crypto tickers (e.g., "ton" → TON/Toncoin),
            // not for the generic catch-all that accepts any 2-5 letter string.
            if let Some(currency_code) = CurrencyDatabase::parse_currency(s) {
                if Self::is_well_known_currency(&currency_code) {
                    let currency_unit = Unit::currency(&currency_code);
                    if currency_unit != primary {
                        alternatives.push(currency_unit);
                    }
                }
            }

            return Ok((primary, alternatives));
        }

        // Try to parse as cryptocurrency or fiat currency alias
        if let Some(currency_code) = CurrencyDatabase::parse_currency(s) {
            let primary = Unit::currency(&currency_code);

            // Check if a case-insensitive mass unit match exists (ambiguity detection)
            if let Some(mass) = MassUnit::parse(&s.to_lowercase()) {
                let mass_unit = Unit::Mass(mass);
                if mass_unit != primary {
                    alternatives.push(mass_unit);
                }
            }

            return Ok((primary, alternatives));
        }

        // Could add more unit types here (duration, length, etc.)

        // If nothing matches, treat as custom unit
        if s.is_empty() {
            Ok((Unit::None, alternatives))
        } else {
            Ok((Unit::Custom(s.to_string()), alternatives))
        }
    }

    /// Checks if a currency code is a well-known fiat or crypto currency.
    ///
    /// Returns `true` for explicitly listed currencies (ISO 4217 fiat codes
    /// and known crypto tickers like TON, BTC, ETH). Returns `false` for codes
    /// that only match the generic "any 2-5 letter string" catch-all pattern
    /// in `CurrencyDatabase::parse_currency`.
    fn is_well_known_currency(code: &str) -> bool {
        let upper = code.to_uppercase();

        // Check if it's a known crypto ticker
        if crypto_api::coingecko_id(&upper).is_some() {
            return true;
        }

        // Check well-known fiat codes (ISO 4217 major currencies)
        matches!(
            upper.as_str(),
            "USD"
                | "EUR"
                | "GBP"
                | "JPY"
                | "CHF"
                | "CNY"
                | "RUB"
                | "INR"
                | "KRW"
                | "CLF"
                | "AUD"
                | "CAD"
                | "NZD"
                | "SEK"
                | "NOK"
                | "DKK"
                | "SGD"
                | "HKD"
                | "MXN"
                | "BRL"
                | "ZAR"
                | "TRY"
                | "PLN"
                | "CZK"
                | "HUF"
                | "RON"
                | "BGN"
                | "ISK"
                | "IDR"
                | "MYR"
                | "PHP"
                | "THB"
                | "KES"
        )
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
