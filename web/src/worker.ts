// Web Worker for running WASM calculations in the background
// This prevents blocking the main UI thread

import init, { Calculator, fetch_exchange_rates } from '@wasm/link_calculator';

interface CalculatorInstance {
  calculate(input: string): string;
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

let calculator: CalculatorInstance | null = null;
let ratesLoaded = false;
let ratesLoading = false;
let ratesError: string | null = null;

async function initWasm() {
  try {
    await init();
    const CalcClass = Calculator as unknown as CalculatorStatic;
    calculator = new CalcClass();
    const version = CalcClass.version();
    self.postMessage({ type: 'ready', data: { version } });

    // Fetch exchange rates in the background after WASM is initialized
    fetchExchangeRates();
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

      // Parse the rates
      const rates = JSON.parse(response.rates_json);

      self.postMessage({
        type: 'ratesLoaded',
        data: {
          success: true,
          base: response.base,
          date: response.date,
          ratesCount: Object.keys(rates).length
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
    fetchExchangeRates();
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
