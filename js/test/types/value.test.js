import { describe, expect, test } from 'vitest';
import { CalculatorError } from '../../src/error.js';
import { tryResult, unwrapResult } from '../../src/rust/core.js';
import { CurrencyDatabase } from '../../src/types/currency.js';
import { DateTime } from '../../src/types/datetime.js';
import { Decimal } from '../../src/types/decimal.js';
import { Rational } from '../../src/types/rational.js';
import { formatDuration } from '../../src/types/value/duration.js';
import { Value } from '../../src/types/value/index.js';

describe('Value (src/types/value/tests.rs)', () => {
  test('number value', () => {
    const v = Value.number(Decimal.new(42));
    expect(v.toDisplayString()).toBe('42');
  });

  test('currency value', () => {
    const v = Value.currency(Decimal.new(100), 'USD');
    expect(v.toDisplayString()).toBe('100 USD');
  });

  test('number addition', () => {
    const a = Value.number(Decimal.new(2));
    const b = Value.number(Decimal.new(3));
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => a.add(b, db));
    expect(result.toDisplayString()).toBe('5');
  });

  test('rational addition', () => {
    const a = Value.rational(Rational.new(1n, 3n));
    const b = Value.rational(Rational.new(1n, 3n));
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => a.add(b, db));
    // 1/3 + 1/3 = 2/3
    expect(result.toFractionString()).toBe('2/3');
  });

  test('currency addition same', () => {
    const a = Value.currency(Decimal.new(100), 'USD');
    const b = Value.currency(Decimal.new(50), 'USD');
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => a.add(b, db));
    expect(result.toDisplayString()).toBe('150 USD');
  });

  test('division by zero', () => {
    const a = Value.number(Decimal.new(10));
    const b = Value.number(Decimal.zero());
    const result = tryResult(() => a.divide(b));
    expect(result.ok).toBe(false);
    expect(result.error).toBe(CalculatorError.DivisionByZero);
  });

  test('rational division by zero', () => {
    const a = Value.rational(Rational.fromInteger(10n));
    const b = Value.rational(Rational.zero());
    const result = tryResult(() => a.divide(b));
    expect(result.ok).toBe(false);
    expect(result.error).toBe(CalculatorError.DivisionByZero);
  });

  test('one third times three', () => {
    // This is the key test for issue #21
    const one = Value.rational(Rational.fromInteger(1n));
    const three = Value.rational(Rational.fromInteger(3n));
    // 1 / 3
    const oneThird = unwrapResult(() => one.divide(three));
    // (1/3) * 3 = 1 (exact!)
    const result = unwrapResult(() => oneThird.multiply(three));
    expect(result.toDisplayString()).toBe('1');
  });

  test('two thirds times three', () => {
    const two = Value.rational(Rational.fromInteger(2n));
    const three = Value.rational(Rational.fromInteger(3n));
    // 2 / 3
    const twoThirds = unwrapResult(() => two.divide(three));
    // (2/3) * 3 = 2 (exact!)
    const result = unwrapResult(() => twoThirds.multiply(three));
    expect(result.toDisplayString()).toBe('2');
  });

  test('format duration', () => {
    expect(formatDuration(0n)).toBe('0 seconds');
    expect(formatDuration(1n)).toBe('1 second');
    expect(formatDuration(60n)).toBe('1 minute');
    expect(formatDuration(3661n)).toBe('1 hour, 1 minute, 1 second');
    expect(formatDuration(86400n)).toBe('1 day');
  });

  test('datetime subtraction', () => {
    const dt1 = Value.datetime(unwrapResult(() => DateTime.parse('2026-01-27')));
    const dt2 = Value.datetime(unwrapResult(() => DateTime.parse('2026-01-25')));
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => dt1.subtract(dt2, db));
    expect(result.kind.tag).toBe('Duration');
  });

  test('datetime plus duration', () => {
    const dt = Value.datetime(unwrapResult(() => DateTime.parse('2026-01-25')));
    const dur = Value.duration(86400n); // 1 day in seconds
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => dt.add(dur, db));
    expect(result.kind.tag).toBe('DateTime');
  });

  test('duration plus datetime', () => {
    const dur = Value.duration(86400n); // 1 day in seconds (issue #8: duration + datetime)
    const dt = Value.datetime(unwrapResult(() => DateTime.parse('2026-01-25')));
    const db = CurrencyDatabase.new();
    const result = unwrapResult(() => dur.add(dt, db));
    expect(result.kind.tag).toBe('DateTime');
  });

  test('issue 8 expression', () => {
    // (Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)
    const dt1 = Value.datetime(unwrapResult(() => DateTime.parse('Jan 27, 8:59am UTC')));
    const dt2 = Value.datetime(unwrapResult(() => DateTime.parse('Jan 25, 12:51pm UTC')));
    const dt3 = Value.datetime(unwrapResult(() => DateTime.parse('Jan 25, 12:51pm UTC')));
    const db = CurrencyDatabase.new();
    // First: dt1 - dt2 = duration
    const duration = unwrapResult(() => dt1.subtract(dt2, db));
    expect(duration.kind.tag).toBe('Duration');
    // Second: duration + dt3 = datetime (this was failing before the fix)
    const result = unwrapResult(() => duration.add(dt3, db));
    expect(result.kind.tag).toBe('DateTime');
  });
});
