import { describe, expect, test } from 'vitest';
import { getVersion, healthCheck, parseConsolidatedLinoRates, parseLinoRate } from '../src/wasm.js';

// Port of the `#[cfg(test)] mod tests` of src/wasm.rs.
describe('wasm', () => {
  test('test_get_version', () => {
    const version = getVersion();
    expect(version).not.toBe('');
    expect(version).toContain('.');
  });

  test('test_health_check', () => {
    expect(healthCheck()).toBe(true);
  });

  test('test_parse_lino_rate', () => {
    const content = `rate:
  from USD
  to EUR
  value 0.92
  date 2026-01-25
  source 'frankfurter.dev (ECB)'`;

    const parsed = JSON.parse(parseLinoRate(content));

    expect(parsed.success).toBe(true);
    expect(parsed.from).toBe('USD');
    expect(parsed.to).toBe('EUR');
    expect(parsed.value).toBe(0.92);
    expect(parsed.date).toBe('2026-01-25');
    expect(parsed.source).toBe('frankfurter.dev (ECB)');
  });

  test('test_parse_lino_rate_missing_fields', () => {
    const content = `rate:
  from USD
  to EUR`;

    const parsed = JSON.parse(parseLinoRate(content));

    expect(parsed.success).toBe(false);
    expect(parsed.error).toContain('Missing');
  });

  test('test_parse_consolidated_lino_rates_legacy_format', () => {
    // Legacy format: rates: as root, data: for rates
    const content = `rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402`;

    const parsed = JSON.parse(parseConsolidatedLinoRates(content));

    expect(parsed.success).toBe(true);
    expect(parsed.from).toBe('USD');
    expect(parsed.to).toBe('EUR');
    expect(parsed.source).toBe('frankfurter.dev (ECB)');

    const rates = parsed.rates;
    expect(rates.length).toBe(3);
    expect(rates[0].date).toBe('2021-01-25');
    expect(rates[0].value).toBe(0.8234);
  });

  test('test_parse_consolidated_lino_rates_new_format', () => {
    // New format: conversion: as root, rates: for rates
    const content = `conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192
    2026-01-25 76.03`;

    const parsed = JSON.parse(parseConsolidatedLinoRates(content));

    expect(parsed.success).toBe(true);
    expect(parsed.from).toBe('USD');
    expect(parsed.to).toBe('RUB');
    expect(parsed.source).toBe('cbr.ru (Central Bank of Russia)');

    const rates = parsed.rates;
    expect(rates.length).toBe(3);
    expect(rates[0].date).toBe('2021-02-08');
    expect(rates[0].value).toBe(74.2602);
  });

  test('test_parse_consolidated_lino_rates_empty', () => {
    const content = `conversion:
  from USD
  to EUR
  source 'test'
  rates:`;

    const parsed = JSON.parse(parseConsolidatedLinoRates(content));

    expect(parsed.success).toBe(false);
    expect(parsed.error).toContain('no rates found');
  });
});
