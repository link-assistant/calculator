// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=39cfab4d8e8a8a39c7eaf0e346f59f26a46096bc1a6341981e0c9f9f1e56fbbd bytes=12546

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * WebAssembly bindings for the calculator (port of `src/wasm.rs`).
// |  *
// |  * This module exposes the same API as the wasm-pack build
// |  * (`import init, { Calculator, fetch_exchange_rates, ... } from '@wasm/link_calculator'`):
// |  * wasm-bindgen keeps the Rust (snake_case) names, so they are kept here too;
// |  * camelCase aliases follow the porting conventions. There is no WebAssembly
// |  * module to load, so `init` resolves immediately.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import * as cryptoApi from './crypto_api.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as currencyApi from './currency_api.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as clock from './clock.js';
// meta-language:translated JavaScript import_statement items=1 sha256=64b351d62f39d50a5e59723ed1d11d0b4b78b3138871ec4ba85121df3f447292
// | import { Calculator as CoreCalculator, VERSION } from './index.js';
use crate::index::{Calculator as CoreCalculator, VERSION};
// meta-language:translated JavaScript import_statement items=1 sha256=df6ad2075e33a2c0da5c18415f2e97b706249fa0300c626524c44150dabe3bdb
// | import { Err } from './rust/core.js';
use crate::rust::core_::Err;
// meta-language:translated JavaScript import_statement items=1 sha256=a2ae0815669f7054bba80f964dad6aba3d95e5ac76315e413af8dd9012011704
// | import { parseF64 } from './rust/num.js';
use crate::rust::num::parseF64;
// meta-language:translated JavaScript import_statement items=1 sha256=f993dbd7c68acedcfa1f0082be5d0debedd93a4cebe60ebd33148d5fde32bcef
// | import { lines, splitWhitespace, stripPrefix, trim, trimEndMatches, trimStartMatches } from './rust/str.js';
use crate::rust::str_::{lines, splitWhitespace, stripPrefix, trim, trimEndMatches, trimStartMatches};

// meta-language:translated JavaScript lexical_declaration items=1 sha256=168e7c49ed11b603b000beaabf59348eadff8c6ba7b08c57143a94a3b7d71980
// | /** The base currency for CBR rates (all CBR rates are expressed relative to RUB). */
// | const CBR_BASE_CURRENCY = 'RUB';
pub const CBR_BASE_CURRENCY: &str = "RUB";

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The default export of a wasm-pack module: loads the WebAssembly module.
// |  * Nothing to load here; runs `wasmInit` like `#[wasm_bindgen(start)]`.
// |  * @param {unknown} [_moduleOrPath]
// |  * @returns {Promise<void>}
// |  */
// | export default async function init(_moduleOrPath) {
// |   wasmInit();
// | }

// meta-language:carried JavaScript export_statement (syntax)
// | /**
// |  * The synchronous variant of `init` that wasm-pack generates.
// |  * @param {unknown} [_module]
// |  */
// | export function initSync(_module) {
// |   wasmInit();
// | }

// meta-language:translated JavaScript export_statement items=1 sha256=f8af801ae83019d9d83831279f02715144365d32eb8589dfbe2c2d9a9a2d7d0f
// | /** Initializes the WASM module. Call this once before using other functions. */
// | export function wasmInit() {
// |   // Nothing to set up: JavaScript errors already carry their messages.
// | }
pub fn wasm_init() -> () {
    ()
}

// meta-language:translated JavaScript export_statement items=1 sha256=30a8e87a2a52804e110c042389729a7a3562e622e000e68273400a88576e71dc
// | /**
// |  * Returns the current version of the calculator library.
// |  * @returns {string}
// |  */
// | export function getVersion() {
// |   return VERSION;
// | }
pub fn get_version() -> String {
    String::from(VERSION)
}

// meta-language:translated JavaScript export_statement items=1 sha256=0d76458881bff7bc3c113df23fff4c13cf71c765df58dad8ab16de0fde75ac29
// | /**
// |  * A simple health check function.
// |  * @returns {boolean}
// |  */
// | export function healthCheck() {
// |   return true;
// | }
pub fn health_check() -> bool {
    true
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Response from fetching exchange rates. */
// | export class ExchangeRatesResponse {
// |   /**
// |    * @param {{ success: boolean, date: string, base: string, error: string | null, ratesJson: string }} fields
// |    */
// |   constructor({ success, date, base, error, ratesJson }) {
// |     /** Whether the fetch was successful. */
// |     this.success = success;
// |     /** The date of the rates. */
// |     this.date = date;
// |     /** The base currency. */
// |     this.base = base;
// |     /** Error message if fetch failed. */
// |     this.error = error;
// |     /** The rates as a JSON string (currency -> rate). */
// |     this.ratesJson = ratesJson;
// |   }
// |
// |   /** Gets the rates as a JSON string (the wasm-bindgen getter name). */
// |   get rates_json() {
// |     return this.ratesJson;
// |   }
// |
// |   /** serde's JSON. */
// |   toJSON() {
// |     return {
// |       success: this.success,
// |       date: this.date,
// |       base: this.base,
// |       error: this.error,
// |       rates_json: this.ratesJson,
// |     };
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Response from fetching cryptocurrency rates. */
// | export class CryptoRatesResponse extends ExchangeRatesResponse {}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::to_string` of a rates map.
// |  * @param {Map<string, number>} rates
// |  */
// | function ratesToJson(rates) {
// |   return JSON.stringify(Object.fromEntries(rates));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Runs a rate fetch and builds the response JSON, as the `future_to_promise`
// |  * blocks of `src/wasm.rs` do: an `Err` becomes a failed response, never a
// |  * rejected promise.
// |  * @param {() => Promise<[string, Map<string, number>]>} fetchRates
// |  * @param {string} base
// |  * @returns {Promise<string>}
// |  */
// | async function ratesResponse(fetchRates, base) {
// |   let date;
// |   let rates;
// |   try {
// |     [date, rates] = await fetchRates();
// |   } catch (thrown) {
// |     if (!(thrown instanceof Err)) {
// |       throw thrown;
// |     }
// |     const response = new ExchangeRatesResponse({
// |       success: false,
// |       date: '',
// |       base,
// |       error: String(thrown.error),
// |       ratesJson: '',
// |     });
// |     return JSON.stringify(response);
// |   }
// |   const response = new ExchangeRatesResponse({ success: true, date, base, error: null, ratesJson: ratesToJson(rates) });
// |   return JSON.stringify(response);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches current exchange rates for a base currency.
// |  * Returns a Promise that resolves to a JSON string with the rates.
// |  * @param {string} baseCurrency
// |  * @returns {Promise<string>}
// |  */
// | export function fetchExchangeRates(baseCurrency) {
// |   return ratesResponse(() => currencyApi.fetchCurrentRates(baseCurrency), baseCurrency.toUpperCase());
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches historical exchange rates for a specific date.
// |  * Returns a Promise that resolves to a JSON string with the rates.
// |  * @param {string} baseCurrency
// |  * @param {string} date
// |  * @returns {Promise<string>}
// |  */
// | export function fetchHistoricalRates(baseCurrency, date) {
// |   return ratesResponse(() => currencyApi.fetchHistoricalRates(baseCurrency, date), baseCurrency.toUpperCase());
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches current exchange rates from the Central Bank of Russia (cbr.ru).
// |  *
// |  * Returns a Promise that resolves to a JSON string with `ExchangeRatesResponse`.
// |  * The base currency is always "RUB", and the rates are "1 CURRENCY = X RUB".
// |  * These rates should be used for all RUB-related currency conversions.
// |  * @returns {Promise<string>}
// |  */
// | export function fetchCbrRates() {
// |   return ratesResponse(() => currencyApi.fetchCbrRates(), CBR_BASE_CURRENCY);
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * Removes the quotes around a `source` value.
// |  * @param {string} rest
// |  */
// | function unquoteSource(rest) {
// |   let src = trim(rest);
// |   src = trimEndMatches(trimStartMatches(src, "'"), "'");
// |   src = trimEndMatches(trimStartMatches(src, '"'), '"');
// |   return src;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `s.parse::<f64>().ok()`.
// |  * @param {string} s
// |  * @returns {number | null}
// |  */
// | function parseF64Ok(s) {
// |   try {
// |     return parseF64(s);
// |   } catch (thrown) {
// |     if (thrown instanceof Err) {
// |       return null;
// |     }
// |     throw thrown;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses a .lino rate file content and returns the rate data as JSON.
// |  * This allows the web app to parse .lino files without loading them into the calculator.
// |  * @param {string} content The .lino file content
// |  * @returns {string} A JSON string with the parsed rate information
// |  */
// | export function parseLinoRate(content) {
// |   let fromCurrency = null;
// |   let toCurrency = null;
// |   let value = null;
// |   let date = null;
// |   let source = null;
// |
// |   for (const raw of lines(content)) {
// |     const line = trim(raw);
// |     if (line === '' || line === 'rate:') {
// |       continue;
// |     }
// |
// |     let rest;
// |     if ((rest = stripPrefix(line, 'from ')) !== null) {
// |       fromCurrency = trim(rest).toUpperCase();
// |     } else if ((rest = stripPrefix(line, 'to ')) !== null) {
// |       toCurrency = trim(rest).toUpperCase();
// |     } else if ((rest = stripPrefix(line, 'value ')) !== null) {
// |       value = parseF64Ok(trim(rest));
// |     } else if ((rest = stripPrefix(line, 'date ')) !== null) {
// |       date = trim(rest);
// |     } else if ((rest = stripPrefix(line, 'source ')) !== null) {
// |       source = unquoteSource(rest);
// |     }
// |   }
// |
// |   const success = fromCurrency !== null && toCurrency !== null && value !== null && date !== null;
// |   return JSON.stringify({
// |     success,
// |     from: fromCurrency,
// |     to: toCurrency,
// |     value,
// |     date,
// |     source,
// |     error: success ? null : 'Missing required fields',
// |   });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses a consolidated .lino rate file and returns the rate data as JSON.
// |  *
// |  * Supports both the new format (conversion/rates) and legacy format (rates/data):
// |  *
// |  * New format:
// |  * ```text
// |  * conversion:
// |  *   from USD
// |  *   to EUR
// |  *   source 'frankfurter.dev (ECB)'
// |  *   rates:
// |  *     2021-01-25 0.8234
// |  *     2021-02-01 0.8315
// |  * ```
// |  *
// |  * Legacy format:
// |  * ```text
// |  * rates:
// |  *   from USD
// |  *   to EUR
// |  *   source 'frankfurter.dev (ECB)'
// |  *   data:
// |  *     2021-01-25 0.8234
// |  *     2021-02-01 0.8315
// |  * ```
// |  * @param {string} content The consolidated .lino file content
// |  * @returns {string} A JSON string with the parsed rates array
// |  */
// | export function parseConsolidatedLinoRates(content) {
// |   let fromCurrency = null;
// |   let toCurrency = null;
// |   let source = null;
// |   /** @type {{ date: string, value: number }[]} */
// |   const rates = [];
// |   let inDataSection = false;
// |
// |   for (const line of lines(content)) {
// |     const trimmed = trim(line);
// |
// |     // Skip empty lines
// |     if (trimmed === '') {
// |       continue;
// |     }
// |
// |     // Handle 'rates:' based on context:
// |     // - If we haven't parsed from_currency yet, it's a root marker (legacy format), skip it
// |     // - If we have parsed from_currency, it's the data section marker (new format)
// |     if (trimmed === 'rates:') {
// |       if (fromCurrency !== null) {
// |         // New format: 'rates:' marks the start of data section
// |         inDataSection = true;
// |       }
// |       // Either way, continue to next line (skip the 'rates:' line itself)
// |       continue;
// |     }
// |
// |     // Skip root marker for new format
// |     if (trimmed === 'conversion:') {
// |       continue;
// |     }
// |
// |     // Check for legacy data section marker
// |     if (trimmed === 'data:') {
// |       inDataSection = true;
// |       continue;
// |     }
// |
// |     let rest;
// |     if (inDataSection) {
// |       // Parse date and value: "2021-01-25 0.8234"
// |       const parts = splitWhitespace(trimmed);
// |       if (parts.length >= 2) {
// |         const value = parseF64Ok(parts[1]);
// |         if (value !== null) {
// |           rates.push({ date: parts[0], value });
// |         }
// |       }
// |     } else if ((rest = stripPrefix(trimmed, 'from ')) !== null) {
// |       fromCurrency = trim(rest).toUpperCase();
// |     } else if ((rest = stripPrefix(trimmed, 'to ')) !== null) {
// |       toCurrency = trim(rest).toUpperCase();
// |     } else if ((rest = stripPrefix(trimmed, 'source ')) !== null) {
// |       source = unquoteSource(rest);
// |     }
// |   }
// |
// |   const success = fromCurrency !== null && toCurrency !== null && rates.length > 0;
// |   return JSON.stringify({
// |     success,
// |     from: fromCurrency,
// |     to: toCurrency,
// |     source,
// |     rates,
// |     error: success ? null : 'Missing required fields or no rates found',
// |   });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fetches current cryptocurrency prices in a given fiat currency.
// |  *
// |  * Fetches prices for the most popular cryptocurrencies from CoinGecko (free API, no key needed).
// |  * @param {string} vsCurrency The fiat currency to price in (e.g., "usd", "eur")
// |  * @returns {Promise<string>} A JSON string with `CryptoRatesResponse`.
// |  */
// | export async function fetchCryptoRates(vsCurrency) {
// |   const tickers = ['TON', 'BTC', 'ETH', 'BNB', 'SOL', 'XRP', 'ADA', 'DOGE', 'DOT', 'LTC', 'LINK', 'UNI'];
// |   const today = clock.now().format('%Y-%m-%d');
// |   const base = vsCurrency.toUpperCase();
// |
// |   let prices;
// |   try {
// |     prices = await cryptoApi.fetchCryptoPrices(tickers, vsCurrency);
// |   } catch (thrown) {
// |     if (!(thrown instanceof Err)) {
// |       throw thrown;
// |     }
// |     return JSON.stringify(new CryptoRatesResponse({
// |       success: false,
// |       date: today,
// |       base,
// |       error: String(thrown.error),
// |       ratesJson: '',
// |     }));
// |   }
// |   // Build a rates map: ticker -> price
// |   const ratesMap = new Map([...prices].map(([ticker, info]) => [ticker, info.rate]));
// |   return JSON.stringify(new CryptoRatesResponse({
// |     success: true,
// |     date: today,
// |     base,
// |     error: null,
// |     ratesJson: ratesToJson(ratesMap),
// |   }));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The calculator with the method names wasm-bindgen exports (snake_case), as
// |  * the web worker calls them; the camelCase methods are inherited.
// |  */
// | export class Calculator extends CoreCalculator {
// |   constructor() {
// |     super();
// |   }
// |
// |   /** wasm-bindgen's `free()`: nothing to release. */
// |   free() {}
// |
// |   /** @param {number} offsetMinutes */
// |   set_timezone_offset(offsetMinutes) {
// |     this.setTimezoneOffset(offsetMinutes);
// |   }
// |
// |   clear_timezone_offset() {
// |     this.clearTimezoneOffset();
// |   }
// |
// |   /** @param {string} base @param {string} date @param {string} ratesJson @returns {number} */
// |   update_rates_from_api(base, date, ratesJson) {
// |     return this.updateRatesFromApi(base, date, ratesJson);
// |   }
// |
// |   /** @param {string} date @param {string} ratesJson @returns {number} */
// |   update_cbr_rates_from_api(date, ratesJson) {
// |     return this.updateCbrRatesFromApi(date, ratesJson);
// |   }
// |
// |   /** @param {string} base @param {string} date @param {string} ratesJson @returns {number} */
// |   update_crypto_rates_from_api(base, date, ratesJson) {
// |     return this.updateCryptoRatesFromApi(base, date, ratesJson);
// |   }
// |
// |   /** @param {string} content @returns {number} */
// |   load_rates_from_consolidated_lino(content) {
// |     return this.loadRatesFromConsolidatedLino(content);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | // wasm-bindgen's export names.
// | export {
// |   fetchCbrRates as fetch_cbr_rates,
// |   fetchCryptoRates as fetch_crypto_rates,
// |   fetchExchangeRates as fetch_exchange_rates,
// |   fetchHistoricalRates as fetch_historical_rates,
// |   getVersion as get_version,
// |   healthCheck as health_check,
// |   parseConsolidatedLinoRates as parse_consolidated_lino_rates,
// |   parseLinoRate as parse_lino_rate,
// |   wasmInit as wasm_init,
// | };
