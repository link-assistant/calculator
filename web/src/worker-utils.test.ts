import { describe, it, expect } from 'vitest';
import { isMissingRatesError } from './worker-utils';

describe('isMissingRatesError', () => {
  it('returns false for successful results', () => {
    expect(isMissingRatesError({ success: true })).toBe(false);
  });

  it('returns true for currencyConversion error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.currencyConversion' },
    })).toBe(true);
  });

  it('returns true for noExchangeRate error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.noExchangeRate' },
    })).toBe(true);
  });

  it('returns true for unknownCurrency error_info key', () => {
    expect(isMissingRatesError({
      success: false,
      error_info: { key: 'errors.unknownCurrency' },
    })).toBe(true);
  });

  it('returns true when error message contains "No exchange rate available"', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Cannot convert TON to RUB: No exchange rate available',
    })).toBe(true);
  });

  it('returns true when error message contains "Cannot convert"', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Cannot convert BTC to USD',
    })).toBe(true);
  });

  it('returns false for non-currency errors', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Division by zero',
      error_info: { key: 'errors.divisionByZero' },
    })).toBe(false);
  });

  it('returns false for failed result with unrelated error', () => {
    expect(isMissingRatesError({
      success: false,
      error: 'Parse error: unexpected token',
    })).toBe(false);
  });
});
