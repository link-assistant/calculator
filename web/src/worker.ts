// Web Worker for running WASM calculations in the background
// This prevents blocking the main UI thread

import init, { Calculator } from '../pkg/link_calculator';

interface CalculatorInstance {
  calculate(input: string): string;
}

interface CalculatorStatic {
  new (): CalculatorInstance;
  version(): string;
}

let calculator: CalculatorInstance | null = null;

async function initWasm() {
  try {
    await init();
    const CalcClass = Calculator as unknown as CalculatorStatic;
    calculator = new CalcClass();
    const version = CalcClass.version();
    self.postMessage({ type: 'ready', data: { version } });
  } catch (error) {
    console.error('Failed to initialize WASM:', error);
    self.postMessage({
      type: 'error',
      data: { error: 'Failed to initialize calculator engine' }
    });
  }
}

self.onmessage = async (e: MessageEvent) => {
  const { type, expression } = e.data;

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
  }
};

// Initialize WASM when worker starts
initWasm();
