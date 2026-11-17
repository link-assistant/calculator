import { describe, expect, test } from 'vitest';
import { DataSizeUnit, DurationUnit, MassUnit, Unit } from '../../src/types/unit.js';
import { f64FromBits, loadVectors, sameF64 } from '../vectors.js';

describe('Unit (src/types/unit.rs)', () => {
  test('unit display', () => {
    expect(Unit.None.toString()).toBe('');
    expect(Unit.currency('USD').toString()).toBe('USD');
    expect(Unit.Duration(DurationUnit.Hours).toString()).toBe('hours');
  });

  test('unit compatibility', () => {
    const usd = Unit.currency('USD');
    const eur = Unit.currency('EUR');
    const none = Unit.None;
    expect(usd.isCompatibleForOperation(eur, '+')).toBe(true);
    expect(usd.isCompatibleForOperation(none, '*')).toBe(true);
    expect(usd.isCompatibleForOperation(eur, '*')).toBe(false);
  });

  test('duration conversion', () => {
    expect(DurationUnit.Minutes.toSecs(2.0)).toBe(120.0);
    expect(DurationUnit.Hours.secsToUnit(3600.0)).toBe(1.0);
  });

  test('data size unit display', () => {
    expect(DataSizeUnit.Kilobyte.toString()).toBe('KB');
    expect(DataSizeUnit.Kibibyte.toString()).toBe('KiB');
    expect(DataSizeUnit.Megabyte.toString()).toBe('MB');
    expect(DataSizeUnit.Mebibyte.toString()).toBe('MiB');
    expect(Unit.DataSize(DataSizeUnit.Kilobyte).toString()).toBe('KB');
  });

  test('data size unit parse', () => {
    expect(DataSizeUnit.parse('KB')).toBe(DataSizeUnit.Kilobyte);
    expect(DataSizeUnit.parse('KiB')).toBe(DataSizeUnit.Kibibyte);
    expect(DataSizeUnit.parse('MB')).toBe(DataSizeUnit.Megabyte);
    expect(DataSizeUnit.parse('MiB')).toBe(DataSizeUnit.Mebibyte);
    expect(DataSizeUnit.parse('mebibytes')).toBe(DataSizeUnit.Mebibyte);
    expect(DataSizeUnit.parse('kilobytes')).toBe(DataSizeUnit.Kilobyte);
    expect(DataSizeUnit.parse('invalid')).toBe(null);
    expect(DataSizeUnit.parse('USD')).toBe(null);
  });

  test('data size bits', () => {
    expect(DataSizeUnit.Byte.bits()).toBe(8n);
    expect(DataSizeUnit.Kilobyte.bits()).toBe(8_000n);
    expect(DataSizeUnit.Kibibyte.bits()).toBe(8_192n);
    expect(DataSizeUnit.Megabyte.bits()).toBe(8_000_000n);
    expect(DataSizeUnit.Mebibyte.bits()).toBe(8_388_608n);
  });

  test('data size conversion within si', () => {
    const result = DataSizeUnit.Kilobyte.convert(741.0, DataSizeUnit.Megabyte);
    expect(Math.abs(result - 0.741)).toBeLessThan(1e-10);
  });

  test('data size conversion within iec', () => {
    const result = DataSizeUnit.Kibibyte.convert(741.0, DataSizeUnit.Mebibyte);
    expect(Math.abs(result - 741.0 / 1024.0)).toBeLessThan(1e-10);
  });

  test('data size conversion cross standard', () => {
    const result = DataSizeUnit.Kilobyte.convert(741.0, DataSizeUnit.Mebibyte);
    expect(Math.abs(result - 741_000.0 / 1_048_576.0)).toBeLessThan(1e-10);
  });

  test('data size unit compatibility', () => {
    const kb = Unit.DataSize(DataSizeUnit.Kilobyte);
    const mib = Unit.DataSize(DataSizeUnit.Mebibyte);
    expect(kb.isCompatibleForOperation(mib, '+')).toBe(true);
    expect(kb.isCompatibleForOperation(mib, '-')).toBe(true);
    expect(kb.isCompatibleForOperation(mib, '*')).toBe(false);
  });
});

describe('Rust vectors: data size and mass conversions', () => {
  const v = loadVectors('units');
  const values = v.values.map(f64FromBits);

  test('DataSizeUnit.convert matches Rust bit for bit', () => {
    let i = 0;
    for (const a of v.sizes) {
      for (const b of v.sizes) {
        const ua = /** @type {DataSizeUnit} */ (DataSizeUnit.parse(a));
        const ub = /** @type {DataSizeUnit} */ (DataSizeUnit.parse(b));
        for (const x of values) {
          expect(sameF64(ua.convert(x, ub), v.data_size[i]), `${x} ${a} -> ${b}`).toBe(true);
          i += 1;
        }
      }
    }
  });

  test('MassUnit.convert matches Rust bit for bit', () => {
    let i = 0;
    for (const a of v.masses) {
      for (const b of v.masses) {
        const ua = /** @type {MassUnit} */ (MassUnit.parse(a));
        const ub = /** @type {MassUnit} */ (MassUnit.parse(b));
        for (const x of values) {
          expect(sameF64(ua.convert(x, ub), v.mass[i]), `${x} ${a} -> ${b}`).toBe(true);
          i += 1;
        }
      }
    }
  });
});
