import { describe, expect, test } from 'vitest';
import { Err, Panic } from '../../src/rust/core.js';
import {
  bigintDiv,
  bigintDivFloor,
  bigintDivRem,
  bigintFromF64,
  bigintFromStr,
  bigintIsEven,
  bigintModFloor,
  bigintPow,
  bigintRem,
  bigintToF64,
  bigintToI128,
  bigintToI64,
  bigintToU128,
  bigintToU64,
  gcd,
  lcm,
  ParseBigIntError,
  Ratio,
} from '../../src/rust/ratio.js';
import { f64Bits, f64FromBits, loadVectors } from '../vectors.js';

const vectors = loadVectors('ratio');

/**
 * A vector's "numer/denom" as a raw `Ratio`.
 * @param {string} s
 */
function ratio(s) {
  const slash = s.indexOf('/');
  return Ratio.newRaw(BigInt(s.slice(0, slash)), BigInt(s.slice(slash + 1)));
}

/**
 * How the vectors write a value: "numer/denom", a bigint as a string, an
 * `Option` as `null`, an f64 as its bits, a bool or `Ordering` as a number.
 * @param {unknown} value
 * @returns {unknown}
 */
function encode(value) {
  if (value instanceof Ratio) {
    return `${value.numer()}/${value.denom()}`;
  }
  if (typeof value === 'bigint') {
    return value.toString();
  }
  if (typeof value === 'boolean') {
    return value ? 1 : 0;
  }
  return value;
}

/**
 * `encode(fn())`, or "!" and the message when it panics.
 * @param {() => unknown} fn
 */
function run(fn) {
  try {
    return encode(fn());
  } catch (thrown) {
    if (thrown instanceof Panic) {
      return `!${thrown.message}`;
    }
    throw thrown;
  }
}

/**
 * The f64 column: its bits, or `null` for `None`.
 * @param {number | null} x
 */
function hex(x) {
  return x === null ? null : f64Bits(x);
}

/**
 * Compares each row of `rows` with `actual(row)` and returns the mismatches.
 * @param {any[][]} rows
 * @param {(row: any[]) => unknown[]} actual
 * @param {number} [inputs] how many leading columns are inputs
 */
function mismatches(rows, actual, inputs = 1) {
  const out = [];
  for (const row of rows) {
    const got = [...row.slice(0, inputs), ...actual(row)];
    if (JSON.stringify(got) !== JSON.stringify(row)) {
      out.push({ expected: row, got });
    }
  }
  return out;
}

describe('BigInt helpers against num-bigint 0.5.1', () => {
  test('to_f64', () => {
    expect(vectors.bigint_to_f64.length).toBeGreaterThan(700);
    expect(mismatches(vectors.bigint_to_f64, ([n]) => [f64Bits(bigintToF64(BigInt(n)))])).toEqual([]);
  });

  test('from_f64', () => {
    expect(vectors.bigint_from_f64.length).toBeGreaterThan(400);
    expect(mismatches(vectors.bigint_from_f64, ([h]) => [encode(bigintFromF64(f64FromBits(h)))])).toEqual([]);
  });

  test('to_i64, to_i128, to_u64, to_u128', () => {
    expect(mismatches(vectors.bigint_to_int, ([s]) => {
      const n = BigInt(s);
      return [bigintToI64(n), bigintToI128(n), bigintToU64(n), bigintToU128(n)].map(encode);
    })).toEqual([]);
  });

  test('FromStr', () => {
    expect(vectors.bigint_parse.length).toBeGreaterThan(150);
    expect(mismatches(vectors.bigint_parse, ([s]) => {
      try {
        return [bigintFromStr(s).toString(), null, null];
      } catch (thrown) {
        if (!(thrown instanceof Err) || !(thrown.error instanceof ParseBigIntError)) {
          throw thrown;
        }
        return [null, thrown.error.toString(), thrown.error.fmtDebug()];
      }
    })).toEqual([]);
  });

  test('ParseBigIntError Display and Debug', () => {
    expect(mismatches(vectors.bigint_parse_errors, ([kind]) => {
      const error = new ParseBigIntError(kind);
      return [error.toString(), error.fmtDebug()];
    })).toEqual([]);
    expect(new ParseBigIntError('Empty').equals(new ParseBigIntError('Empty'))).toBe(true);
    expect(new ParseBigIntError('Empty').equals(new ParseBigIntError('InvalidDigit'))).toBe(false);
  });

  test('pow', () => {
    expect(mismatches(vectors.bigint_pow, ([b, e]) => [bigintPow(BigInt(b), e).toString()], 2)).toEqual([]);
  });

  test('gcd, lcm, div_floor, mod_floor, div_rem, /, %, is_even', () => {
    expect(vectors.bigint_int.length).toBeGreaterThan(300);
    expect(mismatches(vectors.bigint_int, ([as, bs]) => {
      const a = BigInt(as);
      const b = BigInt(bs);
      return [
        run(() => gcd(a, b)),
        run(() => lcm(a, b)),
        run(() => bigintDivFloor(a, b)),
        run(() => bigintModFloor(a, b)),
        run(() => bigintDivRem(a, b)[0]),
        run(() => bigintDivRem(a, b)[1]),
        run(() => bigintDiv(a, b)),
        run(() => bigintRem(a, b)),
        encode(bigintIsEven(a)),
      ];
    }, 2)).toEqual([]);
  });
});

describe('Ratio<BigInt> against num-rational 0.4.2', () => {
  test('new', () => {
    expect(mismatches(vectors.ratio_new, ([n, d]) => [run(() => Ratio.new(BigInt(n), BigInt(d)))], 2))
      .toEqual([]);
  });

  test('unary methods, predicates, formatting and conversions', () => {
    expect(vectors.ratio_unary.length).toBeGreaterThan(700);
    expect(mismatches(vectors.ratio_unary, ([s]) => {
      const r = ratio(s);
      return [
        run(() => r.neg()),
        run(() => r.abs()),
        run(() => r.signum()),
        run(() => r.recip()),
        run(() => r.floor()),
        run(() => r.ceil()),
        run(() => r.round()),
        run(() => r.trunc()),
        run(() => r.fract()),
        run(() => r.toInteger()),
        encode(r.isInteger()),
        encode(r.isZero()),
        encode(r.isPositive()),
        encode(r.isNegative()),
        encode(r.isOne()),
        r.toString(),
        r.fmtDebug(),
        hex(r.toF64()),
        run(() => r.toI64()),
        run(() => r.toU64()),
        run(() => r.toI128()),
        run(() => r.toU128()),
      ];
    })).toEqual([]);
  });

  test('Ratio op Ratio, cmp and eq', () => {
    expect(vectors.ratio_binary.length).toBeGreaterThan(1300);
    expect(mismatches(vectors.ratio_binary, ([as, bs]) => {
      const a = ratio(as);
      const b = ratio(bs);
      return [
        run(() => a.add(b)),
        run(() => a.sub(b)),
        run(() => a.mul(b)),
        run(() => a.div(b)),
        run(() => a.rem(b)),
        run(() => a.cmp(b)),
        run(() => a.equals(b)),
      ];
    }, 2)).toEqual([]);
  });

  test('Ratio op BigInt', () => {
    expect(mismatches(vectors.ratio_binary_int, ([as, ts]) => {
      const a = ratio(as);
      const t = BigInt(ts);
      return [
        run(() => a.add(t)),
        run(() => a.sub(t)),
        run(() => a.mul(t)),
        run(() => a.div(t)),
        run(() => a.rem(t)),
      ];
    }, 2)).toEqual([]);
  });

  test('Pow<i32> and Pow<u32>', () => {
    for (const rows of [vectors.ratio_pow, vectors.ratio_pow_u32]) {
      expect(mismatches(rows, ([s, e]) => [run(() => ratio(s).pow(e))], 2)).toEqual([]);
    }
  });

  test('to_f64 (ratio_to_f64)', () => {
    expect(vectors.ratio_to_f64.length).toBeGreaterThan(700);
    expect(mismatches(vectors.ratio_to_f64, ([s]) => [hex(ratio(s).toF64())])).toEqual([]);
  });

  test('values are frozen and shared constants stay intact', () => {
    const r = Ratio.new(6n, -4n);
    expect(Object.isFrozen(r)).toBe(true);
    expect(r.toString()).toBe('-3/2');
    expect(Ratio.zero().fmtDebug()).toBe('Ratio { numer: 0, denom: 1 }');
    expect(Ratio.one().toString()).toBe('1');
    expect(Ratio.fromInteger(-7n).isInteger()).toBe(true);
    expect(r.equals(Ratio.newRaw(-6n, 4n))).toBe(true);
    expect(r.equals(-3n)).toBe(false);
  });
});
