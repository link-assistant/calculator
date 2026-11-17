import { describe, expect, test } from 'vitest';
import { eq } from '../../src/rust/core.js';
import { Dimension } from '../../src/types/dimension.js';
import { Rational } from '../../src/types/rational.js';
import {
  lookupSymbol,
  lookupUnit,
  namedUnitFor,
  parseNumber,
  units,
} from '../../src/types/unit_db.js';
import { loadVectors, sameF64 } from '../vectors.js';

/** @param {string} name */
function resolved(name) {
  const unit = lookupUnit(name);
  if (unit === null) {
    throw new Error(`unknown unit ${name}`);
  }
  return unit;
}

/** @param {string} name */
const factor = (name) => resolved(name).factor.toF64();
/** @param {string} name */
const symbol = (name) => resolved(name).symbol;

/** @param {Rational} r */
const ratio = (r) => `${r.numerBigint()}/${r.denomBigint()}`;

describe('UnitDatabase (src/types/unit_db.rs)', () => {
  test('database loads', () => {
    expect(units().units.length).toBeGreaterThan(100);
  });

  test('si prefixes including micro spellings', () => {
    for (const name of ['µm', 'μm', 'um', 'micrometre', 'micrometers']) {
      expect(symbol(name), name).toBe('µm');
      expect(Math.abs(factor(name) - 1e-6), name).toBeLessThan(1e-18);
    }
    expect(symbol('kilometres')).toBe('km');
    expect(symbol('dam')).toBe('dam');
    expect(symbol('mL')).toBe('mL');
    expect(symbol('ml')).toBe('mL');
    expect(symbol('kΩ')).toBe('kΩ');
    expect(symbol('Mt')).toBe('Mt');
  });

  test('binary prefixes', () => {
    expect(symbol('KiB')).toBe('KiB');
    expect(Math.abs(factor('KiB') - 8192.0)).toBeLessThan(1e-9);
    expect(symbol('mebibits')).toBe('Mibit');
  });

  test('exact names win over prefixes', () => {
    expect(symbol('ft')).toBe('ft');
    expect(symbol('min')).toBe('min');
    expect(symbol('Pa')).toBe('Pa');
    expect(symbol('cd')).toBe('cd');
    expect(symbol('pt')).toBe('pt');
    expect(lookupUnit('am')).toBe(null);
    expect(lookupUnit('as')).toBe(null);
    expect(lookupUnit('ft2')).toBe(null);
  });

  test('aliases and case', () => {
    expect(symbol('feet')).toBe('ft');
    expect(symbol('Feet')).toBe('ft');
    expect(symbol('metres')).toBe('m');
    expect(symbol('Celsius')).toBe('°C');
    expect(symbol('Ω')).toBe('Ω');
    expect(lookupUnit('CUP')).toBe(null);
  });

  test('definitions', () => {
    expect(Math.abs(factor('ft') - 0.3048)).toBeLessThan(1e-15);
    expect(Math.abs(factor('mi') - 1609.344)).toBeLessThan(1e-9);
    expect(Math.abs(factor('ly') - 9_460_730_472_580_800.0)).toBeLessThan(1.0);
    expect(Math.abs(factor('mph') - 0.44704)).toBeLessThan(1e-12);
    expect(Math.abs(factor('J') - 1.0)).toBeLessThan(1e-15);
    const m = /** @type {Dimension} */ (Dimension.base('length'));
    const kg = /** @type {Dimension} */ (Dimension.base('mass'));
    const s = /** @type {Dimension} */ (Dimension.base('time'));
    expect(eq(resolved('J').dimension(), kg.mul(m.pow(2)).div(s.pow(2)))).toBe(true);
    const fahrenheit = resolved('°F');
    expect(Math.abs(fahrenheit.factor.toF64() - 5.0 / 9.0)).toBeLessThan(1e-15);
    expect(/** @type {Rational} */ (fahrenheit.offset()).toF64()).toBe(459.67);
  });

  test('named units', () => {
    expect(namedUnitFor(resolved('V').dimension())).toBe('V');
    expect(namedUnitFor(resolved('Hz').dimension())).toBe(null);
  });

  test('parse number', () => {
    expect(eq(parseNumber('5/9'), Rational.new(5, 9))).toBe(true);
    expect(eq(parseNumber('0.3048'), Rational.new(3048, 10000))).toBe(true);
    expect(eq(parseNumber('1.5e3'), Rational.fromInteger(1500))).toBe(true);
    expect(parseNumber('m')).toBe(null);
  });
});

describe('Rust vectors: unit lookups', () => {
  test('lookupUnit matches Rust for every alias, case and prefix', () => {
    const { lookups } = loadVectors('units');
    for (const [name, want] of Object.entries(lookups)) {
      const unit = lookupUnit(name);
      if (want === null) {
        expect(unit, name).toBe(null);
        continue;
      }
      expect(unit, name).not.toBe(null);
      const u = /** @type {import('../../src/types/unit_db.js').ResolvedUnit} */ (unit);
      const offset = u.offset();
      const back = lookupSymbol(u.symbol);
      const got = [
        u.symbol,
        ratio(u.factor),
        sameF64(u.factor.toF64(), want[2]) ? want[2] : u.factor.toF64(),
        u.dimension().toString(),
        offset === null ? null : ratio(offset),
        namedUnitFor(u.dimension()),
        back === null ? null : back.symbol,
      ];
      expect(got, name).toEqual(want);
    }
  });
});
