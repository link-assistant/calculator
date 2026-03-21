// Utility functions for the worker, extracted for testability.

// Rate cache TTL constants
export const ECB_CACHE_TTL_MS = 12 * 60 * 60 * 1000;    // 12 hours for ECB (central bank) rates
export const CBR_CACHE_TTL_MS = 12 * 60 * 60 * 1000;    // 12 hours for CBR (central bank) rates
export const CRYPTO_CACHE_TTL_MS = 5 * 60 * 1000;       // 5 minutes for CoinGecko (real-time) rates

// localStorage keys for rate caching
export const RATE_CACHE_KEY_ECB = 'lc_rate_cache_ecb';
export const RATE_CACHE_KEY_CBR = 'lc_rate_cache_cbr';
export const RATE_CACHE_KEY_CRYPTO = 'lc_rate_cache_crypto';

export interface RateCacheEntry {
  timestamp: number;
  base: string;
  date: string;
  rates_json: string;
}

export interface CbrRateCacheEntry {
  timestamp: number;
  date: string;
  rates_json: string;
}

/**
 * Check if a calculation result failed due to missing exchange rates.
 */
export function isMissingRatesError(result: { success: boolean; error?: string; error_info?: { key: string } }): boolean {
  if (result.success) return false;
  // Check error_info key for currency-related errors
  if (result.error_info?.key === 'errors.currencyConversion' ||
      result.error_info?.key === 'errors.noExchangeRate' ||
      result.error_info?.key === 'errors.unknownCurrency') {
    return true;
  }
  // Fallback: check error message text
  if (result.error && (
    result.error.includes('No exchange rate available') ||
    result.error.includes('Cannot convert')
  )) {
    return true;
  }
  return false;
}

/**
 * Load a cached rate entry from localStorage.
 * Returns null if not found or expired.
 */
export function loadCachedRate<T extends { timestamp: number }>(
  key: string,
  ttlMs: number,
  now: number = Date.now()
): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const entry: T = JSON.parse(raw);
    if (now - entry.timestamp >= ttlMs) return null;
    return entry;
  } catch {
    return null;
  }
}

/**
 * Save a rate entry to localStorage cache.
 */
export function saveCachedRate(key: string, entry: object): void {
  try {
    localStorage.setItem(key, JSON.stringify(entry));
  } catch { /* localStorage may be unavailable */ }
}
