/**
 * Calculation planning — determines data requirements without executing
 * (port of `src/plan.rs`).
 *
 * The plan→execute pipeline:
 * 1. `Calculator::plan(expr)` parses the AST and returns a `CalculationPlan`
 *    with required rate sources, LINO interpretation, and alternatives.
 * 2. The worker fetches only the required rate sources.
 * 3. `Calculator::execute(expr)` runs the calculation with rates available.
 */

import { coingeckoId } from "./crypto_api.js";
import { compareStrings } from "./rust/core.js";

/**
 * Rate sources that the calculator can fetch data from.
 *
 * These correspond to the three independent APIs the web worker integrates with:
 * - `ecb` — Fiat currency rates via Frankfurter API (European Central Bank data)
 * - `cbr` — RUB-based rates from the Central Bank of Russia
 * - `crypto` — Cryptocurrency rates via CoinGecko
 */
export class RateSource {
  /** @param {'Ecb' | 'Cbr' | 'Crypto'} tag */
  constructor(tag) {
    this.tag = tag;
    Object.freeze(this);
  }

  /** European Central Bank (fiat currencies: USD, EUR, GBP, JPY, …). */
  static Ecb = new RateSource("Ecb");
  /** Central Bank of Russia (RUB-based rates). */
  static Cbr = new RateSource("Cbr");
  /** CoinGecko (cryptocurrencies: BTC, ETH, TON, …). */
  static Crypto = new RateSource("Crypto");

  /**
   * `#[serde(rename_all = "lowercase")]`.
   * @returns {string}
   */
  toJSON() {
    return this.tag.toLowerCase();
  }
}

/**
 * A calculation plan produced by `Calculator::plan()`.
 *
 * Contains everything the worker needs to know *before* executing the
 * calculation: which rate sources to fetch, how the expression was
 * interpreted, and what alternatives exist.
 */
export class CalculationPlan {
  /**
   * @param {{ expression: string, linoInterpretation: string, alternativeLino: string[] | null,
   *   requiredSources: RateSource[], currencies: string[], isLiveTime: boolean, success: boolean,
   *   error: string | null }} fields
   */
  constructor({
    expression,
    linoInterpretation,
    alternativeLino,
    requiredSources,
    currencies,
    isLiveTime,
    success,
    error,
  }) {
    /** The input expression, trimmed. */
    this.expression = expression;
    /** Links notation interpretation of the expression (default interpretation). */
    this.linoInterpretation = linoInterpretation;
    /**
     * Alternative links notation interpretations, if any.
     * The first element is always the default (same as `linoInterpretation`).
     */
    this.alternativeLino = alternativeLino;
    /**
     * Rate sources that must be loaded before this expression can be executed.
     * Empty for pure math expressions.
     */
    this.requiredSources = requiredSources;
    /** Currency codes found in the expression (e.g., `["USD", "RUB", "TON"]`). */
    this.currencies = currencies;
    /** Whether the expression contains a live time reference (auto-refresh needed). */
    this.isLiveTime = isLiveTime;
    /** Whether the expression was parsed successfully. */
    this.success = success;
    /** Error message if parsing failed. */
    this.error = error;
  }

  /**
   * serde's JSON (field names and order, `skip_serializing_if = "Option::is_none"`).
   * @returns {Record<string, unknown>}
   */
  toJSON() {
    /** @type {Record<string, unknown>} */
    const json = {
      expression: this.expression,
      lino_interpretation: this.linoInterpretation,
    };
    if (this.alternativeLino !== null) {
      json.alternative_lino = this.alternativeLino;
    }
    json.required_sources = this.requiredSources.map((s) => s.toJSON());
    json.currencies = this.currencies;
    json.is_live_time = this.isLiveTime;
    json.success = this.success;
    if (this.error !== null) {
      json.error = this.error;
    }
    return json;
  }
}

/**
 * Maps a currency code to the rate source that is its primary provider.
 * @param {string} code
 * @returns {RateSource}
 */
function primarySource(code) {
  const upper = code.toUpperCase();

  // Cryptocurrencies are provided by CoinGecko
  if (coingeckoId(upper) !== null) {
    return RateSource.Crypto;
  }

  // RUB is provided by CBR
  if (upper === "RUB") {
    return RateSource.Cbr;
  }

  // Currencies available from CBR but not from ECB/Frankfurter
  if (CBR_ONLY.has(upper)) {
    return RateSource.Cbr;
  }

  // All other fiat currencies are provided by ECB/Frankfurter
  return RateSource.Ecb;
}

/** The currencies of the `matches!` in `primary_source`. */
const CBR_ONLY = new Set([
  "VND",
  "KZT",
  "AZN",
  "DZD",
  "AMD",
  "BHD",
  "BYN",
  "BOB",
  "GEL",
  "AED",
  "EGP",
  "IRR",
  "KGS",
  "CUP",
  "MDL",
  "MNT",
  "NGN",
  "OMR",
  "QAR",
  "SAR",
  "TJS",
  "BDT",
  "TMT",
  "UZS",
  "UAH",
  "ETB",
  "RSD",
  "MMK",
]);

/** The currencies of the `matches!` of `cbr_fiat` in `can_also_serve`. */
const CBR_FIAT = new Set([
  "EUR",
  "GBP",
  "JPY",
  "CHF",
  "CNY",
  "INR",
  "AUD",
  "CAD",
  "SGD",
  "HKD",
  "DKK",
  "SEK",
  "NOK",
  "CZK",
  "PLN",
  "HUF",
  "RON",
  "BGN",
  "TRY",
  "KRW",
  "BRL",
  "MXN",
  "ZAR",
  "IDR",
  "MYR",
  "PHP",
  "THB",
  ...CBR_ONLY,
]);

/**
 * Returns true if a source can also provide rates for a currency, even though
 * it is not the primary source for that currency.
 *
 * This captures the real-world overlap between rate providers. The key
 * constraint is the runtime converter, which only triangulates through USD
 * as a bridge currency. A source can "also serve" a currency only when the
 * converter can actually use the rates it provides for that currency.
 *
 * Coverage:
 * - CBR publishes rates for ~50 currencies against RUB, including USD.
 *   Since the converter bridges through USD, CBR's USD↔RUB rate lets it
 *   serve USD for any RUB-involving conversion. CBR also serves other fiat
 *   currencies (EUR, GBP, etc.) for direct conversion to/from RUB.
 * - CoinGecko crypto rates are denominated in USD, so Crypto covers USD.
 * - ECB provides rates for major fiat currencies relative to EUR/USD.
 * @param {RateSource} source
 * @param {string} code
 * @param {string[]} allCurrencies
 * @returns {boolean}
 */
function canAlsoServe(source, code, allCurrencies) {
  const upper = code.toUpperCase();
  switch (source.tag) {
    // CBR provides X↔RUB rates for many currencies. It can serve a fiat
    // currency if the expression involves RUB (which it must, since CBR
    // is only in the set when RUB is present). For direct RUB↔X conversion,
    // CBR has the rate. For USD specifically, CBR provides USD↔RUB which
    // the converter can use directly.
    //
    // However, CBR cannot bridge between two non-RUB fiat currencies
    // (e.g., GBP→EUR) because the converter triangulates through USD,
    // not RUB. So CBR can only serve a non-RUB fiat currency when it's
    // being converted directly to/from RUB — which happens when all
    // other currencies in the expression are either RUB or served by
    // another source.
    case "Cbr": {
      // CBR always covers USD (via USD↔RUB rate)
      if (upper === "USD") {
        return true;
      }
      // CBR can serve other fiat currencies for direct RUB conversion,
      // but only when all fiat currencies in the expression either are
      // RUB or are the single target fiat currency (so conversion is
      // always fiat↔RUB, never fiat↔fiat through CBR).
      const cbrFiat = CBR_FIAT.has(upper);
      if (!cbrFiat) {
        return false;
      }
      // Count how many distinct ECB-primary fiat currencies are in the expression.
      // If there's only one (the one we're checking), CBR can handle the
      // direct RUB↔fiat conversion. If there are multiple, CBR can't
      // convert between them (e.g., GBP↔EUR requires ECB).
      const ecbFiatCount = allCurrencies.filter((c) => {
        const u = c.toUpperCase();
        return u !== "RUB" && coingeckoId(u) === null && u !== "USD";
      }).length;
      return ecbFiatCount <= 1;
    }
    // CoinGecko rates are denominated in USD, so if Crypto is already
    // required, USD conversion is available without ECB.
    case "Crypto":
      return upper === "USD";
    // ECB provides rates for major fiat currencies (but not RUB or crypto).
    case "Ecb":
      return upper !== "RUB" && coingeckoId(upper) === null;
  }
}

/**
 * Creates a `CalculationPlan` from a parsed expression.
 *
 * Walks the AST to collect currencies, maps them to rate sources,
 * generates LINO interpretation, and detects live time references.
 * @param {string} input
 * @param {import('./types/expression.js').Expression} expr
 * @returns {CalculationPlan}
 */
export function createPlan(input, expr) {
  const lino = expr.toLino();
  const alternatives = expr.alternativeLino();
  const isLiveTime = expr.containsLiveTime() || expr.evaluatesToDatetime();

  // Collect currencies from the AST and map to rate sources
  const currenciesSet = expr.collectCurrencies();
  const currencies = [...currenciesSet];
  currencies.sort(compareStrings);

  // Map each currency to its primary source
  /** @type {Set<RateSource>} */
  const sourcesSet = new Set();
  /** @type {Map<RateSource, string[]>} */
  const sourceCurrencies = new Map();
  for (const code of currencies) {
    const source = primarySource(code);
    sourcesSet.add(source);
    let codes = sourceCurrencies.get(source);
    if (codes === undefined) {
      codes = [];
      sourceCurrencies.set(source, codes);
    }
    codes.push(code.toUpperCase());
  }

  // Optimization: remove a source if every currency that triggered its
  // inclusion is also covered by another source already in the set.
  // This generalizes for any target currency — if the expression converts
  // to a final currency, the remaining sources must form a path to it.
  // (Rust iterates a `HashSet` here; the result does not depend on the order,
  // because `other_sources` comes from the fixed `candidates`.)
  const candidates = [...sourcesSet];
  for (const source of candidates) {
    const codes = sourceCurrencies.get(source);
    if (codes !== undefined) {
      const otherSources = candidates.filter((s) => s !== source);
      // Check if every currency for this source is covered by some other source
      const allCovered = codes.every((code) =>
        otherSources.some((other) => canAlsoServe(other, code, currencies)),
      );
      if (allCovered && otherSources.length !== 0) {
        sourcesSet.delete(source);
      }
    }
  }

  const order = { Ecb: 0, Cbr: 1, Crypto: 2 };
  const requiredSources = [...sourcesSet];
  requiredSources.sort((a, b) => order[a.tag] - order[b.tag]);

  return new CalculationPlan({
    expression: input,
    linoInterpretation: lino,
    alternativeLino: alternatives,
    requiredSources,
    currencies,
    isLiveTime,
    success: true,
    error: null,
  });
}

/**
 * Creates a failed `CalculationPlan` for an empty input.
 * @returns {CalculationPlan}
 */
export function emptyPlan() {
  return new CalculationPlan({
    expression: "",
    linoInterpretation: "",
    alternativeLino: null,
    requiredSources: [],
    currencies: [],
    isLiveTime: false,
    success: false,
    error: "Empty input",
  });
}

/**
 * Creates a failed `CalculationPlan` for a parse error.
 * @param {string} input
 * @param {string} error
 * @returns {CalculationPlan}
 */
export function errorPlan(input, error) {
  return new CalculationPlan({
    expression: input,
    linoInterpretation: "",
    alternativeLino: null,
    requiredSources: [],
    currencies: [],
    isLiveTime: false,
    success: false,
    error,
  });
}
