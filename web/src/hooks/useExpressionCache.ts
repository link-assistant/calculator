import { useCallback } from 'react';
import type { CalculationResult } from '../types';

const CACHE_KEY_PREFIX = 'lc_cache_v1_';
const MAX_CACHE_ENTRIES = 50;
const CACHE_INDEX_KEY = 'lc_cache_index_v1';

interface CacheEntry {
  expression: string;
  result: CalculationResult;
  appVersion: string;
  timestamp: number;
}

/**
 * Get the cache key for a given expression.
 * Normalizes whitespace to improve cache hit rate.
 */
function getCacheKey(expression: string): string {
  return CACHE_KEY_PREFIX + expression.trim();
}

/**
 * Get the ordered list of cached expression keys (for eviction).
 */
function getCacheIndex(): string[] {
  try {
    const raw = localStorage.getItem(CACHE_INDEX_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

/**
 * Persist the cache index.
 */
function saveCacheIndex(index: string[]): void {
  try {
    localStorage.setItem(CACHE_INDEX_KEY, JSON.stringify(index));
  } catch {
    // localStorage may be unavailable (private browsing, quota exceeded)
  }
}

/**
 * Hook that provides caching of calculation results in localStorage.
 *
 * The cache stores results keyed by expression and tagged with the app version.
 * When loading a cached result, if the stored version does not match the
 * current app version the cached entry is ignored and the expression is
 * recalculated fresh.
 *
 * Cache size is bounded to MAX_CACHE_ENTRIES; the oldest entry is evicted
 * when the limit is exceeded (LRU-like, by insertion order).
 */
export function useExpressionCache(appVersion: string) {
  /**
   * Read a cached result for the given expression.
   * Returns null if no cache entry exists or if the version does not match.
   */
  const getCachedResult = useCallback(
    (expression: string): CalculationResult | null => {
      if (!expression.trim()) return null;

      try {
        const raw = localStorage.getItem(getCacheKey(expression));
        if (!raw) return null;

        const entry: CacheEntry = JSON.parse(raw);

        // Invalidate if app version changed
        if (entry.appVersion !== appVersion) return null;

        return entry.result;
      } catch {
        return null;
      }
    },
    [appVersion]
  );

  /**
   * Store a calculation result in the cache for the given expression.
   * Only successful results are cached.
   */
  const cacheResult = useCallback(
    (expression: string, result: CalculationResult): void => {
      if (!expression.trim() || !result.success) return;

      try {
        const key = getCacheKey(expression);
        const entry: CacheEntry = {
          expression: expression.trim(),
          result,
          appVersion,
          timestamp: Date.now(),
        };

        // Update the index: remove existing entry for this key, add to end
        const index = getCacheIndex();
        const existingIdx = index.indexOf(key);
        if (existingIdx !== -1) {
          index.splice(existingIdx, 1);
        }
        index.push(key);

        // Evict oldest entries if over limit
        while (index.length > MAX_CACHE_ENTRIES) {
          const evicted = index.shift();
          if (evicted) {
            localStorage.removeItem(evicted);
          }
        }

        localStorage.setItem(key, JSON.stringify(entry));
        saveCacheIndex(index);
      } catch {
        // localStorage may be unavailable or quota exceeded — fail silently
      }
    },
    [appVersion]
  );

  return { getCachedResult, cacheResult };
}
