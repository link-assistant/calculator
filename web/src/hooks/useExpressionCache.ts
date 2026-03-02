import { useCallback } from 'react';
import { Parser } from 'links-notation';
import { escapeReference } from 'lino-objects-codec';
import type { CalculationResult } from '../types';

const CACHE_KEY_PREFIX = 'lc_cache_v3_';
const MAX_CACHE_ENTRIES = 50;
const CACHE_INDEX_KEY = 'lc_cache_index_v3';

interface CacheEntry {
  expression: string;
  result: CalculationResult;
  appVersion: string;
  timestamp: number;
}

// Shared parser instance
const parser = new Parser();

/**
 * Get the cache key for a given expression.
 * Normalizes whitespace to improve cache hit rate.
 */
function getCacheKey(expression: string): string {
  return CACHE_KEY_PREFIX + expression.trim();
}

/**
 * Serialize a CacheEntry to indented Links Notation format.
 *
 * The format follows the same convention used for currency rates in this
 * application — each logical section is a top-level indented block:
 *
 * ```
 * cache-entry:
 *   expression '2 + 3'
 *   appVersion 0.5.1
 *   timestamp 1740925200000
 * result:
 *   result 5
 *   lino_interpretation '((2) + (3))'
 *   success true
 * steps:
 *   'Input expression: 2 + 3'
 *   'Compute: 2 + 3'
 *   '= 5'
 * ```
 */
function serializeCacheEntry(entry: CacheEntry): string {
  const lines: string[] = [];

  // Section 1: cache-entry header
  lines.push('cache-entry:');
  lines.push(`  expression ${escapeReference({ value: entry.expression })}`);
  lines.push(`  appVersion ${escapeReference({ value: entry.appVersion })}`);
  lines.push(`  timestamp ${entry.timestamp}`);

  // Section 2: result fields
  lines.push('result:');
  lines.push(`  result ${escapeReference({ value: entry.result.result })}`);
  lines.push(
    `  lino_interpretation ${escapeReference({ value: entry.result.lino_interpretation })}`
  );
  lines.push(`  success ${entry.result.success}`);

  if (entry.result.error !== undefined) {
    lines.push(`  error ${escapeReference({ value: entry.result.error })}`);
  }
  if (entry.result.latex_input !== undefined) {
    lines.push(`  latex_input ${escapeReference({ value: entry.result.latex_input })}`);
  }
  if (entry.result.latex_result !== undefined) {
    lines.push(`  latex_result ${escapeReference({ value: entry.result.latex_result })}`);
  }

  // Section 3: steps (always written, may be empty)
  lines.push('steps:');
  for (const step of entry.result.steps) {
    lines.push(`  ${escapeReference({ value: step })}`);
  }

  // Section 4: alternative_lino (optional)
  if (entry.result.alternative_lino && entry.result.alternative_lino.length > 0) {
    lines.push('alternative-lino:');
    for (const alt of entry.result.alternative_lino) {
      lines.push(`  ${escapeReference({ value: alt })}`);
    }
  }

  return lines.join('\n');
}

/**
 * Deserialize a CacheEntry from indented Links Notation format.
 * Returns null if the format is unrecognised or required sections are missing.
 */
function deserializeCacheEntry(lino: string): CacheEntry | null {
  try {
    const links = parser.parse(lino);

    // Locate the sections we need
    let entrySection: (typeof links)[number] | undefined;
    let resultSection: (typeof links)[number] | undefined;
    let stepsSection: (typeof links)[number] | undefined;
    let altLinoSection: (typeof links)[number] | undefined;

    for (const link of links) {
      if (link.id === 'cache-entry') entrySection = link;
      else if (link.id === 'result') resultSection = link;
      else if (link.id === 'steps') stepsSection = link;
      else if (link.id === 'alternative-lino') altLinoSection = link;
    }

    if (!entrySection || !resultSection) return null;

    // Helper: extract key→value pairs from a section's values array
    function extractPairs(
      section: NonNullable<typeof entrySection>
    ): Record<string, string> {
      const data: Record<string, string> = {};
      for (const val of section.values ?? []) {
        if (val.values && val.values.length === 2) {
          const key = val.values[0].id;
          const value = val.values[1].id;
          if (key !== null && key !== undefined && value !== null && value !== undefined) {
            data[String(key)] = String(value);
          }
        }
      }
      return data;
    }

    // Helper: extract a list of scalar values from a section (e.g., steps / alt-lino)
    function extractList(
      section: NonNullable<typeof stepsSection> | undefined
    ): string[] {
      if (!section) return [];
      const items: string[] = [];
      for (const val of section.values ?? []) {
        // Items may appear as a Link with id only (e.g., when value is unquoted)
        if (val.id !== null && val.id !== undefined && (val.values?.length ?? 0) === 0) {
          items.push(String(val.id));
        } else if (val.values && val.values.length === 1 && val.values[0].id !== null) {
          items.push(String(val.values[0].id));
        }
      }
      return items;
    }

    const entryData = extractPairs(entrySection);
    const resultData = extractPairs(resultSection);
    const steps = extractList(stepsSection);
    const altLino = extractList(altLinoSection);

    const expression = entryData['expression'];
    const appVersion = entryData['appVersion'];
    const timestamp = parseInt(entryData['timestamp'] ?? '0', 10);

    if (!expression || !appVersion) return null;

    const result: CalculationResult = {
      result: String(resultData['result'] ?? ''),
      lino_interpretation: String(resultData['lino_interpretation'] ?? ''),
      steps,
      success: resultData['success'] === 'true',
    };

    if ('error' in resultData) {
      result.error = String(resultData['error']);
    }
    if ('latex_input' in resultData) {
      result.latex_input = String(resultData['latex_input']);
    }
    if ('latex_result' in resultData) {
      result.latex_result = String(resultData['latex_result']);
    }
    if (altLino.length > 0) {
      result.alternative_lino = altLino;
    }

    return {
      expression,
      result,
      appVersion,
      timestamp: isNaN(timestamp) ? 0 : timestamp,
    };
  } catch {
    return null;
  }
}

/**
 * Serialize the cache index (ordered list of cache keys) to indented lino.
 *
 * ```
 * cache-index:
 *   'lc_cache_v3_2 + 3'
 *   'lc_cache_v3_100 USD in EUR'
 * ```
 */
function serializeCacheIndex(keys: string[]): string {
  const lines = ['cache-index:'];
  for (const key of keys) {
    lines.push(`  ${escapeReference({ value: key })}`);
  }
  return lines.join('\n');
}

/**
 * Deserialize the cache index from indented lino.
 */
function deserializeCacheIndex(lino: string): string[] {
  try {
    const links = parser.parse(lino);
    const indexLink = links.find((l) => l.id === 'cache-index');
    if (!indexLink) return [];

    const keys: string[] = [];
    for (const val of indexLink.values ?? []) {
      if (val.id !== null && val.id !== undefined && (val.values?.length ?? 0) === 0) {
        keys.push(String(val.id));
      }
    }
    return keys;
  } catch {
    return [];
  }
}

/**
 * Get the ordered list of cached expression keys (for eviction tracking).
 */
function getCacheIndex(): string[] {
  try {
    const raw = localStorage.getItem(CACHE_INDEX_KEY);
    if (!raw) return [];
    return deserializeCacheIndex(raw);
  } catch {
    return [];
  }
}

/**
 * Persist the cache index in indented Links Notation format.
 */
function saveCacheIndex(index: string[]): void {
  try {
    localStorage.setItem(CACHE_INDEX_KEY, serializeCacheIndex(index));
  } catch {
    // localStorage may be unavailable (private browsing, quota exceeded)
  }
}

/**
 * Hook that provides caching of calculation results in localStorage.
 *
 * Cache entries are stored in indented Links Notation format (human-readable),
 * following the same convention used for currency rates in this application.
 * Each entry is stored as multiple top-level sections:
 *
 * ```
 * cache-entry:
 *   expression '2 + 3'
 *   appVersion 0.5.1
 *   timestamp 1740925200000
 * result:
 *   result 5
 *   lino_interpretation '((2) + (3))'
 *   success true
 * steps:
 *   'Input expression: 2 + 3'
 *   'Compute: 2 + 3'
 *   '= 5'
 * ```
 *
 * Each entry includes the app version so stale results are automatically
 * discarded after an upgrade. Cache size is bounded to MAX_CACHE_ENTRIES;
 * the oldest entry is evicted when the limit is exceeded (LRU-like, by
 * insertion order).
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

        const entry = deserializeCacheEntry(raw);
        if (!entry) return null;

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

        localStorage.setItem(key, serializeCacheEntry(entry));
        saveCacheIndex(index);
      } catch {
        // localStorage may be unavailable or quota exceeded — fail silently
      }
    },
    [appVersion]
  );

  return { getCachedResult, cacheResult };
}
