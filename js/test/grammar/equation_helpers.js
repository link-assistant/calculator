// Expression builders for the equation tests (the JS parser is not ported yet).
import { BinaryOp, Decimal, Expression } from '../../src/types/index.js';

/** @param {string} s */
export const n = (s) => Expression.number(Decimal.fromStr(s));
/** @param {string} name */
export const v = (name) => Expression.Variable(name);
/** @param {Expression} left @param {Expression} right */
export const add = (left, right) => Expression.Binary({ left, op: BinaryOp.Add, right });
/** @param {Expression} left @param {Expression} right */
export const sub = (left, right) => Expression.Binary({ left, op: BinaryOp.Subtract, right });
/** @param {Expression} left @param {Expression} right */
export const mul = (left, right) => Expression.Binary({ left, op: BinaryOp.Multiply, right });
/** @param {Expression} left @param {Expression} right */
export const div = (left, right) => Expression.Binary({ left, op: BinaryOp.Divide, right });
/** @param {Expression} base @param {Expression} exponent */
export const pow = (base, exponent) => Expression.Power({ base, exponent });
/** @param {Expression} inner */
export const group = (inner) => Expression.Group(inner);
