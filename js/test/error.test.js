import { describe, expect, test } from 'vitest';
import { eq } from '../src/rust/core.js';
import { CalculatorError, ErrorInfo } from '../src/error.js';

describe('CalculatorError (src/error.rs)', () => {
  test('parse error', () => {
    const err = CalculatorError.parse('invalid syntax');
    expect(err.toString()).toBe('Parse error: invalid syntax');
  });

  test('unexpected token', () => {
    const err = CalculatorError.unexpectedToken('+', 'number', 5);
    expect(err.toString()).toContain('Unexpected token');
    expect(err.toString()).toContain('+');
  });

  test('unit mismatch', () => {
    const err = CalculatorError.unitMismatch('add', 'USD', 'hours');
    expect(err.toString()).toContain('Unit mismatch');
    expect(err.toString()).toContain('USD');
    expect(err.toString()).toContain('hours');
  });

  test('division by zero', () => {
    const err = CalculatorError.DivisionByZero;
    expect(err.toString()).toBe('Division by zero');
  });

  test('invalid datetime', () => {
    const err = CalculatorError.InvalidDateTime('not a date');
    expect(err.toString()).toContain('Invalid datetime');
  });
});

// Display, Debug and `to_error_info()` of every variant, as printed by the
// Rust module (experiments/issue-254/foundation-vectors).
const VARIANTS = [
  [
    CalculatorError.parse('invalid syntax'),
    'Parse error: invalid syntax',
    'ParseError("invalid syntax")',
    'errors.parseError',
    { message: 'invalid syntax' },
  ],
  [
    CalculatorError.unexpectedToken('+', 'number', 5),
    "Unexpected token '+' at position 5, expected number",
    'UnexpectedToken { found: "+", expected: "number", position: 5 }',
    'errors.unexpectedToken',
    { found: '+', expected: 'number', position: '5' },
  ],
  [
    CalculatorError.unitMismatch('add', 'USD', 'hours'),
    "Unit mismatch: cannot add 'USD' and 'hours'",
    'UnitMismatch { operation: "add", left_unit: "USD", right_unit: "hours" }',
    'errors.unitMismatch',
    { operation: 'add', leftUnit: 'USD', rightUnit: 'hours' },
  ],
  [
    CalculatorError.eval('bad "quote"\n\tline \u{e9} \u{7f} \u{200b}'),
    'Evaluation error: bad "quote"\n\tline \u{e9} \u{7f} \u{200b}',
    'EvaluationError("bad \\"quote\\"\\n\\tline \u{e9} \\u{7f} \\u{200b}")',
    'errors.evaluationError',
    { message: 'bad "quote"\n\tline \u{e9} \u{7f} \u{200b}' },
  ],
  [CalculatorError.DivisionByZero, 'Division by zero', 'DivisionByZero', 'errors.divisionByZero', null],
  [
    CalculatorError.InvalidDateTime('not a date'),
    'Invalid datetime format: not a date',
    'InvalidDateTime("not a date")',
    'errors.invalidDateTime',
    { format: 'not a date' },
  ],
  [
    CalculatorError.UnknownCurrency('XYZ'),
    'Unknown currency: XYZ',
    'UnknownCurrency("XYZ")',
    'errors.unknownCurrency',
    { currency: 'XYZ' },
  ],
  [
    CalculatorError.CurrencyConversion({ from: 'USD', to: 'EUR', reason: 'no rate' }),
    'Cannot convert USD to EUR: no rate',
    'CurrencyConversion { from: "USD", to: "EUR", reason: "no rate" }',
    'errors.currencyConversion',
    { from: 'USD', to: 'EUR', reason: 'no rate' },
  ],
  [
    CalculatorError.NoHistoricalRate({ currency: 'USD', date: '2020-01-01' }),
    'No exchange rate available for USD on 2020-01-01',
    'NoHistoricalRate { currency: "USD", date: "2020-01-01" }',
    'errors.noHistoricalRate',
    { currency: 'USD', date: '2020-01-01' },
  ],
  [CalculatorError.Overflow, 'Numeric overflow', 'Overflow', 'errors.overflow', null],
  [
    CalculatorError.InvalidOperation('Cannot add'),
    'Invalid operation: Cannot add',
    'InvalidOperation("Cannot add")',
    'errors.invalidOperation',
    { message: 'Cannot add' },
  ],
  [CalculatorError.EmptyInput, 'Empty input', 'EmptyInput', 'errors.emptyInput', null],
  [
    CalculatorError.unknownFunction('foo'),
    'Unknown function: foo',
    'UnknownFunction("foo")',
    'errors.unknownFunction',
    { name: 'foo' },
  ],
  [
    CalculatorError.invalidArgs('sqrt', 'needs 1 argument'),
    "Invalid arguments for function 'sqrt': needs 1 argument",
    'InvalidFunctionArgs { function: "sqrt", reason: "needs 1 argument" }',
    'errors.invalidFunctionArgs',
    { function: 'sqrt', reason: 'needs 1 argument' },
  ],
  [
    CalculatorError.domain('sqrt of negative'),
    'Domain error: sqrt of negative',
    'DomainError("sqrt of negative")',
    'errors.domainError',
    { message: 'sqrt of negative' },
  ],
  [
    CalculatorError.SymbolicResult({
      expression: 'integrate x dx',
      result: 'x^2/2 + C',
      latexInput: '\\int x\\,dx',
      latexResult: '\\frac{x^{2}}{2} + C',
    }),
    'x^2/2 + C',
    'SymbolicResult { expression: "integrate x dx", result: "x^2/2 + C", '
      + 'latex_input: "\\\\int x\\\\,dx", latex_result: "\\\\frac{x^{2}}{2} + C" }',
    'result.symbolic',
    { result: 'x^2/2 + C' },
  ],
];

describe('CalculatorError variants (vectors from experiments/issue-254/foundation-vectors)', () => {
  test.each(VARIANTS)('%s', (error, display, debug, key, params) => {
    expect(error.toString()).toBe(display);
    expect(error.fmtDebug()).toBe(debug);
    const info = error.toErrorInfo();
    expect(info).toBeInstanceOf(ErrorInfo);
    expect(info.key).toBe(key);
    expect(info.params === null ? null : Object.fromEntries(info.params)).toEqual(params);
    // `#[serde(skip_serializing_if = "Option::is_none")]` drops `params`.
    const json = JSON.parse(JSON.stringify(info));
    expect(Object.keys(json)).toEqual(params === null ? ['key'] : ['key', 'params']);
    expect(json).toEqual(params === null ? { key } : { key, params });
  });

  test('variants are frozen and compare by value', () => {
    const error = CalculatorError.unexpectedToken('+', 'number', 5);
    expect(Object.isFrozen(error)).toBe(true);
    expect(eq(error, CalculatorError.UnexpectedToken({ found: '+', expected: 'number', position: 5 }))).toBe(true);
    expect(eq(error, CalculatorError.unexpectedToken('+', 'number', 6))).toBe(false);
    expect(eq(CalculatorError.parse('x'), CalculatorError.ParseError('x'))).toBe(true);
    expect(eq(CalculatorError.parse('x'), CalculatorError.eval('x'))).toBe(false);
    expect(CalculatorError.Overflow.tag).toBe('Overflow');
  });
});

describe('ErrorInfo (src/error.rs)', () => {
  test('new has no params', () => {
    const info = ErrorInfo.new('errors.emptyInput');
    expect(info.params).toBe(null);
    expect(JSON.stringify(info)).toBe('{"key":"errors.emptyInput"}');
  });

  test('with params keeps the map', () => {
    const info = ErrorInfo.withParams('errors.unknownCurrency', new Map([['currency', 'XYZ']]));
    expect(JSON.stringify(info)).toBe('{"key":"errors.unknownCurrency","params":{"currency":"XYZ"}}');
  });
});
