//! Currency types and exchange rate database.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::CalculatorError;
use crate::types::DateTime;

/// Information about an exchange rate, including its source and timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRateInfo {
    /// The exchange rate value.
    pub rate: f64,
    /// The source of this rate (e.g., "frankfurter.dev (ECB)", "cbr.ru (Central Bank of Russia)", "default").
    pub source: String,
    /// The date this rate is from (ISO format: YYYY-MM-DD).
    pub date: String,
    /// When this rate was fetched/updated (ISO timestamp).
    pub fetched_at: Option<String>,
}

impl ExchangeRateInfo {
    /// Creates a new exchange rate info with all metadata.
    #[must_use]
    pub fn new(rate: f64, source: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            rate,
            source: source.into(),
            date: date.into(),
            fetched_at: None,
        }
    }

    /// Creates a default rate (used when no API data is available).
    #[must_use]
    pub fn default_rate(rate: f64) -> Self {
        Self {
            rate,
            source: "default (hardcoded)".to_string(),
            date: "unknown".to_string(),
            fetched_at: None,
        }
    }

    /// Sets the fetched_at timestamp.
    #[must_use]
    pub fn with_fetched_at(mut self, timestamp: impl Into<String>) -> Self {
        self.fetched_at = Some(timestamp.into());
        self
    }

    /// Formats this rate info for display in calculation steps.
    #[must_use]
    pub fn format_for_display(&self, from: &str, to: &str) -> String {
        format!(
            "1 {} = {} {} (source: {}, date: {})",
            from.to_uppercase(),
            self.rate,
            to.to_uppercase(),
            self.source,
            self.date
        )
    }
}

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
    /// Exchange rates with full metadata: (from, to) -> rate info
    /// Rate means: 1 unit of 'from' = rate units of 'to'
    rates: HashMap<(String, String), ExchangeRateInfo>,
    /// Legacy rates map for compatibility (will be deprecated)
    #[allow(dead_code)]
    legacy_rates: HashMap<(String, String), f64>,
    /// Historical rates: (from, to, `date_string`) -> rate info
    historical_rates: HashMap<(String, String, String), ExchangeRateInfo>,
    /// The last rate info used in a conversion (for step display).
    last_used_rate: Option<(String, String, ExchangeRateInfo)>,
}

impl CurrencyDatabase {
    /// Creates a new currency database with default currencies and rates.
    #[must_use]
    pub fn new() -> Self {
        let mut db = Self {
            currencies: HashMap::new(),
            rates: HashMap::new(),
            legacy_rates: HashMap::new(),
            historical_rates: HashMap::new(),
            last_used_rate: None,
        };
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
        // Default fallback rates - these are used only when API rates are unavailable.
        // In production, rates should be fetched from the currency API.
        // Note: These are approximate rates and may be outdated.

        // USD base rates
        self.set_rate_with_info("USD", "EUR", ExchangeRateInfo::default_rate(0.92));
        self.set_rate_with_info("USD", "GBP", ExchangeRateInfo::default_rate(0.79));
        self.set_rate_with_info("USD", "JPY", ExchangeRateInfo::default_rate(148.5));
        self.set_rate_with_info("USD", "CHF", ExchangeRateInfo::default_rate(0.88));
        self.set_rate_with_info("USD", "CNY", ExchangeRateInfo::default_rate(7.25));
        self.set_rate_with_info("USD", "RUB", ExchangeRateInfo::default_rate(89.5));

        // EUR base rates
        self.set_rate_with_info("EUR", "USD", ExchangeRateInfo::default_rate(1.087));
        self.set_rate_with_info("EUR", "GBP", ExchangeRateInfo::default_rate(0.86));
        self.set_rate_with_info("EUR", "JPY", ExchangeRateInfo::default_rate(161.5));
        self.set_rate_with_info("EUR", "CHF", ExchangeRateInfo::default_rate(0.96));

        // GBP base rates
        self.set_rate_with_info("GBP", "USD", ExchangeRateInfo::default_rate(1.27));
        self.set_rate_with_info("GBP", "EUR", ExchangeRateInfo::default_rate(1.16));

        // Add some historical rates for demonstration
        self.set_historical_rate_with_info(
            "USD",
            "EUR",
            "2026-01-22",
            ExchangeRateInfo::new(0.921, "default (hardcoded)", "2026-01-22"),
        );
        self.set_historical_rate_with_info(
            "USD",
            "EUR",
            "2026-01-20",
            ExchangeRateInfo::new(0.918, "default (hardcoded)", "2026-01-20"),
        );
        self.set_historical_rate_with_info(
            "USD",
            "EUR",
            "2026-01-15",
            ExchangeRateInfo::new(0.925, "default (hardcoded)", "2026-01-15"),
        );
    }

    /// Sets an exchange rate with full metadata.
    pub fn set_rate_with_info(&mut self, from: &str, to: &str, info: ExchangeRateInfo) {
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();

        // Store the forward rate
        self.rates
            .insert((from_upper.clone(), to_upper.clone()), info.clone());

        // Also add the inverse rate
        if info.rate != 0.0 {
            let inverse_info = ExchangeRateInfo {
                rate: 1.0 / info.rate,
                source: info.source.clone(),
                date: info.date.clone(),
                fetched_at: info.fetched_at,
            };
            self.rates.insert((to_upper, from_upper), inverse_info);
        }
    }

    /// Sets an exchange rate (legacy method for compatibility).
    pub fn set_rate(&mut self, from: &str, to: &str, rate: f64) {
        self.set_rate_with_info(from, to, ExchangeRateInfo::default_rate(rate));
    }

    /// Sets a historical exchange rate with full metadata.
    pub fn set_historical_rate_with_info(
        &mut self,
        from: &str,
        to: &str,
        date: &str,
        info: ExchangeRateInfo,
    ) {
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();

        self.historical_rates.insert(
            (from_upper.clone(), to_upper.clone(), date.to_string()),
            info.clone(),
        );

        // Also add the inverse rate
        if info.rate != 0.0 {
            let inverse_info = ExchangeRateInfo {
                rate: 1.0 / info.rate,
                source: info.source.clone(),
                date: info.date.clone(),
                fetched_at: info.fetched_at,
            };
            self.historical_rates
                .insert((to_upper, from_upper, date.to_string()), inverse_info);
        }
    }

    /// Sets a historical exchange rate for a specific date (legacy method).
    pub fn set_historical_rate(&mut self, from: &str, to: &str, date: &str, rate: f64) {
        self.set_historical_rate_with_info(
            from,
            to,
            date,
            ExchangeRateInfo::new(rate, "default (hardcoded)", date),
        );
    }

    /// Gets the current exchange rate info.
    #[must_use]
    pub fn get_rate_info(&self, from: &str, to: &str) -> Option<&ExchangeRateInfo> {
        if from.eq_ignore_ascii_case(to) {
            return None; // Same currency, no rate needed
        }
        self.rates.get(&(from.to_uppercase(), to.to_uppercase()))
    }

    /// Gets the current exchange rate.
    #[must_use]
    pub fn get_rate(&self, from: &str, to: &str) -> Option<f64> {
        if from.eq_ignore_ascii_case(to) {
            return Some(1.0);
        }
        self.rates
            .get(&(from.to_uppercase(), to.to_uppercase()))
            .map(|info| info.rate)
    }

    /// Gets the last used rate info (for display in calculation steps).
    #[must_use]
    pub fn get_last_used_rate(&self) -> Option<&(String, String, ExchangeRateInfo)> {
        self.last_used_rate.as_ref()
    }

    /// Clears the last used rate info.
    pub fn clear_last_used_rate(&mut self) {
        self.last_used_rate = None;
    }

    /// Gets a historical exchange rate for a specific date.
    #[must_use]
    pub fn get_historical_rate(&self, from: &str, to: &str, date: &DateTime) -> Option<f64> {
        if from.eq_ignore_ascii_case(to) {
            return Some(1.0);
        }

        let date_str = format!("{}", date.as_chrono().format("%Y-%m-%d"));

        // First try exact date match
        if let Some(info) =
            self.historical_rates
                .get(&(from.to_uppercase(), to.to_uppercase(), date_str))
        {
            return Some(info.rate);
        }

        // Fall back to current rate if no historical data
        self.get_rate(from, to)
    }

    /// Converts an amount from one currency to another, tracking the rate used.
    pub fn convert(&mut self, amount: f64, from: &str, to: &str) -> Result<f64, CalculatorError> {
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();

        if from_upper == to_upper {
            self.last_used_rate = None;
            return Ok(amount);
        }

        match self.rates.get(&(from_upper.clone(), to_upper.clone())) {
            Some(info) => {
                self.last_used_rate = Some((from_upper, to_upper, info.clone()));
                Ok(amount * info.rate)
            }
            None => Err(CalculatorError::CurrencyConversion {
                from: from_upper,
                to: to_upper,
                reason: "No exchange rate available".to_string(),
            }),
        }
    }

    /// Converts with a specific date for historical rates.
    pub fn convert_at_date(
        &mut self,
        amount: f64,
        from: &str,
        to: &str,
        date: &DateTime,
    ) -> Result<f64, CalculatorError> {
        let from_upper = from.to_uppercase();
        let to_upper = to.to_uppercase();
        let date_str = format!("{}", date.as_chrono().format("%Y-%m-%d"));

        if from_upper == to_upper {
            self.last_used_rate = None;
            return Ok(amount);
        }

        // First try historical rates
        if let Some(info) = self
            .historical_rates
            .get(&(from_upper.clone(), to_upper.clone(), date_str))
            .cloned()
        {
            self.last_used_rate = Some((from_upper, to_upper, info.clone()));
            return Ok(amount * info.rate);
        }

        // Fall back to current rates
        if let Some(info) = self
            .rates
            .get(&(from_upper.clone(), to_upper.clone()))
            .cloned()
        {
            self.last_used_rate = Some((from_upper, to_upper, info.clone()));
            return Ok(amount * info.rate);
        }

        Err(CalculatorError::NoHistoricalRate {
            currency: format!("{}/{}", from_upper, to_upper),
            date: date.to_string(),
        })
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
        let mut db = CurrencyDatabase::new();
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

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_exchange_rate_info() {
        let info = ExchangeRateInfo::new(1.5, "test-api", "2026-01-25");
        assert_eq!(info.rate, 1.5);
        assert_eq!(info.source, "test-api");
        assert_eq!(info.date, "2026-01-25");
    }

    #[test]
    fn test_rate_info_display() {
        let info = ExchangeRateInfo::new(89.5, "cbr.ru (Central Bank of Russia)", "2026-01-25");
        let display = info.format_for_display("USD", "RUB");
        assert!(display.contains("1 USD = 89.5 RUB"));
        assert!(display.contains("cbr.ru (Central Bank of Russia)"));
        assert!(display.contains("2026-01-25"));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_set_rate_with_info() {
        let mut db = CurrencyDatabase::new();
        let info = ExchangeRateInfo::new(75.0, "test-api", "2026-01-25");
        db.set_rate_with_info("USD", "RUB", info);

        let rate_info = db.get_rate_info("USD", "RUB").unwrap();
        assert_eq!(rate_info.rate, 75.0);
        assert_eq!(rate_info.source, "test-api");

        // Check inverse rate was also set
        let inverse_info = db.get_rate_info("RUB", "USD").unwrap();
        assert!((inverse_info.rate - 1.0 / 75.0).abs() < 0.0001);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_convert_tracks_rate_used() {
        let mut db = CurrencyDatabase::new();
        let info = ExchangeRateInfo::new(75.0, "test-api", "2026-01-25");
        db.set_rate_with_info("USD", "RUB", info);

        let result = db.convert(100.0, "USD", "RUB").unwrap();
        assert_eq!(result, 7500.0);

        let last_rate = db.get_last_used_rate().unwrap();
        assert_eq!(last_rate.0, "USD");
        assert_eq!(last_rate.1, "RUB");
        assert_eq!(last_rate.2.rate, 75.0);
        assert_eq!(last_rate.2.source, "test-api");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_same_currency_no_rate_tracking() {
        let mut db = CurrencyDatabase::new();
        let result = db.convert(100.0, "USD", "USD").unwrap();
        assert_eq!(result, 100.0);
        assert!(db.get_last_used_rate().is_none());
    }
}
