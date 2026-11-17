// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3cefdf13c9849c0b480f77d3a692feb2ec70d4d6f62ae72eb33787c81cdc8050 bytes=5866

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Powers, roots and functions of dimensional values (issue #235).
// |  *
// |  * `(2 m)^2` is `4 m^2`, `sqrt(4 m^2)` is `2 m` and `sqrt(2 m)` fails with a
// |  * non-integer dimension. Rounding keeps the unit, trigonometric functions
// |  * accept angles (`sin(30°)`), and other functions need plain numbers.
// |  *
// |  * Port of `src/grammar/dimensional_functions.rs`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=234ac7040c5dcb58f4b62f9ddbe30f7246f7dc5022ce4a4eb5010dd9dd621c8e
// | import { eq, err } from '../rust/core.js';
use crate::rust::core_::{eq, err};
// meta-language:translated JavaScript import_statement items=1 sha256=244efa3dd154e8dd122584d793652d5328e81fb0eec77e3eee517553d303d5ca
// | import { I32, I64, round } from '../rust/num.js';
use crate::rust::num::{I32, I64, round};
// meta-language:translated JavaScript import_statement items=1 sha256=d299b9b45b9e26f5230f1248ef8828f6c8cd562eeee49ce7d5167a41ef8023c0
// | import { Dimension, normalizeAffine, Rational, Unit, Value } from '../types/index.js';
use crate::types::index::{Dimension, normalizeAffine, Rational, Unit, Value};

// meta-language:translated JavaScript import_statement items=1 sha256=48f643c4ed1738fb4ba46cd20b48f76ff75e1d9d76a21300e45d06e86dee2ee3
// | import { evaluateFunctionValues } from './math_functions.js';
use crate::grammar::math_functions::evaluateFunctionValues;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Reads a temperature result in kelvin unless it was converted explicitly:
// |  * `20 °C` is `293.15 K` as in Numbat, while `25 °C in °F` stays `77 °F`.
// |  * Inside expressions `°C` keeps its offset, so `25 °C * 2` is rejected.
// |  * @param {import('../types/expression.js').Expression} expr
// |  * @param {Value} value
// |  * @returns {Value}
// |  */
// | export function kelvinResult(expr, value) {
// |   let inner = expr;
// |   while (inner.tag === 'Group') {
// |     inner = inner._0;
// |   }
// |   if (inner.tag === 'UnitConversion') {
// |     return value;
// |   }
// |   const amount = value.toRational();
// |   if (amount === null) {
// |     return value;
// |   }
// |   const [normalized, unit] = normalizeAffine(amount, value.unit);
// |   if (eq(unit, value.unit)) {
// |     return value;
// |   }
// |   return Value.rationalWithUnit(normalized, unit);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Raises a dimensional value to a rational power; `None` (`null`) when the
// |  * base has no dimensional unit. The `Some(Err)` case throws
// |  * `Err(CalculatorError)`.
// |  * @param {Value} base
// |  * @param {Value} exponent
// |  * @returns {Value | null}
// |  */
// | export function dimensionalPower(base, exponent) {
// |   if (base.physicalBase() === null) {
// |     return null;
// |   }
// |   if (!eq(exponent.unit, Unit.None)) {
// |     return err(CalculatorError.domain('an exponent must be a plain number'));
// |   }
// |   const exponentRational = exponent.toRational();
// |   if (exponentRational === null) {
// |     return err(CalculatorError.domain('an exponent must be a plain number'));
// |   }
// |   let power = I32.tryFrom(exponentRational.numer());
// |   if (power !== null && !((power < 0n ? -power : power) <= 1000n)) {
// |     power = null;
// |   }
// |   let root = I32.tryFrom(exponentRational.denom());
// |   if (root !== null && !(root <= 1000n)) {
// |     root = null;
// |   }
// |   if (power === null || root === null) {
// |     return err(
// |       CalculatorError.domain(
// |         `power ${exponentRational} of a quantity with units is not supported`,
// |       ),
// |     );
// |   }
// |   if (root === 1n) {
// |     return base.physicalPower(Number(power));
// |   }
// |   const rooted = base.physicalRoot(Number(root));
// |   if (rooted === null) {
// |     return null;
// |   }
// |   return rooted.physicalPower(Number(power)) ?? rooted;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Evaluates a one-argument function of a dimensional value; `None` (`null`)
// |  * when the argument is a plain number or the function has no dimensional
// |  * meaning. The `Some(Err)` case throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {Value[]} args
// |  * @returns {Value | null}
// |  */
// | export function dimensionalFunction(name, args) {
// |   if (args.length !== 1) {
// |     return null;
// |   }
// |   const arg = args[0];
// |   const physical = arg.physicalBase();
// |   if (physical === null) {
// |     return null;
// |   }
// |   const [base, dimension] = physical;
// |   const lower = name.toLowerCase();
// |   switch (lower) {
// |     case 'sqrt':
// |       return arg.physicalRoot(2);
// |     case 'cbrt':
// |       return arg.physicalRoot(3);
// |     case 'abs':
// |     case 'floor':
// |     case 'ceil':
// |     case 'round':
// |     case 'trunc': {
// |       const amount = arg.toRational();
// |       if (amount === null) {
// |         return null;
// |       }
// |       const value = evaluateFunctionValues(name, [Value.rational(amount)]);
// |       const rounded = value.toRational();
// |       if (rounded === null) {
// |         return err(CalculatorError.invalidArgs(name, 'expected numeric result'));
// |       }
// |       return Value.rationalWithUnit(rounded, arg.unit);
// |     }
// |     default:
// |   }
// |   if (
// |     (lower === 'sin' ||
// |       lower === 'cos' ||
// |       lower === 'tan' ||
// |       lower === 'sec' ||
// |       lower === 'csc' ||
// |       lower === 'cot') &&
// |     eq(Dimension.base('angle'), dimension)
// |   ) {
// |     return trigonometric(name, lower, base, arg);
// |   }
// |   if (arg.unit.tag === 'Physical') {
// |     return err(
// |       CalculatorError.invalidArgs(name, `expected a plain number, got ${arg.toDisplayString()}`),
// |     );
// |   }
// |   return null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Evaluates a trigonometric function of an angle in radians, giving exact
// |  * values at multiples of a right angle (`sin(180°)` is `0`).
// |  *
// |  * Throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {string} lower
// |  * @param {Rational} radians
// |  * @param {Value} arg
// |  * @returns {Value}
// |  */
// | function trigonometric(name, lower, radians, arg) {
// |   const value = evaluateFunctionValues(name, [Value.rational(radians)]);
// |   const quarterTurns = radians.toF64() / (Math.PI / 2);
// |   const onAxis = Math.abs(quarterTurns - round(quarterTurns)) < 1e-9;
// |   if (!onAxis) {
// |     return value;
// |   }
// |   const rounded = I64.fromF64(round(quarterTurns));
// |   const quadrant = Number(((rounded % 4n) + 4n) % 4n);
// |   const [sin, cos] = [
// |     [0, 1],
// |     [1, 0],
// |     [0, -1],
// |     [-1, 0],
// |   ][quadrant];
// |   /** @type {[number, number] | null} */
// |   let exact;
// |   switch (lower) {
// |     case 'sin':
// |       exact = [sin, 1];
// |       break;
// |     case 'cos':
// |       exact = [cos, 1];
// |       break;
// |     case 'tan':
// |       exact = [sin, cos];
// |       break;
// |     case 'cot':
// |       exact = [cos, sin];
// |       break;
// |     case 'sec':
// |       exact = [1, cos];
// |       break;
// |     case 'csc':
// |       exact = [1, sin];
// |       break;
// |     default:
// |       exact = null;
// |   }
// |   if (exact === null) {
// |     return value;
// |   }
// |   const [numer, denom] = exact;
// |   if (denom === 0) {
// |     return err(CalculatorError.domain(`${name} is undefined at ${arg.toDisplayString()}`));
// |   }
// |   return Value.rational(Rational.new(numer, denom));
// | }
