import { expect, test } from 'vitest';
import { Lexer } from '../../src/grammar/lexer.js';

test('test_tokenize_number', () => {
  const tokens = Lexer.new('42').tokenize();
  expect(tokens.length).toBe(2); // number + eof
  expect(tokens[0].kind.tag === 'Number' && tokens[0].kind._0 === '42').toBe(true);
});

test('test_tokenize_decimal', () => {
  const tokens = Lexer.new('3.14').tokenize();
  expect(tokens[0].kind.tag === 'Number' && tokens[0].kind._0 === '3.14').toBe(true);
});

test('test_tokenize_expression', () => {
  const tokens = Lexer.new('2 + 3').tokenize();
  expect(tokens.length).toBe(4); // 2 + 3 eof
  expect(tokens[1].kind.tag).toBe('Plus');
});

test('test_tokenize_parentheses', () => {
  const tokens = Lexer.new('(2 + 3)').tokenize();
  expect(tokens[0].kind.tag).toBe('LeftParen');
  expect(tokens[4].kind.tag).toBe('RightParen');
});

test('test_tokenize_identifier', () => {
  const tokens = Lexer.new('USD EUR').tokenize();
  expect(tokens[0].kind.tag === 'Identifier' && tokens[0].kind._0 === 'USD').toBe(true);
  expect(tokens[1].kind.tag === 'Identifier' && tokens[1].kind._0 === 'EUR').toBe(true);
});

test('test_tokenize_at_keyword', () => {
  const tokens = Lexer.new('value at time').tokenize();
  expect(tokens[1].kind.tag).toBe('At');
});

test('test_tokenize_currency_expression', () => {
  const tokens = Lexer.new('84 USD - 34 EUR').tokenize();
  expect(tokens.length).toBe(6); // 84 USD - 34 EUR eof
});

test('test_tokenize_percent', () => {
  const tokens = Lexer.new('3%').tokenize();
  expect(tokens.length).toBe(3); // 3 % eof
  expect(tokens[0].kind.tag === 'Number' && tokens[0].kind._0 === '3').toBe(true);
  expect(tokens[1].kind.tag).toBe('Percent');
});

test('test_tokenize_numeric_date_literals', () => {
  for (const input of ['2026-01-22', '15/10/2025', '01/22/2026', '15.10.2025', '2025.10.15']) {
    const tokens = Lexer.new(input).tokenize();
    expect(tokens.length, `${input} should be one date token + eof`).toBe(2);
    expect(
      tokens[0].kind.tag === 'DateLiteral' && tokens[0].kind._0 === input,
      `${input} should tokenize as a single DateLiteral, got ${tokens[0].kind.fmtDebug()}`,
    ).toBe(true);
  }
});

test('test_tokenize_date_in_expression', () => {
  const tokens = Lexer.new('15.10.2025 + 180 days').tokenize();
  expect(tokens[0].kind.tag === 'DateLiteral' && tokens[0].kind._0 === '15.10.2025').toBe(true);
  expect(tokens[1].kind.tag).toBe('Plus');
  expect(tokens[2].kind.tag === 'Number' && tokens[2].kind._0 === '180').toBe(true);
});

test('test_arithmetic_not_treated_as_date', () => {
  // Decimals, fractions and spaced subtraction must stay numeric tokens.
  expect(Lexer.new('3.14').tokenize()[0].kind.tag).toBe('Number');

  const tokens = Lexer.new('10/2').tokenize();
  expect(tokens[0].kind.tag).toBe('Number');
  expect(tokens[1].kind.tag).toBe('Slash');

  // Invalid calendar date (month 20) falls back to plain numbers.
  expect(Lexer.new('2026-20-5').tokenize()[0].kind.tag).toBe('Number');
});

test('test_tokenize_percent_expression', () => {
  const tokens = Lexer.new('3% * 50').tokenize();
  expect(tokens.length).toBe(5); // 3 % * 50 eof
  expect(tokens[1].kind.tag).toBe('Percent');
  expect(tokens[2].kind.tag).toBe('Star');
});
