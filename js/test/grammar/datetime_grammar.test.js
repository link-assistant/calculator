import { expect, test } from 'vitest';
import { DateTimeGrammar } from '../../src/grammar/datetime_grammar.js';
import { isOk } from '../../src/rust/core.js';

test('test_looks_like_datetime_month_name', () => {
  expect(DateTimeGrammar.looksLikeDatetime('Jan 22, 2026')).toBe(true);
  expect(DateTimeGrammar.looksLikeDatetime('22 January 2026')).toBe(true);
  expect(DateTimeGrammar.looksLikeDatetime('Feb 15')).toBe(true);
});

test('test_looks_like_datetime_time', () => {
  expect(DateTimeGrammar.looksLikeDatetime('8:59am')).toBe(true);
  expect(DateTimeGrammar.looksLikeDatetime('12:30pm')).toBe(true);
  expect(DateTimeGrammar.looksLikeDatetime('14:30 UTC')).toBe(true);
});

test('test_looks_like_datetime_iso', () => {
  expect(DateTimeGrammar.looksLikeDatetime('2026-01-22')).toBe(true);
});

test('test_looks_like_datetime_negative', () => {
  expect(DateTimeGrammar.looksLikeDatetime('42')).toBe(false);
  expect(DateTimeGrammar.looksLikeDatetime('USD')).toBe(false);
  expect(DateTimeGrammar.looksLikeDatetime('hello world')).toBe(false);
});

test('test_parse_various_formats', () => {
  const grammar = DateTimeGrammar.new();

  expect(isOk(() => grammar.parse('Jan 22, 2026'))).toBe(true);
  expect(isOk(() => grammar.parse('2026-01-22'))).toBe(true);
  expect(isOk(() => grammar.parse('8:59am UTC'))).toBe(true);
  expect(isOk(() => grammar.parse('Jan 27, 8:59am UTC'))).toBe(true);
});
