// Web Worker for running WASM calculations in the background
// This prevents blocking the main UI thread

import init, { Calculator, fetch_exchange_rates, fetch_crypto_rates, fetch_cbr_rates, parse_consolidated_lino_rates } from '@wasm/link_calculator';
import { createRateCoordination } from './worker-rate-coordination';

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

// On-demand rate coordination: calculations detect which rate sources they
// need and wait only for those. Sources that aren't needed are never fetched.
const rateCoordination = createRateCoordination();

async function initWasm() {
  try {
    await init();
    const CalcClass = Calculator as unknown as CalculatorStatic;
    calculator = new CalcClass();
    const version = CalcClass.version();
    self.postMessage({ type: 'ready', data: { version } });

    // Register rate fetchers so the coordination module can trigger them
    // on demand. No rates are fetched eagerly — they load when a calculation
    // first needs them.
    rateCoordination.registerFetcher('ecb', fetchExchangeRates);
    rateCoordination.registerFetcher('cbr', fetchCbrRates);
    rateCoordination.registerFetcher('crypto', fetchCryptoRates);
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
    self.postMessage({ type: 'ratesLoading', data: { loading: false } });
    rateCoordination.markLoaded('ecb');
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
    rateCoordination.markLoaded('cbr');
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
    rateCoordination.markLoaded('crypto');
  }
}

/**
 * Execute a calculation, loading only the exchange rate sources it needs.
 *
 * 1. Scans the expression for currency keywords to detect required sources.
 * 2. Triggers fetching for sources that haven't been loaded yet.
 * 3. Waits only for the required sources (already-loaded sources resolve instantly).
 * 4. Executes the calculation once rates are available.
 *
 * Pure math expressions skip rate loading entirely (zero delay).
 */
async function executeCalculation(expression: string): Promise<void> {
  if (!calculator) {
    self.postMessage({
      type: 'error',
      data: { error: 'Calculator not initialized' }
    });
    return;
  }

  // Wait only for the rate sources this expression needs.
  // Pure math expressions (no currency references) proceed immediately.
  await rateCoordination.ensureRatesForExpression(expression);

  try {
    const resultJson = calculator.calculate(expression);
    const result = JSON.parse(resultJson);
    self.postMessage({ type: 'result', data: result });
  } catch (error) {
    console.error('Calculation error:', error);
    self.postMessage({
      type: 'error',
      data: { error: `Calculation failed: ${error}` }
    });
  }
}

self.onmessage = async (e: MessageEvent) => {
  const { type, expression, baseCurrency } = e.data;

  if (type === 'calculate') {
    await executeCalculation(expression);
  } else if (type === 'refreshRates') {
    // Manual refresh: re-fetch all sources that were previously loaded
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
