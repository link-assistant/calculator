import { describe, expect, test } from 'vitest';
import { Err, tryResult } from '../../src/rust/core.js';
import {
  fmax,
  fmin,
  I128,
  I16,
  I32,
  I64,
  I8,
  intToF64,
  ParseFloatError,
  ParseIntError,
  parseF64,
  remEuclid,
  round,
  signum,
  toDegrees,
  toRadians,
  U128,
  U32,
  U64,
  U8,
} from '../../src/rust/num.js';
import { f64Bits, f64FromBits, loadVectors, sameF64 } from '../vectors.js';

const vectors = loadVectors('std');

/** @type {Record<string, import('../../src/rust/num.js').IntType>} */
const INT_TYPES = { i8: I8, u8: U8, i32: I32, u32: U32, i64: I64, u64: U64, i128: I128, u128: U128 };

describe('f64 methods against Rust', () => {
  test('round, signum, to_degrees, to_radians, max, min and `as` casts', () => {
    const mismatches = [];
    for (const c of vectors.floats) {
      const x = f64FromBits(c.bits);
      const checks = {
        round: sameF64(round(x), c.round),
        signum: sameF64(signum(x), c.signum),
        degrees: sameF64(toDegrees(x), c.degrees),
        radians: sameF64(toRadians(x), c.radians),
        max_nan: sameF64(fmax(x, Number.NaN), c.max_nan),
        min_one: sameF64(fmin(x, 1), c.min_one),
        as_i64: I64.fromF64(x).toString() === c.as_i64,
        as_i32: I32.fromF64(x).toString() === c.as_i32,
        as_u32: U32.fromF64(x).toString() === c.as_u32,
        as_u64: U64.fromF64(x).toString() === c.as_u64,
        as_u8: U8.fromF64(x).toString() === c.as_u8,
        as_i128: I128.fromF64(x).toString() === c.as_i128,
      };
      for (const [name, passed] of Object.entries(checks)) {
        if (!passed) {
          mismatches.push({ bits: c.bits, name });
        }
      }
    }
    expect(mismatches).toEqual([]);
    expect(vectors.floats.length).toBeGreaterThan(700);
  });

  test('round is half away from zero and keeps the sign of zero', () => {
    expect(round(2.5)).toBe(3);
    expect(round(-2.5)).toBe(-3);
    expect(round(0.49999999999999994)).toBe(0);
    expect(Object.is(round(-0.4), -0)).toBe(true);
  });
});

describe('str::parse::<f64>() against Rust', () => {
  test('values and errors', () => {
    const mismatches = [];
    for (const c of vectors.parse_f64) {
      const result = tryResult(() => parseF64(c.input));
      const passed = result.ok ? c.ok !== undefined && sameF64(result.value, c.ok) : String(result.error) === c.err;
      if (!passed) {
        mismatches.push({ input: c.input, expected: c.ok ?? c.err, actual: result.ok ? f64Bits(result.value) : String(result.error) });
      }
    }
    expect(mismatches).toEqual([]);
  });

  test('errors are ParseFloatError values thrown as Err', () => {
    expect(() => parseF64('')).toThrow(Err);
    expect(tryResult(() => parseF64('')).error).toEqual(new ParseFloatError('Empty'));
    expect(tryResult(() => parseF64('1 ')).error.fmtDebug()).toBe('ParseFloatError { kind: Invalid }');
  });
});

describe('str::parse::<integer>() against Rust', () => {
  test('every integer type', () => {
    const mismatches = [];
    for (const c of vectors.parse_int) {
      for (const [name, type] of Object.entries(INT_TYPES)) {
        const result = tryResult(() => type.parse(c.input));
        const actual = result.ok ? { ok: result.value.toString() } : { err: String(result.error) };
        if (JSON.stringify(actual) !== JSON.stringify(c[name])) {
          mismatches.push({ input: c.input, type: name, expected: c[name], actual });
        }
      }
    }
    expect(mismatches).toEqual([]);
  });

  test('ParseIntError formats as Rust', () => {
    const error = tryResult(() => I32.parse('x')).error;
    expect(error).toEqual(new ParseIntError('InvalidDigit'));
    expect(error.fmtDebug()).toBe('ParseIntError { kind: InvalidDigit }');
    expect(I32.fromStrRadix('-ff', 16)).toBe(-255n);
  });
});

describe('IntType arithmetic', () => {
  test('checked operations return null on overflow', () => {
    expect(I8.checkedAdd(127n, 1n)).toBeNull();
    expect(I8.checkedSub(-128n, 1n)).toBeNull();
    expect(I8.checkedMul(16n, 8n)).toBeNull();
    expect(I8.checkedDiv(-128n, -1n)).toBeNull();
    expect(I8.checkedDiv(1n, 0n)).toBeNull();
    expect(I8.checkedRem(-128n, -1n)).toBeNull();
    expect(I8.checkedRem(-7n, 2n)).toBe(-1n);
    expect(I8.checkedAbs(-128n)).toBeNull();
    expect(I8.checkedNeg(-128n)).toBeNull();
    expect(U8.checkedNeg(0n)).toBe(0n);
    expect(U8.checkedNeg(1n)).toBeNull();
    expect(I64.checkedPow(10n, 18)).toBe(10n ** 18n);
    expect(I64.checkedPow(10n, 19)).toBeNull();
    expect(I64.checkedPow(-2n, 63)).toBe(I64.MIN);
    expect(I64.checkedPow(7n, 0)).toBe(1n);
  });

  test('saturating, wrapping and conversions', () => {
    expect(I16.saturatingAdd(32767n, 1n)).toBe(32767n);
    expect(U8.saturatingSub(0n, 1n)).toBe(0n);
    expect(I8.saturatingMul(-100n, 2n)).toBe(-128n);
    expect(U8.wrappingAdd(255n, 1n)).toBe(0n);
    expect(I8.wrappingSub(-128n, 1n)).toBe(127n);
    expect(I32.wrappingMul(65536n, 65536n)).toBe(0n);
    expect(I8.wrap(200)).toBe(-56n);
    expect(U32.tryFrom(-1)).toBeNull();
    expect(U32.tryFrom(4294967295n)).toBe(4294967295n);
  });

  test('i128 as f64 rounds to nearest, ties to even', () => {
    expect(intToF64(9007199254740993n)).toBe(9007199254740992);
    expect(intToF64(9007199254740995n)).toBe(9007199254740996);
  });

  test('rem_euclid is never negative', () => {
    expect(remEuclid(-7, 3)).toBe(2);
    expect(remEuclid(7, -3)).toBe(1);
    expect(remEuclid(-7n, -3n)).toBe(2n);
    expect(Object.is(remEuclid(-6, 3), 0)).toBe(true);
  });
});
