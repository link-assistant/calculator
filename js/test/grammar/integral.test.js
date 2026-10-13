import { describe, expect, test } from 'vitest';
import { BinaryOp, Decimal, Expression } from '../../src/types/index.js';
import { symbolicResultToLatex, trySymbolicIntegral } from '../../src/grammar/integral.js';

describe('integral (src/grammar/integral.rs)', () => {
  test('sin x over x', () => {
    // sin(x)/x -> Si(x) + C
    const integrand = Expression.binary(
      Expression.functionCall('sin', [Expression.variable('x')]),
      BinaryOp.Divide,
      Expression.variable('x'),
    );
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('Si(x) + C');
  });

  test('cos x over x', () => {
    // cos(x)/x -> Ci(x) + C
    const integrand = Expression.binary(
      Expression.functionCall('cos', [Expression.variable('x')]),
      BinaryOp.Divide,
      Expression.variable('x'),
    );
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('Ci(x) + C');
  });

  test('x squared', () => {
    // x^2 -> x^3/3 + C
    const integrand = Expression.power(Expression.variable('x'), Expression.number(Decimal.new(2)));
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('x^3/(3) + C');
  });

  test('just x', () => {
    // x -> x²/2 + C
    const integrand = Expression.variable('x');
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('x²/2 + C');
  });

  test('constant', () => {
    // 5 -> 5 * x + C
    const integrand = Expression.number(Decimal.new(5));
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('5 * x + C');
  });

  test('sin x', () => {
    // sin(x) -> -cos(x) + C
    const integrand = Expression.functionCall('sin', [Expression.variable('x')]);
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('-cos(x) + C');
  });

  test('cos x', () => {
    // cos(x) -> sin(x) + C
    const integrand = Expression.functionCall('cos', [Expression.variable('x')]);
    const result = trySymbolicIntegral(integrand, 'x');
    expect(result).toBe('sin(x) + C');
  });

  test('symbolic result to latex', () => {
    expect(symbolicResultToLatex('Si(x) + C')).toBe('\\text{Si}(x) + C');
    expect(symbolicResultToLatex('ln|x| + C')).toBe('\\ln|x| + C');
    expect(symbolicResultToLatex('x²/2 + C')).toBe('x^{2}/2 + C');
  });
});
