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
function makeResult(result: string, linoInterpretation = result): CalculationResult {
  return {
    result,
    lino_interpretation: linoInterpretation,
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

// Helper to build a result with steps
function makeResultWithSteps(result: string, steps: string[]): CalculationResult {
  return {
    result,
    lino_interpretation: result,
    steps,
    success: true,
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
      expect(cached).not.toBeNull();
      expect(cached!.result).toBe('5');
      expect(cached!.success).toBe(true);
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
      expect(result.current.getCachedResult('2 + 3')).not.toBeNull();
    });

    it('should return null for corrupted cache entry', () => {
      localStorage.setItem('lc_cache_v2_broken', 'not-valid-lino!!!@@@###');
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      expect(result.current.getCachedResult('broken')).toBeNull();
    });

    it('should preserve result as string for numeric calculation results', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      // "5" looks like a number but must remain a string after round-trip
      const calcResult = makeResult('5', '((2) + (3))');

      act(() => {
        result.current.cacheResult('2 + 3', calcResult);
      });

      const cached = result.current.getCachedResult('2 + 3');
      expect(cached).not.toBeNull();
      expect(typeof cached!.result).toBe('string');
      expect(cached!.result).toBe('5');
    });

    it('should preserve lino_interpretation as string even when it looks like a number', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResult('42', '42');

      act(() => {
        result.current.cacheResult('42', calcResult);
      });

      const cached = result.current.getCachedResult('42');
      expect(cached).not.toBeNull();
      expect(typeof cached!.result).toBe('string');
      expect(cached!.result).toBe('42');
      expect(typeof cached!.lino_interpretation).toBe('string');
      expect(cached!.lino_interpretation).toBe('42');
    });

    it('should preserve steps array through round-trip', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResultWithSteps('5', [
        'Input expression: 2 + 3',
        'Compute: 2 + 3',
        '= 5',
      ]);

      act(() => {
        result.current.cacheResult('2 + 3', calcResult);
      });

      const cached = result.current.getCachedResult('2 + 3');
      expect(cached).not.toBeNull();
      expect(cached!.steps).toEqual(['Input expression: 2 + 3', 'Compute: 2 + 3', '= 5']);
    });

    it('should preserve empty steps array through round-trip', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResult('5');

      act(() => {
        result.current.cacheResult('2 + 3', calcResult);
      });

      const cached = result.current.getCachedResult('2 + 3');
      expect(cached).not.toBeNull();
      expect(Array.isArray(cached!.steps)).toBe(true);
    });

    it('should store entries in Links Notation format', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('2 + 3', makeResult('5'));
      });

      const raw = localStorage.getItem('lc_cache_v2_2 + 3');
      expect(raw).not.toBeNull();
      // Links Notation uses parentheses for structure, not JSON braces
      expect(raw).toContain('(');
      expect(raw).not.toMatch(/^\{/); // should NOT start with JSON object brace
    });

    it('should store the cache index in Links Notation format', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('2 + 3', makeResult('5'));
      });

      const raw = localStorage.getItem('lc_cache_index_v2');
      expect(raw).not.toBeNull();
      // Array in Links Notation is parenthesized: ('item1' 'item2')
      expect(raw).toContain('(');
      expect(raw).not.toMatch(/^\[/); // should NOT start with JSON array bracket
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

      // Nothing should be stored for empty expressions
      expect(localStorage.getItem('lc_cache_v2_')).toBeNull();
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

    it('should cache results with complex lino_interpretation', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult: CalculationResult = {
        result: '91.50 EUR',
        lino_interpretation: '((100 USD) in EUR)',
        steps: [
          'Input expression: 100 USD in EUR',
          'Exchange rate: 1 USD = 0.915 EUR',
          '= 91.5 EUR',
        ],
        success: true,
      };

      act(() => {
        result.current.cacheResult('100 USD in EUR', calcResult);
      });

      const cached = result.current.getCachedResult('100 USD in EUR');
      expect(cached).not.toBeNull();
      expect(cached!.result).toBe('91.50 EUR');
      expect(cached!.lino_interpretation).toBe('((100 USD) in EUR)');
      expect(cached!.steps).toHaveLength(3);
    });

    it('should cache results with alternative_lino interpretations', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult: CalculationResult = {
        result: '0.(3)',
        lino_interpretation: '(1/3)',
        alternative_lino: ['0.333...', '0.3̅'],
        steps: [],
        success: true,
      };

      act(() => {
        result.current.cacheResult('1 / 3', calcResult);
      });

      const cached = result.current.getCachedResult('1 / 3');
      expect(cached).not.toBeNull();
      expect(cached!.result).toBe('0.(3)');
      expect(Array.isArray(cached!.alternative_lino)).toBe(true);
    });

    it('should handle expressions with special characters', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));
      const calcResult = makeResult('1.41421356', '((sqrt 2))');

      act(() => {
        result.current.cacheResult('sqrt(2)', calcResult);
      });

      const cached = result.current.getCachedResult('sqrt(2)');
      expect(cached).not.toBeNull();
      expect(cached!.result).toBe('1.41421356');
    });

    it('should handle multiple different expressions independently', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      act(() => {
        result.current.cacheResult('2 + 3', makeResult('5'));
        result.current.cacheResult('10 * 10', makeResult('100'));
        result.current.cacheResult('100 USD in EUR', makeResult('91.50 EUR'));
      });

      expect(result.current.getCachedResult('2 + 3')?.result).toBe('5');
      expect(result.current.getCachedResult('10 * 10')?.result).toBe('100');
      expect(result.current.getCachedResult('100 USD in EUR')?.result).toBe('91.50 EUR');
    });

    it('should keep cached results after partial eviction', () => {
      const { result } = renderHook(() => useExpressionCache('1.0.0'));

      // Fill cache to limit
      act(() => {
        for (let i = 1; i <= 50; i++) {
          result.current.cacheResult(`expr ${i}`, makeResult(`${i}`));
        }
      });

      // Add 5 more — should evict the 5 oldest (1-5)
      act(() => {
        for (let i = 51; i <= 55; i++) {
          result.current.cacheResult(`expr ${i}`, makeResult(`${i}`));
        }
      });

      // Entries 1-5 should be evicted
      for (let i = 1; i <= 5; i++) {
        expect(result.current.getCachedResult(`expr ${i}`)).toBeNull();
      }

      // Entries 6-55 should still be present
      for (let i = 6; i <= 55; i++) {
        expect(result.current.getCachedResult(`expr ${i}`)).not.toBeNull();
      }
    });
  });
});
