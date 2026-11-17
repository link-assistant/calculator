import { describe, expect, test } from 'vitest';
import { Err, Panic, eq } from '../../src/rust/core.js';
import { DecimalError } from '../../src/rust/decimal.js';
import { Decimal } from '../../src/types/decimal.js';

describe('Decimal (src/types/decimal.rs)', () => {
  test('decimal creation', () => {
    const d = Decimal.new(42);
    expect(d.toString()).toBe('42');
  });

  test('decimal from str', () => {
    const d = Decimal.fromStr('3.14159');
    expect(d.toString().startsWith('3.14')).toBe(true);
  });

  test('decimal operations', () => {
    const a = Decimal.new(10);
    const b = Decimal.new(3);

    expect(a.add(b).toString()).toBe('13');
    expect(a.sub(b).toString()).toBe('7');
    expect(a.mul(b).toString()).toBe('30');
  });

  test('decimal division', () => {
    const a = Decimal.new(10);
    const b = Decimal.new(4);
    expect(a.div(b).toString()).toBe('2.5');
  });

  test('decimal checked div', () => {
    const a = Decimal.new(10);
    const zero = Decimal.zero();
    expect(a.checkedDiv(zero)).toBeNull();
  });

  test('decimal is negative', () => {
    const pos = Decimal.new(5);
    const neg = Decimal.new(-5);
    const zero = Decimal.zero();

    expect(pos.isNegative()).toBe(false);
    expect(neg.isNegative()).toBe(true);
    expect(zero.isNegative()).toBe(false);
  });
});

describe('Decimal wrapper semantics', () => {
  test('Display normalizes; Debug and Serialize do not', () => {
    const d = Decimal.new(10).div(Decimal.new(4));
    expect(d.toString()).toBe('2.5');
    expect(d.fmtDebug()).toBe('Decimal(2.50)');
    expect(JSON.stringify(d)).toBe('"2.50"');
    expect(Decimal.zero().neg().toString()).toBe('0');
    expect(Decimal.zero().neg().fmtDebug()).toBe('Decimal(-0)');
    expect(Decimal.zero().neg().isNegative()).toBe(false);
  });

  test('constructors', () => {
    expect(Decimal.one().toString()).toBe('1');
    expect(Decimal.default().toString()).toBe('0');
    expect(Decimal.fromI64(-7n).toString()).toBe('-7');
    expect(Decimal.fromI32(-7).toString()).toBe('-7');
    expect(Decimal.fromScientific('1.5e-6').toString()).toBe('0.0000015');
    expect(Decimal.fromStr('1.5E3').toString()).toBe('1500');
    expect(Object.isFrozen(Decimal.one())).toBe(true);
  });

  test('parse errors', () => {
    const thrown = (/** @type {() => unknown} */ fn) => {
      try {
        fn();
      } catch (e) {
        return e;
      }
      return undefined;
    };
    const e = thrown(() => Decimal.fromStr('abc'));
    expect(e).toBeInstanceOf(Err);
    expect(/** @type {Err} */ (e).error).toBeInstanceOf(DecimalError);
    expect(String(/** @type {Err} */ (e).error)).toBe('Invalid decimal: unknown character');
    const s = thrown(() => Decimal.fromScientific('1e29'));
    expect(/** @type {DecimalError} */ (/** @type {Err} */ (s).error).fmtDebug()).toBe('ScaleExceedsMaximumPrecision(29)');
  });

  test('floats', () => {
    expect(Decimal.tryFromF64(0.1).toString()).toBe('0.1');
    expect(Decimal.tryFromF64(Number.NaN)).toBeNull();
    expect(Decimal.fromF64(Number.POSITIVE_INFINITY).toString()).toBe('0');
    expect(Decimal.fromF64(1e30).toString()).toBe('0');
    expect(Decimal.fromStr('0.1').toF64()).toBe(0.1);
    expect(Decimal.fromStr('-2.5').abs().toF64()).toBe(2.5);
  });

  test('rounding and normalization', () => {
    expect(Decimal.fromStr('2.345').round(2).fmtDebug()).toBe('Decimal(2.34)');
    expect(Decimal.fromStr('2.355').round(2).fmtDebug()).toBe('Decimal(2.36)');
    expect(Decimal.fromStr('1.500').normalize().fmtDebug()).toBe('Decimal(1.5)');
  });

  test('checked operations', () => {
    const max = Decimal.fromStr('79228162514264337593543950335');
    expect(max.checkedMul(Decimal.new(2))).toBeNull();
    expect(Decimal.new(6).checkedMul(Decimal.new(7)).toString()).toBe('42');
    expect(Decimal.new(1).checkedDiv(Decimal.new(3)).toString()).toBe('0.3333333333333333333333333333');
    expect(Decimal.new(7).checkedRem(Decimal.new(3)).toString()).toBe('1');
    expect(Decimal.new(7).checkedRem(Decimal.zero())).toBeNull();
    expect(() => max.add(max)).toThrow(Panic);
    expect(() => Decimal.new(1).div(Decimal.zero())).toThrow('Division by zero');
  });

  test('comparison and equality', () => {
    const a = Decimal.fromStr('1.0');
    const b = Decimal.new(1);
    expect(a.equals(b)).toBe(true);
    expect(eq(a, b)).toBe(true);
    expect(a.cmp(Decimal.new(2))).toBe(-1);
    expect(Decimal.new(2).cmp(a)).toBe(1);
    expect(a.neg().toString()).toBe('-1');
  });
});
