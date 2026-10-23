import { describe, expect, test } from 'vitest';
import { solve } from '../../src/grammar/linear_equation.js';
import { tryResult } from '../../src/rust/core.js';
import { add, div, group, mul, n, pow, sub, v } from './equation_helpers.js';

// src/grammar/linear_equation.rs has no unit tests; its behaviour is covered by
// tests/issue_174_placeholder_equation_tests.rs through the parser. These cases
// apply the same inputs (built by hand, as the parser is not ported yet) to the
// module directly, with the results and steps those tests expect.

/** @param {import('../../src/types/index.js').Expression} l @param {import('../../src/types/index.js').Expression} r */
function display(l, r) {
  return solve(l, r).toValue().toDisplayString();
}

/** @param {() => unknown} fn */
function errorOf(fn) {
  const result = tryResult(fn);
  expect(result.ok).toBe(false);
  return /** @type {import('../../src/error.js').CalculatorError} */ (result.error).fmtDebug();
}

describe('linear_equation', () => {
  test('solves placeholder equations', () => {
    expect(display(add(mul(n('2'), v('?')), n('3')), n('11'))).toBe('? = 4');
    expect(display(add(mul(n('2'), v('*')), n('3')), n('11'))).toBe('* = 4');
    expect(display(add(v('?'), v('?')), n('4'))).toBe('? = 2');
    expect(display(div(v('x'), n('4')), n('0.5'))).toBe('x = 2');
    expect(display(sub(n('10'), v('x')), n('3'))).toBe('x = 7');
  });

  test('prefers placeholder targets', () => {
    expect(display(add(v('x'), v('?')), n('5'))).toBe('? = 5 - x');
    expect(display(add(v('x'), v('*')), n('5'))).toBe('* = 5 - x');
  });

  test('solves symbolically with detailed trace', () => {
    const solution = solve(add(mul(n('2'), v('x')), mul(n('3'), v('y'))), n('12'));
    const value = solution.toValue();
    expect(value.toDisplayString()).toBe('x = 6 - 1.5*y');
    expect(value.kind.tag).toBe('SymbolicEquationSolution');
    expect(value.kind.expression).toBe('6 - 1.5*y');
    expect(solution.derivationSteps()).toEqual([
      'Original equation: 2*x + 3*y = 12',
      'Linear form: 2*x + 3*y = 12',
      'Choose target variable: x',
      'Combine target coefficients: 2 - 0 = 2',
      'Move non-target variable terms to the right: y: 0 - 3 = -3',
      'Move constants to the right: 12 - 0 = 12',
      'Right side after moving terms: 12 - 3*y',
      'Isolate variable term: 2*x = 12 - 3*y',
      'Divide both sides by 2: x = 6 - 1.5*y',
      'Solve for x: x = 6 - 1.5*y',
    ]);
  });

  test('solves more than 500 named multi variable equations', () => {
    const variables = ['x', 'y', 'z', 'a', 'b', 'c', 'm', 'n', 'p', 'q', 'r', 's'];
    let caseCount = 0;
    for (const target of variables) {
      for (const other of variables) {
        if (target === other) {
          continue;
        }
        for (let total = 2; total <= 6; total += 1) {
          const solution = solve(add(v(target), v(other)), n(String(total)));
          expect(solution.toValue().toDisplayString()).toBe(`${target} = ${total} - ${other}`);
          expect(solution.derivationSteps()).toHaveLength(10);
          caseCount += 1;
        }
      }
    }
    expect(caseCount).toBeGreaterThan(500);
  });

  test('star placeholder formatting', () => {
    const solution = solve(add(mul(n('2'), v('*')), n('3')), n('11'));
    expect(solution.derivationSteps()).toContain('Isolate variable term: 2*(*) = 8');
    expect(solution.derivationSteps()).toContain(
      'Move non-target variable terms to the right: no other variables to move',
    );
  });

  test('rejects unsupported equations', () => {
    expect(errorOf(() => solve(v('x'), v('x')))).toBe(
      'InvalidOperation("linear equation has no unique solution")',
    );
    expect(errorOf(() => solve(n('1'), n('1')))).toBe(
      'InvalidOperation("linear equation has no unique solution")',
    );
    expect(errorOf(() => solve(mul(v('x'), v('x')), n('4')))).toBe(
      'InvalidOperation("equation is not linear")',
    );
    expect(errorOf(() => solve(div(n('1'), v('x')), n('4')))).toBe(
      'InvalidOperation("equation has a variable denominator")',
    );
    expect(errorOf(() => solve(div(v('x'), group(sub(n('1'), n('1')))), n('4')))).toBe(
      'DivisionByZero',
    );
    expect(errorOf(() => solve(pow(v('x'), n('2')), n('4')))).toBe(
      'InvalidOperation("only linear equations with unitless numbers are supported")',
    );
  });
});
