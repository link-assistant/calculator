import { describe, expect, test } from 'vitest';
import { Duration, Utc } from '../../src/rust/chrono.js';
import { unwrapResult } from '../../src/rust/core.js';
import { DateTime } from '../../src/types/datetime.js';

describe('DateTime (src/types/datetime_tests.rs)', () => {
  test('parse iso date', () => {
    const dt = unwrapResult(() => DateTime.parse('2026-01-22'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.hasTimeValue).toBe(false);
  });

  test('parse us date', () => {
    const dt = unwrapResult(() => DateTime.parse('01/22/2026'));
    expect(dt.hasDateValue).toBe(true);
  });

  test('parse dot date european', () => {
    // DD.MM.YYYY (German/Russian convention) — issue #166.
    const dt = unwrapResult(() => DateTime.parse('15.10.2025'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.year()).toBe(2025);
    expect(dt.toString()).toBe('2025-10-15');
  });

  test('parse dot date iso', () => {
    // YYYY.MM.DD
    const dt = unwrapResult(() => DateTime.parse('2025.10.15'));
    expect(dt.toString()).toBe('2025-10-15');
  });

  test('parse month name date', () => {
    const dt = unwrapResult(() => DateTime.parse('Jan 22, 2026'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.year()).toBe(2026);
  });

  test('parse time 12h', () => {
    const dt = unwrapResult(() => DateTime.parse('8:59am'));
    expect(dt.hasTimeValue).toBe(true);
  });

  test('parse time with utc', () => {
    const dt = unwrapResult(() => DateTime.parse('8:59am UTC'));
    expect(dt.hasTimeValue).toBe(true);
    expect(dt.offsetSecondsValue).not.toBe(null);
  });

  test('parse datetime with partial date', () => {
    const dt = unwrapResult(() => DateTime.parse('Jan 27, 8:59am UTC'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.hasTimeValue).toBe(true);
  });

  test('datetime subtraction', () => {
    const dt1 = unwrapResult(() => DateTime.parse('Jan 27, 8:59am UTC'));
    const dt2 = unwrapResult(() => DateTime.parse('Jan 25, 12:51pm UTC'));
    const diff = dt1.subtract(dt2);
    // Should be approximately 44 hours and 8 minutes
    const hours = diff.asSecs() / 3600n;
    expect(hours > 40n && hours < 50n).toBe(true);
  });

  test('today uses requested timezone date', () => {
    const now = Utc.now();
    const [offsetSeconds, expectedDate] = now.time().hour() < 12
      ? [-12 * 60 * 60, now.sub(Duration.hours(12)).dateNaive()]
      : [14 * 60 * 60, now.add(Duration.hours(14)).dateNaive()];

    const today = DateTime.today(offsetSeconds);
    expect(today.inner.dateNaive().equals(expectedDate)).toBe(true);
    expect(today.hasDate()).toBe(true);
    expect(today.hasTime()).toBe(false);
  });

  test('22 jan 2026 format', () => {
    const dt = unwrapResult(() => DateTime.parse('22 Jan 2026'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.year()).toBe(2026);
  });

  /** Issue #212: "08 Aug 2026 22:35" (date followed by a 24-hour time,
   * without a comma) failed to parse. */
  test('date then time without comma', () => {
    const dt = unwrapResult(() => DateTime.parse('08 Aug 2026 22:35'));
    expect(dt.hasDateValue).toBe(true);
    expect(dt.hasTimeValue).toBe(true);
    expect(dt.inner.format('%Y-%m-%d %H:%M')).toBe('2026-08-08 22:35');
  });

  test('date then time variants', () => {
    for (const input of [
      '2026-08-08 22:35',
      'Aug 08, 2026 22:35',
      'August 8 2026 10:35pm',
      '08.08.2026 22:35',
    ]) {
      const dt = unwrapResult(() => DateTime.parse(input));
      expect(dt.inner.format('%Y-%m-%d %H:%M'), `input: ${input}`).toBe('2026-08-08 22:35');
    }
  });

  test('date then time with timezone', () => {
    let dt = unwrapResult(() => DateTime.parse('08 Aug 2026 22:35 UTC'));
    expect(dt.offsetSecondsValue).toBe(0);
    expect(dt.inner.format('%Y-%m-%d %H:%M')).toBe('2026-08-08 22:35');

    // +3 local time is 3 hours ahead of UTC.
    dt = unwrapResult(() => DateTime.parse('08 Aug 2026 22:35 MSK'));
    expect(dt.offsetSecondsValue).toBe(3 * 3600);
    expect(dt.inner.format('%Y-%m-%d %H:%M')).toBe('2026-08-08 19:35');
  });
});
