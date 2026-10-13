import { describe, expect, test } from 'vitest';
import { Panic } from '../../src/rust/core.js';
import {
  byteToIndex,
  charCount,
  charIndices,
  contains,
  countMatches,
  debugChar,
  debugStr,
  endsWith,
  eqIgnoreAsciiCase,
  find,
  fromDigit,
  indexToByte,
  isAlphabetic,
  isAlphanumeric,
  isAsciiDigit,
  isDigit,
  isLowercase,
  isNumeric,
  isUppercase,
  isWhitespace,
  lenUtf8,
  lines,
  replace,
  rfind,
  rsplitOnce,
  sliceBytes,
  split,
  splitn,
  splitOnce,
  splitWhitespace,
  startsWith,
  stripPrefix,
  stripSuffix,
  toAsciiLowercase,
  toAsciiUppercase,
  toDigit,
  trim,
  trimEnd,
  trimEndMatches,
  trimMatches,
  trimStart,
  trimStartMatches,
  utf8Len,
} from '../../src/rust/str.js';
import { loadVectors } from '../vectors.js';

const vectors = loadVectors('std');

/**
 * Inclusive code point ranges where `property` holds, as the vectors list
 * them (surrogates are not `char`s).
 * @param {(c: string) => boolean} property
 */
function ranges(property) {
  const result = [];
  let start = null;
  for (let code = 0; code <= 0x110000; code += 1) {
    const holds = code <= 0x10ffff && (code < 0xd800 || code > 0xdfff) && property(String.fromCodePoint(code));
    if (holds && start === null) {
      start = code;
    } else if (!holds && start !== null) {
      result.push([start, code - 1]);
      start = null;
    }
  }
  return result;
}

describe('char classes against Rust', () => {
  const properties = {
    whitespace: isWhitespace,
    alphabetic: isAlphabetic,
    numeric: isNumeric,
    alphanumeric: isAlphanumeric,
    lowercase: isLowercase,
    uppercase: isUppercase,
    debug_str_escaped: (/** @type {string} */ c) => debugStr(c) !== `"${c}"`,
    debug_char_escaped: (/** @type {string} */ c) => debugChar(c) !== `'${c}'`,
  };
  for (const [name, property] of Object.entries(properties)) {
    test(name, () => {
      expect(ranges(property)).toEqual(vectors.chars[name]);
    });
  }

  test('ASCII digits and radix conversion', () => {
    expect(isAsciiDigit('٣')).toBe(false);
    expect(toDigit('f', 16)).toBe(15);
    expect(toDigit('F', 16)).toBe(15);
    expect(toDigit('g', 16)).toBeNull();
    expect(isDigit('7', 8)).toBe(true);
    expect(isDigit('8', 8)).toBe(false);
    expect(fromDigit(11, 16)).toBe('b');
    expect(fromDigit(16, 16)).toBeNull();
  });
});

describe('str methods against Rust', () => {
  const isWhitespaceOrDigit = (/** @type {string} */ c) => isAsciiDigit(c) || c === ' ';
  for (const c of vectors.strings.cases) {
    test(debugStr(c.input), () => {
      const s = c.input;
      expect({
        trim: trim(s),
        trim_start: trimStart(s),
        trim_end: trimEnd(s),
        split_whitespace: splitWhitespace(s),
        lines: lines(s),
        to_lowercase: s.toLowerCase(),
        to_uppercase: s.toUpperCase(),
        to_ascii_lowercase: toAsciiLowercase(s),
        to_ascii_uppercase: toAsciiUppercase(s),
        debug: debugStr(s),
        len: utf8Len(s),
        char_count: charCount(s),
        char_indices: charIndices(s).map(([offset]) => offset),
        split_space: split(s, ' '),
        split_empty: split(s, ''),
        replace_empty: replace(s, '', '-'),
        replace_dollar: replace(s, 'a', '$&'),
        splitn2: splitn(s, 2, isWhitespace),
        split_once_ws: splitOnce(s, isWhitespace),
        rsplit_once_ws: rsplitOnce(s, isWhitespace),
        trim_start_digits: trimStartMatches(s, isWhitespaceOrDigit),
        trim_end_ws_chars: trimEndMatches(s, [' ', '\n', '\r']),
        matches_a: countMatches(s, 'a'),
      }).toEqual(Object.fromEntries(Object.entries(c).filter(([key]) => key !== 'input')));
    });
  }

  test('eq_ignore_ascii_case', () => {
    for (const [a, b, expected] of vectors.strings.eq_ignore_ascii_case) {
      expect(eqIgnoreAsciiCase(a, b)).toBe(expected);
    }
  });

  test('trim_start_matches and trim_end_matches with &str patterns', () => {
    for (const [s, pattern, start, end] of vectors.strings.trim_matches) {
      expect([trimStartMatches(s, pattern), trimEndMatches(s, pattern)]).toEqual([start, end]);
    }
  });

  test('patterns: &str, char predicates and char slices', () => {
    expect(trimMatches('--a-b--', '-')).toBe('a-b');
    expect(trimMatches('12ab34', isAsciiDigit)).toBe('ab');
    expect(stripPrefix('USD 5', 'USD ')).toBe('5');
    expect(stripPrefix('5', 'USD')).toBeNull();
    expect(stripPrefix('😀x', isAlphabetic)).toBeNull();
    expect(stripPrefix('éx', isAlphabetic)).toBe('x');
    expect(stripSuffix('5%', ['%', '‰'])).toBe('5');
    expect(startsWith('abc', ['a', 'b'])).toBe(true);
    expect(endsWith('abc', isNumeric)).toBe(false);
    expect(find('a😀b', 'b')).toBe(3);
    expect(find('a😀b', isAlphabetic)).toBe(0);
    expect(rfind('a b c', ' ')).toBe(3);
    expect(rfind('a b c', isWhitespace)).toBe(3);
    expect(find('abc', 'x')).toBeNull();
    expect(contains('abc', ['x', 'c'])).toBe(true);
    expect(splitn('a b c', 0, ' ')).toEqual([]);
    expect(splitn('a b c', 1, ' ')).toEqual(['a b c']);
    expect(replace('a.b.c', '.', '$&')).toBe('a$&b$&c');
  });
});

describe('UTF-8 byte offsets against Rust', () => {
  test('&s[from..to] and &s[from..] panic as Rust does', () => {
    for (const c of vectors.slice_panics) {
      const to = c.to ?? undefined;
      let actual;
      try {
        actual = { ok: sliceBytes(c.input, c.from, to) };
      } catch (thrown) {
        expect(thrown).toBeInstanceOf(Panic);
        actual = { panic: /** @type {Panic} */ (thrown).message };
      }
      expect(actual).toEqual(c.ok === undefined ? { panic: c.panic } : { ok: c.ok });
    }
  });

  test('index conversions', () => {
    const s = 'a😀é';
    expect(indexToByte(s, 3)).toBe(5);
    expect(byteToIndex(s, 5)).toBe(3);
    expect(byteToIndex(s, utf8Len(s))).toBe(s.length);
    expect(() => byteToIndex(s, 2)).toThrow(Panic);
    expect(lenUtf8('😀')).toBe(4);
    expect(charIndices(s)).toEqual([[0, 'a'], [1, '😀'], [5, 'é']]);
  });
});
