//! Cryptocurrency price API client.
//!
//! This module provides functionality to fetch real-time cryptocurrency prices.
//!
//! Primary source: CoinMarketCap API (https://coinmarketcap.com/)
//! - Provides prices for thousands of cryptocurrencies
//! - Updated frequently (near real-time)
//! - Requires a free API key from https://pro.coinmarketcap.com/
//!
//! Fallback source: CoinGecko API (https://www.coingecko.com/en/api)
//! - Free tier available (no API key required for basic usage)
//! - Updated every minute
//! - Supports 10,000+ cryptocurrencies
//!
//! The module fetches prices relative to a fiat quote currency (e.g., USD)
//! and returns them as exchange rate information compatible with the
//! `CurrencyDatabase`.

// Allow futures that are not Send, as these are WASM-only functions running in a single-threaded context
#![allow(clippy::future_not_send)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::types::ExchangeRateInfo;

/// The API source identifier for rates fetched from CoinGecko.
pub const COINGECKO_SOURCE: &str = "coingecko.com";

/// The API source identifier for rates fetched from CoinMarketCap.
pub const COINMARKETCAP_SOURCE: &str = "coinmarketcap.com";

/// CoinGecko coin ID mapping for common cryptocurrencies.
///
/// Maps the ticker symbol (uppercase) to the CoinGecko ID.
pub fn coingecko_id(ticker: &str) -> Option<&'static str> {
    match ticker.to_uppercase().as_str() {
        "TON" | "TONCOIN" => Some("the-open-network"),
        "BTC" | "BITCOIN" => Some("bitcoin"),
        "ETH" | "ETHEREUM" => Some("ethereum"),
        "BNB" | "BINANCECOIN" => Some("binancecoin"),
        "SOL" | "SOLANA" => Some("solana"),
        "XRP" | "RIPPLE" => Some("ripple"),
        "ADA" | "CARDANO" => Some("cardano"),
        "DOGE" | "DOGECOIN" => Some("dogecoin"),
        "DOT" | "POLKADOT" => Some("polkadot"),
        "LTC" | "LITECOIN" => Some("litecoin"),
        "LINK" | "CHAINLINK" => Some("chainlink"),
        "UNI" | "UNISWAP" => Some("uniswap"),
        _ => None,
    }
}

/// Error type for crypto API operations.
#[derive(Debug, Clone)]
pub enum CryptoApiError {
    /// Network request failed.
    NetworkError(String),
    /// Failed to parse response.
    ParseError(String),
    /// Coin not found.
    CoinNotFound { ticker: String },
}

impl std::fmt::Display for CryptoApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::CoinNotFound { ticker } => {
                write!(f, "Cryptocurrency not found: {}", ticker)
            }
        }
    }
}

/// Fetches the current price of a cryptocurrency in a given fiat currency.
///
/// Uses the CoinGecko API (free, no key required for basic usage).
///
/// # Arguments
/// * `ticker` - The cryptocurrency ticker (e.g., "TON", "BTC", "ETH")
/// * `vs_currency` - The fiat currency to price in (e.g., "usd", "eur")
///
/// # Returns
/// An `ExchangeRateInfo` with the current price (1 [ticker] = price [vs_currency])
pub async fn fetch_crypto_price(
    ticker: &str,
    vs_currency: &str,
) -> Result<ExchangeRateInfo, CryptoApiError> {
    let coin_id = coingecko_id(ticker).ok_or_else(|| CryptoApiError::CoinNotFound {
        ticker: ticker.to_uppercase(),
    })?;

    let vs = vs_currency.to_lowercase();
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        coin_id, vs
    );

    let (price, date) = fetch_coingecko_price(&url, coin_id, &vs).await?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    Ok(ExchangeRateInfo::new(price, COINGECKO_SOURCE, &date).with_fetched_at(timestamp))
}

/// Fetches prices for multiple cryptocurrencies against a fiat currency.
///
/// # Arguments
/// * `tickers` - Slice of cryptocurrency tickers (e.g., `["TON", "BTC", "ETH"]`)
/// * `vs_currency` - The fiat currency to price in (e.g., "usd", "eur")
///
/// # Returns
/// A map from ticker → `ExchangeRateInfo`
pub async fn fetch_crypto_prices(
    tickers: &[&str],
    vs_currency: &str,
) -> Result<HashMap<String, ExchangeRateInfo>, CryptoApiError> {
    // Map tickers to CoinGecko IDs
    let mut id_to_ticker: HashMap<String, String> = HashMap::new();
    let mut ids: Vec<String> = Vec::new();

    for ticker in tickers {
        let upper = ticker.to_uppercase();
        if let Some(id) = coingecko_id(&upper) {
            id_to_ticker.insert(id.to_string(), upper.clone());
            ids.push(id.to_string());
        }
    }

    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let vs = vs_currency.to_lowercase();
    let ids_str = ids.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
        ids_str, vs
    );

    let json = fetch_json_value(&url).await?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut result = HashMap::new();

    for (id, ticker) in &id_to_ticker {
        if let Some(coin_data) = json.get(id.as_str()).and_then(|v| v.as_object()) {
            if let Some(price) = coin_data.get(vs.as_str()).and_then(serde_json::Value::as_f64) {
                let info = ExchangeRateInfo::new(price, COINGECKO_SOURCE, &today)
                    .with_fetched_at(&timestamp);
                result.insert(ticker.clone(), info);
            }
        }
    }

    Ok(result)
}

/// Fetches a single coin price from CoinGecko.
async fn fetch_coingecko_price(
    url: &str,
    coin_id: &str,
    vs_currency: &str,
) -> Result<(f64, String), CryptoApiError> {
    let json = fetch_json_value(url).await?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let price = json
        .get(coin_id)
        .and_then(|c| c.get(vs_currency))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| {
            CryptoApiError::ParseError(format!(
                "Price not found for {} in {}",
                coin_id, vs_currency
            ))
        })?;

    Ok((price, today))
}

/// Performs an HTTP GET request and parses the response as JSON.
async fn fetch_json_value(url: &str) -> Result<serde_json::Value, CryptoApiError> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| CryptoApiError::NetworkError(format!("Failed to create request: {:?}", e)))?;

    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| CryptoApiError::NetworkError(format!("Failed to set headers: {:?}", e)))?;

    let global = js_sys::global();
    let resp_value = if let Some(window) = global.dyn_ref::<web_sys::Window>() {
        JsFuture::from(window.fetch_with_request(&request)).await
    } else if let Some(worker) = global.dyn_ref::<web_sys::WorkerGlobalScope>() {
        JsFuture::from(worker.fetch_with_request(&request)).await
    } else {
        return Err(CryptoApiError::NetworkError(
            "Neither Window nor WorkerGlobalScope available".to_string(),
        ));
    }
    .map_err(|e| CryptoApiError::NetworkError(format!("Fetch failed: {:?}", e)))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| CryptoApiError::NetworkError("Invalid response type".to_string()))?;

    if !resp.ok() {
        return Err(CryptoApiError::NetworkError(format!(
            "HTTP error: {}",
            resp.status()
        )));
    }

    let json_future = resp
        .json()
        .map_err(|e| CryptoApiError::ParseError(format!("Failed to get JSON: {:?}", e)))?;

    let js_value = JsFuture::from(json_future)
        .await
        .map_err(|e| CryptoApiError::ParseError(format!("Failed to await JSON: {:?}", e)))?;

    serde_wasm_bindgen::from_value(js_value)
        .map_err(|e| CryptoApiError::ParseError(format!("Failed to deserialize: {:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coingecko_id_ton() {
        assert_eq!(coingecko_id("TON"), Some("the-open-network"));
        assert_eq!(coingecko_id("ton"), Some("the-open-network"));
        assert_eq!(coingecko_id("TONCOIN"), Some("the-open-network"));
    }

    #[test]
    fn test_coingecko_id_bitcoin() {
        assert_eq!(coingecko_id("BTC"), Some("bitcoin"));
        assert_eq!(coingecko_id("BITCOIN"), Some("bitcoin"));
        assert_eq!(coingecko_id("btc"), Some("bitcoin"));
    }

    #[test]
    fn test_coingecko_id_ethereum() {
        assert_eq!(coingecko_id("ETH"), Some("ethereum"));
        assert_eq!(coingecko_id("ethereum"), Some("ethereum"));
    }

    #[test]
    fn test_coingecko_id_unknown() {
        assert_eq!(coingecko_id("UNKNOWN"), None);
        assert_eq!(coingecko_id("XYZ"), None);
    }

    #[test]
    fn test_crypto_api_error_display() {
        let err = CryptoApiError::NetworkError("timeout".to_string());
        assert!(err.to_string().contains("Network error"));

        let err = CryptoApiError::ParseError("invalid json".to_string());
        assert!(err.to_string().contains("Parse error"));

        let err = CryptoApiError::CoinNotFound {
            ticker: "XYZ".to_string(),
        };
        assert!(err.to_string().contains("XYZ"));
    }
}
