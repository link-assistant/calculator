//! Currency exchange rate API client.
//!
//! This module provides functionality to fetch real-time and historical exchange
//! rates from the fawazahmed0/currency-api service.
//!
//! API Documentation: https://github.com/fawazahmed0/exchange-api

// Allow futures that are not Send, as these are WASM-only functions running in a single-threaded context
#![allow(clippy::future_not_send)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::types::ExchangeRateInfo;

/// The API source identifier for rates fetched from this API.
pub const API_SOURCE: &str = "fawazahmed0/currency-api";

/// CDN URL for the currency API (primary).
const CDN_URL: &str = "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api";

/// Fallback URL for the currency API.
const FALLBACK_URL: &str = "https://latest.currency-api.pages.dev";

/// Response from the currency API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CurrencyApiResponse {
    /// The date of the rates (YYYY-MM-DD format).
    pub date: String,
    /// The rates for the base currency.
    #[serde(flatten)]
    pub rates: HashMap<String, HashMap<String, f64>>,
}

/// Error type for currency API operations.
#[derive(Debug, Clone)]
pub enum CurrencyApiError {
    /// Network request failed.
    NetworkError(String),
    /// Failed to parse response.
    ParseError(String),
    /// Rate not found for currency pair.
    RateNotFound { from: String, to: String },
}

impl std::fmt::Display for CurrencyApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::RateNotFound { from, to } => {
                write!(f, "Rate not found for {}/{}", from, to)
            }
        }
    }
}

/// Fetches the current exchange rates for a base currency.
///
/// # Arguments
/// * `base_currency` - The base currency code (e.g., "usd")
///
/// # Returns
/// A map of currency codes to exchange rates
pub async fn fetch_current_rates(
    base_currency: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let base = base_currency.to_lowercase();
    let url = format!("{}@latest/v1/currencies/{}.json", CDN_URL, base);

    fetch_rates_from_url(&url, &base).await
}

/// Fetches historical exchange rates for a specific date.
///
/// # Arguments
/// * `base_currency` - The base currency code (e.g., "usd")
/// * `date` - The date in YYYY-MM-DD format
///
/// # Returns
/// A map of currency codes to exchange rates
pub async fn fetch_historical_rates(
    base_currency: &str,
    date: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let base = base_currency.to_lowercase();
    let url = format!("{}@{}/v1/currencies/{}.json", CDN_URL, date, base);

    fetch_rates_from_url(&url, &base).await
}

/// Fetches rates from a specific URL.
async fn fetch_rates_from_url(
    url: &str,
    base_currency: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    // First try the primary CDN URL
    if let Ok((date, rates)) = fetch_json(url).await {
        Ok((date, rates))
    } else {
        // Try fallback URL
        let fallback = format!("{}/v1/currencies/{}.json", FALLBACK_URL, base_currency);
        fetch_json(&fallback).await
    }
}

/// Performs the actual fetch and JSON parsing.
/// Works in both Window and Web Worker contexts.
async fn fetch_json(url: &str) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| {
        CurrencyApiError::NetworkError(format!("Failed to create request: {:?}", e))
    })?;

    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| CurrencyApiError::NetworkError(format!("Failed to set headers: {:?}", e)))?;

    // Use the global fetch function which works in both Window and Worker contexts.
    // In a browser window, this is window.fetch; in a worker, this is self.fetch.
    let global = js_sys::global();
    let resp_value = if let Some(window) = global.dyn_ref::<web_sys::Window>() {
        // Running in a Window context
        JsFuture::from(window.fetch_with_request(&request)).await
    } else if let Some(worker) = global.dyn_ref::<web_sys::WorkerGlobalScope>() {
        // Running in a Web Worker context
        JsFuture::from(worker.fetch_with_request(&request)).await
    } else {
        return Err(CurrencyApiError::NetworkError(
            "Neither Window nor WorkerGlobalScope available".to_string(),
        ));
    }
    .map_err(|e| CurrencyApiError::NetworkError(format!("Fetch failed: {:?}", e)))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| CurrencyApiError::NetworkError("Invalid response type".to_string()))?;

    if !resp.ok() {
        return Err(CurrencyApiError::NetworkError(format!(
            "HTTP error: {}",
            resp.status()
        )));
    }

    let json = JsFuture::from(
        resp.json()
            .map_err(|e| CurrencyApiError::ParseError(format!("Failed to get JSON: {:?}", e)))?,
    )
    .await
    .map_err(|e| CurrencyApiError::ParseError(format!("Failed to parse JSON: {:?}", e)))?;

    // Parse the JSON response
    let response: CurrencyApiResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| CurrencyApiError::ParseError(format!("Failed to deserialize: {:?}", e)))?;

    // Extract the rates from the nested structure
    // The API returns: { "date": "YYYY-MM-DD", "usd": { "eur": 0.92, ... } }
    let base_lower = url
        .split('/')
        .next_back()
        .and_then(|s| s.strip_suffix(".json"))
        .unwrap_or("usd");

    let rates = response.rates.get(base_lower).cloned().unwrap_or_default();

    Ok((response.date, rates))
}

/// Converts raw API rates to `ExchangeRateInfo` objects.
#[allow(clippy::implicit_hasher)]
pub fn rates_to_exchange_info(
    base: &str,
    date: &str,
    rates: &HashMap<String, f64>,
) -> Vec<(String, String, ExchangeRateInfo)> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let base_upper = base.to_uppercase();

    rates
        .iter()
        .map(|(target, rate)| {
            let target_upper = target.to_uppercase();
            let info = ExchangeRateInfo::new(*rate, API_SOURCE, date).with_fetched_at(&timestamp);
            (base_upper.clone(), target_upper, info)
        })
        .collect()
}

/// Fetches a single exchange rate.
pub async fn fetch_rate(from: &str, to: &str) -> Result<ExchangeRateInfo, CurrencyApiError> {
    let (date, rates) = fetch_current_rates(from).await?;
    let to_lower = to.to_lowercase();

    rates
        .get(&to_lower)
        .map(|rate| {
            let timestamp = chrono::Utc::now().to_rfc3339();
            ExchangeRateInfo::new(*rate, API_SOURCE, &date).with_fetched_at(timestamp)
        })
        .ok_or_else(|| CurrencyApiError::RateNotFound {
            from: from.to_uppercase(),
            to: to.to_uppercase(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_source_constant() {
        assert_eq!(API_SOURCE, "fawazahmed0/currency-api");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_rates_to_exchange_info() {
        let mut rates = HashMap::new();
        rates.insert("eur".to_string(), 0.92);
        rates.insert("gbp".to_string(), 0.79);

        let result = rates_to_exchange_info("USD", "2026-01-25", &rates);

        assert_eq!(result.len(), 2);

        // Find EUR rate
        let eur_rate = result
            .iter()
            .find(|(_, to, _)| to == "EUR")
            .expect("EUR rate should exist");
        assert_eq!(eur_rate.0, "USD");
        assert_eq!(eur_rate.1, "EUR");
        assert_eq!(eur_rate.2.rate, 0.92);
        assert_eq!(eur_rate.2.source, API_SOURCE);
        assert_eq!(eur_rate.2.date, "2026-01-25");
    }

    #[test]
    fn test_currency_api_error_display() {
        let err = CurrencyApiError::NetworkError("timeout".to_string());
        assert!(err.to_string().contains("Network error"));

        let err = CurrencyApiError::ParseError("invalid json".to_string());
        assert!(err.to_string().contains("Parse error"));

        let err = CurrencyApiError::RateNotFound {
            from: "USD".to_string(),
            to: "XYZ".to_string(),
        };
        assert!(err.to_string().contains("USD/XYZ"));
    }
}
