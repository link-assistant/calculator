import { describe, expect, test } from 'vitest';
import { eq } from '../../src/rust/core.js';
import { Dimension } from '../../src/types/dimension.js';
import { PhysicalUnit } from '../../src/types/physical.js';
import { DataSizeUnit, DurationUnit, MassUnit, Unit } from '../../src/types/unit.js';
import { loadVectors, sameF64 } from '../vectors.js';

/** @param {[string, number][]} terms */
const unit = (terms) => PhysicalUnit.fromTerms(terms);

/**
 * @param {[string, number][]} terms
 * @returns {[string, number]}
 */
function simplified(terms) {
  const [u, scale] = unit(terms).simplify();
  return [u.toString(), scale.toF64()];
}

/** @param {string} name */
function parsed(name) {
  const u = PhysicalUnit.parse(name);
  if (u === null) {
    throw new Error(`unknown unit ${name}`);
  }
  return u;
}

describe('PhysicalUnit (src/types/physical.rs)', () => {
  test('display uses ascii powers', () => {
    expect(unit([['kg', 1], ['m', 1], ['s', -2]]).toString()).toBe('kg m/s^2');
    expect(unit([['kg', -1]]).toString()).toBe('kg^-1');
    expect(unit([['m', 1], ['s', -1], ['kg', -1]]).toString()).toBe('m s^-1 kg^-1');
    expect(unit([['km', 1], ['day', -1]]).toString()).toBe('km/day');
  });

  test('simplify merges symbols and dimensions', () => {
    expect(simplified([['m', 1], ['m', 1]])).toEqual(['m^2', 1.0]);
    expect(simplified([['cm', 1], ['m', 1]])).toEqual(['cm^2', 100.0]);
    expect(simplified([['m', 1], ['cm', -1]])).toEqual(['', 100.0]);
    const [text, scale] = simplified([['lb', -1], ['kg', -1]]);
    expect(text).toBe('lb^-2');
    expect(Math.abs(scale - 0.453_592_37)).toBeLessThan(1e-12);
  });

  test('simplify expands and collapses', () => {
    expect(simplified([['mph', 1], ['h', 1]])).toEqual(['mi', 1.0]);
    expect(simplified([['V', 1], ['Ω', -1]])).toEqual(['A', 1.0]);
    expect(simplified([['A', 1], ['Ω', 1]])).toEqual(['V', 1.0]);
    expect(simplified([['kg', 1], ['m', 1], ['s', -2]])).toEqual(['kg m/s^2', 1.0]);
    expect(simplified([['kW', 1], ['h', 1]])).toEqual(['kW h', 1.0]);
  });

  test('legacy bridge round trips', () => {
    for (const u of [
      Unit.Mass(MassUnit.Pound),
      Unit.Duration(DurationUnit.Days),
      Unit.DataSize(DataSizeUnit.Kibibyte),
      Unit.DataSize(DataSizeUnit.Kilobit),
    ]) {
      const physical = PhysicalUnit.fromUnit(u);
      expect(physical).not.toBe(null);
      expect(eq(/** @type {PhysicalUnit} */ (physical).intoUnit(), u)).toBe(true);
    }
    expect(PhysicalUnit.fromUnit(Unit.currency('USD'))).toBe(null);
  });

  // tests/issue_235_dimensional_units_tests.rs
  test('unit database exposes dimensions', () => {
    const newton = parsed('N');
    const product = parsed('kg').product(parsed('m')).product(parsed('s').powi(-2));
    expect(eq(newton.dimension(), product.dimension())).toBe(true);
    expect(eq(parsed('ft').dimension(), Dimension.base('length'))).toBe(true);
    expect(parsed('°C').hasAffine()).toBe(true);
  });
});

describe('Rust vectors: unit products', () => {
  test('product, simplify, root and intoUnit match Rust', () => {
    const v = loadVectors('units');
    const sample = v.sample;
    let k = 0;
    for (let i = 0; i < sample.length; i += 1) {
      for (let j = 0; j < sample.length; j += 1) {
        for (const pa of [1, 2]) {
          for (const pb of [-1]) {
            let product = parsed(sample[i]).powi(pa).product(parsed(sample[j]).powi(pb));
            if ((i + j) % 5 === 0) {
              const c = sample[(i * 7 + j) % sample.length];
              product = product.product(parsed(c).powi(-1));
            }
            const want = v.products[k];
            k += 1;
            const [u, scale] = product.simplify();
            const offset = product.affineOffset();
            const root = product.root(2);
            const got = [
              product.terms().map((t) => [t.symbol, t.power]),
              product.toString(),
              product.dimension().toString(),
              `${product.factor().numerBigint()}/${product.factor().denomBigint()}`,
              product.hasAffine(),
              offset === null ? null : `${offset.numerBigint()}/${offset.denomBigint()}`,
              root === null ? null : root.toString(),
              u.toString(),
              `${scale.numerBigint()}/${scale.denomBigint()}`,
              sameF64(scale.toF64(), want[9]) ? want[9] : scale.toF64(),
              u.clone().intoUnit().toString(),
            ];
            expect(got, product.toString()).toEqual(want);
          }
        }
      }
    }
    expect(k).toBe(v.products.length);
  });
});
