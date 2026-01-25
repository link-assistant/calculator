//! WebAssembly bindings for the calculator.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use js_sys::Promise;

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
                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                    format!(r#"{{"success":false,"error":"{}"}}"#, e)
                });
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
                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                    format!(r#"{{"success":false,"error":"{}"}}"#, e)
                });
                Ok(JsValue::from_str(&json))
            }
        }
    })
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
}
