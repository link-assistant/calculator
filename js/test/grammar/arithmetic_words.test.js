import { expect, test } from 'vitest';
import { parseVocabulary, scan, vocabulary } from '../../src/grammar/arithmetic_words.js';
import { TokenKind } from '../../src/grammar/lexer.js';
import { eq } from '../../src/rust/core.js';

/** @param {string} input */
function kinds(input) {
  const tokens = scan([...input], 0);
  return tokens === null ? null : tokens.map((t) => t[0]);
}

/** @param {string} input */
function number(input) {
  const k = kinds(input);
  if (k === null) {
    return null;
  }
  return k.length === 1 && k[0].tag === 'Number' ? k[0]._0 : null;
}

test('test_vocabulary_parses', () => {
  const v = vocabulary();
  expect(v.words.size > 150).toBe(true);
  expect(v.phrases.some(([w]) => eq(w, ['multiplied', 'by']))).toBe(true);
});

test('test_malformed_link_is_rejected', () => {
  expect(typeof parseVocabulary("(number en x 'one')")).toBe('string');
  expect(typeof parseVocabulary("(number en 1 'one two')")).toBe('string');
  expect(typeof parseVocabulary("(number en 1 'one')\n(number es 2 'one')")).toBe('string');
});

test('test_compound_numbers', () => {
  expect(number('seven')).toBe('7');
  expect(number('twenty one')).toBe('21');
  expect(number('twenty-one')).toBe('21');
  expect(number('one hundred twenty three')).toBe('123');
  expect(number('two thousand twenty six')).toBe('2026');
  expect(number('сто двадцать три')).toBe('123');
  expect(number('две тысячи')).toBe('2000');
  expect(number('दो सौ')).toBe('200');
  expect(number('二十三')).toBe('23');
  expect(number('三百二十')).toBe('320');
  expect(number('十五')).toBe('15');
  expect(number('一万二千')).toBe('12000');
});

test('test_number_words_do_not_run_together', () => {
  // "two three" is two numbers, so only "two" is consumed.
  const tokens = scan([...'two three'], 0);
  expect(eq(tokens, [[TokenKind.Number('2'), 0, 3, 'two']])).toBe(true);
  // A hyphen between non-combining words stays a minus sign.
  expect(/** @type {any} */ (scan([...'five-two'], 0))[0][2]).toBe(4);
});

test('test_operator_phrases', () => {
  expect(eq(kinds('multiplied by 3'), [TokenKind.Star])).toBe(true);
  expect(eq(kinds('умножить на 7'), [TokenKind.Star])).toBe(true);
  expect(eq(kinds('по модулю 3'), [TokenKind.Percent])).toBe(true);
  expect(eq(kinds('dividido por 2'), [TokenKind.Slash])).toBe(true);
  expect(kinds('mod(17, 5)')).toBe(null);
  expect(kinds('по мск')).toBe(null);
});

test('test_cjk_runs', () => {
  expect(eq(kinds('二加二'), [TokenKind.Number('2'), TokenKind.Plus, TokenKind.Number('2')])).toBe(true);
  expect(kinds('千克')).toBe(null);
  expect(kinds('十二月')).toBe(null);
});

test('test_uppercase_currency_codes_are_not_words', () => {
  expect(kinds('ONE')).toBe(null);
  expect(number('one')).toBe('1');
});
