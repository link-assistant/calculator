/**
 * Variable substitution for plotting (port of `src/substitution.rs`, an
 * `impl Calculator` block; `Calculator.substituteVariable` delegates here).
 */

import { Decimal, Expression } from './types/index.js';

/**
 * Substitutes a variable with a numeric value in an expression.
 * @param {Expression} expr
 * @param {string} variable
 * @param {number} value
 * @returns {Expression}
 */
export function substituteVariable(expr, variable, value) {
  /** @param {Expression} e */
  const sub = (e) => substituteVariable(e, variable, value);
  switch (expr.tag) {
    case 'Variable':
      if (expr._0 === variable) {
        return Expression.number(Decimal.fromF64(value));
      }
      return expr;
    case 'Number':
    case 'DateTime':
    case 'Now':
    case 'Today':
      return expr;
    case 'Until':
      return Expression.Until(sub(expr._0));
    case 'Binary':
      return Expression.binary(sub(expr.left), expr.op, sub(expr.right));
    case 'Negate':
      return Expression.negate(sub(expr._0));
    case 'Group':
      return Expression.group(sub(expr._0));
    case 'Power':
      return Expression.power(sub(expr.base), sub(expr.exponent));
    case 'FunctionCall':
      return Expression.functionCall(
        expr.name,
        expr.args.map((/** @type {Expression} */ a) => sub(a)),
      );
    case 'AtTime':
      return Expression.atTime(sub(expr.value), sub(expr.time));
    case 'IndefiniteIntegral':
      return Expression.indefiniteIntegral(sub(expr.integrand), expr.variable);
    case 'UnitConversion':
      return Expression.unitConversion(sub(expr.value), expr.targetUnit);
    case 'Equality':
      return Expression.equality(sub(expr.left), sub(expr.right));
    case 'Comparison':
      return Expression.comparison(sub(expr.left), expr.op, sub(expr.right));
    default:
      return expr;
  }
}
