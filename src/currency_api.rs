//! Currency exchange rate API client.
//!
//! This module provides functionality to fetch real-time and historical exchange
//! rates from official Central Bank APIs.
//!
//! Primary source: Frankfurter API (https://frankfurter.dev/) - European Central Bank (ECB) data
//! - Provides exchange rates for 30+ currencies
//! - Updated daily at around 16:00 CET
//! - Data available from 1999
//!
//! Secondary source: Central Bank of Russia (https://cbr.ru) - CBR data
//! - Provides RUB exchange rates for 50+ currencies
//! - Updated daily
//! - Used for all RUB-related conversions

// Allow futures that are not Send, as these are WASM-only functions running in a single-threaded context
#![allow(clippy::future_not_send)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::types::ExchangeRateInfo;

/// The API source identifier for rates fetched from ECB via Frankfurter API.
pub const API_SOURCE: &str = "frankfurter.dev (ECB)";

/// The API source identifier for rates fetched from the Central Bank of Russia.
pub const CBR_API_SOURCE: &str = "cbr.ru (Central Bank of Russia)";

/// Primary URL for the Frankfurter API (European Central Bank data).
const FRANKFURTER_URL: &str = "https://api.frankfurter.app";

/// URL for the Central Bank of Russia daily exchange rates (XML API).
/// Returns all currency rates relative to RUB for the current business day.
const CBR_DAILY_URL: &str = "https://www.cbr.ru/scripts/XML_daily.asp";

/// Response from the Frankfurter API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CurrencyApiResponse {
    /// The amount (always 1 for our requests).
    #[serde(default)]
    pub amount: f64,
    /// The base currency.
    pub base: String,
    /// The date of the rates (YYYY-MM-DD format).
    pub date: String,
    /// The rates for the base currency.
    pub rates: HashMap<String, f64>,
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

/// Fetches the current exchange rates for a base currency from ECB via Frankfurter API.
///
/// # Arguments
/// * `base_currency` - The base currency code (e.g., "USD", "EUR")
///
/// # Returns
/// A tuple of (date, rates map) where rates map currency codes to exchange rates
///
/// # Note
/// The ECB publishes rates daily at around 16:00 CET.
/// RUB is not available through ECB - use historical .lino files for RUB rates.
pub async fn fetch_current_rates(
    base_currency: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let base = base_currency.to_uppercase();
    let url = format!("{}/latest?from={}", FRANKFURTER_URL, base);

    fetch_rates_from_url(&url).await
}

/// Fetches historical exchange rates for a specific date from ECB via Frankfurter API.
///
/// # Arguments
/// * `base_currency` - The base currency code (e.g., "USD", "EUR")
/// * `date` - The date in YYYY-MM-DD format (ECB data available from 1999-01-04)
///
/// # Returns
/// A tuple of (date, rates map) where rates map currency codes to exchange rates
///
/// # Note
/// RUB is not available through ECB - use historical .lino files for RUB rates.
pub async fn fetch_historical_rates(
    base_currency: &str,
    date: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let base = base_currency.to_uppercase();
    let url = format!("{}/{}?from={}", FRANKFURTER_URL, date, base);

    fetch_rates_from_url(&url).await
}

/// Fetches rates from the Frankfurter API URL.
async fn fetch_rates_from_url(
    url: &str,
) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    fetch_json(url).await
}

/// Performs the actual fetch and JSON parsing for the Frankfurter API.
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

    // Parse the JSON response from Frankfurter API
    // The API returns: { "amount": 1, "base": "USD", "date": "YYYY-MM-DD", "rates": { "EUR": 0.92, ... } }
    let response: CurrencyApiResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| CurrencyApiError::ParseError(format!("Failed to deserialize: {:?}", e)))?;

    // Convert rate keys to lowercase for consistency with other API consumers
    let rates: HashMap<String, f64> = response
        .rates
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

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

/// Fetches current exchange rates from the Central Bank of Russia (cbr.ru).
///
/// The CBR API returns all rates relative to RUB. This function converts them
/// to a map where keys are currency codes (lowercase) and values are the rate
/// expressed as "1 CURRENCY = X RUB" (i.e., how many rubles per unit of foreign currency).
///
/// # Returns
/// A tuple of (date_string, rates_map) where:
/// - `date_string` is in YYYY-MM-DD format
/// - `rates_map` maps lowercase currency code → rate (1 unit of that currency in RUB)
pub async fn fetch_cbr_rates() -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(CBR_DAILY_URL, &opts).map_err(|e| {
        CurrencyApiError::NetworkError(format!("Failed to create CBR request: {:?}", e))
    })?;

    // CBR returns Windows-1251 encoded XML, but modern browsers decode it correctly
    request
        .headers()
        .set("Accept", "application/xml, text/xml, */*")
        .map_err(|e| CurrencyApiError::NetworkError(format!("Failed to set headers: {:?}", e)))?;

    let global = js_sys::global();
    let resp_value = if let Some(window) = global.dyn_ref::<web_sys::Window>() {
        JsFuture::from(window.fetch_with_request(&request)).await
    } else if let Some(worker) = global.dyn_ref::<web_sys::WorkerGlobalScope>() {
        JsFuture::from(worker.fetch_with_request(&request)).await
    } else {
        return Err(CurrencyApiError::NetworkError(
            "Neither Window nor WorkerGlobalScope available".to_string(),
        ));
    }
    .map_err(|e| CurrencyApiError::NetworkError(format!("CBR fetch failed: {:?}", e)))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| CurrencyApiError::NetworkError("Invalid CBR response type".to_string()))?;

    if !resp.ok() {
        return Err(CurrencyApiError::NetworkError(format!(
            "CBR HTTP error: {}",
            resp.status()
        )));
    }

    // Get the response as text (XML)
    let text_promise = resp.text().map_err(|e| {
        CurrencyApiError::ParseError(format!("Failed to get CBR response text: {:?}", e))
    })?;

    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| CurrencyApiError::ParseError(format!("Failed to await CBR text: {:?}", e)))?;

    let xml_text = text_value
        .as_string()
        .ok_or_else(|| CurrencyApiError::ParseError("CBR response is not a string".to_string()))?;

    parse_cbr_xml(&xml_text)
}

/// Parses the CBR XML response and extracts exchange rates.
///
/// The XML format from cbr.ru looks like:
/// ```xml
/// <ValCurs Date="25.02.2026" name="Foreign Currency Market">
///   <Valute ID="R01235">
///     <NumCode>840</NumCode>
///     <CharCode>USD</CharCode>
///     <Nominal>1</Nominal>
///     <Name>Доллар США</Name>
///     <Value>76,6342</Value>
///   </Valute>
///   ...
/// </ValCurs>
/// ```
///
/// Note: The Value uses comma as decimal separator (Russian locale).
/// The rate returned is: 1 unit of CURRENCY = Value/Nominal RUB
pub fn parse_cbr_xml(xml: &str) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    // Extract the date from ValCurs Date attribute: Date="25.02.2026"
    let date = extract_cbr_date(xml)?;

    // Extract all Valute entries
    let mut rates = HashMap::new();

    // Simple XML parsing by finding Valute blocks
    let mut remaining = xml;
    while let Some(valute_start) = remaining.find("<Valute") {
        let block_start = valute_start;
        let end_tag = "</Valute>";
        let block_end = remaining[block_start..].find(end_tag);
        let block_end = match block_end {
            Some(e) => block_start + e + end_tag.len(),
            None => break,
        };

        let valute_block = &remaining[block_start..block_end];

        if let Some((code, rate)) = parse_valute_block(valute_block) {
            rates.insert(code.to_lowercase(), rate);
        }

        remaining = &remaining[block_end..];
    }

    if rates.is_empty() {
        return Err(CurrencyApiError::ParseError(
            "No exchange rates found in CBR response".to_string(),
        ));
    }

    Ok((date, rates))
}

/// Extracts the date from the CBR XML ValCurs element.
/// The date is in dd.mm.yyyy format and is converted to yyyy-mm-dd.
fn extract_cbr_date(xml: &str) -> Result<String, CurrencyApiError> {
    // Look for Date="dd.mm.yyyy" pattern
    let date_prefix = "Date=\"";
    let date_start = xml.find(date_prefix).ok_or_else(|| {
        CurrencyApiError::ParseError("Date attribute not found in CBR response".to_string())
    })?;

    let after_prefix = &xml[date_start + date_prefix.len()..];
    let date_end = after_prefix.find('"').ok_or_else(|| {
        CurrencyApiError::ParseError("Date attribute closing quote not found".to_string())
    })?;

    let date_str = &after_prefix[..date_end];

    // Convert from dd.mm.yyyy to yyyy-mm-dd
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return Err(CurrencyApiError::ParseError(format!(
            "Invalid CBR date format: {}",
            date_str
        )));
    }

    Ok(format!("{}-{}-{}", parts[2], parts[1], parts[0]))
}

/// Parses a single Valute XML block and returns (currency_code, rate_per_unit_in_rub).
fn parse_valute_block(block: &str) -> Option<(String, f64)> {
    let char_code = extract_xml_text(block, "CharCode")?;
    let nominal_str = extract_xml_text(block, "Nominal")?;
    let value_str = extract_xml_text(block, "Value")?;

    let nominal: f64 = nominal_str.parse().ok()?;
    // CBR uses comma as decimal separator
    let value: f64 = value_str.replace(',', ".").parse().ok()?;

    if nominal <= 0.0 || value <= 0.0 {
        return None;
    }

    // Rate: 1 unit of CharCode = (value / nominal) RUB
    let rate = value / nominal;

    Some((char_code, rate))
}

/// Extracts text content from an XML element by tag name.
fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    let start = xml.find(&open_tag)? + open_tag.len();
    let end = xml[start..].find(&close_tag)?;

    Some(xml[start..start + end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_source_constant() {
        // API source should be ECB via Frankfurter
        assert_eq!(API_SOURCE, "frankfurter.dev (ECB)");
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

    #[test]
    fn test_cbr_api_source_constant() {
        assert_eq!(CBR_API_SOURCE, "cbr.ru (Central Bank of Russia)");
    }

    #[test]
    fn test_parse_cbr_xml_basic() {
        let xml = r#"<?xml version="1.0" encoding="windows-1251"?>
<ValCurs Date="25.02.2026" name="Foreign Currency Market">
  <Valute ID="R01235">
    <NumCode>840</NumCode>
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Name>Доллар США</Name>
    <Value>76,6342</Value>
    <VunitRate>76,6342</VunitRate>
  </Valute>
  <Valute ID="R01239">
    <NumCode>978</NumCode>
    <CharCode>EUR</CharCode>
    <Nominal>1</Nominal>
    <Name>Евро</Name>
    <Value>90,5821</Value>
    <VunitRate>90,5821</VunitRate>
  </Valute>
  <Valute ID="R01270">
    <NumCode>356</NumCode>
    <CharCode>INR</CharCode>
    <Nominal>100</Nominal>
    <Name>Индийская рупия</Name>
    <Value>84,2448</Value>
    <VunitRate>0,842448</VunitRate>
  </Valute>
</ValCurs>"#;

        let result = parse_cbr_xml(xml);
        assert!(result.is_ok(), "Should parse CBR XML successfully");

        let (date, rates) = result.unwrap();
        assert_eq!(date, "2026-02-25");

        // USD rate: 1 USD = 76.6342 RUB
        let usd_rate = rates.get("usd").expect("USD rate should exist");
        assert!(
            (*usd_rate - 76.6342).abs() < 0.001,
            "USD rate should be ~76.6342"
        );

        // EUR rate: 1 EUR = 90.5821 RUB
        let eur_rate = rates.get("eur").expect("EUR rate should exist");
        assert!(
            (*eur_rate - 90.5821).abs() < 0.001,
            "EUR rate should be ~90.5821"
        );

        // INR rate: 100 INR = 84.2448 RUB → 1 INR = 0.842448 RUB
        let inr_rate = rates.get("inr").expect("INR rate should exist");
        assert!(
            (*inr_rate - 0.842_448).abs() < 0.001,
            "INR rate should be ~0.842448 (100 INR = 84.2448 RUB)"
        );
    }

    #[test]
    fn test_parse_cbr_xml_date_conversion() {
        let xml = r#"<ValCurs Date="01.03.2026" name="Foreign Currency Market">
  <Valute ID="R01235">
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Value>76,0000</Value>
  </Valute>
</ValCurs>"#;

        let (date, _) = parse_cbr_xml(xml).unwrap();
        assert_eq!(
            date, "2026-03-01",
            "Date should be converted from dd.mm.yyyy to yyyy-mm-dd"
        );
    }

    #[test]
    fn test_parse_cbr_xml_missing_date() {
        let xml = r#"<ValCurs name="Foreign Currency Market">
  <Valute ID="R01235">
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Value>76,0000</Value>
  </Valute>
</ValCurs>"#;

        assert!(
            parse_cbr_xml(xml).is_err(),
            "Should fail without Date attribute"
        );
    }

    #[test]
    fn test_parse_cbr_xml_nominal_division() {
        // Test that nominal > 1 is handled correctly (e.g., 100 JPY = X RUB)
        let xml = r#"<ValCurs Date="25.02.2026" name="Foreign Currency Market">
  <Valute ID="R01820">
    <CharCode>JPY</CharCode>
    <Nominal>100</Nominal>
    <Value>49,4989</Value>
  </Valute>
</ValCurs>"#;

        let (_, rates) = parse_cbr_xml(xml).unwrap();
        let jpy_rate = rates.get("jpy").expect("JPY rate should exist");
        // 100 JPY = 49.4989 RUB → 1 JPY = 0.494989 RUB
        assert!(
            (*jpy_rate - 0.494_989).abs() < 0.001,
            "JPY rate should be ~0.494989"
        );
    }

    #[test]
    fn test_parse_cbr_xml_empty() {
        let xml = r#"<ValCurs Date="25.02.2026" name="Foreign Currency Market">
</ValCurs>"#;

        assert!(parse_cbr_xml(xml).is_err(), "Should fail with no rates");
    }

    #[test]
    fn test_extract_xml_text() {
        let block = "<CharCode>USD</CharCode><Nominal>1</Nominal>";
        assert_eq!(extract_xml_text(block, "CharCode"), Some("USD".to_string()));
        assert_eq!(extract_xml_text(block, "Nominal"), Some("1".to_string()));
        assert_eq!(extract_xml_text(block, "Missing"), None);
    }
}
