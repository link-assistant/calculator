import { describe, expect, test } from 'vitest';
import { Err, Panic } from '../../src/rust/core.js';
import { DecimalError, RustDecimal } from '../../src/rust/decimal.js';
import { f64Bits, f64FromBits, loadVectors } from '../vectors.js';

const vectors = loadVectors('decimal');

/**
 * A vector's "[-]mantissa:scale" as a raw `RustDecimal`.
 * @param {string} s
 */
function dec(s) {
  const negative = s.startsWith('-');
  const [mantissa, scale] = (negative ? s.slice(1) : s).split(':');
  return new RustDecimal(BigInt(mantissa), Number(scale), negative);
}

/**
 * How the vectors write a decimal: its raw parts, the sign flag as "-".
 * @param {RustDecimal} d
 */
function enc(d) {
  return `${d.negative ? '-' : ''}${d.magnitude}:${d.places}`;
}

/**
 * A `Result<Decimal, Error>` column: the parts, or `{ d: Debug, e: Display }`.
 * @param {() => RustDecimal} fn
 */
function result(fn) {
  try {
    return enc(fn());
  } catch (thrown) {
    if (thrown instanceof Err && thrown.error instanceof DecimalError) {
      return { d: thrown.error.fmtDebug(), e: thrown.error.toString() };
    }
    throw thrown;
  }
}

/**
 * An operator column: the parts, or "!" and the panic message; also checks
 * that the `checked_*` form agrees (`null` exactly when the operator panics).
 * @param {() => RustDecimal} operator
 * @param {() => RustDecimal | null} checked
 */
function op(operator, checked) {
  const c = checked();
  try {
    const value = enc(operator());
    return c !== null && enc(c) === value ? value : `checked: ${c && enc(c)}`;
  } catch (thrown) {
    if (thrown instanceof Panic) {
      return c === null ? `!${thrown.message}` : `checked: ${enc(c)}`;
    }
    throw thrown;
  }
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

describe('RustDecimal against rust_decimal 1.43.0', () => {
  test('FromStr', () => {
    expect(vectors.parse.length).toBeGreaterThan(2000);
    expect(mismatches(vectors.parse, ([s]) => [result(() => RustDecimal.fromStr(s))])).toEqual([]);
  });

  test('from_scientific and the FromStr fallback', () => {
    expect(vectors.scientific.length).toBeGreaterThan(1000);
    expect(mismatches(vectors.scientific, ([s]) => [
      result(() => RustDecimal.fromScientific(s)),
      result(() => RustDecimal.fromStr(s)),
    ])).toEqual([]);
  });

  test('try_from(f64)', () => {
    expect(vectors.from_f64.length).toBeGreaterThan(2000);
    expect(mismatches(vectors.from_f64, ([h]) => {
      const d = RustDecimal.tryFromF64(f64FromBits(h));
      return [d === null ? null : enc(d)];
    })).toEqual([]);
  });

  test('Display, Serialize, to_f64, normalize, round_dp', () => {
    expect(vectors.unary.length).toBeGreaterThan(2000);
    expect(mismatches(vectors.unary, ([s, , , , , rounds]) => {
      const d = dec(s);
      const f = d.toF64();
      return [
        d.toString(),
        JSON.parse(JSON.stringify(d)),
        f === null ? null : f64Bits(f),
        enc(d.normalize()),
        rounds.map(([dp]) => [dp, enc(d.roundDp(dp))]),
      ];
    })).toEqual([]);
  });

  test('Debug', () => {
    expect(mismatches(vectors.debug, ([s]) => {
      const d = dec(s);
      return [d.fmtDebug(), d.toString()];
    })).toEqual([]);
  });

  test('add, sub, mul, div, rem and their checked forms; cmp', () => {
    expect(vectors.binary.length).toBeGreaterThan(3000);
    expect(mismatches(vectors.binary, ([x, y]) => {
      const a = dec(x);
      const b = dec(y);
      return [
        op(() => a.add(b), () => a.checkedAdd(b)),
        op(() => a.sub(b), () => a.checkedSub(b)),
        op(() => a.mul(b), () => a.checkedMul(b)),
        op(() => a.div(b), () => a.checkedDiv(b)),
        op(() => a.rem(b), () => a.checkedRem(b)),
        a.cmp(b),
      ];
    }, 2)).toEqual([]);
  });
});

describe('RustDecimal API', () => {
  test('constants and integer conversions', () => {
    expect(RustDecimal.ZERO.toString()).toBe('0');
    expect(RustDecimal.ONE.toString()).toBe('1');
    expect(enc(RustDecimal.fromI64(-42))).toBe('-42:0');
    expect(enc(RustDecimal.fromI64(-9223372036854775808n))).toBe('-9223372036854775808:0');
    expect(enc(RustDecimal.fromI32(7))).toBe('7:0');
    expect(Object.isFrozen(RustDecimal.fromI32(7))).toBe(true);
  });

  test('accessors', () => {
    const d = dec('-12345:3');
    expect(d.scale()).toBe(3);
    expect(d.mantissa()).toBe(-12345n);
    expect(d.isSignNegative()).toBe(true);
    expect(d.isZero()).toBe(false);
    expect(d.abs().toString()).toBe('12.345');
    expect(d.neg().toString()).toBe('12.345');
    expect(RustDecimal.ZERO.neg().toString()).toBe('-0');
  });

  test('equality is numeric', () => {
    expect(dec('10:1').equals(RustDecimal.ONE)).toBe(true);
    expect(dec('-0:2').equals(RustDecimal.ZERO)).toBe(true);
    expect(dec('1:1').equals(RustDecimal.ONE)).toBe(false);
    expect(dec('1:1').cmp(RustDecimal.ONE)).toBe(-1);
  });

  test('DecimalError', () => {
    expect(DecimalError.ExceedsMaximumPossibleValue.toString()).toBe(
      'Number exceeds maximum value that can be represented.',
    );
    expect(DecimalError.LessThanMinimumPossibleValue.toString()).toBe(
      'Number less than minimum value that can be represented.',
    );
    expect(DecimalError.Underflow.toString()).toBe('Number has a high precision that can not be represented.');
    expect(DecimalError.ScaleExceedsMaximumPrecision(29).toString()).toBe(
      'Scale exceeds the maximum precision allowed: 29 > 28',
    );
    expect(DecimalError.ScaleExceedsMaximumPrecision(29).fmtDebug()).toBe('ScaleExceedsMaximumPrecision(29)');
    expect(DecimalError.ConversionTo('f64').toString()).toBe('Error while converting to f64');
    expect(DecimalError.ConversionTo('f64').fmtDebug()).toBe('ConversionTo("f64")');
    expect(DecimalError.ErrorString('a "b"').fmtDebug()).toBe('ErrorString("a \\"b\\"")');
    expect(DecimalError.Underflow.fmtDebug()).toBe('Underflow');
    expect(DecimalError.Underflow.tag).toBe('Underflow');
  });
});
