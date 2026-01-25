//! Currency types and exchange rate database.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::CalculatorError;
use crate::types::DateTime;

/// Represents a currency with its code and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency {
    /// ISO 4217 currency code (e.g., "USD", "EUR").
    pub code: String,
    /// Full name of the currency.
    pub name: String,
    /// Currency symbol (e.g., "$", "€").
    pub symbol: String,
    /// Number of decimal places typically used.
    pub decimals: u8,
}

impl Currency {
    /// Creates a new currency.
    #[must_use]
    pub fn new(code: &str, name: &str, symbol: &str, decimals: u8) -> Self {
        Self {
            code: code.to_uppercase(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals,
        }
    }

    /// Common currencies database.
    #[must_use]
    pub fn usd() -> Self {
        Self::new("USD", "US Dollar", "$", 2)
    }

    #[must_use]
    pub fn eur() -> Self {
        Self::new("EUR", "Euro", "€", 2)
    }

    #[must_use]
    pub fn gbp() -> Self {
        Self::new("GBP", "British Pound", "£", 2)
    }

    #[must_use]
    pub fn jpy() -> Self {
        Self::new("JPY", "Japanese Yen", "¥", 0)
    }

    #[must_use]
    pub fn chf() -> Self {
        Self::new("CHF", "Swiss Franc", "Fr", 2)
    }

    #[must_use]
    pub fn cny() -> Self {
        Self::new("CNY", "Chinese Yuan", "¥", 2)
    }

    #[must_use]
    pub fn rub() -> Self {
        Self::new("RUB", "Russian Ruble", "₽", 2)
    }
}

/// A database of exchange rates, supporting historical data.
#[derive(Debug, Clone, Default)]
pub struct CurrencyDatabase {
    /// Known currencies.
    currencies: HashMap<String, Currency>,
    /// Exchange rates: (from, to) -> rate
    /// Rate means: 1 unit of 'from' = rate units of 'to'
    rates: HashMap<(String, String), f64>,
    /// Historical rates: (from, to, `date_string`) -> rate
    historical_rates: HashMap<(String, String, String), f64>,
}

impl CurrencyDatabase {
    /// Creates a new currency database with default currencies and rates.
    #[must_use]
    pub fn new() -> Self {
        let mut db = Self::default();
        db.initialize_default_currencies();
        db.initialize_default_rates();
        db
    }

    fn initialize_default_currencies(&mut self) {
        let currencies = vec![
            Currency::usd(),
            Currency::eur(),
            Currency::gbp(),
            Currency::jpy(),
            Currency::chf(),
            Currency::cny(),
            Currency::rub(),
        ];

        for currency in currencies {
            self.currencies.insert(currency.code.clone(), currency);
        }
    }

    fn initialize_default_rates(&mut self) {
        // Approximate rates as of January 2026 (for demonstration)
        // In a real application, these would be fetched from an API

        // USD base rates
        self.set_rate("USD", "EUR", 0.92);
        self.set_rate("USD", "GBP", 0.79);
        self.set_rate("USD", "JPY", 148.5);
        self.set_rate("USD", "CHF", 0.88);
        self.set_rate("USD", "CNY", 7.25);
        self.set_rate("USD", "RUB", 89.5);

        // EUR base rates
        self.set_rate("EUR", "USD", 1.087);
        self.set_rate("EUR", "GBP", 0.86);
        self.set_rate("EUR", "JPY", 161.5);
        self.set_rate("EUR", "CHF", 0.96);

        // GBP base rates
        self.set_rate("GBP", "USD", 1.27);
        self.set_rate("GBP", "EUR", 1.16);

        // Add some historical rates for demonstration
        self.set_historical_rate("USD", "EUR", "2026-01-22", 0.921);
        self.set_historical_rate("USD", "EUR", "2026-01-20", 0.918);
        self.set_historical_rate("USD", "EUR", "2026-01-15", 0.925);
    }

    /// Sets an exchange rate.
    pub fn set_rate(&mut self, from: &str, to: &str, rate: f64) {
        self.rates
            .insert((from.to_uppercase(), to.to_uppercase()), rate);
        // Also add the inverse rate
        if rate != 0.0 {
            self.rates
                .insert((to.to_uppercase(), from.to_uppercase()), 1.0 / rate);
        }
    }

    /// Sets a historical exchange rate for a specific date.
    pub fn set_historical_rate(&mut self, from: &str, to: &str, date: &str, rate: f64) {
        self.historical_rates.insert(
            (from.to_uppercase(), to.to_uppercase(), date.to_string()),
            rate,
        );
        // Also add the inverse rate
        if rate != 0.0 {
            self.historical_rates.insert(
                (to.to_uppercase(), from.to_uppercase(), date.to_string()),
                1.0 / rate,
            );
        }
    }

    /// Gets the current exchange rate.
    #[must_use]
    pub fn get_rate(&self, from: &str, to: &str) -> Option<f64> {
        if from.eq_ignore_ascii_case(to) {
            return Some(1.0);
        }
        self.rates
            .get(&(from.to_uppercase(), to.to_uppercase()))
            .copied()
    }

    /// Gets a historical exchange rate for a specific date.
    #[must_use]
    pub fn get_historical_rate(&self, from: &str, to: &str, date: &DateTime) -> Option<f64> {
        if from.eq_ignore_ascii_case(to) {
            return Some(1.0);
        }

        let date_str = format!("{}", date.as_chrono().format("%Y-%m-%d"));

        // First try exact date match
        if let Some(rate) =
            self.historical_rates
                .get(&(from.to_uppercase(), to.to_uppercase(), date_str))
        {
            return Some(*rate);
        }

        // Fall back to current rate if no historical data
        self.get_rate(from, to)
    }

    /// Converts an amount from one currency to another.
    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Result<f64, CalculatorError> {
        self.get_rate(from, to).map_or_else(
            || {
                Err(CalculatorError::CurrencyConversion {
                    from: from.to_uppercase(),
                    to: to.to_uppercase(),
                    reason: "No exchange rate available".to_string(),
                })
            },
            |rate| Ok(amount * rate),
        )
    }

    /// Converts with a specific date for historical rates.
    pub fn convert_at_date(
        &self,
        amount: f64,
        from: &str,
        to: &str,
        date: &DateTime,
    ) -> Result<f64, CalculatorError> {
        self.get_historical_rate(from, to, date).map_or_else(
            || {
                Err(CalculatorError::NoHistoricalRate {
                    currency: format!("{}/{}", from.to_uppercase(), to.to_uppercase()),
                    date: date.to_string(),
                })
            },
            |rate| Ok(amount * rate),
        )
    }

    /// Checks if a currency code is known.
    #[must_use]
    pub fn is_known_currency(&self, code: &str) -> bool {
        self.currencies.contains_key(&code.to_uppercase())
    }

    /// Gets currency metadata.
    #[must_use]
    pub fn get_currency(&self, code: &str) -> Option<&Currency> {
        self.currencies.get(&code.to_uppercase())
    }

    /// Returns all supported currency codes.
    #[must_use]
    pub fn supported_currencies(&self) -> Vec<String> {
        self.currencies.keys().cloned().collect()
    }

    /// Parses a currency code from a string.
    #[must_use]
    pub fn parse_currency(input: &str) -> Option<String> {
        let input = input.trim().to_uppercase();
        // Common currency codes
        match input.as_str() {
            "USD" | "US$" | "$" => Some("USD".to_string()),
            "EUR" | "€" => Some("EUR".to_string()),
            "GBP" | "£" => Some("GBP".to_string()),
            "JPY" | "¥" => Some("JPY".to_string()),
            "CHF" | "FR" => Some("CHF".to_string()),
            "CNY" | "RMB" => Some("CNY".to_string()),
            "RUB" | "₽" => Some("RUB".to_string()),
            code if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) => {
                Some(code.to_string())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_creation() {
        let usd = Currency::usd();
        assert_eq!(usd.code, "USD");
        assert_eq!(usd.symbol, "$");
    }

    #[test]
    fn test_database_creation() {
        let db = CurrencyDatabase::new();
        assert!(db.is_known_currency("USD"));
        assert!(db.is_known_currency("EUR"));
        assert!(!db.is_known_currency("XYZ"));
    }

    #[test]
    fn test_get_rate() {
        let db = CurrencyDatabase::new();
        let rate = db.get_rate("USD", "EUR");
        assert!(rate.is_some());
        assert!(rate.unwrap() > 0.8 && rate.unwrap() < 1.0);
    }

    #[test]
    fn test_same_currency_rate() {
        let db = CurrencyDatabase::new();
        assert_eq!(db.get_rate("USD", "USD"), Some(1.0));
    }

    #[test]
    fn test_convert() {
        let db = CurrencyDatabase::new();
        let result = db.convert(100.0, "USD", "EUR").unwrap();
        assert!(result > 80.0 && result < 100.0);
    }

    #[test]
    fn test_parse_currency() {
        assert_eq!(
            CurrencyDatabase::parse_currency("USD"),
            Some("USD".to_string())
        );
        assert_eq!(
            CurrencyDatabase::parse_currency("$"),
            Some("USD".to_string())
        );
        assert_eq!(
            CurrencyDatabase::parse_currency("eur"),
            Some("EUR".to_string())
        );
        assert_eq!(
            CurrencyDatabase::parse_currency("€"),
            Some("EUR".to_string())
        );
    }

    #[test]
    fn test_inverse_rate() {
        let db = CurrencyDatabase::new();
        let usd_eur = db.get_rate("USD", "EUR").unwrap();
        let eur_usd = db.get_rate("EUR", "USD").unwrap();
        // The rates should be inverses of each other (approximately)
        let product = usd_eur * eur_usd;
        assert!((product - 1.0).abs() < 0.01);
    }
}
