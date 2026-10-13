import { describe, expect, test } from 'vitest';
import { solve } from '../../src/grammar/polynomial_equation.js';
import { tryResult } from '../../src/rust/core.js';
import { add, div, group, mul, n, pow, sub, v } from './equation_helpers.js';

// src/grammar/polynomial_equation.rs has no unit tests; its behaviour is
// covered by tests/issue_174_placeholder_equation_tests.rs through the parser.
// These cases apply the same inputs (built by hand, as the parser is not ported
// yet) to the module directly, with the results those tests expect.

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

const x = v('x');
/** @param {string} e */
const xp = (e) => pow(x, n(e));

describe('polynomial_equation', () => {
  test('solves quadratic and higher power equations', () => {
    /** @type {[any, any, string][]} */
    const cases = [
      [xp('2'), n('4'), 'x = -2 or x = 2'],
      [add(sub(xp('2'), mul(n('5'), x)), n('6')), n('0'), 'x = 2 or x = 3'],
      [add(add(xp('2'), mul(n('5'), x)), n('6')), n('0'), 'x = -3 or x = -2'],
      [xp('2'), n('0'), 'x = 0'],
      [xp('3'), n('27'), 'x = 3'],
      [sub(xp('3'), x), n('0'), 'x = -1 or x = 0 or x = 1'],
      [xp('4'), n('16'), 'x = -2 or x = 2'],
      [sub(xp('5'), n('32')), n('0'), 'x = 2'],
      [sub(xp('6'), n('64')), n('0'), 'x = -2 or x = 2'],
      [add(xp('7'), n('128')), n('0'), 'x = -2'],
      [mul(group(sub(x, n('2'))), group(add(x, n('3')))), n('0'), 'x = -3 or x = 2'],
      [pow(group(sub(x, n('1'))), n('3')), n('0'), 'x = 1'],
      [
        mul(mul(group(sub(x, n('1'))), group(sub(x, n('2')))), group(sub(x, n('3')))),
        n('0'),
        'x = 1 or x = 2 or x = 3',
      ],
      [
        mul(
          mul(group(sub(mul(n('2'), x), n('1'))), group(sub(x, n('3')))),
          group(add(x, n('2'))),
        ),
        n('0'),
        'x = -2 or x = 0.5 or x = 3',
      ],
      [sub(mul(n('2'), xp('2')), n('8')), n('0'), 'x = -2 or x = 2'],
      [pow(v('?'), n('2')), n('9'), '? = -3 or ? = 3'],
      [pow(v('*'), n('2')), n('9'), '* = -3 or * = 3'],
      [mul(v('?'), v('?')), n('4'), '? = -2 or ? = 2'],
      [mul(v('*'), v('*')), n('4'), '* = -2 or * = 2'],
    ];
    for (const [l, r, expected] of cases) {
      expect(display(l, r)).toBe(expected);
    }
  });

  test('solves more than 1000 power equation edge cases', () => {
    const variables = ['x', 'y', 'z', 'a', 'b', 'c', 'm', 'n', 'p', 'q'];
    let caseCount = 0;
    for (const variable of variables) {
      for (let power = 2; power <= 11; power += 1) {
        for (let root = -5n; root <= 5n; root += 1n) {
          const rhs = root ** BigInt(power);
          const solution = solve(pow(v(variable), n(String(power))), n(String(rhs)));
          let expected;
          if (root === 0n) {
            expected = `${variable} = 0`;
          } else if (power % 2 === 0) {
            const abs = root < 0n ? -root : root;
            expected = `${variable} = -${abs} or ${variable} = ${abs}`;
          } else {
            expected = `${variable} = ${root}`;
          }
          expect(solution.toValue().toDisplayString()).toBe(expected);
          caseCount += 1;
        }
      }
    }
    expect(caseCount).toBeGreaterThan(1000);
  });

  test('structured polynomial solutions and detailed trace', () => {
    const solution = solve(add(sub(xp('2'), mul(n('5'), x)), n('6')), n('0'));
    const value = solution.toValue();
    expect(value.kind.tag).toBe('EquationSolutions');
    expect(value.kind.variable).toBe('x');
    expect(value.kind.values.map((/** @type {import('../../src/types/index.js').Rational} */ r) => r.toDisplayString())).toEqual([
      '2',
      '3',
    ]);
    expect(solution.derivationSteps()).toEqual([
      'Original equation: x^2 - 5 * x + 6 = 0',
      'Move all terms to the left: x^2 - 5*x + 6 = 0',
      'Polynomial form: x^2 - 5*x + 6 = 0',
      'Polynomial degree: 2',
      'Choose target variable: x',
      'Find real rational roots: test exact rational candidates and keep the values that make the polynomial equal 0',
      'Verify root 2: substituting x = 2 gives 0',
      'Verify root 3: substituting x = 3 gives 0',
      'Factor roots: (x - 2) * (x - 3)',
      'Solutions: x = 2 or x = 3',
    ]);
  });

  test('rejects unsupported polynomial equations', () => {
    expect(errorOf(() => solve(add(xp('2'), v('y')), n('4')))).toBe(
      'InvalidOperation("polynomial equations with multiple variables are not supported")',
    );
    expect(errorOf(() => solve(div(x, x), n('1')))).toBe(
      'InvalidOperation("polynomial equation has a variable denominator")',
    );
    expect(errorOf(() => solve(xp('2'), n('2')))).toBe(
      'InvalidOperation("polynomial equation has no supported real rational solutions")',
    );
    expect(errorOf(() => solve(mul(n('2'), x), n('4')))).toBe(
      'InvalidOperation("polynomial equation is linear")',
    );
    expect(errorOf(() => solve(n('2'), n('4')))).toBe(
      'InvalidOperation("polynomial equation has no variable")',
    );
    expect(errorOf(() => solve(xp('33'), n('0')))).toMatch(/only polynomial equations/);
  });
});
