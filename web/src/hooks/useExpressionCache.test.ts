import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useExpressionCache } from './useExpressionCache';
import type { CalculationResult } from '../types';

// Use a real in-memory localStorage for these tests so that get/set/remove work correctly.
// The global setup in src/test/setup.ts mocks localStorage with vi.fn() stubs that don't
// retain data between calls, which breaks tests for cache read-back.
const realLocalStorage = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
    get length() { return Object.keys(store).length; },
    key: (index: number) => Object.keys(store)[index] ?? null,
    // Allow spreading for Object.keys(localStorage) checks in tests
    [Symbol.iterator]() {
      const keys = Object.keys(store);
      let index = 0;
      return {
        next() {
          if (index < keys.length) {
            return { value: keys[index++], done: false };
          }
          return { value: undefined, done: true };
        },
      };
    },
  };
})();

// Override the global localStorage mock with our real in-memory implementation
vi.stubGlobal('localStorage', realLocalStorage);

// Helper to build a minimal successful CalculationResult
function makeResult(result: string): CalculationResult {
  return {
    result,
    lino_interpretation: result,
    steps: [],
    success: true,
  };
}

// Helper to build a failed CalculationResult
function makeErrorResult(): CalculationResult {
  return {
    result: '',
    lino_interpretation: '',
    steps: [],
    success: false,
    error: 'some error',
  };
}

describe('useExpressionCache', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('getCachedResult', () => {
    it('should return null for an expression not in cache', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      expect(result.current.getCachedResult('2 + 3')).toBeNull();
    });

    it('should return null for empty expression', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      expect(result.current.getCachedResult('')).toBeNull();
      expect(result.current.getCachedResult('   ')).toBeNull();
    });

    it('should return cached result when version matches', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResult('5');

      act(() => {
        result.current.cacheResult('2 + 3', calcResult);
      });

      const cached = result.current.getCachedResult('2 + 3');
      expect(cached).toEqual(calcResult);
    });

    it('should return null when app version changed', () => {
      // Cache with version 1.0.0
      const { result: r1 } = renderHook(() => useExpressionCache('1.0.0'));
      act(() => {
        r1.current.cacheResult('2 + 3', makeResult('5'));
      });

      // Read with version 2.0.0
      const { result: r2 } = renderHook(() => useExpressionCache('2.0.0'));
      expect(r2.current.getCachedResult('2 + 3')).toBeNull();
    });

    it('should normalise whitespace in expression key', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResult('5');

      act(() => {
        result.current.cacheResult('  2 + 3  ', calcResult);
      });

      // Same expression without extra whitespace should still hit the cache
      expect(result.current.getCachedResult('2 + 3')).toEqual(calcResult);
    });

    it('should return null for corrupted cache entry', () => {
      localStorage.setItem('lc_cache_v1_broken', 'not-json');
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      expect(result.current.getCachedResult('broken')).toBeNull();
    });
  });

  describe('cacheResult', () => {
    it('should not cache failed results', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('bad expression', makeErrorResult());
      });

      expect(result.current.getCachedResult('bad expression')).toBeNull();
    });

    it('should not cache empty expression', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('', makeResult('0'));
        result.current.cacheResult('   ', makeResult('0'));
      });

      // Nothing should be stored beyond the (possibly empty) index key
      expect(localStorage.getItem('lc_cache_v1_')).toBeNull();
    });

    it('should overwrite an existing entry for the same expression', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('2 + 3', makeResult('5'));
        result.current.cacheResult('2 + 3', makeResult('five'));
      });

      expect(result.current.getCachedResult('2 + 3')?.result).toBe('five');
    });

    it('should evict oldest entry when cache exceeds 50 entries', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      // Fill the cache to MAX_CACHE_ENTRIES (50)
      act(() => {
        for (let i = 1; i <= 50; i++) {
          result.current.cacheResult(`expression ${i}`, makeResult(`${i}`));
        }
      });

      // All 50 should be present
      expect(result.current.getCachedResult('expression 1')).not.toBeNull();

      // Adding one more should evict the oldest
      act(() => {
        result.current.cacheResult('expression 51', makeResult('51'));
      });

      expect(result.current.getCachedResult('expression 1')).toBeNull();
      expect(result.current.getCachedResult('expression 51')).not.toBeNull();
    });

    it('should update LRU order when re-caching an existing expression', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      // Fill 50 entries
      act(() => {
        for (let i = 1; i <= 50; i++) {
          result.current.cacheResult(`expression ${i}`, makeResult(`${i}`));
        }
      });

      // Re-cache expression 1 (moves it to end of index)
      act(() => {
        result.current.cacheResult('expression 1', makeResult('one'));
      });

      // Adding one more should evict expression 2 (now oldest), not expression 1
      act(() => {
        result.current.cacheResult('expression 51', makeResult('51'));
      });

      expect(result.current.getCachedResult('expression 1')).not.toBeNull();
      expect(result.current.getCachedResult('expression 2')).toBeNull();
    });
  });
});
