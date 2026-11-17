import { describe, expect, test } from 'vitest';
import * as libm from '../../src/rust/libm.js';
import { ok, tryResult, unwrapResult } from '../../src/rust/core.js';
import { E, PI } from '../../src/rust/num.js';
import { Decimal } from '../../src/types/index.js';
import { evaluateFunction, integrate, isMathFunction } from '../../src/grammar/math_functions.js';
import { f64Bits, f64FromBits, loadVectors } from '../vectors.js';

/** @param {number} a @param {number} b @param {number} epsilon */
function approxEq(a, b, epsilon) {
  return Math.abs(a - b) < epsilon;
}

describe('math functions (src/grammar/math_functions.rs)', () => {
  test('sin', () => {
    let result = unwrapResult(() => evaluateFunction('sin', [Decimal.new(0)]));
    expect(approxEq(result.toF64(), 0.0, 1e-10)).toBe(true);

    result = unwrapResult(() => evaluateFunction('sin', [Decimal.fromF64(PI / 2.0)]));
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);
  });

  test('cos', () => {
    let result = unwrapResult(() => evaluateFunction('cos', [Decimal.new(0)]));
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);

    result = unwrapResult(() => evaluateFunction('cos', [Decimal.fromF64(PI)]));
    expect(approxEq(result.toF64(), -1.0, 1e-10)).toBe(true);
  });

  test('sqrt', () => {
    let result = unwrapResult(() => evaluateFunction('sqrt', [Decimal.new(16)]));
    expect(approxEq(result.toF64(), 4.0, 1e-10)).toBe(true);

    result = unwrapResult(() => evaluateFunction('sqrt', [Decimal.new(2)]));
    expect(approxEq(result.toF64(), Math.SQRT2, 1e-10)).toBe(true);
  });

  test('sqrt negative', () => {
    expect(() => evaluateFunction('sqrt', [Decimal.fromF64(-1.0)])).toThrow();
  });

  test('ln', () => {
    const result = unwrapResult(() => evaluateFunction('ln', [Decimal.fromF64(E)]));
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);
  });

  test('exp', () => {
    let result = unwrapResult(() => evaluateFunction('exp', [Decimal.new(0)]));
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);

    result = unwrapResult(() => evaluateFunction('exp', [Decimal.new(1)]));
    expect(approxEq(result.toF64(), E, 1e-10)).toBe(true);
  });

  test('pow', () => {
    const result = unwrapResult(() => evaluateFunction('pow', [Decimal.new(2), Decimal.new(3)]));
    expect(approxEq(result.toF64(), 8.0, 1e-10)).toBe(true);
  });

  test('factorial', () => {
    let result = unwrapResult(() => evaluateFunction('factorial', [Decimal.new(5)]));
    expect(approxEq(result.toF64(), 120.0, 1e-10)).toBe(true);

    result = unwrapResult(() => evaluateFunction('factorial', [Decimal.new(0)]));
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);
  });

  test('pi', () => {
    const result = unwrapResult(() => evaluateFunction('pi', []));
    expect(approxEq(result.toF64(), PI, 1e-10)).toBe(true);
  });

  test('e', () => {
    const result = unwrapResult(() => evaluateFunction('e', []));
    expect(approxEq(result.toF64(), E, 1e-10)).toBe(true);
  });

  test('abs', () => {
    const result = unwrapResult(() => evaluateFunction('abs', [Decimal.fromF64(-5.0)]));
    expect(approxEq(result.toF64(), 5.0, 1e-10)).toBe(true);
  });

  test('min max', () => {
    let result = unwrapResult(() =>
      evaluateFunction('min', [Decimal.new(3), Decimal.new(1), Decimal.new(2)]),
    );
    expect(approxEq(result.toF64(), 1.0, 1e-10)).toBe(true);

    result = unwrapResult(() =>
      evaluateFunction('max', [Decimal.new(3), Decimal.new(1), Decimal.new(2)]),
    );
    expect(approxEq(result.toF64(), 3.0, 1e-10)).toBe(true);
  });

  test('integrate constant', () => {
    // Integral of 1 from 0 to 1 should be 1
    const result = integrate(() => 1.0, 0.0, 1.0);
    expect(approxEq(result, 1.0, 1e-6)).toBe(true);
  });

  test('integrate linear', () => {
    // Integral of x from 0 to 1 should be 0.5
    const result = integrate((x) => x, 0.0, 1.0);
    expect(approxEq(result, 0.5, 1e-6)).toBe(true);
  });

  test('integrate quadratic', () => {
    // Integral of x^2 from 0 to 1 should be 1/3
    const result = integrate((x) => x * x, 0.0, 1.0);
    expect(approxEq(result, 1.0 / 3.0, 1e-6)).toBe(true);
  });

  test('integrate sin', () => {
    // Integral of sin(x) from 0 to pi should be 2
    const result = integrate(libm.sin, 0.0, PI);
    expect(approxEq(result, 2.0, 1e-6)).toBe(true);
  });

  test('is math function', () => {
    expect(isMathFunction('sin')).toBe(true);
    expect(isMathFunction('SIN')).toBe(true);
    expect(isMathFunction('sqrt')).toBe(true);
    expect(isMathFunction('pi')).toBe(true);
    expect(isMathFunction('foo')).toBe(false);
    expect(isMathFunction('USD')).toBe(false);
  });
});

describe('math functions vectors (experiments/issue-254/rust-vectors --bin math_functions)', () => {
  const vectors = loadVectors('math_functions');

  /** @param {() => Decimal} f */
  function encode(f) {
    const result = tryResult(f);
    if (result.ok) {
      return /** @type {Decimal} */ (result.value).toString();
    }
    const e = /** @type {any} */ (result.error);
    switch (e.tag) {
      case 'InvalidFunctionArgs':
        return { e: `${e.tag}: ${e.function}: ${e.reason}` };
      case 'Overflow':
        return { e: e.tag };
      default:
        return { e: `${e.tag}: ${e._0}` };
    }
  }

  test('evaluate function', () => {
    for (const [name, args, expected] of vectors.functions) {
      const decimals = args.map((/** @type {string} */ s) =>
        ok(() => Decimal.fromStr(s)) ?? Decimal.fromScientific(s),);
      expect([name, args, encode(() => evaluateFunction(name, decimals))]).toEqual([name, args, expected]);
    }
  });

  test('integrate', () => {
    /** @type {Record<string, (x: number) => number>} */
    const fs = {
      one: () => 1.0,
      x: (x) => x,
      x2: (x) => x * x,
      sin: libm.sin,
      exp: libm.exp,
      sin_over_x: (x) => libm.sin(x) / x,
    };
    for (const [name, a, b, expected] of vectors.integrals) {
      const result = integrate(fs[name], f64FromBits(a), f64FromBits(b));
      expect([name, a, b, f64Bits(result)]).toEqual([name, a, b, expected]);
    }
  });
});
