// Rate coordination for the web worker.
// Manages promises that track when each exchange rate source finishes loading.
// Calculations await allRatesReady() before executing, ensuring rates are
// available on the first attempt — no retry or error-detection needed.

export interface RateCoordination {
  /** Wait for all exchange rate sources to finish loading. Resolves instantly if already loaded. */
  allRatesReady: () => Promise<void>;
  /** Signal that ECB rates have finished loading (success or failure). */
  resolveEcb: () => void;
  /** Signal that CBR rates have finished loading (success or failure). */
  resolveCbr: () => void;
  /** Signal that crypto rates have finished loading (success or failure). */
  resolveCrypto: () => void;
  /** Reset all rate promises (used when manually refreshing rates). */
  reset: () => void;
}

export function createRateCoordination(): RateCoordination {
  let resolveEcb: () => void;
  let resolveCbr: () => void;
  let resolveCrypto: () => void;
  let ecbReady: Promise<void>;
  let cbrReady: Promise<void>;
  let cryptoReady: Promise<void>;

  function init() {
    ecbReady = new Promise<void>((resolve) => { resolveEcb = resolve; });
    cbrReady = new Promise<void>((resolve) => { resolveCbr = resolve; });
    cryptoReady = new Promise<void>((resolve) => { resolveCrypto = resolve; });
  }

  init();

  return {
    allRatesReady: () => Promise.all([ecbReady, cbrReady, cryptoReady]).then(() => {}),
    get resolveEcb() { return resolveEcb; },
    get resolveCbr() { return resolveCbr; },
    get resolveCrypto() { return resolveCrypto; },
    reset: init,
  };
}
