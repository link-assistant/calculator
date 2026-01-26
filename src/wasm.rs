//! WebAssembly bindings for the calculator.

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::currency_api;

/// Initializes the WASM module. Call this once before using other functions.
#[wasm_bindgen(start)]
pub fn wasm_init() {
    // Set up better panic messages in WASM
    console_error_panic_hook::set_once();
}

/// Returns the current version of the calculator library.
#[wasm_bindgen]
pub fn get_version() -> String {
    crate::VERSION.to_string()
}

/// A simple health check function.
#[wasm_bindgen]
pub fn health_check() -> bool {
    true
}

/// Response from fetching exchange rates.
#[wasm_bindgen]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)] // wasm_bindgen adds unsafe methods
pub struct ExchangeRatesResponse {
    /// Whether the fetch was successful.
    pub success: bool,
    /// The date of the rates.
    date: String,
    /// The base currency.
    base: String,
    /// Error message if fetch failed.
    error: Option<String>,
    /// The rates as a JSON string (currency -> rate).
    rates_json: String,
}

#[wasm_bindgen]
impl ExchangeRatesResponse {
    /// Gets the date of the rates.
    #[wasm_bindgen(getter)]
    pub fn date(&self) -> String {
        self.date.clone()
    }

    /// Gets the base currency.
    #[wasm_bindgen(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    /// Gets the error message.
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }

    /// Gets the rates as a JSON string.
    #[wasm_bindgen(getter)]
    pub fn rates_json(&self) -> String {
        self.rates_json.clone()
    }
}

/// Fetches current exchange rates for a base currency.
/// Returns a Promise that resolves to a JSON string with the rates.
#[wasm_bindgen]
pub fn fetch_exchange_rates(base_currency: String) -> Promise {
    future_to_promise(async move {
        match currency_api::fetch_current_rates(&base_currency).await {
            Ok((date, rates)) => {
                let rates_json = serde_json::to_string(&rates).unwrap_or_else(|_| "{}".to_string());
                let response = ExchangeRatesResponse {
                    success: true,
                    date,
                    base: base_currency.to_uppercase(),
                    error: None,
                    rates_json,
                };
                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                    r#"{"success":false,"error":"Serialization failed"}"#.to_string()
                });
                Ok(JsValue::from_str(&json))
            }
            Err(e) => {
                let response = ExchangeRatesResponse {
                    success: false,
                    date: String::new(),
                    base: base_currency.to_uppercase(),
                    error: Some(e.to_string()),
                    rates_json: String::new(),
                };
                let json = serde_json::to_string(&response)
                    .unwrap_or_else(|_| format!(r#"{{"success":false,"error":"{}"}}"#, e));
                Ok(JsValue::from_str(&json))
            }
        }
    })
}

/// Fetches historical exchange rates for a specific date.
/// Returns a Promise that resolves to a JSON string with the rates.
#[wasm_bindgen]
pub fn fetch_historical_rates(base_currency: String, date: String) -> Promise {
    future_to_promise(async move {
        match currency_api::fetch_historical_rates(&base_currency, &date).await {
            Ok((actual_date, rates)) => {
                let rates_json = serde_json::to_string(&rates).unwrap_or_else(|_| "{}".to_string());
                let response = ExchangeRatesResponse {
                    success: true,
                    date: actual_date,
                    base: base_currency.to_uppercase(),
                    error: None,
                    rates_json,
                };
                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                    r#"{"success":false,"error":"Serialization failed"}"#.to_string()
                });
                Ok(JsValue::from_str(&json))
            }
            Err(e) => {
                let response = ExchangeRatesResponse {
                    success: false,
                    date: String::new(),
                    base: base_currency.to_uppercase(),
                    error: Some(e.to_string()),
                    rates_json: String::new(),
                };
                let json = serde_json::to_string(&response)
                    .unwrap_or_else(|_| format!(r#"{{"success":false,"error":"{}"}}"#, e));
                Ok(JsValue::from_str(&json))
            }
        }
    })
}

/// Parses a .lino rate file content and returns the rate data as JSON.
/// This allows the web app to parse .lino files without loading them into the calculator.
///
/// # Arguments
/// * `content` - The .lino file content
///
/// # Returns
/// A JSON string with the parsed rate information
#[wasm_bindgen]
#[allow(clippy::needless_pass_by_value)] // wasm_bindgen requires owned String
pub fn parse_lino_rate(content: String) -> String {
    #[derive(serde::Serialize)]
    struct ParsedRate {
        success: bool,
        from: Option<String>,
        to: Option<String>,
        value: Option<f64>,
        date: Option<String>,
        source: Option<String>,
        error: Option<String>,
    }

    let mut from_currency: Option<String> = None;
    let mut to_currency: Option<String> = None;
    let mut value: Option<f64> = None;
    let mut date: Option<String> = None;
    let mut source: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line == "rate:" {
            continue;
        }

        if let Some(rest) = line.strip_prefix("from ") {
            from_currency = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = line.strip_prefix("to ") {
            to_currency = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = line.strip_prefix("value ") {
            value = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("date ") {
            date = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("source ") {
            let src = rest.trim();
            let src = src.trim_start_matches('\'').trim_end_matches('\'');
            let src = src.trim_start_matches('"').trim_end_matches('"');
            source = Some(src.to_string());
        }
    }

    let result =
        if from_currency.is_some() && to_currency.is_some() && value.is_some() && date.is_some() {
            ParsedRate {
                success: true,
                from: from_currency,
                to: to_currency,
                value,
                date,
                source,
                error: None,
            }
        } else {
            ParsedRate {
                success: false,
                from: from_currency,
                to: to_currency,
                value,
                date,
                source,
                error: Some("Missing required fields".to_string()),
            }
        };

    serde_json::to_string(&result)
        .unwrap_or_else(|_| r#"{"success":false,"error":"Serialization failed"}"#.to_string())
}

/// Parses a consolidated .lino rate file and returns the rate data as JSON.
/// The consolidated format stores all rates for a currency pair in one file:
/// ```text
/// rates:
///   from USD
///   to EUR
///   source 'frankfurter.dev (ECB)'
///   data:
///     2021-01-25 0.8234
///     2021-02-01 0.8315
/// ```
///
/// # Arguments
/// * `content` - The consolidated .lino file content
///
/// # Returns
/// A JSON string with the parsed rates array
#[wasm_bindgen]
#[allow(clippy::needless_pass_by_value)] // wasm_bindgen requires owned String
pub fn parse_consolidated_lino_rates(content: String) -> String {
    #[derive(serde::Serialize)]
    struct ParsedConsolidatedRates {
        success: bool,
        from: Option<String>,
        to: Option<String>,
        source: Option<String>,
        rates: Vec<RateEntry>,
        error: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct RateEntry {
        date: String,
        value: f64,
    }

    let mut from_currency: Option<String> = None;
    let mut to_currency: Option<String> = None;
    let mut source: Option<String> = None;
    let mut rates: Vec<RateEntry> = Vec::new();
    let mut in_data_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed == "rates:" {
            continue;
        }

        if trimmed == "data:" {
            in_data_section = true;
            continue;
        }

        if in_data_section {
            // Parse date and value: "2021-01-25 0.8234"
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    rates.push(RateEntry {
                        date: parts[0].to_string(),
                        value,
                    });
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("from ") {
            from_currency = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = trimmed.strip_prefix("to ") {
            to_currency = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = trimmed.strip_prefix("source ") {
            let src = rest.trim();
            let src = src.trim_start_matches('\'').trim_end_matches('\'');
            let src = src.trim_start_matches('"').trim_end_matches('"');
            source = Some(src.to_string());
        }
    }

    let result = if from_currency.is_some() && to_currency.is_some() && !rates.is_empty() {
        ParsedConsolidatedRates {
            success: true,
            from: from_currency,
            to: to_currency,
            source,
            rates,
            error: None,
        }
    } else {
        ParsedConsolidatedRates {
            success: false,
            from: from_currency,
            to: to_currency,
            source,
            rates,
            error: Some("Missing required fields or no rates found".to_string()),
        }
    };

    serde_json::to_string(&result)
        .unwrap_or_else(|_| r#"{"success":false,"error":"Serialization failed"}"#.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }

    #[test]
    fn test_parse_lino_rate() {
        let content = "rate:
  from USD
  to EUR
  value 0.92
  date 2026-01-25
  source 'fawazahmed0/currency-api'";

        let result = parse_lino_rate(content.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["from"], "USD");
        assert_eq!(parsed["to"], "EUR");
        assert_eq!(parsed["value"], 0.92);
        assert_eq!(parsed["date"], "2026-01-25");
        assert_eq!(parsed["source"], "fawazahmed0/currency-api");
    }

    #[test]
    fn test_parse_lino_rate_missing_fields() {
        let content = "rate:
  from USD
  to EUR";

        let result = parse_lino_rate(content.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].as_str().unwrap().contains("Missing"));
    }

    #[test]
    fn test_parse_consolidated_lino_rates() {
        let content = "rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402";

        let result = parse_consolidated_lino_rates(content.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["from"], "USD");
        assert_eq!(parsed["to"], "EUR");
        assert_eq!(parsed["source"], "frankfurter.dev (ECB)");

        let rates = parsed["rates"].as_array().unwrap();
        assert_eq!(rates.len(), 3);
        assert_eq!(rates[0]["date"], "2021-01-25");
        assert_eq!(rates[0]["value"], 0.8234);
    }

    #[test]
    fn test_parse_consolidated_lino_rates_empty() {
        let content = "rates:
  from USD
  to EUR
  source 'test'
  data:";

        let result = parse_consolidated_lino_rates(content.to_string());
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["success"], false);
        assert!(parsed["error"].as_str().unwrap().contains("no rates found"));
    }
}
