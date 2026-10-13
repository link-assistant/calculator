/**
 * Currency exchange rate API client (port of `src/currency_api.rs`).
 *
 * This module provides functionality to fetch real-time and historical exchange
 * rates from official Central Bank APIs.
 *
 * Primary source: Frankfurter API (https://frankfurter.dev/) - European Central Bank (ECB) data
 * - Provides exchange rates for 30+ currencies
 * - Updated daily at around 16:00 CET
 * - Data available from 1999
 *
 * Secondary source: Central Bank of Russia (https://cbr.ru) - CBR data
 * - Provides RUB exchange rates for 50+ currencies
 * - Updated daily
 * - Used for all RUB-related conversions
 *
 * Rust fetches through `web_sys` (`window.fetch` / `self.fetch`); this port
 * calls `globalThis.fetch` (Node 20+, Bun, browsers, workers) with the same
 * requests and the same error variants and messages.
 */

import * as clock from './clock.js';
import { debugJsValue } from './crypto_api.js';
import { err, ok, tryResult } from './rust/core.js';
import { debugStruct, debugTuple } from './rust/fmt.js';
import { parseF64 } from './rust/num.js';
import { debugStr, find, replace, split, trim } from './rust/str.js';
import { ExchangeRateInfo } from './types/currency.js';

/** The API source identifier for rates fetched from ECB via Frankfurter API. */
export const API_SOURCE = 'frankfurter.dev (ECB)';

/** The API source identifier for rates fetched from the Central Bank of Russia. */
export const CBR_API_SOURCE = 'cbr.ru (Central Bank of Russia)';

/** Primary URL for the Frankfurter API (European Central Bank data). */
const FRANKFURTER_URL = 'https://api.frankfurter.app';

/**
 * URL for the Central Bank of Russia daily exchange rates (XML API).
 * Returns all currency rates relative to RUB for the current business day.
 */
const CBR_DAILY_URL = 'https://www.cbr.ru/scripts/XML_daily.asp';

/** Response from the Frankfurter API. */
export class CurrencyApiResponse {
  /** @param {{ amount: number, base: string, date: string, rates: Map<string, number> }} fields */
  constructor({ amount, base, date, rates }) {
    /** The amount (always 1 for our requests). */
    this.amount = amount;
    /** The base currency. */
    this.base = base;
    /** The date of the rates (YYYY-MM-DD format). */
    this.date = date;
    /** The rates for the base currency. */
    this.rates = rates;
  }

  /**
   * `#[derive(Deserialize)]` from the parsed JSON (`amount` defaults to 0).
   * Throws `Err(string)` with a serde-like message when the shape is wrong.
   * @param {unknown} json
   * @returns {CurrencyApiResponse}
   */
  static deserialize(json) {
    if (typeof json !== 'object' || json === null || Array.isArray(json)) {
      return err('invalid type: expected struct CurrencyApiResponse');
    }
    const o = /** @type {Record<string, unknown>} */ (json);
    const amount = o.amount === undefined ? 0 : o.amount;
    if (typeof amount !== 'number') {
      return err('invalid type: expected f64');
    }
    for (const field of ['base', 'date', 'rates']) {
      if (o[field] === undefined) {
        return err(`missing field \`${field}\``);
      }
    }
    if (typeof o.base !== 'string' || typeof o.date !== 'string') {
      return err('invalid type: expected a string');
    }
    if (typeof o.rates !== 'object' || o.rates === null || Array.isArray(o.rates)) {
      return err('invalid type: expected a map');
    }
    /** @type {Map<string, number>} */
    const rates = new Map();
    for (const [k, v] of Object.entries(o.rates)) {
      if (typeof v !== 'number') {
        return err('invalid type: expected f64');
      }
      rates.set(k, v);
    }
    return new CurrencyApiResponse({ amount, base: o.base, date: o.date, rates });
  }
}

/** Error type for currency API operations. */
export class CurrencyApiError {
  /**
   * @param {string} tag the variant name
   * @param {{ _0?: string, from?: string, to?: string }} [fields]
   */
  constructor(tag, fields) {
    this.tag = tag;
    Object.assign(this, fields);
    Object.freeze(this);
  }

  /**
   * Network request failed.
   * @param {string} value
   */
  static NetworkError(value) {
    return new CurrencyApiError('NetworkError', { _0: value });
  }

  /**
   * Failed to parse response.
   * @param {string} value
   */
  static ParseError(value) {
    return new CurrencyApiError('ParseError', { _0: value });
  }

  /**
   * Rate not found for currency pair.
   * @param {{ from: string, to: string }} fields
   */
  static RateNotFound({ from, to }) {
    return new CurrencyApiError('RateNotFound', { from, to });
  }

  /** @returns {string} */
  toString() {
    const f = /** @type {{ _0: string, from: string, to: string }} */ (/** @type {unknown} */ (this));
    switch (this.tag) {
      case 'NetworkError':
        return `Network error: ${f._0}`;
      case 'ParseError':
        return `Parse error: ${f._0}`;
      default:
        return `Rate not found for ${f.from}/${f.to}`;
    }
  }

  /** `#[derive(Debug)]`. */
  fmtDebug() {
    const f = /** @type {{ _0: string, from: string, to: string }} */ (/** @type {unknown} */ (this));
    if (this.tag === 'RateNotFound') {
      return debugStruct('RateNotFound', [
        ['from', debugStr(f.from)],
        ['to', debugStr(f.to)],
      ]);
    }
    return debugTuple(this.tag, [debugStr(f._0)]);
  }
}

/**
 * Fetches the current exchange rates for a base currency from ECB via Frankfurter API.
 *
 * Returns a tuple of (date, rates map) where rates map currency codes to exchange rates.
 *
 * Note: The ECB publishes rates daily at around 16:00 CET.
 * RUB is not available through ECB - use historical .lino files for RUB rates.
 * @param {string} baseCurrency The base currency code (e.g., "USD", "EUR")
 * @returns {Promise<[string, Map<string, number>]>}
 */
export async function fetchCurrentRates(baseCurrency) {
  const base = baseCurrency.toUpperCase();
  const url = `${FRANKFURTER_URL}/latest?from=${base}`;

  return fetchRatesFromUrl(url);
}

/**
 * Fetches historical exchange rates for a specific date from ECB via Frankfurter API.
 *
 * Returns a tuple of (date, rates map) where rates map currency codes to exchange rates.
 *
 * Note: RUB is not available through ECB - use historical .lino files for RUB rates.
 * @param {string} baseCurrency The base currency code (e.g., "USD", "EUR")
 * @param {string} date The date in YYYY-MM-DD format (ECB data available from 1999-01-04)
 * @returns {Promise<[string, Map<string, number>]>}
 */
export async function fetchHistoricalRates(baseCurrency, date) {
  const base = baseCurrency.toUpperCase();
  const url = `${FRANKFURTER_URL}/${date}?from=${base}`;

  return fetchRatesFromUrl(url);
}

/**
 * Fetches rates from the Frankfurter API URL.
 * @param {string} url
 * @returns {Promise<[string, Map<string, number>]>}
 */
async function fetchRatesFromUrl(url) {
  return fetchJson(url);
}

/**
 * Sends a GET request with the given `Accept` header through the global
 * `fetch` (the Window / WorkerGlobalScope branches of Rust), mapping each
 * failure to the `NetworkError` Rust returns. `label` is `""` or `"CBR "`.
 * @param {string} url
 * @param {string} accept
 * @param {string} label
 * @returns {Promise<Response>}
 */
async function sendRequest(url, accept, label) {
  /** @type {Request} */
  let request;
  try {
    request = new Request(url, { method: 'GET', mode: 'cors' });
  } catch (e) {
    throw err(CurrencyApiError.NetworkError(`Failed to create ${label}request: ${debugJsValue(e)}`));
  }

  try {
    request.headers.set('Accept', accept);
  } catch (e) {
    throw err(CurrencyApiError.NetworkError(`Failed to set headers: ${debugJsValue(e)}`));
  }

  // Use the global fetch function which works in both Window and Worker contexts.
  if (typeof globalThis.fetch !== 'function') {
    throw err(CurrencyApiError.NetworkError('Neither Window nor WorkerGlobalScope available'));
  }
  /** @type {Response} */
  let resp;
  try {
    resp = await globalThis.fetch(request);
  } catch (e) {
    throw err(CurrencyApiError.NetworkError(`${label === '' ? 'Fetch' : 'CBR fetch'} failed: ${debugJsValue(e)}`));
  }

  if (!resp.ok) {
    throw err(CurrencyApiError.NetworkError(`${label}HTTP error: ${resp.status}`));
  }
  return resp;
}

/**
 * Performs the actual fetch and JSON parsing for the Frankfurter API.
 * Works in both Window and Web Worker contexts.
 * @param {string} url
 * @returns {Promise<[string, Map<string, number>]>}
 */
async function fetchJson(url) {
  const resp = await sendRequest(url, 'application/json', '');

  /** @type {unknown} */
  let json;
  try {
    json = await resp.json();
  } catch (e) {
    throw err(CurrencyApiError.ParseError(`Failed to parse JSON: ${debugJsValue(e)}`));
  }

  return parseFrankfurterJson(json);
}

/**
 * The parsing half of `fetch_json`, split out so tests can run it without the
 * network.
 *
 * The API returns: { "amount": 1, "base": "USD", "date": "YYYY-MM-DD", "rates": { "EUR": 0.92, ... } }
 * @internal
 * @param {unknown} json
 * @returns {[string, Map<string, number>]}
 */
export function parseFrankfurterJson(json) {
  const result = tryResult(() => CurrencyApiResponse.deserialize(json));
  if (!result.ok) {
    const message = /** @type {string} */ (result.error);
    throw err(CurrencyApiError.ParseError(`Failed to deserialize: Error(${debugStr(message)})`));
  }
  const response = /** @type {CurrencyApiResponse} */ (result.value);

  // Convert rate keys to lowercase for consistency with other API consumers
  /** @type {Map<string, number>} */
  const rates = new Map();
  for (const [k, v] of response.rates) {
    rates.set(k.toLowerCase(), v);
  }

  return [response.date, rates];
}

/**
 * Converts raw API rates to `ExchangeRateInfo` objects.
 * @param {string} base
 * @param {string} date
 * @param {Map<string, number>} rates
 * @returns {Array<[string, string, ExchangeRateInfo]>}
 */
export function ratesToExchangeInfo(base, date, rates) {
  const timestamp = clock.now().toRfc3339();
  const baseUpper = base.toUpperCase();

  return [...rates].map(([target, rate]) => {
    const targetUpper = target.toUpperCase();
    const info = ExchangeRateInfo.new(rate, API_SOURCE, date).withFetchedAt(timestamp);
    return [baseUpper, targetUpper, info];
  });
}

/**
 * Fetches a single exchange rate.
 * @param {string} from
 * @param {string} to
 * @returns {Promise<ExchangeRateInfo>}
 */
export async function fetchRate(from, to) {
  const [date, rates] = await fetchCurrentRates(from);
  const toLower = to.toLowerCase();

  const rate = rates.get(toLower);
  if (rate === undefined) {
    throw err(CurrencyApiError.RateNotFound({ from: from.toUpperCase(), to: to.toUpperCase() }));
  }
  const timestamp = clock.now().toRfc3339();
  return ExchangeRateInfo.new(rate, API_SOURCE, date).withFetchedAt(timestamp);
}

/**
 * Fetches current exchange rates from the Central Bank of Russia (cbr.ru).
 *
 * The CBR API returns all rates relative to RUB. This function converts them
 * to a map where keys are currency codes (lowercase) and values are the rate
 * expressed as "1 CURRENCY = X RUB" (i.e., how many rubles per unit of foreign currency).
 *
 * Returns a tuple of (date_string, rates_map) where:
 * - `date_string` is in YYYY-MM-DD format
 * - `rates_map` maps lowercase currency code -> rate (1 unit of that currency in RUB)
 * @returns {Promise<[string, Map<string, number>]>}
 */
export async function fetchCbrRates() {
  // CBR returns Windows-1251 encoded XML, but modern browsers decode it correctly
  const resp = await sendRequest(CBR_DAILY_URL, 'application/xml, text/xml, */*', 'CBR ');

  // Get the response as text (XML)
  /** @type {unknown} */
  let textValue;
  try {
    textValue = await resp.text();
  } catch (e) {
    throw err(CurrencyApiError.ParseError(`Failed to await CBR text: ${debugJsValue(e)}`));
  }

  if (typeof textValue !== 'string') {
    throw err(CurrencyApiError.ParseError('CBR response is not a string'));
  }

  return parseCbrXml(textValue);
}

/**
 * Parses the CBR XML response and extracts exchange rates.
 *
 * The XML format from cbr.ru looks like:
 * ```xml
 * <ValCurs Date="25.02.2026" name="Foreign Currency Market">
 *   <Valute ID="R01235">
 *     <NumCode>840</NumCode>
 *     <CharCode>USD</CharCode>
 *     <Nominal>1</Nominal>
 *     <Name>Доллар США</Name>
 *     <Value>76,6342</Value>
 *   </Valute>
 *   ...
 * </ValCurs>
 * ```
 *
 * Note: The Value uses comma as decimal separator (Russian locale).
 * The rate returned is: 1 unit of CURRENCY = Value/Nominal RUB
 * @param {string} xml
 * @returns {[string, Map<string, number>]}
 */
export function parseCbrXml(xml) {
  // Extract the date from ValCurs Date attribute: Date="25.02.2026"
  const date = extractCbrDate(xml);

  // Extract all Valute entries
  /** @type {Map<string, number>} */
  const rates = new Map();

  // Simple XML parsing by finding Valute blocks (UTF-16 indexes: only used to slice)
  let remaining = xml;
  let valuteStart;
  while ((valuteStart = find(remaining, '<Valute')) !== null) {
    const blockStart = valuteStart;
    const endTag = '</Valute>';
    const e = find(remaining.slice(blockStart), endTag);
    if (e === null) {
      break;
    }
    const blockEnd = blockStart + e + endTag.length;

    const valuteBlock = remaining.slice(blockStart, blockEnd);

    const parsed = parseValuteBlock(valuteBlock);
    if (parsed !== null) {
      const [code, rate] = parsed;
      rates.set(code.toLowerCase(), rate);
    }

    remaining = remaining.slice(blockEnd);
  }

  if (rates.size === 0) {
    throw err(CurrencyApiError.ParseError('No exchange rates found in CBR response'));
  }

  return [date, rates];
}

/**
 * Extracts the date from the CBR XML ValCurs element.
 * The date is in dd.mm.yyyy format and is converted to yyyy-mm-dd.
 * @param {string} xml
 * @returns {string}
 */
function extractCbrDate(xml) {
  // Look for Date="dd.mm.yyyy" pattern
  const datePrefix = 'Date="';
  const dateStart = find(xml, datePrefix);
  if (dateStart === null) {
    throw err(CurrencyApiError.ParseError('Date attribute not found in CBR response'));
  }

  const afterPrefix = xml.slice(dateStart + datePrefix.length);
  const dateEnd = find(afterPrefix, '"');
  if (dateEnd === null) {
    throw err(CurrencyApiError.ParseError('Date attribute closing quote not found'));
  }

  const dateStr = afterPrefix.slice(0, dateEnd);

  // Convert from dd.mm.yyyy to yyyy-mm-dd
  const parts = split(dateStr, '.');
  if (parts.length !== 3) {
    throw err(CurrencyApiError.ParseError(`Invalid CBR date format: ${dateStr}`));
  }

  return `${parts[2]}-${parts[1]}-${parts[0]}`;
}

/**
 * Parses a single Valute XML block and returns (currency_code, rate_per_unit_in_rub).
 * @param {string} block
 * @returns {[string, number] | null}
 */
function parseValuteBlock(block) {
  const charCode = extractXmlText(block, 'CharCode');
  if (charCode === null) return null;
  const nominalStr = extractXmlText(block, 'Nominal');
  if (nominalStr === null) return null;
  const valueStr = extractXmlText(block, 'Value');
  if (valueStr === null) return null;

  const nominal = ok(() => parseF64(nominalStr));
  if (nominal === null) return null;
  // CBR uses comma as decimal separator
  const value = ok(() => parseF64(replace(valueStr, ',', '.')));
  if (value === null) return null;

  if (nominal <= 0.0 || value <= 0.0) {
    return null;
  }

  // Rate: 1 unit of CharCode = (value / nominal) RUB
  const rate = value / nominal;

  return [charCode, rate];
}

/**
 * Extracts text content from an XML element by tag name.
 * @internal
 * @param {string} xml
 * @param {string} tag
 * @returns {string | null}
 */
export function extractXmlText(xml, tag) {
  const openTag = `<${tag}>`;
  const closeTag = `</${tag}>`;

  const found = find(xml, openTag);
  if (found === null) return null;
  const start = found + openTag.length;
  const end = find(xml.slice(start), closeTag);
  if (end === null) return null;

  return trim(xml.slice(start, start + end));
}
