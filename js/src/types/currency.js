/**
 * Currency types and exchange rate database (port of `src/types/currency.rs`).
 */

import { CalculatorError } from '../error.js';
import { err } from '../rust/core.js';
import { fmtF64 } from '../rust/fmt.js';
import { eqIgnoreAsciiCase, isAsciiAlphabetic, trim, utf8Len } from '../rust/str.js';
import { BUNDLED_LATEST_RATES } from '../generated/bundled-rates.js';

/**
 * A `(from, to, source, date, rate)` entry of a bundled rates table.
 * @typedef {readonly [string, string, string, string, number]} BundledRate
 */

/**
 * A `(from, to, rate info)` entry of the rates used by the last conversion.
 * @typedef {[string, string, ExchangeRateInfo]} UsedRate
 */

/** Information about an exchange rate, including its source and timestamp. */
export class ExchangeRateInfo {
  /** @param {{ rate: number, source: string, date: string, fetchedAt: string | null }} fields */
  constructor({ rate, source, date, fetchedAt }) {
    /** The exchange rate value. */
    this.rate = rate;
    /** The source of this rate (e.g., "frankfurter.dev (ECB)", "cbr.ru (Central Bank of Russia)", "default"). */
    this.source = source;
    /** The date this rate is from (ISO format: YYYY-MM-DD). */
    this.date = date;
    /** When this rate was fetched/updated (ISO timestamp). */
    this.fetchedAt = fetchedAt;
    Object.freeze(this);
  }

  /**
   * Creates a new exchange rate info with all metadata.
   * @param {number} rate
   * @param {string} source
   * @param {string} date
   */
  static new(rate, source, date) {
    return new ExchangeRateInfo({ rate, source, date, fetchedAt: null });
  }

  /**
   * Creates a default rate (used when no API data is available).
   * @param {number} rate
   */
  static defaultRate(rate) {
    return new ExchangeRateInfo({ rate, source: 'default (hardcoded)', date: 'unknown', fetchedAt: null });
  }

  /**
   * Sets the fetched_at timestamp (returns a copy: the value is frozen).
   * @param {string} timestamp
   */
  withFetchedAt(timestamp) {
    return new ExchangeRateInfo({ ...this, fetchedAt: timestamp });
  }

  /**
   * Formats this rate info for display in calculation steps.
   * @param {string} from
   * @param {string} to
   */
  formatForDisplay(from, to) {
    return `1 ${from.toUpperCase()} = ${fmtF64(this.rate)} ${to.toUpperCase()} (source: ${this.source}, date: ${this.date})`;
  }

  /** serde: `{ rate, source, date, fetched_at }`. */
  toJSON() {
    return { rate: this.rate, source: this.source, date: this.date, fetched_at: this.fetchedAt };
  }
}

/** Represents a currency with its code and metadata. */
export class Currency {
  /** @param {{ code: string, name: string, symbol: string, decimals: number }} fields */
  constructor({ code, name, symbol, decimals }) {
    /** ISO 4217 currency code (e.g., "USD", "EUR"). */
    this.code = code;
    /** Full name of the currency. */
    this.name = name;
    /** Currency symbol (e.g., "$", "€"). */
    this.symbol = symbol;
    /** Number of decimal places typically used. */
    this.decimals = decimals;
    Object.freeze(this);
  }

  /**
   * Creates a new currency.
   * @param {string} code
   * @param {string} name
   * @param {string} symbol
   * @param {number} decimals
   */
  static new(code, name, symbol, decimals) {
    return new Currency({ code: code.toUpperCase(), name, symbol, decimals });
  }

  /** Common currencies database. */
  static usd() {
    return Currency.new('USD', 'US Dollar', '$', 2);
  }

  static eur() {
    return Currency.new('EUR', 'Euro', '€', 2);
  }

  static gbp() {
    return Currency.new('GBP', 'British Pound', '£', 2);
  }

  static jpy() {
    return Currency.new('JPY', 'Japanese Yen', '¥', 0);
  }

  static chf() {
    return Currency.new('CHF', 'Swiss Franc', 'Fr', 2);
  }

  static cny() {
    return Currency.new('CNY', 'Chinese Yuan', '¥', 2);
  }

  static rub() {
    return Currency.new('RUB', 'Russian Ruble', '₽', 2);
  }

  static inr() {
    return Currency.new('INR', 'Indian Rupee', '₹', 2);
  }

  static clf() {
    return Currency.new('CLF', 'Chilean Unidad de Fomento', 'UF', 4);
  }

  static kzt() {
    return Currency.new('KZT', 'Kazakhstani Tenge', '₸', 2);
  }
}

/**
 * The string key of a `(from, to)` pair. Currency codes never contain U+0000.
 * @param {string} from
 * @param {string} to
 */
function pairKey(from, to) {
  return `${from}\u0000${to}`;
}

/**
 * The string key of a `(from, to, date)` triple.
 * @param {string} from
 * @param {string} to
 * @param {string} date
 */
function tripleKey(from, to, date) {
  return `${from}\u0000${to}\u0000${date}`;
}

/** A database of exchange rates, supporting historical data. */
export class CurrencyDatabase {
  /**
   * @param {{
   *   currencies: Map<string, Currency>,
   *   rates: Map<string, ExchangeRateInfo>,
   *   legacyRates: Map<string, number>,
   *   historicalRates: Map<string, ExchangeRateInfo>,
   *   lastUsedRates: UsedRate[],
   * }} fields
   */
  constructor({ currencies, rates, legacyRates, historicalRates, lastUsedRates }) {
    /** Known currencies. */
    this.currencies = currencies;
    /**
     * Exchange rates with full metadata: (from, to) -> rate info
     * Rate means: 1 unit of 'from' = rate units of 'to'
     * (keyed by `pairKey(from, to)`).
     */
    this.rates = rates;
    /** Legacy rates map for compatibility (will be deprecated) */
    this.legacyRates = legacyRates;
    /** Historical rates: (from, to, `date_string`) -> rate info (keyed by `tripleKey`). */
    this.historicalRates = historicalRates;
    /**
     * All rate infos used in the last conversion (for step display).
     * May contain multiple entries for cross-rate (triangulated) conversions.
     */
    this.lastUsedRates = lastUsedRates;
  }

  /** `#[derive(Default)]`: an empty database. */
  static default() {
    return new CurrencyDatabase({
      currencies: new Map(),
      rates: new Map(),
      legacyRates: new Map(),
      historicalRates: new Map(),
      lastUsedRates: [],
    });
  }

  /** Creates a new currency database with default currencies and rates. */
  static new() {
    return CurrencyDatabase.withBundledRates(BUNDLED_LATEST_RATES);
  }

  /**
   * Creates a currency database with the default currencies and rates and
   * the given table instead of the rates bundled at build time.
   *
   * Each entry is `(from, to, source, date, rate)`. The parity fixtures
   * (`tests/parity_goldens.rs`) pass a frozen snapshot, so the daily rate
   * updates of `data/currency/` do not change their results.
   * @param {ReadonlyArray<BundledRate>} bundled
   */
  static withBundledRates(bundled) {
    const db = CurrencyDatabase.default();
    db.initializeDefaultCurrencies();
    db.initializeDefaultRates();
    db.initializeBundledRates(bundled);
    return db;
  }

  /**
   * Adds the latest rates bundled from `data/currency/*.lino` at build time
   * (issue #224) for currencies that have no fallback rate at all.
   *
   * Without this, currencies such as HKD, AUD or SEK failed with
   * "No exchange rate available" until live API rates were loaded. Existing
   * hardcoded defaults are kept unchanged, and live API rates loaded later
   * override the bundled ones.
   * @param {ReadonlyArray<BundledRate>} bundled
   */
  initializeBundledRates(bundled) {
    for (const [from, to, source, date, rate] of bundled) {
      if (this.hasAnyRate(from) && this.hasAnyRate(to)) {
        continue;
      }
      const info = ExchangeRateInfo.new(rate, source, date);
      this.setRateWithInfo(from, to, info);
      this.setHistoricalRateWithInfo(from, to, date, info);
    }
  }

  /**
   * Returns `true` if any current rate involves `code`.
   * @param {string} code
   */
  hasAnyRate(code) {
    for (const key of this.rates.keys()) {
      const [from, to] = key.split('\u0000');
      if (from === code || to === code) {
        return true;
      }
    }
    return false;
  }

  initializeDefaultCurrencies() {
    const currencies = [
      Currency.usd(),
      Currency.eur(),
      Currency.gbp(),
      Currency.jpy(),
      Currency.chf(),
      Currency.cny(),
      Currency.rub(),
      Currency.inr(),
      Currency.clf(),
      Currency.kzt(),
    ];

    for (const currency of currencies) {
      this.currencies.set(currency.code, currency);
    }
  }

  initializeDefaultRates() {
    // Default fallback rates - these are used only when API rates are unavailable.
    // In production, rates should be fetched from the currency API.
    // Note: These are approximate rates and may be outdated.

    // USD base rates
    this.setRateWithInfo('USD', 'EUR', ExchangeRateInfo.defaultRate(0.92));
    this.setRateWithInfo('USD', 'GBP', ExchangeRateInfo.defaultRate(0.79));
    this.setRateWithInfo('USD', 'JPY', ExchangeRateInfo.defaultRate(148.5));
    this.setRateWithInfo('USD', 'CHF', ExchangeRateInfo.defaultRate(0.88));
    this.setRateWithInfo('USD', 'CNY', ExchangeRateInfo.defaultRate(7.25));
    this.setRateWithInfo('USD', 'RUB', ExchangeRateInfo.defaultRate(89.5));
    this.setRateWithInfo('USD', 'INR', ExchangeRateInfo.defaultRate(86.5));
    // CLF (Chilean Unidad de Fomento, also known as UF): 1 USD ≈ 0.022 CLF (1 CLF ≈ 45 USD)
    this.setRateWithInfo('USD', 'CLF', ExchangeRateInfo.defaultRate(0.022));

    // VND (Vietnamese Dong): CBR rate ~32.33 RUB per 10,000 VND → 1 USD ≈ 25,810 VND
    this.setRateWithInfo('USD', 'VND', ExchangeRateInfo.defaultRate(25_810.0));

    // KZT (Kazakhstani Tenge): ~470 KZT per USD (approximate, from CBR data)
    this.setRateWithInfo('USD', 'KZT', ExchangeRateInfo.defaultRate(470.0));

    // EUR base rates
    this.setRateWithInfo('EUR', 'USD', ExchangeRateInfo.defaultRate(1.087));
    this.setRateWithInfo('EUR', 'GBP', ExchangeRateInfo.defaultRate(0.86));
    this.setRateWithInfo('EUR', 'JPY', ExchangeRateInfo.defaultRate(161.5));
    this.setRateWithInfo('EUR', 'CHF', ExchangeRateInfo.defaultRate(0.96));

    // GBP base rates
    this.setRateWithInfo('GBP', 'USD', ExchangeRateInfo.defaultRate(1.27));
    this.setRateWithInfo('GBP', 'EUR', ExchangeRateInfo.defaultRate(1.16));

    // Add some historical rates for demonstration
    this.setHistoricalRateWithInfo(
      'USD',
      'EUR',
      '2026-01-22',
      ExchangeRateInfo.new(0.921, 'default (hardcoded)', '2026-01-22'),
    );
    this.setHistoricalRateWithInfo(
      'USD',
      'EUR',
      '2026-01-20',
      ExchangeRateInfo.new(0.918, 'default (hardcoded)', '2026-01-20'),
    );
    this.setHistoricalRateWithInfo(
      'USD',
      'EUR',
      '2026-01-15',
      ExchangeRateInfo.new(0.925, 'default (hardcoded)', '2026-01-15'),
    );
  }

  /**
   * Sets an exchange rate with full metadata.
   * @param {string} from
   * @param {string} to
   * @param {ExchangeRateInfo} info
   */
  setRateWithInfo(from, to, info) {
    const fromUpper = from.toUpperCase();
    const toUpper = to.toUpperCase();

    // Store the forward rate
    this.rates.set(pairKey(fromUpper, toUpper), info);

    // Also add the inverse rate
    if (info.rate !== 0.0) {
      const inverseInfo = new ExchangeRateInfo({
        rate: 1.0 / info.rate,
        source: info.source,
        date: info.date,
        fetchedAt: info.fetchedAt,
      });
      this.rates.set(pairKey(toUpper, fromUpper), inverseInfo);
    }
  }

  /**
   * Sets an exchange rate (legacy method for compatibility).
   * @param {string} from
   * @param {string} to
   * @param {number} rate
   */
  setRate(from, to, rate) {
    this.setRateWithInfo(from, to, ExchangeRateInfo.defaultRate(rate));
  }

  /**
   * Sets a historical exchange rate with full metadata.
   * @param {string} from
   * @param {string} to
   * @param {string} date
   * @param {ExchangeRateInfo} info
   */
  setHistoricalRateWithInfo(from, to, date, info) {
    const fromUpper = from.toUpperCase();
    const toUpper = to.toUpperCase();

    this.historicalRates.set(tripleKey(fromUpper, toUpper, date), info);

    // Also add the inverse rate
    if (info.rate !== 0.0) {
      const inverseInfo = new ExchangeRateInfo({
        rate: 1.0 / info.rate,
        source: info.source,
        date: info.date,
        fetchedAt: info.fetchedAt,
      });
      this.historicalRates.set(tripleKey(toUpper, fromUpper, date), inverseInfo);
    }
  }

  /**
   * Sets a historical exchange rate for a specific date (legacy method).
   * @param {string} from
   * @param {string} to
   * @param {string} date
   * @param {number} rate
   */
  setHistoricalRate(from, to, date, rate) {
    this.setHistoricalRateWithInfo(from, to, date, ExchangeRateInfo.new(rate, 'default (hardcoded)', date));
  }

  /**
   * Gets the current exchange rate info.
   * @param {string} from
   * @param {string} to
   * @returns {ExchangeRateInfo | null}
   */
  getRateInfo(from, to) {
    if (eqIgnoreAsciiCase(from, to)) {
      return null; // Same currency, no rate needed
    }
    return this.rates.get(pairKey(from.toUpperCase(), to.toUpperCase())) ?? null;
  }

  /**
   * Gets the current exchange rate.
   * @param {string} from
   * @param {string} to
   * @returns {number | null}
   */
  getRate(from, to) {
    if (eqIgnoreAsciiCase(from, to)) {
      return 1.0;
    }
    const info = this.rates.get(pairKey(from.toUpperCase(), to.toUpperCase()));
    return info === undefined ? null : info.rate;
  }

  /**
   * Gets all rate infos used in the last conversion (for display in calculation steps).
   * Returns multiple entries for cross-rate (triangulated) conversions.
   * @returns {ReadonlyArray<UsedRate>}
   */
  getLastUsedRates() {
    return this.lastUsedRates;
  }

  /** Clears the last used rate info. */
  clearLastUsedRate() {
    this.lastUsedRates = [];
  }

  /**
   * Gets a historical exchange rate for a specific date.
   * @param {string} from
   * @param {string} to
   * @param {import('./datetime.js').DateTime} date
   * @returns {number | null}
   */
  getHistoricalRate(from, to, date) {
    if (eqIgnoreAsciiCase(from, to)) {
      return 1.0;
    }

    const info = this.getHistoricalRateInfo(from, to, date);
    return info === null ? null : info.rate;
  }

  /**
   * @param {string} from
   * @param {string} to
   * @param {import('./datetime.js').DateTime} date
   * @returns {ExchangeRateInfo | null}
   */
  getHistoricalRateInfo(from, to, date) {
    const fromUpper = from.toUpperCase();
    const toUpper = to.toUpperCase();
    const dateStr = date.asChrono().format('%Y-%m-%d');

    const exact = this.historicalRates.get(tripleKey(fromUpper, toUpper, dateStr));
    if (exact !== undefined) {
      return exact;
    }

    // `filter_map(...).max_by_key(rate_date)`: the dates are distinct per pair.
    /** @type {string | null} */
    let bestDate = null;
    /** @type {ExchangeRateInfo | null} */
    let best = null;
    for (const [key, info] of this.historicalRates) {
      const [rateFrom, rateTo, rateDate] = key.split('\u0000');
      if (rateFrom === fromUpper && rateTo === toUpper && rateDate <= dateStr) {
        if (bestDate === null || rateDate >= bestDate) {
          bestDate = rateDate;
          best = info;
        }
      }
    }
    return best;
  }

  /**
   * Converts an amount from one currency to another, tracking the rate used.
   * Throws `Err(CalculatorError)`.
   * @param {number} amount
   * @param {string} from
   * @param {string} to
   * @returns {number}
   */
  convert(amount, from, to) {
    const fromUpper = from.toUpperCase();
    const toUpper = to.toUpperCase();

    if (fromUpper === toUpper) {
      this.lastUsedRates = [];
      return amount;
    }

    const info = this.rates.get(pairKey(fromUpper, toUpper));
    if (info !== undefined) {
      const result = amount * info.rate;
      this.lastUsedRates = [/** @type {UsedRate} */ ([fromUpper, toUpper, info])];
      return result;
    }

    // Try triangulation via USD as a bridge currency.
    // This handles cross-currency pairs like INR→RUB or RUB→INR
    // where no direct rate exists but both currencies have USD rates.
    if (fromUpper !== 'USD' && toUpper !== 'USD') {
      const fromUsdInfo = this.rates.get(pairKey(fromUpper, 'USD'));
      const usdToInfo = this.rates.get(pairKey('USD', toUpper));
      if (fromUsdInfo !== undefined && usdToInfo !== undefined) {
        const triangulatedRate = fromUsdInfo.rate * usdToInfo.rate;
        // Store both individual rate steps so callers can show each hop explicitly
        this.lastUsedRates = [
          /** @type {UsedRate} */ ([fromUpper, 'USD', fromUsdInfo]),
          /** @type {UsedRate} */ (['USD', toUpper, usdToInfo]),
        ];
        return amount * triangulatedRate;
      }
    }

    throw err(
      CalculatorError.CurrencyConversion({
        from: fromUpper,
        to: toUpper,
        reason: 'No exchange rate available',
      }),
    );
  }

  /**
   * Converts with a specific date for historical rates.
   * Throws `Err(CalculatorError)`.
   * @param {number} amount
   * @param {string} from
   * @param {string} to
   * @param {import('./datetime.js').DateTime} date
   * @returns {number}
   */
  convertAtDate(amount, from, to, date) {
    const fromUpper = from.toUpperCase();
    const toUpper = to.toUpperCase();

    if (fromUpper === toUpper) {
      this.lastUsedRates = [];
      return amount;
    }

    const info = this.getHistoricalRateInfo(fromUpper, toUpper, date);
    if (info !== null) {
      this.lastUsedRates = [/** @type {UsedRate} */ ([fromUpper, toUpper, info])];
      return amount * info.rate;
    }

    throw err(
      CalculatorError.NoHistoricalRate({
        currency: `${fromUpper}/${toUpper}`,
        date: date.toString(),
      }),
    );
  }

  /**
   * Checks if a currency code is known.
   * @param {string} code
   */
  isKnownCurrency(code) {
    return this.currencies.has(code.toUpperCase());
  }

  /**
   * Gets currency metadata.
   * @param {string} code
   * @returns {Currency | null}
   */
  getCurrency(code) {
    return this.currencies.get(code.toUpperCase()) ?? null;
  }

  /**
   * Returns all supported currency codes (in insertion order; Rust's
   * `HashMap` order is unspecified).
   * @returns {string[]}
   */
  supportedCurrencies() {
    return [...this.currencies.keys()];
  }

  /**
   * Parses a currency code from a string.
   *
   * Handles:
   * - Standard ISO 4217 fiat currency codes (USD, EUR, GBP, etc.)
   * - Common fiat currency symbols ($, €, £, ¥)
   * - Natural language fiat names (dollars, euros, pounds, yen)
   * - Cryptocurrency codes and natural language names (TON, BTC, ETH, bitcoin, etc.)
   * @param {string} input
   * @returns {string | null}
   */
  static parseCurrency(input) {
    input = trim(input).toUpperCase();
    // Fiat currency codes and symbols
    switch (input) {
      case 'USD': case 'US$': case '$':
        return 'USD';
      case 'EUR': case '€':
        return 'EUR';
      case 'GBP': case '£':
        return 'GBP';
      case 'JPY': case '¥':
        return 'JPY';
      case 'CHF': case 'FR':
        return 'CHF';
      case 'CNY': case 'RMB':
        return 'CNY';
      case 'RUB': case '₽':
        return 'RUB';
      case 'VND': case '₫':
        return 'VND';
      case 'INR': case '₹':
        return 'INR';
      case 'KRW': case '₩':
        return 'KRW';
      case 'KZT': case '₸':
        return 'KZT';
      // CLF is the ISO 4217 code; UF is the widely used Chilean abbreviation
      case 'CLF': case 'UF':
        return 'CLF';
      case 'BTC': case '₿':
        return 'BTC';
      default:
    }

    // Natural language fiat currency names (lowercase comparison)
    const inputLower = input.toLowerCase();
    switch (inputLower) {
      case 'dollar': case 'dollars': case 'usdollar': case 'usdollars': case 'us dollar': case 'us dollars':
        return 'USD';
      case 'euro': case 'euros':
        return 'EUR';
      case 'pound': case 'pounds': case 'sterling': case 'british pound': case 'british pounds':
        return 'GBP';
      case 'yen': case 'japanese yen':
        return 'JPY';
      case 'franc': case 'francs': case 'swiss franc': case 'swiss francs':
        return 'CHF';
      case 'yuan': case 'renminbi': case 'chinese yuan':
        return 'CNY';
      case 'ruble': case 'rubles': case 'rouble': case 'roubles':
        return 'RUB';
      // Russian language names for RUB (all grammatical cases/forms)
      // Nominative: рубль, рубли; Genitive: рубля, рублей;
      // Dative: рублю, рублям; Instrumental: рублём/рублем, рублями;
      // Prepositional: рубле, рублях
      case 'рубль': case 'рубля': case 'рубле': case 'рубли': case 'рублей': case 'рублям': case 'рублю':
      case 'рублём': case 'рублем': case 'рублями': case 'рублях':
        return 'RUB';
      // Common Russian abbreviations for RUB. The trailing dot of "руб." / "рубл."
      // is stripped by the lexer, so only the dotless forms are listed here.
      // A bare "р" is intentionally not accepted: single Latin/Cyrillic letters are
      // used as variable names in equations.
      case 'руб': case 'рубл':
        return 'RUB';
      // Russian language names for VND (Vietnamese Dong, all grammatical cases/forms)
      // Nominative: донг, донги; Genitive: донга, донгов;
      // Dative: донгу, донгам; Instrumental: донгом, донгами;
      // Prepositional: донге, донгах
      case 'донг': case 'донга': case 'донге': case 'донги': case 'донгов': case 'донгам': case 'донгу':
      case 'донгом': case 'донгами': case 'донгах':
        return 'VND';
      // English and Vietnamese names for VND
      case 'dong': case 'dongs': case 'đồng':
        return 'VND';
      // Russian language names for USD (all grammatical cases/forms)
      // Nominative: доллар, доллары; Genitive: доллара, долларов;
      // Dative: доллару, долларам; Accusative: доллар, доллары;
      // Instrumental: долларом, долларами; Prepositional: долларе, долларах
      case 'доллар': case 'доллара': case 'долларе': case 'доллары': case 'долларов': case 'долларам':
      case 'доллару': case 'долларом': case 'долларами': case 'долларах':
        return 'USD';
      // Russian language names for EUR (all grammatical cases/forms)
      // Nominative: евро (indeclinable in Russian)
      case 'евро':
        return 'EUR';
      // Russian language names for GBP (all grammatical cases/forms)
      // фунт (pound), фунт стерлингов (pound sterling)
      case 'фунт': case 'фунта': case 'фунте': case 'фунты': case 'фунтов': case 'фунтам': case 'фунту':
      case 'фунтом': case 'фунтами': case 'фунтах':
        return 'GBP';
      // Russian language names for CNY (all grammatical cases/forms)
      // юань (yuan)
      case 'юань': case 'юаня': case 'юане': case 'юани': case 'юаней': case 'юаням': case 'юаню':
      case 'юанем': case 'юанями': case 'юанях':
        return 'CNY';
      // Russian language names for JPY (all grammatical cases/forms)
      // иена (yen)
      case 'иена': case 'иены': case 'иене': case 'иену': case 'иеной': case 'иенами': case 'иенах':
      case 'иен':
        return 'JPY';
      // Russian language names for INR (Indian Rupee, all grammatical cases/forms)
      case 'рупия': case 'рупии': case 'рупий': case 'рупию': case 'рупией': case 'рупиях':
        return 'INR';
      // English names for INR
      case 'rupee': case 'rupees': case 'indian rupee': case 'indian rupees':
        return 'INR';
      // CLF: Chilean Unidad de Fomento (also known as UF)
      case 'unidad de fomento': case 'unidad fomento': case 'fomento':
        return 'CLF';
      // German language names for currencies
      // USD: Dollar (der Dollar) - "in" preposition identical to English
      case 'us-dollar':
        return 'USD';
      // EUR: Euro - same as English, already handled above
      // GBP: Pfund (das Pfund) - invariant with numbers; "pfund sterling" formal
      case 'pfund': case 'pfund sterling': case 'britisches pfund':
        return 'GBP';
      // JPY: Yen - same as English, already handled above
      // CHF: Franken (der Franken) - German uses "Franken" not "Franc"
      case 'franken': case 'schweizer franken':
        return 'CHF';
      // CNY: Yuan - same as English; "renminbi" already handled above; "chinesischer yuan" added
      case 'chinesischer yuan':
        return 'CNY';
      // RUB: Rubel (der Rubel) - German spelling; "rubeln" dative plural
      case 'rubel': case 'rubeln': case 'russischer rubel':
        return 'RUB';
      // INR: Rupie (die Rupie) - German; plural "rupien"
      case 'rupie': case 'rupien': case 'indische rupie':
        return 'INR';
      // French language names for currencies ("en" preposition added to lexer)
      // USD: dollar - singular "dollar" already handled; add French-specific forms
      case 'dollar américain': case 'dollars américains': case 'dollar americain': case 'dollars americains':
        return 'USD';
      // EUR: euro/euros - already handled above
      // GBP: livre sterling (la livre sterling) - "sterling" invariable
      case 'livre sterling': case 'livres sterling': case 'livres sterlings': case 'livre': case 'livres':
        return 'GBP';
      // JPY: yen/yens - "yen" already handled; add "yens" and French full form
      case 'yens': case 'yen japonais': case 'yens japonais':
        return 'JPY';
      // CHF: franc suisse - "franc"/"francs" already handled; add "franc suisse" forms
      case 'franc suisse': case 'francs suisses':
        return 'CHF';
      // CNY: yuan/yuans - "yuan" already handled; add French plural and full forms
      case 'yuans': case 'yuan chinois': case 'yuans chinois':
        return 'CNY';
      // RUB: French "rouble russe" / "roubles russes" (English "ruble"/"rubles" and base
      // "rouble"/"roubles" already handled in the English section above)
      case 'rouble russe': case 'roubles russes':
        return 'RUB';
      // INR: roupie/roupies (French spelling differs from English "rupee")
      case 'roupie': case 'roupies': case 'roupie indienne': case 'roupies indiennes':
        return 'INR';
      // Chinese (Simplified) language names for currencies ("换成"/"兑换成" etc. added to lexer)
      // USD: 美元 (měi yuán) standard; 美金 (měi jīn) colloquial
      case '美元': case '美金':
        return 'USD';
      // EUR: 欧元 (ōu yuán)
      case '欧元':
        return 'EUR';
      // GBP: 英镑 (yīng bàng)
      case '英镑':
        return 'GBP';
      // JPY: 日元 (rì yuán) simplified; 日圆 (rì yuán) traditional
      case '日元': case '日圆':
        return 'JPY';
      // CHF: 瑞士法郎 (ruì shì fǎ láng); 法郎 short form
      case '瑞士法郎': case '法郎':
        return 'CHF';
      // CNY: 人民币 (rén mín bì) official name; 元 (yuán) unit; 块 (kuài) colloquial
      case '人民币': case '元': case '块':
        return 'CNY';
      // RUB: 卢布 (lú bù)
      case '卢布':
        return 'RUB';
      // INR: 卢比 (lú bǐ) short; 印度卢比 full form
      case '卢比': case '印度卢比':
        return 'INR';
      // Hindi language names for currencies ("में" postposition added to lexer)
      // USD: डॉलर (ḍŏlar); अमेरिकी डॉलर full form
      case 'डॉलर': case 'अमेरिकी डॉलर':
        return 'USD';
      // EUR: यूरो (yūro) - invariable
      case 'यूरो':
        return 'EUR';
      // GBP: पाउंड (pā'uṃḍ); ब्रिटिश पाउंड; पाउंड स्टर्लिंग
      case 'पाउंड': case 'ब्रिटिश पाउंड': case 'पाउंड स्टर्लिंग':
        return 'GBP';
      // JPY: येन (yen); जापानी येन full form
      case 'येन': case 'जापानी येन':
        return 'JPY';
      // CHF: फ्रैंक (frank); स्विस फ्रैंक full form
      case 'फ्रैंक': case 'स्विस फ्रैंक':
        return 'CHF';
      // CNY: युआन (yuan); चीनी युआन full form; रेनमिनबी alt.
      case 'युआन': case 'चीनी युआन': case 'रेनमिनबी':
        return 'CNY';
      // RUB: रूबल (rūbal); रूसी रूबल; रशियन रूबल
      case 'रूबल': case 'रूसी रूबल': case 'रशियन रूबल':
        return 'RUB';
      // INR: रुपया (rupayā) singular; रुपये (rupaye) plural/oblique; रुपयों oblique plural; भारतीय रुपया full
      case 'रुपया': case 'रुपये': case 'रुपयों': case 'भारतीय रुपया':
        return 'INR';
      // Arabic language names for currencies ("إلى" preposition added to lexer)
      // USD: دولار (dūlār); دولارات plural; دولار أمريكي full
      case 'دولار': case 'دولارات': case 'دولار أمريكي': case 'دولارات أمريكية':
        return 'USD';
      // EUR: يورو (yūrū) - invariable loanword
      case 'يورو':
        return 'EUR';
      // GBP: جنيه إسترليني; جنيهات إسترلينية plural; جنيه short; جنيهات short plural
      case 'جنيه إسترليني': case 'جنيهات إسترلينية': case 'جنيه': case 'جنيهات':
        return 'GBP';
      // JPY: ين ياباني; ين short
      case 'ين ياباني': case 'ين':
        return 'JPY';
      // CHF: فرنك سويسري; فرنكات سويسرية plural; فرنك short
      case 'فرنك سويسري': case 'فرنكات سويسرية': case 'فرنك':
        return 'CHF';
      // CNY: يوان صيني; يوان short; رنمينبي alt.
      case 'يوان صيني': case 'يوان': case 'رنمينبي':
        return 'CNY';
      // RUB: روبل روسي; روبلات روسية plural; روبل short
      case 'روبل روسي': case 'روبلات روسية': case 'روبل':
        return 'RUB';
      // INR: روبية هندية; روبيات هندية plural; روبية short; روبيات short plural
      case 'روبية هندية': case 'روبيات هندية': case 'روبية': case 'روبيات':
        return 'INR';
      // Russian and Kazakh language names for KZT (Kazakhstani Tenge)
      // "тенге" is invariable in Russian (same form for all grammatical cases).
      // "теңге" is the Kazakh spelling (with ң, a Kazakh letter).
      case 'тенге': case 'теңге':
        return 'KZT';
      // English names for KZT
      case 'tenge': case 'tenges': case 'kazakhstani tenge':
        return 'KZT';
      // Cryptocurrency natural language names and common aliases
      case 'toncoin': case 'the open network':
        return 'TON';
      case 'bitcoin':
        return 'BTC';
      case 'ethereum': case 'ether':
        return 'ETH';
      case 'litecoin':
        return 'LTC';
      case 'dogecoin': case 'doge':
        return 'DOGE';
      case 'solana': case 'sol':
        return 'SOL';
      case 'ripple': case 'xrp':
        return 'XRP';
      case 'cardano': case 'ada':
        return 'ADA';
      case 'polkadot': case 'dot':
        return 'DOT';
      case 'chainlink': case 'link':
        return 'LINK';
      case 'uniswap': case 'uni':
        return 'UNI';
      case 'binancecoin': case 'bnb': case 'binance coin':
        return 'BNB';
      
      default:
    }

    // Standard 3-letter alphabetic codes (handles most crypto tickers: BTC, ETH, TON, etc.)
    if (utf8Len(input) === 3 && [...input].every(isAsciiAlphabetic)) {
      return input;
    }

    // Allow 2-5 letter crypto tickers (e.g., BNB=3, DOGE=4, MATIC=5, SOL=3)
    // Only accept if all characters are ASCII alphabetic
    if (utf8Len(input) >= 2 && utf8Len(input) <= 5 && [...input].every(isAsciiAlphabetic)) {
      return input;
    }

    return null;
  }
}
