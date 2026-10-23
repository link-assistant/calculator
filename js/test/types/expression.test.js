import { describe, expect, test } from 'vitest';
import { Decimal } from '../../src/types/decimal.js';
import { BinaryOp, Expression } from '../../src/types/expression.js';

describe('Expression (src/types/expression_tests.rs)', () => {
  test('number expression', () => {
    const expr = Expression.number(Decimal.new(42));
    expect(expr.toString()).toBe('42');
    expect(expr.toLino()).toBe('42');
  });

  test('binary expression', () => {
    const left = Expression.number(Decimal.new(2));
    const right = Expression.number(Decimal.new(3));
    const expr = Expression.binary(left, BinaryOp.Add, right);
    expect(expr.toString()).toBe('2 + 3');
    expect(expr.toLino()).toBe('(2 + 3)');
  });

  test('complex expression', () => {
    const usd = Expression.currency(Decimal.new(84), 'USD');
    const eur = Expression.currency(Decimal.new(34), 'EUR');
    const expr = Expression.binary(usd, BinaryOp.Subtract, eur);
    expect(expr.toLino()).toContain('84 USD');
    expect(expr.toLino()).toContain('34 EUR');
  });

  test('binary op precedence', () => {
    expect(BinaryOp.Multiply.precedence() > BinaryOp.Add.precedence()).toBe(true);
    expect(BinaryOp.Modulo.precedence()).toBe(BinaryOp.Multiply.precedence());
    expect(BinaryOp.Add.precedence()).toBe(BinaryOp.Subtract.precedence());
  });

  test('depth', () => {
    const simple = Expression.number(Decimal.new(1));
    expect(simple.depth()).toBe(1);
    const binary = Expression.binary(
      Expression.number(Decimal.new(1)),
      BinaryOp.Add,
      Expression.number(Decimal.new(2)),
    );
    expect(binary.depth()).toBe(2);
  });
});
