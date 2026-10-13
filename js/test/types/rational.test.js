import { describe, expect, test } from 'vitest';
import { Err, Panic } from '../../src/rust/core.js';
import { Ratio } from '../../src/rust/ratio.js';
import { Decimal } from '../../src/types/decimal.js';
import { Rational } from '../../src/types/rational.js';
import { f64Bits, f64FromBits, loadVectors } from '../vectors.js';

describe('Rational (src/types/rational.rs)', () => {
  test('rational creation', () => {
    const r = Rational.new(1, 3);
    expect(r.numer()).toBe(1n);
    expect(r.denom()).toBe(3n);
  });

  test('rational from integer', () => {
    const r = Rational.fromInteger(42);
    expect(r.isInteger()).toBe(true);
    expect(r.numer()).toBe(42n);
    expect(r.denom()).toBe(1n);
  });

  test('rational simplification', () => {
    const r = Rational.new(2, 6);
    expect(r.numer()).toBe(1n);
    expect(r.denom()).toBe(3n);
  });

  test('one third times three', () => {
    const oneThird = Rational.new(1, 3);
    const three = Rational.fromInteger(3);
    const result = oneThird.mul(three);
    expect(result.isInteger()).toBe(true);
    expect(result.numer()).toBe(1n);
    expect(result.toDisplayString()).toBe('1');
  });

  test('two thirds times three', () => {
    const result = Rational.new(2, 3).mul(Rational.fromInteger(3));
    expect(result.isInteger()).toBe(true);
    expect(result.numer()).toBe(2n);
  });

  test('one sixth times six', () => {
    const result = Rational.new(1, 6).mul(Rational.fromInteger(6));
    expect(result.isInteger()).toBe(true);
    expect(result.numer()).toBe(1n);
  });

  test('repeating decimal one third', () => {
    const rd = Rational.new(1, 3).toRepeatingDecimalNotation();
    expect(rd?.integerPart).toBe('0');
    expect(rd?.nonRepeating).toBe('');
    expect(rd?.repeating).toBe('3');
    expect(rd?.toParenthesisNotation()).toBe('0.(3)');
  });

  test('repeating decimal one sixth', () => {
    const rd = Rational.new(1, 6).toRepeatingDecimalNotation();
    expect(rd?.integerPart).toBe('0');
    expect(rd?.nonRepeating).toBe('1');
    expect(rd?.repeating).toBe('6');
    expect(rd?.toParenthesisNotation()).toBe('0.1(6)');
  });

  test('repeating decimal one seventh', () => {
    const rd = Rational.new(1, 7).toRepeatingDecimalNotation();
    expect(rd?.integerPart).toBe('0');
    expect(rd?.nonRepeating).toBe('');
    expect(rd?.repeating).toBe('142857');
    expect(rd?.toParenthesisNotation()).toBe('0.(142857)');
  });

  test('terminating decimal', () => {
    const rd = Rational.new(1, 4).toRepeatingDecimalNotation();
    expect(rd?.repeating).toBe('');
    expect(rd?.nonRepeating).toBe('25');
  });

  test('fraction string', () => {
    expect(Rational.new(1, 3).toFractionString()).toBe('1/3');
  });

  test('from decimal', () => {
    const r = Rational.fromDecimal(Decimal.fromF64(0.5));
    expect(r.numer()).toBe(1n);
    expect(r.denom()).toBe(2n);
  });

  test('checked div by zero', () => {
    expect(Rational.fromInteger(10).checkedDiv(Rational.zero())).toBeNull();
  });

  test('negative rational', () => {
    const r = Rational.new(-1, 3);
    expect(r.isNegative()).toBe(true);
    expect(r.toRepeatingDecimalNotation()?.isNegative).toBe(true);
  });

  test('big power', () => {
    const result = Rational.fromInteger(10).powI32(100);
    expect(result.isInteger()).toBe(true);
    const s = result.toDisplayString();
    expect(s.startsWith('1')).toBe(true);
    expect(s.length).toBe(101); // 1 followed by 100 zeros
    expect([...s.slice(1)].every((c) => c === '0')).toBe(true);
  });

  test('negative power', () => {
    const result = Rational.fromInteger(2).powI32(-1);
    expect(result.numer()).toBe(1n);
    expect(result.denom()).toBe(2n);
  });
});

describe('Rational semantics', () => {
  test('panics and saturation', () => {
    expect(() => Rational.new(1, 0)).toThrow(new Panic('Denominator cannot be zero'));
    expect(() => Rational.zero().powI32(-1)).toThrow(Panic);
    const huge = Rational.fromBigint(1n << 200n);
    expect(huge.numer()).toBe((1n << 127n) - 1n);
    expect(huge.neg().numer()).toBe(-(1n << 127n));
  });

  test('debug, serde, ordering', () => {
    const r = Rational.new(-2, 4);
    expect(r.fmtDebug()).toBe('Rational { inner: Ratio { numer: -1, denom: 2 } }');
    expect(JSON.stringify(r)).toBe('{"inner":["-1","2"]}');
    expect(Rational.fromJSON({ inner: ['2', '-4'] }).equals(r)).toBe(true);
    expect(() => Rational.fromJSON({ inner: ['x', '1'] })).toThrow(Err);
    expect(r.cmp(Rational.zero())).toBe(-1);
    expect(Rational.default().isZero()).toBe(true);
    expect(Rational.fromI32(3).toString()).toBe('3');
  });
});

describe('Rust vectors (experiments/issue-254/rust-vectors --bin rational)', () => {
  const vectors = loadVectors('rational');

  /** @param {Rational} x */
  const enc = (x) => `${x.numerBigint()}/${x.denomBigint()}`;
  /** @param {string} s */
  const dec = (s) => {
    const i = s.indexOf('/');
    return new Rational({ inner: Ratio.newRaw(BigInt(s.slice(0, i)), BigInt(s.slice(i + 1))) });
  };
  /** @param {() => Rational} fn */
  const run = (fn) => {
    try {
      return enc(fn());
    } catch (e) {
      if (e instanceof Panic) {
        return `!${e.message}`;
      }
      throw e;
    }
  };

  test('from_f64', () => {
    for (const [bits, expected] of vectors.from_f64) {
      expect([bits, run(() => Rational.fromF64(f64FromBits(bits)))]).toEqual([bits, expected]);
    }
  });

  test('unary', () => {
    for (const row of vectors.unary) {
      const x = dec(row.x);
      const rd = x.toRepeatingDecimalNotation();
      expect({
        x: row.x,
        display: x.toDisplayString(),
        fraction: x.toFractionString(),
        f64: f64Bits(x.toF64()),
        decimal: x.toDecimal().toString(),
        numer: x.numer().toString(),
        denom: x.denom().toString(),
        repeating: rd === null ? null : [
          rd.toVinculumNotation(),
          rd.toParenthesisNotation(),
          rd.toEllipsisNotation(),
          rd.toLatex(),
        ],
        json: JSON.parse(JSON.stringify(x)),
        pow: [-3, -1, 2, 3].map((e) => run(() => x.powI32(e))),
      }).toEqual(row);
    }
  });

  test('binary', () => {
    for (const row of vectors.binary) {
      const a = dec(row.a);
      const b = dec(row.b);
      const div = a.checkedDiv(b);
      const rem = a.remainder(b);
      expect({
        a: row.a,
        b: row.b,
        add: enc(a.add(b)),
        sub: enc(a.sub(b)),
        mul: enc(a.mul(b)),
        div: div === null ? null : enc(div),
        rem: rem === null ? null : enc(rem),
        cmp: a.cmp(b),
      }).toEqual(row);
    }
  });

  test('from_str', () => {
    for (const [s, expected] of vectors.from_str) {
      let actual;
      try {
        actual = enc(Rational.fromStr(s));
      } catch (e) {
        if (!(e instanceof Err)) {
          throw e;
        }
        actual = { e: e.error };
      }
      expect([s, actual]).toEqual([s, expected]);
    }
  });
});
