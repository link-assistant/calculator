import { expect, test } from 'vitest';
import { NumberGrammar } from '../../src/grammar/number_grammar.js';
import { eq, unwrapResult } from '../../src/rust/core.js';
import { Unit } from '../../src/types/index.js';

test('test_parse_integer', () => {
  const grammar = NumberGrammar.new();
  const n = unwrapResult(() => grammar.parseNumber('42'));
  expect(n.toString()).toBe('42');
});

test('test_parse_decimal', () => {
  const grammar = NumberGrammar.new();
  const n = unwrapResult(() => grammar.parseNumber('3.14'));
  expect(n.toString().startsWith('3.14')).toBe(true);
});

test('test_parse_negative', () => {
  const grammar = NumberGrammar.new();
  const n = unwrapResult(() => grammar.parseNumber('-5'));
  expect(n.toString()).toBe('-5');
});

test('test_parse_number_with_currency', () => {
  const grammar = NumberGrammar.new();
  const [n, unit] = unwrapResult(() => grammar.parseNumberWithUnit('100', 'USD'));
  expect(n.toString()).toBe('100');
  expect(eq(unit, Unit.currency('USD'))).toBe(true);
});

test('test_looks_like_number', () => {
  expect(NumberGrammar.looksLikeNumber('42')).toBe(true);
  expect(NumberGrammar.looksLikeNumber('-5')).toBe(true);
  expect(NumberGrammar.looksLikeNumber('3.14')).toBe(true);
  expect(NumberGrammar.looksLikeNumber('abc')).toBe(false);
  expect(NumberGrammar.looksLikeNumber('')).toBe(false);
});

test('test_parse_currency_unit', () => {
  const grammar = NumberGrammar.new();
  const unit = unwrapResult(() => grammar.parseUnit('EUR'));
  expect(eq(unit, Unit.currency('EUR'))).toBe(true);
});
