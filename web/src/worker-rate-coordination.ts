// On-demand rate coordination for the web worker.
//
// Instead of eagerly loading all rate sources at startup and blocking until
// all finish, this module detects which rate sources a given expression needs
// and loads only those. Calculations wait only for the sources they require.
//
// Rate sources:
//   ECB  — fiat currencies via Frankfurter API (USD, EUR, GBP, …)
//   CBR  — RUB-based rates from Central Bank of Russia
//   Crypto — cryptocurrency rates via CoinGecko (BTC, ETH, TON, …)

/** The three independent rate sources the calculator supports. */
export type RateSource = 'ecb' | 'cbr' | 'crypto';

// ---------------------------------------------------------------------------
// Expression → required rate sources detection
// ---------------------------------------------------------------------------

// Crypto tickers supported by our CoinGecko integration (must stay in sync
// with the hardcoded list in src/crypto_api.rs).
const CRYPTO_TICKERS = [
  'TON', 'BTC', 'ETH', 'BNB', 'SOL', 'XRP',
  'ADA', 'DOGE', 'DOT', 'LTC', 'LINK', 'UNI',
];

// Natural-language crypto names (case-insensitive matching).
const CRYPTO_NAMES = [
  'bitcoin', 'ethereum', 'ether', 'toncoin',
  'solana', 'ripple', 'cardano', 'dogecoin',
  'polkadot', 'litecoin', 'chainlink', 'uniswap',
  'binancecoin', 'binance coin',
];

// RUB-related tokens that signal we need CBR rates.
// Includes Russian-language currency names in all grammatical cases.
const RUB_TOKENS = [
  'RUB', '₽',
  'рубль', 'рубля', 'рубли', 'рублей', 'рублями', 'рублях',
  'руб',
];

// Fiat currency codes/symbols/names that signal ECB rates.
// We only need a representative subset — the parser accepts any 2-5 letter
// code, but we only detect known fiat indicators to trigger ECB loading.
const FIAT_TOKENS = [
  // Codes & symbols
  'USD', 'EUR', 'GBP', 'JPY', 'CHF', 'CNY', 'INR', 'KRW', 'CLF',
  'AUD', 'CAD', 'NZD', 'SGD', 'HKD', 'MXN', 'BRL', 'ZAR',
  'SEK', 'NOK', 'DKK', 'CZK', 'HUF', 'PLN', 'RON', 'BGN',
  'TRY', 'IDR', 'MYR', 'PHP', 'THB', 'ISK', 'ILS',
  '$', '€', '£', '¥', '₹', '₩',
  // English names
  'dollar', 'dollars', 'euro', 'euros', 'pound', 'pounds', 'sterling',
  'yen', 'franc', 'francs', 'yuan', 'rupee', 'rupees',
  // Russian fiat names (not RUB)
  'доллар', 'долларов', 'долларами', 'евро', 'фунт', 'фунтов',
  'юань', 'иена', 'иен', 'франк', 'франков', 'рупия', 'рупий',
  // German
  'franken',
  // Chinese
  '美元', '美金', '欧元', '英镑', '日元', '瑞士法郎', '人民币', '卢比',
  // Hindi
  'डॉलर', 'यूरो', 'पाउंड', 'येन', 'फ्रैंक', 'युआन', 'रुपया', 'रुपये',
  // Arabic
  'دولار', 'يورو', 'جنيه', 'ين', 'فرنك', 'يوان', 'روبية',
];

/**
 * Detect which rate sources an expression requires by scanning for currency
 * keywords. This is a fast, conservative heuristic — it may return sources
 * that turn out to be unnecessary, but never misses a required source.
 *
 * Returns an empty set for expressions with no currency references (pure math).
 */
export function detectRequiredSources(expression: string): Set<RateSource> {
  const sources = new Set<RateSource>();
  const upper = expression.toUpperCase();

  // Check crypto tickers (case-insensitive via uppercased expression)
  for (const ticker of CRYPTO_TICKERS) {
    if (upper.includes(ticker)) {
      sources.add('crypto');
      break;
    }
  }

  // Check crypto natural names (case-insensitive via lowercased expression)
  if (!sources.has('crypto')) {
    const lower = expression.toLowerCase();
    for (const name of CRYPTO_NAMES) {
      if (lower.includes(name)) {
        sources.add('crypto');
        break;
      }
    }
  }

  // Check RUB tokens (case-sensitive for Cyrillic, case-insensitive for code)
  for (const token of RUB_TOKENS) {
    if (token === 'RUB' || token === '₽') {
      if (upper.includes(token)) {
        sources.add('cbr');
        break;
      }
    } else {
      // Cyrillic tokens — case-sensitive match in original expression
      if (expression.includes(token)) {
        sources.add('cbr');
        break;
      }
    }
  }

  // Check fiat tokens
  for (const token of FIAT_TOKENS) {
    // Single-char symbols and multi-byte Unicode — check original expression
    if (token.length <= 2 && !/[A-Z]/.test(token)) {
      if (expression.includes(token)) {
        sources.add('ecb');
        break;
      }
    } else if (/^[A-Z]+$/.test(token)) {
      // Uppercase codes — check uppercased expression
      if (upper.includes(token)) {
        sources.add('ecb');
        break;
      }
    } else {
      // Natural-language names — check original (handles Cyrillic, CJK, etc.)
      if (expression.includes(token) || expression.toLowerCase().includes(token)) {
        sources.add('ecb');
        break;
      }
    }
  }

  // If any currency conversion is happening, we likely need ECB as a base
  // for triangulation (e.g., TON→USD needs crypto + ecb for cross-rates).
  // If CBR is needed (RUB), ECB may also help for triangulation.
  // However, we keep it minimal: only add ECB if fiat tokens are found.

  return sources;
}

// ---------------------------------------------------------------------------
// Rate source state management
// ---------------------------------------------------------------------------

export type SourceState = 'idle' | 'loading' | 'loaded';

export interface RateCoordination {
  /** Detect required sources for the expression and wait for them to be loaded. */
  ensureRatesForExpression: (expression: string) => Promise<Set<RateSource>>;
  /** Mark a source as loaded. Called from fetch callbacks. */
  markLoaded: (source: RateSource) => void;
  /** Mark a source as loading. Called when fetch starts. */
  markLoading: (source: RateSource) => void;
  /** Get the current state of a source. */
  getState: (source: RateSource) => SourceState;
  /** Register a fetch function for a source. */
  registerFetcher: (source: RateSource, fetcher: () => Promise<void>) => void;
}

export function createRateCoordination(): RateCoordination {
  const states: Record<RateSource, SourceState> = {
    ecb: 'idle',
    cbr: 'idle',
    crypto: 'idle',
  };

  // Promises that resolve when the source finishes loading.
  // Created lazily when a source starts loading.
  const loadPromises: Partial<Record<RateSource, Promise<void>>> = {};
  const resolvers: Partial<Record<RateSource, () => void>> = {};

  // Fetcher functions registered by the worker for each source.
  const fetchers: Partial<Record<RateSource, () => Promise<void>>> = {};

  function getOrCreatePromise(source: RateSource): Promise<void> {
    if (states[source] === 'loaded') {
      return Promise.resolve();
    }
    if (!loadPromises[source]) {
      loadPromises[source] = new Promise<void>((resolve) => {
        resolvers[source] = resolve;
      });
    }
    return loadPromises[source]!;
  }

  function markLoaded(source: RateSource): void {
    states[source] = 'loaded';
    if (resolvers[source]) {
      resolvers[source]!();
      // Clean up — future calls go through the fast 'loaded' check
      delete resolvers[source];
      delete loadPromises[source];
    }
  }

  function markLoading(source: RateSource): void {
    if (states[source] !== 'loaded') {
      states[source] = 'loading';
      // Ensure a promise exists for waiters
      getOrCreatePromise(source);
    }
  }

  function registerFetcher(source: RateSource, fetcher: () => Promise<void>): void {
    fetchers[source] = fetcher;
  }

  async function ensureRatesForExpression(expression: string): Promise<Set<RateSource>> {
    const needed = detectRequiredSources(expression);

    if (needed.size === 0) {
      return needed;
    }

    const waitFor: Promise<void>[] = [];

    for (const source of needed) {
      if (states[source] === 'loaded') {
        continue; // Already available — no delay
      }

      if (states[source] === 'idle') {
        // Not yet started — trigger the fetch
        const fetcher = fetchers[source];
        if (fetcher) {
          markLoading(source);
          fetcher(); // Fire and forget — markLoaded is called in the finally block
        }
      }

      // Wait for it (whether we just started it or it was already in-flight)
      waitFor.push(getOrCreatePromise(source));
    }

    if (waitFor.length > 0) {
      await Promise.all(waitFor);
    }

    return needed;
  }

  return {
    ensureRatesForExpression,
    markLoaded,
    markLoading,
    getState: (source) => states[source],
    registerFetcher,
  };
}
