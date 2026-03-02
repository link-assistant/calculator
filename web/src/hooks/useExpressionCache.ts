import { useCallback } from 'react';
import { jsonToLino, linoToJson } from 'lino-objects-codec';
import type { CalculationResult } from '../types';

const CACHE_KEY_PREFIX = 'lc_cache_v2_';
const MAX_CACHE_ENTRIES = 50;
const CACHE_INDEX_KEY = 'lc_cache_index_v2';

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
    if (!raw) return [];
    const parsed = linoToJson({ lino: raw });
    // linoToJson unwraps single-element arrays to scalars; normalise back to array
    if (Array.isArray(parsed)) return parsed as string[];
    if (typeof parsed === 'string') return [parsed];
    return [];
  } catch {
    return [];
  }
}

/**
 * Persist the cache index in Links Notation format.
 */
function saveCacheIndex(index: string[]): void {
  try {
    localStorage.setItem(CACHE_INDEX_KEY, jsonToLino({ json: index }));
  } catch {
    // localStorage may be unavailable (private browsing, quota exceeded)
  }
}

/**
 * Restore CalculationResult field types after linoToJson round-trip.
 *
 * linoToJson converts numeric-looking strings (e.g., "5") to numbers.
 * CalculationResult.result and .lino_interpretation are always strings,
 * so we coerce them back explicitly.
 */
function coerceResult(raw: unknown): CalculationResult {
  const r = raw as Record<string, unknown>;
  return {
    ...(r as CalculationResult),
    result: r.result !== undefined ? String(r.result) : '',
    lino_interpretation: r.lino_interpretation !== undefined ? String(r.lino_interpretation) : '',
  };
}

/**
 * Hook that provides caching of calculation results in localStorage.
 *
 * Cache entries are stored in Links Notation format (human-readable),
 * matching the convention used for currency rates and preferences in
 * this application. Each entry includes the app version so stale
 * results are automatically discarded after an upgrade.
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

        const entry = linoToJson({ lino: raw }) as Record<string, unknown> | null;
        if (!entry || typeof entry !== 'object') return null;

        // Invalidate if app version changed
        if (entry.appVersion !== appVersion) return null;

        return coerceResult(entry.result);
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

        localStorage.setItem(key, jsonToLino({ json: entry }));
        saveCacheIndex(index);
      } catch {
        // localStorage may be unavailable or quota exceeded — fail silently
      }
    },
    [appVersion]
  );

  return { getCachedResult, cacheResult };
}
