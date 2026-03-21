import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  isMissingRatesError,
  loadCachedRate,
  saveCachedRate,
  ECB_CACHE_TTL_MS,
  CBR_CACHE_TTL_MS,
  CRYPTO_CACHE_TTL_MS,
  RATE_CACHE_KEY_ECB,
  RATE_CACHE_KEY_CBR,
  RATE_CACHE_KEY_CRYPTO,
  type RateCacheEntry,
  type CbrRateCacheEntry,
} from './worker-utils';

// Use a real in-memory localStorage for these tests
// (the global mock from setup.ts uses vi.fn() which doesn't store data)
function createRealLocalStorage(): Storage {
  const store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { for (const key in store) delete store[key]; },
    get length() { return Object.keys(store).length; },
    key: (index: number) => Object.keys(store)[index] ?? null,
  };
}

describe('isMissingRatesError', () => {
  it('returns false for successful results', () => {
    expect(isMissingRatesError({ success: true })).toBe(false);
  });

  it('returns true for currencyConversion error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.currencyConversion' },
    })).toBe(true);
  });

  it('returns true for noExchangeRate error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.noExchangeRate' },
    })).toBe(true);
  });

  it('returns true for unknownCurrency error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.unknownCurrency' },
    })).toBe(true);
  });

  it('returns true when error message contains "No exchange rate available"', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Cannot convert TON to RUB: No exchange rate available',
    })).toBe(true);
  });

  it('returns true when error message contains "Cannot convert"', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Cannot convert BTC to USD',
    })).toBe(true);
  });

  it('returns false for non-currency errors', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Division by zero',
      error_info: { key: 'errors.divisionByZero' },
    })).toBe(false);
  });

  it('returns false for failed result with unrelated error', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Parse error: unexpected token',
    })).toBe(false);
  });
});

describe('Rate cache (loadCachedRate / saveCachedRate)', () => {
  let originalLocalStorage: Storage;

  beforeEach(() => {
    originalLocalStorage = window.localStorage;
    Object.defineProperty(window, 'localStorage', {
      value: createRealLocalStorage(),
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    Object.defineProperty(window, 'localStorage', {
      value: originalLocalStorage,
      writable: true,
      configurable: true,
    });
  });

  it('returns null for non-existent key', () => {
    expect(loadCachedRate('nonexistent', 1000)).toBeNull();
  });

  it('saves and loads a rate entry', () => {
    const entry: RateCacheEntry = {
      timestamp: Date.now(),
      base: 'usd',
      date: '2026-03-21',
      rates_json: '{"eur": 0.92}',
    };
    saveCachedRate(RATE_CACHE_KEY_ECB, entry);
    const loaded = loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_ECB, ECB_CACHE_TTL_MS);
    expect(loaded).not.toBeNull();
    expect(loaded!.base).toBe('usd');
    expect(loaded!.rates_json).toBe('{"eur": 0.92}');
  });

  it('returns null when cache entry is expired', () => {
    const entry: RateCacheEntry = {
      timestamp: Date.now() - ECB_CACHE_TTL_MS - 1000, // expired
      base: 'usd',
      date: '2026-03-21',
      rates_json: '{"eur": 0.92}',
    };
    saveCachedRate(RATE_CACHE_KEY_ECB, entry);
    expect(loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_ECB, ECB_CACHE_TTL_MS)).toBeNull();
  });

  it('returns valid entry just before expiration', () => {
    const entry: RateCacheEntry = {
      timestamp: Date.now() - ECB_CACHE_TTL_MS + 5000, // 5s before expiry
      base: 'usd',
      date: '2026-03-21',
      rates_json: '{"eur": 0.92}',
    };
    saveCachedRate(RATE_CACHE_KEY_ECB, entry);
    expect(loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_ECB, ECB_CACHE_TTL_MS)).not.toBeNull();
  });

  it('handles corrupt localStorage data gracefully', () => {
    localStorage.setItem(RATE_CACHE_KEY_ECB, 'not valid json');
    expect(loadCachedRate(RATE_CACHE_KEY_ECB, ECB_CACHE_TTL_MS)).toBeNull();
  });

  it('CBR cache uses different TTL and key', () => {
    const entry: CbrRateCacheEntry = {
      timestamp: Date.now(),
      date: '2026-03-21',
      rates_json: '{"usd": 92.5}',
    };
    saveCachedRate(RATE_CACHE_KEY_CBR, entry);
    const loaded = loadCachedRate<CbrRateCacheEntry>(RATE_CACHE_KEY_CBR, CBR_CACHE_TTL_MS);
    expect(loaded).not.toBeNull();
    expect(loaded!.rates_json).toBe('{"usd": 92.5}');
  });

  it('crypto cache expires after 5 minutes', () => {
    const entry: RateCacheEntry = {
      timestamp: Date.now() - CRYPTO_CACHE_TTL_MS - 1000, // expired
      base: 'usd',
      date: '2026-03-21',
      rates_json: '{"btc": 67000}',
    };
    saveCachedRate(RATE_CACHE_KEY_CRYPTO, entry);
    expect(loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_CRYPTO, CRYPTO_CACHE_TTL_MS)).toBeNull();
  });

  it('crypto cache is valid within 5 minutes', () => {
    const entry: RateCacheEntry = {
      timestamp: Date.now() - CRYPTO_CACHE_TTL_MS + 30000, // 30s before expiry
      base: 'usd',
      date: '2026-03-21',
      rates_json: '{"btc": 67000}',
    };
    saveCachedRate(RATE_CACHE_KEY_CRYPTO, entry);
    expect(loadCachedRate<RateCacheEntry>(RATE_CACHE_KEY_CRYPTO, CRYPTO_CACHE_TTL_MS)).not.toBeNull();
  });
});

describe('TTL constants', () => {
  it('ECB TTL is 12 hours', () => {
    expect(ECB_CACHE_TTL_MS).toBe(12 * 60 * 60 * 1000);
  });

  it('CBR TTL is 12 hours', () => {
    expect(CBR_CACHE_TTL_MS).toBe(12 * 60 * 60 * 1000);
  });

  it('Crypto TTL is 5 minutes', () => {
    expect(CRYPTO_CACHE_TTL_MS).toBe(5 * 60 * 1000);
  });
});
