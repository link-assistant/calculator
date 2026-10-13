// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0a8012af36e61c01215a449c2d5d7577100d9b222967d4b90437e73ec4e61965 bytes=9547

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Cryptocurrency price API client (port of `src/crypto_api.rs`).
// |  *
// |  * This module provides functionality to fetch real-time cryptocurrency prices.
// |  *
// |  * Primary source: CoinMarketCap API (https://coinmarketcap.com/)
// |  * - Provides prices for thousands of cryptocurrencies
// |  * - Updated frequently (near real-time)
// |  * - Requires a free API key from https://pro.coinmarketcap.com/
// |  *
// |  * Fallback source: CoinGecko API (https://www.coingecko.com/en/api)
// |  * - Free tier available (no API key required for basic usage)
// |  * - Updated every minute
// |  * - Supports 10,000+ cryptocurrencies
// |  *
// |  * The module fetches prices relative to a fiat quote currency (e.g., USD)
// |  * and returns them as exchange rate information compatible with the
// |  * `CurrencyDatabase`.
// |  *
// |  * Rust fetches through `web_sys` (`window.fetch` / `self.fetch`); this port
// |  * calls `globalThis.fetch` (Node 20+, Bun, browsers, workers) with the same
// |  * request and the same error variants and messages.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import * as clock from './clock.js';
// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from './rust/core.js';
use crate::rust::core_::err;
// meta-language:translated JavaScript import_statement items=1 sha256=ed0c33f0945575ce66bd32d6f90adaee2df0fac7a1ee6cba0c39326bfa9a4520
// | import { debugStruct, debugTuple } from './rust/fmt.js';
use crate::rust::fmt::{debugStruct, debugTuple};
// meta-language:translated JavaScript import_statement items=1 sha256=c3cb20a246b48eceec768a0ec33382e903699a8492bff980cf6321ff00ce5112
// | import { debugStr } from './rust/str.js';
use crate::rust::str_::debugStr;
// meta-language:translated JavaScript import_statement items=1 sha256=c38c408c3b33a97e7d4fe1f03c71ddc225dde4425177e192f52a167e77e8efd1
// | import { ExchangeRateInfo } from './types/currency.js';
use crate::types::currency::ExchangeRateInfo;

// meta-language:translated JavaScript export_statement items=1 sha256=50679d069c79fd7b0fd1b3e7090c9bf427faca6e999862409a79668c329ae89b
// | /** The API source identifier for rates fetched from CoinGecko. */
// | export const COINGECKO_SOURCE = 'coingecko.com';
pub const COINGECKO_SOURCE: &str = "coingecko.com";

// meta-language:translated JavaScript export_statement items=1 sha256=62b190ab31697bf2252585237cc1fdae2af7d96b713fd7e947aa78744c8de5bc
// | /** The API source identifier for rates fetched from CoinMarketCap. */
// | export const COINMARKETCAP_SOURCE = 'coinmarketcap.com';
pub const COINMARKETCAP_SOURCE: &str = "coinmarketcap.com";

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * CoinGecko coin ID mapping for common cryptocurrencies.
// |  *
// |  * Maps the ticker symbol (uppercase) to the CoinGecko ID.
// |  * @param {string} ticker
// |  * @returns {string | null}
// |  */
// | export function coingeckoId(ticker) {
// |   switch (ticker.toUpperCase()) {
// |     case 'TON':
// |     case 'TONCOIN':
// |       return 'the-open-network';
// |     case 'BTC':
// |     case 'BITCOIN':
// |       return 'bitcoin';
// |     case 'ETH':
// |     case 'ETHEREUM':
// |       return 'ethereum';
// |     case 'BNB':
// |     case 'BINANCECOIN':
// |       return 'binancecoin';
// |     case 'SOL':
// |     case 'SOLANA':
// |       return 'solana';
// |     case 'XRP':
// |     case 'RIPPLE':
// |       return 'ripple';
// |     case 'ADA':
// |     case 'CARDANO':
// |       return 'cardano';
// |     case 'DOGE':
// |     case 'DOGECOIN':
// |       return 'dogecoin';
// |     case 'DOT':
// |     case 'POLKADOT':
// |       return 'polkadot';
// |     case 'LTC':
// |     case 'LITECOIN':
// |       return 'litecoin';
// |     case 'LINK':
// |     case 'CHAINLINK':
// |       return 'chainlink';
// |     case 'UNI':
// |     case 'UNISWAP':
// |       return 'uniswap';
// |     default:
// |       return null;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Error type for crypto API operations. */
// | export class CryptoApiError {
// |   /**
// |    * @param {string} tag the variant name
// |    * @param {{ _0?: string, ticker?: string }} [fields]
// |    */
// |   constructor(tag, fields) {
// |     this.tag = tag;
// |     Object.assign(this, fields);
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * Network request failed.
// |    * @param {string} value
// |    */
// |   static NetworkError(value) {
// |     return new CryptoApiError('NetworkError', { _0: value });
// |   }
// |
// |   /**
// |    * Failed to parse response.
// |    * @param {string} value
// |    */
// |   static ParseError(value) {
// |     return new CryptoApiError('ParseError', { _0: value });
// |   }
// |
// |   /**
// |    * Coin not found.
// |    * @param {{ ticker: string }} fields
// |    */
// |   static CoinNotFound({ ticker }) {
// |     return new CryptoApiError('CoinNotFound', { ticker });
// |   }
// |
// |   /** @returns {string} */
// |   toString() {
// |     const f = /** @type {{ _0: string, ticker: string }} */ (/** @type {unknown} */ (this));
// |     switch (this.tag) {
// |       case 'NetworkError':
// |         return `Network error: ${f._0}`;
// |       case 'ParseError':
// |         return `Parse error: ${f._0}`;
// |       default:
// |         return `Cryptocurrency not found: ${f.ticker}`;
// |     }
// |   }
// |
// |   /** `#[derive(Debug)]`. */
// |   fmtDebug() {
// |     const f = /** @type {{ _0: string, ticker: string }} */ (/** @type {unknown} */ (this));
// |     if (this.tag === 'CoinNotFound') {
// |       return debugStruct('CoinNotFound', [['ticker', debugStr(f.ticker)]]);
// |     }
// |     return debugTuple(this.tag, [debugStr(f._0)]);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches the current price of a cryptocurrency in a given fiat currency.
// |  *
// |  * Uses the CoinGecko API (free, no key required for basic usage).
// |  *
// |  * Returns an `ExchangeRateInfo` with the current price
// |  * (1 [ticker] = price [vs_currency]); rejects with `Err(CryptoApiError)`.
// |  * @param {string} ticker The cryptocurrency ticker (e.g., "TON", "BTC", "ETH")
// |  * @param {string} vsCurrency The fiat currency to price in (e.g., "usd", "eur")
// |  * @returns {Promise<ExchangeRateInfo>}
// |  */
// | export async function fetchCryptoPrice(ticker, vsCurrency) {
// |   const coinId = coingeckoId(ticker);
// |   if (coinId === null) {
// |     throw err(CryptoApiError.CoinNotFound({ ticker: ticker.toUpperCase() }));
// |   }
// |
// |   const vs = vsCurrency.toLowerCase();
// |   const url = `https://api.coingecko.com/api/v3/simple/price?ids=${coinId}&vs_currencies=${vs}`;
// |
// |   const [price, date] = await fetchCoingeckoPrice(url, coinId, vs);
// |
// |   const timestamp = clock.now().toRfc3339();
// |   return ExchangeRateInfo.new(price, COINGECKO_SOURCE, date).withFetchedAt(timestamp);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches prices for multiple cryptocurrencies against a fiat currency.
// |  *
// |  * Returns a map from ticker -> `ExchangeRateInfo`.
// |  * @param {string[]} tickers Cryptocurrency tickers (e.g., `["TON", "BTC", "ETH"]`)
// |  * @param {string} vsCurrency The fiat currency to price in (e.g., "usd", "eur")
// |  * @returns {Promise<Map<string, ExchangeRateInfo>>}
// |  */
// | export async function fetchCryptoPrices(tickers, vsCurrency) {
// |   // Map tickers to CoinGecko IDs
// |   /** @type {Map<string, string>} */
// |   const idToTicker = new Map();
// |   /** @type {string[]} */
// |   const ids = [];
// |
// |   for (const ticker of tickers) {
// |     const upper = ticker.toUpperCase();
// |     const id = coingeckoId(upper);
// |     if (id !== null) {
// |       idToTicker.set(id, upper);
// |       ids.push(id);
// |     }
// |   }
// |
// |   if (ids.length === 0) {
// |     return new Map();
// |   }
// |
// |   const vs = vsCurrency.toLowerCase();
// |   const idsStr = ids.join(',');
// |   const url = `https://api.coingecko.com/api/v3/simple/price?ids=${idsStr}&vs_currencies=${vs}`;
// |
// |   const json = await fetchJsonValue(url);
// |
// |   const timestamp = clock.now().toRfc3339();
// |   const today = clock.now().format('%Y-%m-%d');
// |
// |   /** @type {Map<string, ExchangeRateInfo>} */
// |   const result = new Map();
// |
// |   for (const [id, ticker] of idToTicker) {
// |     const coinData = asObject(get(json, id));
// |     if (coinData !== null) {
// |       const price = asF64(get(coinData, vs));
// |       if (price !== null) {
// |         const info = ExchangeRateInfo.new(price, COINGECKO_SOURCE, today).withFetchedAt(timestamp);
// |         result.set(ticker, info);
// |       }
// |     }
// |   }
// |
// |   return result;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Fetches a single coin price from CoinGecko.
// |  * @param {string} url
// |  * @param {string} coinId
// |  * @param {string} vsCurrency
// |  * @returns {Promise<[number, string]>}
// |  */
// | async function fetchCoingeckoPrice(url, coinId, vsCurrency) {
// |   const json = await fetchJsonValue(url);
// |   const today = clock.now().format('%Y-%m-%d');
// |   return [coingeckoPriceFromJson(json, coinId, vsCurrency), today];
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The price lookup of `fetch_coingecko_price`, split out so tests can run it
// |  * without the network.
// |  * @internal
// |  * @param {unknown} json
// |  * @param {string} coinId
// |  * @param {string} vsCurrency
// |  * @returns {number}
// |  */
// | export function coingeckoPriceFromJson(json, coinId, vsCurrency) {
// |   const price = asF64(get(get(json, coinId), vsCurrency));
// |   if (price === null) {
// |     throw err(CryptoApiError.ParseError(`Price not found for ${coinId} in ${vsCurrency}`));
// |   }
// |   return price;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::Value::get(key)` on an object (null for other values).
// |  * @param {unknown} value
// |  * @param {string} key
// |  * @returns {unknown}
// |  */
// | function get(value, key) {
// |   const object = asObject(value);
// |   return object !== null && Object.prototype.hasOwnProperty.call(object, key) ? object[key] : null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::Value::as_object`.
// |  * @param {unknown} value
// |  * @returns {Record<string, unknown> | null}
// |  */
// | function asObject(value) {
// |   return typeof value === 'object' && value !== null && !Array.isArray(value)
// |     ? /** @type {Record<string, unknown>} */ (value)
// |     : null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::Value::as_f64`.
// |  * @param {unknown} value
// |  * @returns {number | null}
// |  */
// | function asF64(value) {
// |   return typeof value === 'number' ? value : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of a `JsValue` error, as wasm-bindgen prints it.
// |  * @internal
// |  * @param {unknown} e
// |  */
// | export function debugJsValue(e) {
// |   return `JsValue(${String(e)})`;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Performs an HTTP GET request and parses the response as JSON.
// |  * @param {string} url
// |  * @returns {Promise<unknown>}
// |  */
// | async function fetchJsonValue(url) {
// |   /** @type {Request} */
// |   let request;
// |   try {
// |     request = new Request(url, { method: 'GET', mode: 'cors' });
// |   } catch (e) {
// |     throw err(CryptoApiError.NetworkError(`Failed to create request: ${debugJsValue(e)}`));
// |   }
// |
// |   try {
// |     request.headers.set('Accept', 'application/json');
// |   } catch (e) {
// |     throw err(CryptoApiError.NetworkError(`Failed to set headers: ${debugJsValue(e)}`));
// |   }
// |
// |   if (typeof globalThis.fetch !== 'function') {
// |     throw err(CryptoApiError.NetworkError('Neither Window nor WorkerGlobalScope available'));
// |   }
// |   /** @type {Response} */
// |   let resp;
// |   try {
// |     resp = await globalThis.fetch(request);
// |   } catch (e) {
// |     throw err(CryptoApiError.NetworkError(`Fetch failed: ${debugJsValue(e)}`));
// |   }
// |
// |   if (!resp.ok) {
// |     throw err(CryptoApiError.NetworkError(`HTTP error: ${resp.status}`));
// |   }
// |
// |   try {
// |     return await resp.json();
// |   } catch (e) {
// |     throw err(CryptoApiError.ParseError(`Failed to await JSON: ${debugJsValue(e)}`));
// |   }
// | }
