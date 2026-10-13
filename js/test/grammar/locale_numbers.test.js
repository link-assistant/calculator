import { describe, expect, test } from 'vitest';
import { CalculatorError } from '../../src/error.js';
import { Err, eq, tryResult } from '../../src/rust/core.js';
import { scan } from '../../src/grammar/locale_numbers.js';

// src/grammar/locale_numbers.rs has no unit tests; its behaviour is covered by
// the lexer tests. These vectors were printed by the Rust module itself
// (experiments/issue-254/foundation-vectors), which also checks 200 000 fuzzed
// inputs against this port (experiments/issue-254/foundation-compare.mjs).
//
// An expected value is `[normalized, end]` or the `{:?}` of the error.

/** The result under `allow_comma = true, decimal_comma = false` also applies. */
const SAME = Symbol('same');

/**
 * `[input, expected]` scanned from 0 with `allow_comma = true,
 * decimal_comma = false`, then optionally the expected result with
 * `decimal_comma = true` (`SAME` if unchanged) and with `allow_comma = false`
 * (`decimal_comma = false`). A missing entry means unchanged.
 * @type {[string, any, any?, any?][]}
 */
const SCANS = [
  ['1,234.5', ['1234.5', 7], SAME, ['1', 1]],
  ['1.234,5', ['1234.5', 7], SAME, ['1.234', 5]],
  ['1 234,5', ['1234.5', 7], SAME, ['1234', 5]],
  ['1\'234.5', ['1234.5', 7]],
  ['1\u{2019}234.5', ['1234.5', 7]],
  ['12,34,567', ['1234567', 9], SAME, ['12', 2]],
  ['1,1', ['11', 3], ['1.1', 3], ['1', 1]],
  ['1,2', ['1.2', 3], SAME, ['1', 1]],
  ['1.000 000', ['1.000000', 9]],
  ['123_456\u{2009}789', ['123456789', 11]],
  ['1,23,4567', 'ParseError("Invalid number separators in \'1,23,4567\' at position 0")', SAME, ['1', 1]],
  ['1\'23', ['1', 1]],
  ['1 23,4', ['1', 1]],
  ['1,234,', ['1234', 5], ['1.234', 5], ['1', 1]],
  ['1,,234', ['1', 1]],
  ['1,234', ['1234', 5], ['1.234', 5], ['1', 1]],
  ['12,345', ['12345', 6], ['12.345', 6], ['12', 2]],
  ['1,234,567.89', ['1234567.89', 12], SAME, ['1', 1]],
  ['1.234.567,89', ['1234567.89', 12], SAME, 'ParseError("Invalid number separators in \'1.234.567\' at position 0")'],
  ['1 234 567', ['1234567', 9]],
  ['1\u{a0}234', ['1234', 5]],
  ['1\u{202f}234,56', ['1234.56', 8], SAME, ['1234', 5]],
  ['1\u{2007}234', ['1234', 5]],
  ['1 2345', ['1', 1]],
  ['1234 567', ['1234', 4]],
  ['0.5', ['0.5', 3]],
  ['.5', ['0.5', 2]],
  [',5', 'ParseError("Invalid number separators in \',5\' at position 0")', SAME, 'ParseError("Invalid number separators in \'\' at position 0")'],
  ['1_000', ['1000', 5]],
  ['1__000', 'ParseError("Invalid number separators in \'1__000\' at position 0")'],
  ['_1', 'ParseError("Invalid number separators in \'_1\' at position 0")'],
  ['1_', 'ParseError("Invalid number separators in \'1_\' at position 0")'],
  ['1_.5', 'ParseError("Invalid number separators in \'1_.5\' at position 0")'],
  ['1._5', ['1', 1]],
  ['1.5_5', ['1.55', 5]],
  ['6\'2"', ['6', 1]],
  ['1\'234\'567', ['1234567', 9]],
  ['1\'234\u{2019}567', ['1234567', 9]],
  ['12\'345', ['12345', 6]],
  ['1\'2345', ['1', 1]],
  ['3.14159', ['3.14159', 7]],
  ['1.2.3', 'ParseError("Invalid number separators in \'1.2.3\' at position 0")'],
  ['1,2,3', 'ParseError("Invalid number separators in \'1,2,3\' at position 0")', SAME, ['1', 1]],
  ['1,23', ['1.23', 4], SAME, ['1', 1]],
  ['1,2345', ['1.2345', 6], SAME, ['1', 1]],
  ['1,234.567.8', 'ParseError("Invalid number separators in \'1,234.567.8\' at position 0")', SAME, ['1', 1]],
  ['1.234,567,8', 'ParseError("Invalid number separators in \'1.234,567,8\' at position 0")', SAME, ['1.234', 5]],
  ['1,234.5 678', ['1234.5678', 11], SAME, ['1', 1]],
  ['0.123 456', ['0.123456', 9]],
  ['0.123 45', ['0.123', 5]],
  ['1 234.5', ['1234.5', 7]],
  ['1 234_567', ['1234567', 9]],
  ['1_234 567', ['1234567', 9]],
  ['0b1010', ['10', 6]],
  ['0B11.1', ['3.5', 6]],
  ['0o17', ['15', 4]],
  ['0x1F', ['31', 4]],
  ['0xff_ff', ['65535', 7]],
  ['0x1.8', ['1.5', 5]],
  ['0b1e10', ['4', 6]],
  ['0b1e-10', ['0.25', 7]],
  ['0x1e+2', ['30', 4]],
  ['0b1e', 'ParseError("Invalid radix exponent")'],
  ['0b1e-', 'ParseError("Invalid radix exponent")'],
  ['0b1e_1', 'ParseError("Invalid radix exponent")'],
  ['0x', ['0', 1]],
  ['0b2', ['0', 1]],
  ['0b', ['0', 1]],
  ['0b1_', ['1', 3]],
  ['0b1__0', ['1', 3]],
  ['0x.8', ['0', 1]],
  ['0b0.1', ['0.5', 5]],
  ['0b0.0001', ['0.0625', 8]],
  ['0x0.1', ['0.0625', 5]],
  ['0o0.1', ['0.125', 5]],
  ['0b0.01e-1', ['0.125', 9]],
  ['0x1.', ['1', 4]],
  ['0b1.0.1', ['1', 5]],
  ['0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF', ['340282366920938463463374607431768211455', 34]],
  ['0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF', 'Overflow'],
  ['0b1e1111111', ['170141183460469231731687303715884105728', 11]],
  ['0b1e100000000000000000000000000000000', 'Overflow'],
  ['0x1e-10', ['30', 4]],
  ['0o1e-77', 'Overflow'],
  ['0b0.00000000000000000000000000001', 'Overflow'],
  ['0x0.000000000000000000000001', 'Overflow'],
  ['123', ['123', 3]],
  ['0', ['0', 1]],
  ['00', ['00', 2]],
  ['007', ['007', 3]],
  ['1e5', ['1', 1]],
  ['12 345 678', ['12345678', 10]],
  ['1 234 5678', ['1234', 5]],
  ['12 34', ['12', 2]],
  ['1 234,5 6', ['1234.5', 7], SAME, ['1234', 5]],
  ['1.234 567,8', 'ParseError("Invalid number separators in \'1.234 567,8\' at position 0")', SAME, ['1.234567', 9]],
  ['1 000.5', ['1000.5', 7]],
  ['1.5', ['1.5', 3]],
  ['1,5', ['1.5', 3], SAME, ['1', 1]],
  ['12,3', ['12.3', 4], SAME, ['12', 2]],
  ['123,4', ['123.4', 5], SAME, ['123', 3]],
  ['1234,5', ['1234.5', 6], SAME, ['1234', 4]],
  ['1,2345,6', 'ParseError("Invalid number separators in \'1,2345,6\' at position 0")', SAME, ['1', 1]],
  ['99,99,999', ['9999999', 9], SAME, ['99', 2]],
  ['9,99,99,999', ['99999999', 11], SAME, ['9', 1]],
  ['1,00,000', ['100000', 8], SAME, ['1', 1]],
  ['10,00,000.50', ['1000000.50', 12], SAME, ['10', 2]],
  ['1,00,00', 'ParseError("Invalid number separators in \'1,00,00\' at position 0")', SAME, ['1', 1]],
  ['100,00,000', 'ParseError("Invalid number separators in \'100,00,000\' at position 0")', SAME, ['100', 3]],
  ['1,000,00', 'ParseError("Invalid number separators in \'1,000,00\' at position 0")', SAME, ['1', 1]],
  ['\u{661}\u{662}\u{663}', 'ParseError("Invalid number separators in \'\' at position 0")'],
  ['1.', ['1', 1]],
  ['1,', ['1', 1]],
  ['1 ', ['1', 1]],
  ['1\'', ['1', 1]],
  ['1_000_000.000_1', ['1000000.0001', 15]],
];

/** `[input, start, expected]` with `allow_comma = true, decimal_comma = false`. */
/** @type {[string, number, any][]} */
const SCANS_FROM = [
  ['x 1,234.5', 2, ['1234.5', 9]],
  ['ab0x1F', 2, ['31', 6]],
  ['\u{e9} 1 234', 2, ['1234', 7]],
  ['\u{1f600}1,1', 1, ['11', 4]],
  ['12', 2, 'ParseError("Invalid number separators in \'\' at position 2")'],
];

/**
 * `scan(input.chars().collect(), ...)` as `[normalized, end]` or the `{:?}`
 * of the error.
 * @param {string} input
 * @param {number} start
 * @param {boolean} allowComma
 * @param {boolean} decimalComma
 */
function scanned(input, start, allowComma, decimalComma) {
  const result = tryResult(() => scan([...input], start, allowComma, decimalComma));
  return result.ok ? result.value : result.error.fmtDebug();
}

describe('scan (src/grammar/locale_numbers.rs)', () => {
  test.each(SCANS)('%j', (input, expected, decimalComma = SAME, noComma = SAME) => {
    expect(scanned(input, 0, true, false)).toEqual(expected);
    expect(scanned(input, 0, true, true)).toEqual(decimalComma === SAME ? expected : decimalComma);
    expect(scanned(input, 0, false, false)).toEqual(noComma === SAME ? expected : noComma);
  });

  test.each(SCANS_FROM)('%j from %i', (input, start, expected) => {
    expect(scanned(input, start, true, false)).toEqual(expected);
  });

  test('fend outlier 1,1 means 11 only without decimal commas', () => {
    expect(scan([...'1,1'], 0, true, false)).toEqual(['11', 3]);
    expect(scan([...'1,1'], 0, true, true)).toEqual(['1.1', 3]);
    expect(scan([...'1,2'], 0, true, false)).toEqual(['1.2', 3]);
  });

  test('invalid separators are a parse error at the start char index', () => {
    const result = tryResult(() => scan([...'x 1,23,4567'], 2, true, false));
    expect(result.ok).toBe(false);
    expect(eq(result.error, CalculatorError.ParseError(
      "Invalid number separators in '1,23,4567' at position 2",
    ))).toBe(true);
  });

  test('radix overflow is a recoverable Err', () => {
    expect(() => scan([...'0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF'], 0, true, false)).toThrow(Err);
    const result = tryResult(() => scan([...'0o1e-77'], 0, true, false));
    expect(result.error).toBe(CalculatorError.Overflow);
  });
});
