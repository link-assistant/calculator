// Web Worker for running WASM calculations in the background
// This prevents blocking the main UI thread

import init, { Calculator, fetch_exchange_rates, fetch_crypto_rates, fetch_cbr_rates, parse_consolidated_lino_rates } from '@wasm/link_calculator';
import {
  ECB_CACHE_TTL_MS, CBR_CACHE_TTL_MS, CRYPTO_CACHE_TTL_MS,
  RATE_CACHE_KEY_ECB, RATE_CACHE_KEY_CBR, RATE_CACHE_KEY_CRYPTO,
  isMissingRatesError, loadCachedRate, saveCachedRate,
  type RateCacheEntry, type CbrRateCacheEntry,
} from './worker-utils';

interface CalculatorInstance {
  calculate(input: string): string;
  update_rates_from_api(base: string, date: string, rates_json: string): number;
  update_crypto_rates_from_api(base: string, date: string, rates_json: string): number;
  update_cbr_rates_from_api(date: string, rates_json: string): number;
}

interface CalculatorStatic {
  new (): CalculatorInstance;
  version(): string;
}

interface ExchangeRatesResponse {
  success: boolean;
  date: string;
  base: string;
  error?: string;
  rates_json: string;
}

interface CryptoRatesResponse {
  success: boolean;
  date: string;
  base: string;
  error?: string;
  rates_json: string;
}

let calculator: CalculatorInstance | null = null;
let ratesLoaded = false;
let ratesLoading = false;
let ratesError: string | null = null;
let cryptoRatesError: string | null = null;

// Track per-source loading status for the wait-for-rates mechanism
let ecbLoaded = false;
let ecbDone = false;   // true when fetch completed (success or failure)
let cbrDone = false;
let cryptoDone = false;

// Pending calculation that is waiting for rates to load
let pendingCalculation: { expression: string } | null = null;

// Callbacks to notify when any rate source finishes loading
let rateLoadedCallbacks: (() => void)[] = [];

function notifyRateLoaded(): void {
  const callbacks = rateLoadedCallbacks.slice();
  rateLoadedCallbacks = [];
  for (const cb of callbacks) {
    cb();
  }
  // If there is a pending calculation, try it now
  if (pendingCalculation) {
    processPendingCalculation();
  }
}

function allRateSourcesDone(): boolean {
  return ecbDone && cbrDone && cryptoDone;
}

/**
 * Process a pending calculation that was waiting for rates.
 * Retries the calculation; if it still fails due to rates and more sources
 * are pending, keeps waiting. Otherwise sends the result.
 */
async function processPendingCalculation(): Promise<void> {
  if (!pendingCalculation || !calculator) return;

  const { expression } = pendingCalculation;

  try {
    const resultJson = calculator.calculate(expression);
    const result = JSON.parse(resultJson);

    if (isMissingRatesError(result) && !allRateSourcesDone()) {
      // Still waiting for more rate sources — keep pending
      return;
    }

    // Calculation succeeded or all rate sources are done (nothing more to wait for)
    pendingCalculation = null;
    self.postMessage({ type: 'result', data: result });
  } catch (error) {
    pendingCalculation = null;
    console.error('Calculation error:', error);
    self.postMessage({
      type: 'error',
      data: { error: `Calculation failed: ${error}` }
    });
  }
}

/**
 * Load cached rates from localStorage and apply to calculator.
 * Returns which rate sources had valid cached data.
 */
function loadCachedRates(): { ecb: boolean; cbr: boolean; crypto: boolean } {
  const result = { ecb: false, cbr: false, crypto: false };
  if (!calculator) return result;

  const ecbEntry = loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_ECB, ECB_CACHE_TTL_MS);
  if (ecbEntry) {
    calculator.update_rates_from_api(ecbEntry.base, ecbEntry.date, ecbEntry.rates_json);
    ratesLoaded = true;
    ecbLoaded = true;
    result.ecb = true;
    console.debug('[Rate cache] Loaded ECB rates from cache');
  }

  const cbrEntry = loadCachedRate<CbrRateCacheEntry>(RATE_CACHE_KEY_CBR, CBR_CACHE_TTL_MS);
  if (cbrEntry) {
    calculator.update_cbr_rates_from_api(cbrEntry.date, cbrEntry.rates_json);
    result.cbr = true;
    console.debug('[Rate cache] Loaded CBR rates from cache');
  }

  const cryptoEntry = loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_CRYPTO, CRYPTO_CACHE_TTL_MS);
  if (cryptoEntry) {
    calculator.update_crypto_rates_from_api(cryptoEntry.base, cryptoEntry.date, cryptoEntry.rates_json);
    result.crypto = true;
    console.debug('[Rate cache] Loaded crypto rates from cache');
  }

  return result;
}

function cacheEcbRates(base: string, date: string, rates_json: string): void {
  saveCachedRate(RATE_CACHE_KEY_ECB, { timestamp: Date.now(), base, date, rates_json } as RateCacheEntry);
}

function cacheCbrRates(date: string, rates_json: string): void {
  saveCachedRate(RATE_CACHE_KEY_CBR, { timestamp: Date.now(), date, rates_json } as CbrRateCacheEntry);
}

function cacheCryptoRates(base: string, date: string, rates_json: string): void {
  saveCachedRate(RATE_CACHE_KEY_CRYPTO, { timestamp: Date.now(), base, date, rates_json } as RateCacheEntry);
}

async function initWasm() {
  try {
    await init();
    const CalcClass = Calculator as unknown as CalculatorStatic;
    calculator = new CalcClass();
    const version = CalcClass.version();
    self.postMessage({ type: 'ready', data: { version } });

    // Load cached rates immediately so calculations can proceed without waiting
    const cached = loadCachedRates();
    if (cached.ecb) {
      self.postMessage({
        type: 'ratesLoaded',
        data: { success: true, source: 'cache' }
      });
    }

    // Fetch fresh rates in the background (will update cache when done)
    // Fetch CBR rates first so RUB conversions use official Russian Central Bank rates
    fetchCbrRates();
    fetchExchangeRates();
    fetchCryptoRates();
  } catch (error) {
    console.error('Failed to initialize WASM:', error);
    self.postMessage({
      type: 'error',
      data: { error: 'Failed to initialize calculator engine' }
    });
  }
}

async function fetchExchangeRates() {
  if (ratesLoading) {
    return;
  }

  ratesLoading = true;
  ratesError = null;
  self.postMessage({ type: 'ratesLoading', data: { loading: true } });

  try {
    // Fetch rates for common base currencies (USD is most common)
    const responseJson = await fetch_exchange_rates('usd');
    const response: ExchangeRatesResponse = JSON.parse(responseJson);

    if (response.success) {
      ratesLoaded = true;
      ecbLoaded = true;
      ratesError = null;

      // Parse the rates and count them
      const rates = JSON.parse(response.rates_json);
      const ratesCount = Object.keys(rates).length;

      // CRITICAL: Apply the fetched rates to the calculator instance
      // This was the root cause of issue #18 - rates were fetched but never applied
      let appliedCount = 0;
      if (calculator) {
        appliedCount = calculator.update_rates_from_api(
          response.base,
          response.date,
          response.rates_json
        );
      }

      // Cache for future page loads
      cacheEcbRates(response.base, response.date, response.rates_json);

      self.postMessage({
        type: 'ratesLoaded',
        data: {
          success: true,
          base: response.base,
          date: response.date,
          ratesCount,
          appliedCount
        }
      });
    } else {
      ratesError = response.error || 'Unknown error fetching rates';
      self.postMessage({
        type: 'ratesLoaded',
        data: {
          success: false,
          error: ratesError
        }
      });
    }
  } catch (error) {
    ratesError = `Failed to fetch exchange rates: ${error}`;
    console.error('Failed to fetch exchange rates:', error);
    self.postMessage({
      type: 'ratesLoaded',
      data: {
        success: false,
        error: ratesError
      }
    });
  } finally {
    ratesLoading = false;
    ecbDone = true;
    self.postMessage({ type: 'ratesLoading', data: { loading: false } });
    notifyRateLoaded();
  }
}

// Currency pairs to load from local .lino files as fallback when CBR CORS fails.
// These files are served from GitHub Pages at /calculator/data/currency/
// and are updated weekly by the update-currency-rates.yml CI workflow.
const LINO_RUB_PAIRS = [
  'usd-rub',
  'eur-rub',
  'gbp-rub',
  'jpy-rub',
  'chf-rub',
  'cny-rub',
  'inr-rub',
];

/// Load CBR rates from local .lino files served via GitHub Pages.
/// This is a fallback for when the cbr.ru API is blocked by CORS.
/// Returns the number of rate pairs successfully loaded.
async function loadCbrRatesFromLinoFiles(): Promise<number> {
  const baseUrl = '/calculator/data/currency';
  const ratesJson: Record<string, number> = {};
  let latestDate = '';

  for (const pair of LINO_RUB_PAIRS) {
    const [from] = pair.split('-');
    try {
      const response = await fetch(`${baseUrl}/${pair}.lino`);
      if (!response.ok) {
        console.debug(`[CBR fallback] ${pair}.lino not found (${response.status})`);
        continue;
      }
      const content = await response.text();
      const parsed = JSON.parse(parse_consolidated_lino_rates(content));

      if (parsed.success && parsed.rates && parsed.rates.length > 0) {
        // Get the latest rate (last entry in chronological order)
        const latestEntry = parsed.rates[parsed.rates.length - 1];
        ratesJson[from] = latestEntry.value;
        if (latestEntry.date > latestDate) {
          latestDate = latestEntry.date;
        }
        console.debug(`[CBR fallback] Loaded ${pair}: 1 ${from.toUpperCase()} = ${latestEntry.value} RUB (${latestEntry.date})`);
      }
    } catch (err) {
      console.debug(`[CBR fallback] Failed to load ${pair}.lino:`, err);
    }
  }

  const loadedCount = Object.keys(ratesJson).length;
  if (loadedCount > 0 && calculator) {
    const ratesJsonStr = JSON.stringify(ratesJson);
    calculator.update_cbr_rates_from_api(latestDate, ratesJsonStr);
    cacheCbrRates(latestDate, ratesJsonStr);
  }

  return loadedCount;
}

async function fetchCbrRates() {
  try {
    const responseJson = await fetch_cbr_rates();
    const response: ExchangeRatesResponse = JSON.parse(responseJson);

    if (response.success) {
      // Apply CBR rates to the calculator (RUB-based rates from Central Bank of Russia)
      if (calculator) {
        calculator.update_cbr_rates_from_api(
          response.date,
          response.rates_json
        );
      }

      // Cache for future page loads
      cacheCbrRates(response.date, response.rates_json);

      self.postMessage({
        type: 'cbrRatesLoaded',
        data: {
          success: true,
          date: response.date
        }
      });
    } else {
      // CBR direct fetch failed - try loading from local .lino files
      console.warn('[CBR] Direct CBR API fetch failed (likely CORS), falling back to local .lino files:', response.error);
      const loadedCount = await loadCbrRatesFromLinoFiles();
      self.postMessage({
        type: 'cbrRatesLoaded',
        data: {
          success: loadedCount > 0,
          error: loadedCount > 0 ? undefined : response.error,
          source: loadedCount > 0 ? 'local .lino files (CBR data)' : undefined,
          loadedCount
        }
      });
    }
  } catch (error) {
    // CBR direct fetch threw (typically CORS error in browser) - try local .lino files fallback
    console.warn('[CBR] Direct CBR API request failed (CORS error expected in browser):', error);
    console.info('[CBR] Falling back to local .lino files served via GitHub Pages...');

    try {
      const loadedCount = await loadCbrRatesFromLinoFiles();
      if (loadedCount > 0) {
        console.info(`[CBR] Successfully loaded ${loadedCount} RUB rate pairs from local .lino files`);
        self.postMessage({
          type: 'cbrRatesLoaded',
          data: {
            success: true,
            source: 'local .lino files (CBR data)',
            loadedCount
          }
        });
      } else {
        console.warn('[CBR] Local .lino files also unavailable. Using hardcoded default rates for RUB.');
        self.postMessage({
          type: 'cbrRatesLoaded',
          data: {
            success: false,
            error: `CBR CORS blocked and local .lino files unavailable. Original error: ${error}`
          }
        });
      }
    } catch (fallbackError) {
      console.error('[CBR] Fallback to local .lino files failed:', fallbackError);
      self.postMessage({
        type: 'cbrRatesLoaded',
        data: { success: false, error: `Failed to fetch CBR rates: ${error}` }
      });
    }
  } finally {
    cbrDone = true;
    notifyRateLoaded();
  }
}

async function fetchCryptoRates(vsCurrency = 'usd') {
  try {
    const responseJson = await fetch_crypto_rates(vsCurrency);
    const response: CryptoRatesResponse = JSON.parse(responseJson);

    if (response.success) {
      cryptoRatesError = null;

      if (calculator) {
        calculator.update_crypto_rates_from_api(
          response.base,
          response.date,
          response.rates_json
        );
      }

      // Cache for future page loads
      cacheCryptoRates(response.base, response.date, response.rates_json);

      self.postMessage({
        type: 'cryptoRatesLoaded',
        data: {
          success: true,
          base: response.base,
          date: response.date
        }
      });
    } else {
      cryptoRatesError = response.error || 'Unknown error fetching crypto rates';
      self.postMessage({
        type: 'cryptoRatesLoaded',
        data: { success: false, error: cryptoRatesError }
      });
    }
  } catch (error) {
    cryptoRatesError = `Failed to fetch crypto rates: ${error}`;
    console.error('Failed to fetch crypto rates:', error);
    self.postMessage({
      type: 'cryptoRatesLoaded',
      data: { success: false, error: cryptoRatesError }
    });
  } finally {
    cryptoDone = true;
    notifyRateLoaded();
  }
}

self.onmessage = async (e: MessageEvent) => {
  const { type, expression, baseCurrency } = e.data;

  if (type === 'calculate') {
    if (!calculator) {
      self.postMessage({
        type: 'error',
        data: { error: 'Calculator not initialized' }
      });
      return;
    }

    try {
      const resultJson = calculator.calculate(expression);
      const result = JSON.parse(resultJson);

      // If calculation failed due to missing rates and rate sources are still loading,
      // queue it and wait for rates to arrive instead of returning an error immediately.
      if (isMissingRatesError(result) && !allRateSourcesDone()) {
        pendingCalculation = { expression };
        // The calculation will be retried automatically when rate sources complete
        // (see notifyRateLoaded). The main thread keeps showing the busy indicator.
        return;
      }

      self.postMessage({ type: 'result', data: result });
    } catch (error) {
      console.error('Calculation error:', error);
      self.postMessage({
        type: 'error',
        data: { error: `Calculation failed: ${error}` }
      });
    }
  } else if (type === 'refreshRates') {
    // Allow manual refresh of exchange rates
    fetchCbrRates();
    fetchExchangeRates();
    fetchCryptoRates();
  } else if (type === 'fetchRates') {
    // Fetch rates for a specific base currency
    if (!baseCurrency) {
      self.postMessage({
        type: 'error',
        data: { error: 'baseCurrency is required' }
      });
      return;
    }

    try {
      const responseJson = await fetch_exchange_rates(baseCurrency);
      const response: ExchangeRatesResponse = JSON.parse(responseJson);
      self.postMessage({ type: 'ratesResult', data: response });
    } catch (error) {
      self.postMessage({
        type: 'ratesResult',
        data: { success: false, error: `Failed to fetch rates: ${error}` }
      });
    }
  } else if (type === 'getRatesStatus') {
    // Return current rates status
    self.postMessage({
      type: 'ratesStatus',
      data: {
        loaded: ratesLoaded,
        loading: ratesLoading,
        error: ratesError
      }
    });
  }
};

// Initialize WASM when worker starts
initWasm();
