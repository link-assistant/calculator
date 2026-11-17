import { describe, expect, test } from 'vitest';
import { unwrap, unwrapResult } from '../../src/rust/core.js';
import { Currency, CurrencyDatabase, ExchangeRateInfo } from '../../src/types/currency.js';

describe('Currency (src/types/currency.rs)', () => {
  test('currency creation', () => {
    const usd = Currency.usd();
    expect(usd.code).toBe('USD');
    expect(usd.symbol).toBe('$');
  });

  test('database creation', () => {
    const db = CurrencyDatabase.new();
    expect(db.isKnownCurrency('USD')).toBe(true);
    expect(db.isKnownCurrency('EUR')).toBe(true);
    expect(db.isKnownCurrency('XYZ')).toBe(false);
  });

  test('get rate', () => {
    const db = CurrencyDatabase.new();
    const rate = db.getRate('USD', 'EUR');
    expect(rate).not.toBeNull();
    expect(unwrap(rate) > 0.8 && unwrap(rate) < 1.0).toBe(true);
  });

  test('same currency rate', () => {
    const db = CurrencyDatabase.new();
    expect(db.getRate('USD', 'USD')).toBe(1.0);
  });

  test('convert', () => {
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => db.convert(100.0, 'USD', 'EUR'));
    expect(result > 80.0 && result < 100.0).toBe(true);
  });

  test('parse currency', () => {
    expect(CurrencyDatabase.parseCurrency('USD')).toBe('USD');
    expect(CurrencyDatabase.parseCurrency('$')).toBe('USD');
    expect(CurrencyDatabase.parseCurrency('eur')).toBe('EUR');
    expect(CurrencyDatabase.parseCurrency('€')).toBe('EUR');
  });

  test('inverse rate', () => {
    const db = CurrencyDatabase.new();
    const usdEur = unwrap(db.getRate('USD', 'EUR'));
    const eurUsd = unwrap(db.getRate('EUR', 'USD'));
    // The rates should be inverses of each other (approximately)
    const product = usdEur * eurUsd;
    expect(Math.abs(product - 1.0) < 0.01).toBe(true);
  });

  test('exchange rate info', () => {
    const info = ExchangeRateInfo.new(1.5, 'test-api', '2026-01-25');
    expect(info.rate).toBe(1.5);
    expect(info.source).toBe('test-api');
    expect(info.date).toBe('2026-01-25');
  });

  test('rate info display', () => {
    const info = ExchangeRateInfo.new(89.5, 'cbr.ru (Central Bank of Russia)', '2026-01-25');
    const display = info.formatForDisplay('USD', 'RUB');
    expect(display).toContain('1 USD = 89.5 RUB');
    expect(display).toContain('cbr.ru (Central Bank of Russia)');
    expect(display).toContain('2026-01-25');
  });

  test('set rate with info', () => {
    const db = CurrencyDatabase.new();
    const info = ExchangeRateInfo.new(75.0, 'test-api', '2026-01-25');
    db.setRateWithInfo('USD', 'RUB', info);

    const rateInfo = unwrap(db.getRateInfo('USD', 'RUB'));
    expect(rateInfo.rate).toBe(75.0);
    expect(rateInfo.source).toBe('test-api');

    // Check inverse rate was also set
    const inverseInfo = unwrap(db.getRateInfo('RUB', 'USD'));
    expect(Math.abs(inverseInfo.rate - 1.0 / 75.0) < 0.0001).toBe(true);
  });

  test('convert tracks rate used', () => {
    const db = CurrencyDatabase.new();
    const info = ExchangeRateInfo.new(75.0, 'test-api', '2026-01-25');
    db.setRateWithInfo('USD', 'RUB', info);

    const result = unwrapResult(() => db.convert(100.0, 'USD', 'RUB'));
    expect(result).toBe(7500.0);

    const lastRates = db.getLastUsedRates();
    expect(lastRates.length).toBe(1);
    const lastRate = lastRates[0];
    expect(lastRate[0]).toBe('USD');
    expect(lastRate[1]).toBe('RUB');
    expect(lastRate[2].rate).toBe(75.0);
    expect(lastRate[2].source).toBe('test-api');
  });

  test('same currency no rate tracking', () => {
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => db.convert(100.0, 'USD', 'USD'));
    expect(result).toBe(100.0);
    expect(db.getLastUsedRates().length).toBe(0);
  });
});
