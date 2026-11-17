import { describe, expect, test } from 'vitest';
import * as libm from '../../src/rust/libm.js';
import { f64Bits, f64FromBits, loadVectors, sameF64 } from '../vectors.js';

/**
 * js/test/vectors/libm.json: for every function, rows of the argument bits,
 * then the bits `libm` 0.2.16 returns, then the bits the `f64` method (the
 * platform C library) returns.
 * @type {{ crate: string, functions: Record<string, string[][]> }}
 */
const vectors = loadVectors('libm');

/** @type {Record<string, (...args: number[]) => number>} */
const functions = {
  sin: libm.sin,
  cos: libm.cos,
  tan: libm.tan,
  asin: libm.asin,
  acos: libm.acos,
  atan: libm.atan,
  atan2: libm.atan2,
  sinh: libm.sinh,
  cosh: libm.cosh,
  tanh: libm.tanh,
  exp: libm.exp,
  log: libm.log,
  log10: libm.log10,
  log2: libm.log2,
  pow: libm.pow,
  cbrt: libm.cbrt,
  sqrt: libm.sqrt,
  fma: libm.fma,
};

describe('libm against the Rust libm crate', () => {
  test('the vectors come from libm 0.2.16 and cover every export', () => {
    expect(vectors.crate).toBe('libm 0.2.16');
    expect(Object.keys(vectors.functions).sort()).toEqual(Object.keys(functions).sort());
    expect(Object.keys(libm).sort()).toEqual(Object.keys(functions).sort());
  });

  for (const [name, rows] of Object.entries(vectors.functions)) {
    test(`${name}: ${rows.length} vectors, bit for bit`, () => {
      const f = functions[name];
      const mismatches = [];
      for (const row of rows) {
        const args = row.slice(0, -2);
        const expected = row[row.length - 2];
        const actual = f(...args.map(f64FromBits));
        if (!sameF64(actual, expected)) {
          mismatches.push({ args, expected, actual: f64Bits(actual) });
        }
      }
      expect(mismatches).toEqual([]);
    });
  }

  test('fma keeps the sticky bit that generic/fma.rs drops', () => {
    // (1 + 2^-26) * (1 - 2^-26 + 2^-52) = 1 + 2^-78, so the exact sum is
    // 2^53 + 1 + 2^-78 and rounds up to 2^53 + 2, as the FMA instruction does.
    expect(libm.fma(1 + 2 ** -26, 1 - 2 ** -26 + 2 ** -52, 2 ** 53)).toBe(2 ** 53 + 2);
    expect(libm.fma(1 + 2 ** -26, -(1 - 2 ** -26 + 2 ** -52), 2 ** 54)).toBe(2 ** 54 - 2);
  });
});
