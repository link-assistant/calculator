import { describe, expect, test } from 'vitest';
import { eq } from '../../src/rust/core.js';
import { Dimension } from '../../src/types/dimension.js';

describe('Dimension (src/types/dimension.rs)', () => {
  test('dimension algebra', () => {
    const length = Dimension.base('length');
    const time = Dimension.base('time');
    const speed = length.div(time);
    expect(speed.toString()).toBe('length·time^-1');
    expect(eq(speed.mul(time), length)).toBe(true);
    expect(eq(length.pow(2).root(2), length)).toBe(true);
    expect(length.root(2)).toBe(null);
    expect(length.div(length).isNone()).toBe(true);
  });
});

// Values printed by the Rust module (experiments/issue-254/foundation-vectors).
describe('Dimension vectors (experiments/issue-254/foundation-vectors)', () => {
  test('display, powers and roots', () => {
    const length = Dimension.base('length');
    const mass = Dimension.base('mass');
    const time = Dimension.base('time');
    const currency = Dimension.base('currency');
    const energy = mass.mul(length.pow(2)).div(time.pow(2));
    /** @param {Dimension | null} d */
    const str = (d) => (d === null ? null : d.toString());
    expect(energy.toString()).toBe('length^2·mass·time^-2');
    expect(str(energy.root(2))).toBe(null);
    expect(energy.pow(-3).toString()).toBe('length^-6·mass^-3·time^6');
    expect(str(energy.root(-2))).toBe(null);
    expect(str(energy.root(0))).toBe(null);
    expect(currency.div(mass).toString()).toBe('mass^-1·currency');
    expect(Dimension.NONE.toString()).toBe('dimensionless');
    expect(Dimension.default().toString()).toBe('dimensionless');
    expect(Dimension.base('speed')).toBe(null);
    expect(energy.pow(0).toString()).toBe('dimensionless');
    expect(str(length.pow(-4).root(2))).toBe('length^-2');
    expect(str(length.pow(-3).root(2))).toBe(null);
  });

  test('instances are frozen', () => {
    expect(Object.isFrozen(Dimension.NONE)).toBe(true);
    expect(Object.isFrozen(Dimension.NONE._0)).toBe(true);
  });
});
