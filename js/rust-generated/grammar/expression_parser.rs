// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=587b0c1ac0c5e3fad83877994e1d2f95c78b16e045425c097d18524652709924 bytes=32770

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Expression parser that combines all grammars (port of
// |  * `src/grammar/expression_parser.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=a9d273d0d0d19a08a5b97dde46204a6498660b7b11ac573012e17879f46112ff
// | import { eq, err, tryResult, unreachable } from '../rust/core.js';
use crate::rust::core_::{eq, err, tryResult, unreachable};
// meta-language:translated JavaScript import_statement items=1 sha256=bc21746fe1ea8f721392a0ad91fec77b2254a6549e9fcdd4e82af18ac20da551
// | import { fma, pow } from '../rust/libm.js';
use crate::rust::libm::{fma, pow};
// meta-language:translated JavaScript import_statement items=1 sha256=f794ec57e30a3721cfa7a9110f6cb1529861e227164553059f1def65f32a079e
// | import { I32 } from '../rust/num.js';
use crate::rust::num::I32;
// meta-language:translated JavaScript import_statement items=1 sha256=b0328aca8cb0949bb3bfaa2dc434589a65f5056db1104a1561dd80c4b878ffcc
// | import { trim } from '../rust/str.js';
use crate::rust::str_::trim;
// meta-language:translated JavaScript import_statement items=1 sha256=00a5edeb748dc5c17b4b0febc245ee62a86ab0b3915e4ec66eb7cd9adf1986f5
// | import {
// |   BinaryOp, ComparisonOp, CurrencyDatabase, Decimal, Rational, Value,
// | } from '../types/index.js';
use crate::types::index::{BinaryOp, ComparisonOp, CurrencyDatabase, Decimal, Rational, Value};
// meta-language:translated JavaScript import_statement items=1 sha256=56c1478162f05e40ea70a94d1cda8081d36a0940d9b7b6496f6357122e0f51fa
// | import { DateTimeGrammar } from './datetime_grammar.js';
use crate::grammar::datetime_grammar::DateTimeGrammar;
// meta-language:translated JavaScript import_statement items=1 sha256=ff09659fa3117ceb702727620949b8e705d946d59766afcc56fe543b2c639a0d
// | import { dimensionalPower, kelvinResult } from './dimensional_functions.js';
use crate::grammar::dimensional_functions::{dimensionalPower, kelvinResult};
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as locale from './expression_parser_locale.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as timezone from './expression_parser_timezone.js';
// meta-language:translated JavaScript import_statement items=1 sha256=40f305b913528bced5ced22e3f071e72ae4ed8d9250447ea4058820243937acf
// | import { evaluateIndefiniteIntegral } from './integral.js';
use crate::grammar::integral::evaluateIndefiniteIntegral;
// meta-language:translated JavaScript import_statement items=1 sha256=4c70e2f8133d89b29a7040e3bb1b46af4ef8578c9d6c9905f2f29257c4ac72d5
// | import { Lexer } from './lexer.js';
use crate::grammar::lexer::Lexer;
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as linearEquation from './linear_equation.js';
// meta-language:translated JavaScript import_statement items=1 sha256=48f643c4ed1738fb4ba46cd20b48f76ff75e1d9d76a21300e45d06e86dee2ee3
// | import { evaluateFunctionValues } from './math_functions.js';
use crate::grammar::math_functions::evaluateFunctionValues;
// meta-language:translated JavaScript import_statement items=1 sha256=9c729167abcd53b361ad55e2ea7f88090270e1ebd62749f3f06a2b472ad02fe2
// | import { NumberGrammar } from './number_grammar.js';
use crate::grammar::number_grammar::NumberGrammar;
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as polynomialEquation from './polynomial_equation.js';
// meta-language:translated JavaScript import_statement items=1 sha256=83f70f18110fd2f11614aa0bd8190a56a8975c29c0731241f144c7b70c946e64
// | import { TokenParser } from './token_parser.js';
use crate::grammar::token_parser::TokenParser;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../types/expression.js').Expression} Expression */
// | /** @typedef {import('../types/datetime.js').DateTime} DateTime */
// | /** `std::cmp::Ordering` as -1, 0 or 1. @typedef {-1 | 0 | 1} Ordering */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Evaluates a power expression, using exact rational arithmetic when possible.
// |  *
// |  * When both base and exponent are rational and the exponent is an integer
// |  * that fits in i32, the computation is exact (arbitrary precision).
// |  * Otherwise, falls back to f64 computation.
// |  *
// |  * This function is exposed so downstream consumers can reproduce the
// |  * exact-versus-floating-point fallback used inside the evaluator.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {Value} baseVal
// |  * @param {Value} expVal
// |  * @returns {Value}
// |  */
// | export function evaluatePower(baseVal, expVal) {
// |   if (baseVal.unit.isOpaqueCompound() || expVal.unit.isOpaqueCompound()) {
// |     return err(CalculatorError.domain(
// |       'powers of compound units are not supported',
// |     ));
// |   }
// |   const dimensional = dimensionalPower(baseVal, expVal);
// |   if (dimensional !== null) {
// |     return dimensional;
// |   }
// |   // Try exact rational exponentiation first
// |   const baseRat = baseVal.toRational();
// |   const expRat = expVal.toRational();
// |   if (baseRat !== null && expRat !== null && expRat.isInteger()) {
// |     // Check exponent fits in i32 (reasonable range for exact computation)
// |     const expI32Big = I32.tryFrom(expRat.numer());
// |     if (expI32Big !== null) {
// |       const expI32 = Number(expI32Big);
// |       // Guard against absurdly large exponents that would consume too much memory
// |       if (Math.abs(expI32) <= 1_000_000) {
// |         if (expI32 < 0 && baseRat.isZero()) {
// |           return err(CalculatorError.domain(
// |             'division by zero (negative exponent with zero base)',
// |           ));
// |         }
// |         const result = baseRat.powI32(expI32);
// |         return Value.rational(result);
// |       }
// |     }
// |   }
// |
// |   // Fallback to f64 for non-integer exponents or very large exponents
// |   const baseDec = baseVal.asDecimal()
// |     ?? err(CalculatorError.InvalidOperation('power base must be numeric'));
// |   const expDec = expVal.asDecimal()
// |     ?? err(CalculatorError.InvalidOperation('power exponent must be numeric'));
// |
// |   const baseF64 = baseDec.toF64();
// |   const expF64 = expDec.toF64();
// |   const result = pow(baseF64, expF64);
// |
// |   if (Number.isNaN(result)) {
// |     return err(CalculatorError.domain('power result is undefined'));
// |   }
// |   if (!Number.isFinite(result)) {
// |     return err(CalculatorError.Overflow);
// |   }
// |
// |   return Value.number(Decimal.fromF64(result));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Self::expression_contains_variable`.
// |  * @param {Expression} expr
// |  * @returns {boolean}
// |  */
// | function expressionContainsVariable(expr) {
// |   switch (expr.tag) {
// |     case 'Variable':
// |       return true;
// |     case 'Until':
// |     case 'Negate':
// |     case 'Group':
// |       return expressionContainsVariable(expr._0);
// |     case 'Binary':
// |     case 'Equality':
// |     case 'Comparison':
// |       return expressionContainsVariable(expr.left) || expressionContainsVariable(expr.right);
// |     case 'Power':
// |       return expressionContainsVariable(expr.base) || expressionContainsVariable(expr.exponent);
// |     case 'AtTime':
// |       return expressionContainsVariable(expr.value) || expressionContainsVariable(expr.time);
// |     case 'FunctionCall':
// |       return expr.args.some(expressionContainsVariable);
// |     case 'IndefiniteIntegral':
// |       return expressionContainsVariable(expr.integrand);
// |     case 'UnitConversion':
// |       return expressionContainsVariable(expr.value);
// |     default:
// |       // Number, DateTime, Now, Today
// |       return false;
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Self::solve_equation`. Throws `Err(CalculatorError)`.
// |  * @param {Expression} left
// |  * @param {Expression} right
// |  * @returns {Value}
// |  */
// | function solveEquation(left, right) {
// |   const linear = tryResult(() => linearEquation.solve(left, right));
// |   if (linear.ok) {
// |     return linear.value.toValue();
// |   }
// |   return polynomialEquation.solve(left, right).toValue();
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Self::duration_seconds_for_comparison`.
// |  * @param {Value} value
// |  * @returns {number | null}
// |  */
// | function durationSecondsForComparison(value) {
// |   const kind = value.kind;
// |   const unit = value.unit;
// |   if (kind.tag === 'Duration' && unit.tag === 'None') {
// |     return Number(kind.seconds);
// |   }
// |   if (kind.tag === 'Number' && unit.tag === 'Duration') {
// |     return unit._0.toSecs(kind._0.toF64());
// |   }
// |   if (kind.tag === 'Rational' && unit.tag === 'Duration') {
// |     return unit._0.toSecs(kind._0.toF64());
// |   }
// |   return null;
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * `Self::ordering_symbol`.
// |  * @param {Ordering} ordering
// |  */
// | function orderingSymbol(ordering) {
// |   if (ordering < 0) {
// |     return '<';
// |   }
// |   return ordering === 0 ? '=' : '>';
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Appends "Exchange rate: ..." steps for the rates used by the last
// |  * conversion (several entries for cross-rate conversions).
// |  * @param {CurrencyDatabase} currencyDb
// |  * @param {string[]} steps
// |  */
// | function pushRateSteps(currencyDb, steps) {
// |   for (const [from, to, rateInfo] of currencyDb.getLastUsedRates()) {
// |     steps.push(`Exchange rate: ${rateInfo.formatForDisplay(from, to)}`);
// |   }
// | }

