// src/types/datetime_parse.rs has no unit tests of its own; these check the
// helpers on inputs whose Rust results follow directly from the source.
import { afterEach, describe, expect, test } from 'vitest';
import { setFixedNow } from '../../src/clock.js';
import { Utc } from '../../src/rust/chrono.js';
import { Panic } from '../../src/rust/core.js';
import {
  extractTimezone, normalizeMonthName, parse12hTime, parseOffset, parsePartialDate,
  parseTzAbbreviation, preprocessNaturalDate, translateMonthNames,
} from '../../src/types/datetime_parse.js';

afterEach(() => setFixedNow(null));

describe('datetime_parse (src/types/datetime_parse.rs)', () => {
  test('translate month names', () => {
    expect(translateMonthNames('17 февраля 2026')).toBe('17 February 2026');
    expect(translateMonthNames('17 ФЕВРАЛЯ 2026')).toBe('17 February 2026');
    expect(translateMonthNames('3 März 2026')).toBe('3 March 2026');
    expect(translateMonthNames('1 janvier')).toBe('1 January');
    // Whole words only: German "januar" must not match inside "january".
    expect(translateMonthNames('January 5')).toBe('January 5');
    // "二月" inside "十二月" is preceded by an alphabetic ideograph.
    expect(translateMonthNames('十二月 5')).toBe('December 5');
    expect(translateMonthNames('nothing here')).toBe('nothing here');
  });

  test('translate month names slices the input at offsets of its lowercase', () => {
    // 'İ' lowercases to two code points (3 bytes instead of 2), so Rust's
    // offsets into the lowercase string land inside the input's 'й' and panic.
    expect(() => translateMonthNames('İ май')).toThrow(Panic);
  });

  test('preprocess natural date', () => {
    expect(preprocessNaturalDate('Monday, January 26th, 2026')).toBe('January 26, 2026');
    expect(preprocessNaturalDate('on January 26th')).toBe('January 26');
    expect(preprocessNaturalDate('8th of August')).toBe('8 August');
    expect(preprocessNaturalDate('17 August 2026 года')).toBe('17 August 2026');
    expect(preprocessNaturalDate('17 August 2026 г.')).toBe('17 August 2026');
    expect(preprocessNaturalDate('1st Ofx')).toBe('1 Ofx');
  });

  test('parse tz abbreviation', () => {
    expect(parseTzAbbreviation('IST')?.localMinusUtc()).toBe(5 * 3600 + 30 * 60);
    expect(parseTzAbbreviation(' est ')?.localMinusUtc()).toBe(-5 * 3600);
    expect(parseTzAbbreviation('мск')?.localMinusUtc()).toBe(3 * 3600);
    expect(parseTzAbbreviation('NST')?.localMinusUtc()).toBe(-(3 * 3600 + 30 * 60));
    expect(parseTzAbbreviation('cst+8')?.localMinusUtc()).toBe(8 * 3600);
    expect(parseTzAbbreviation('XYZ')).toBe(null);
  });

  test('normalize month name', () => {
    expect(normalizeMonthName('jANUARY  22,   2026')).toBe('January 22, 2026');
    expect(normalizeMonthName('')).toBe('');
  });

  test('parse partial date uses the current year', () => {
    setFixedNow(Utc.withYmdAndHms(2026, 3, 15, 12, 0, 0).unwrap());
    expect(parsePartialDate('jan 27')?.toString()).toBe('2026-01-27');
    expect(parsePartialDate('27 January')?.toString()).toBe('2026-01-27');
    expect(parsePartialDate('Feb 30')).toBe(null);
  });

  test('parse 12h time', () => {
    expect(parse12hTime('8:59am')?.toString()).toBe('08:59:00');
    expect(parse12hTime('12:51 PM')?.toString()).toBe('12:51:00');
    expect(parse12hTime('12:05am')?.toString()).toBe('00:05:00');
    expect(parse12hTime('11:30:15pm')?.toString()).toBe('23:30:15');
    expect(parse12hTime('8:59:xx pm')?.toString()).toBe('20:59:00');
    expect(parse12hTime('8 am')).toBe(null);
    expect(parse12hTime('13:00pm')).toBe(null);
    expect(parse12hTime('8:59')).toBe(null);
  });

  test('extract timezone', () => {
    let [part, offset, abbrev] = extractTimezone('8:59am gtm');
    expect([part, offset?.localMinusUtc(), abbrev]).toEqual(['8:59am', 0, 'GMT']);
    [part, offset, abbrev] = extractTimezone('14:30 EST');
    expect([part, offset?.localMinusUtc(), abbrev]).toEqual(['14:30', -5 * 3600, 'EST']);
    [part, offset, abbrev] = extractTimezone('14:30+05:30');
    expect([part, offset?.localMinusUtc(), abbrev]).toEqual(['14:30', 5 * 3600 + 1800, null]);
    [part, offset, abbrev] = extractTimezone('14:30-08:00');
    expect([part, offset?.localMinusUtc(), abbrev]).toEqual(['14:30', -8 * 3600, null]);
    [part, offset, abbrev] = extractTimezone('-08:00');
    expect([part, offset, abbrev]).toEqual(['-08:00', null, null]);
  });

  test('parse offset', () => {
    expect(parseOffset('+05:00')?.localMinusUtc()).toBe(5 * 3600);
    expect(parseOffset('-08')?.localMinusUtc()).toBe(-8 * 3600);
    expect(parseOffset('+24:00')).toBe(null);
    expect(parseOffset('+x')).toBe(null);
  });
});
