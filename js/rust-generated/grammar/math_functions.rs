// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2a601266b3e0f8f53786df4649a249819657954a2dc92312660a47db621815c6 bytes=14497

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Mathematical functions implementation.
// |  *
// |  * This module provides implementations for various mathematical functions
// |  * including trigonometry, logarithms, and numerical integration.
// |  *
// |  * Port of `src/grammar/math_functions.rs`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from '../rust/core.js';
use crate::rust::core_::err;
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as libm from '../rust/libm.js';
// meta-language:translated JavaScript import_statement items=1 sha256=96013caf537d0a269e190eb17882d0b9194fb4af7989e191055544479d2ebb54
// | import { E, fmax, fmin, PI, round, signum, toDegrees, toRadians, U32, U64 } from '../rust/num.js';
use crate::rust::num::{E, fmax, fmin, PI, round, signum, to_degrees, to_radians, U32, U64};
// meta-language:translated JavaScript import_statement items=1 sha256=6a58e9adb3b15ffddb375f0e6d3ccd15d27e918ab78e0d0474c5b280ad9f79ac
// | import { eqIgnoreAsciiCase } from '../rust/str.js';
use crate::rust::str_::eqIgnoreAsciiCase;
// meta-language:translated JavaScript import_statement items=1 sha256=47d2bc07b043da3bb831bba7db6482bc9b1698c6e70d433277ecb355f6eb6469
// | import { Decimal, evaluateRatioFunction, Rational, Value } from '../types/index.js';
use crate::types::index::{Decimal, evaluateRatioFunction, Rational, Value};
// meta-language:translated JavaScript import_statement items=1 sha256=aec1cfa64b680e4c46fa241c044c15b5afef1aad16700ec908b73c1506537d59
// | import { dimensionalFunction } from './dimensional_functions.js';
use crate::grammar::dimensional_functions::dimensionalFunction;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5341b3a146f31c0a1847285b77d6db96f57cd58f2fc1473240ef6e9bc27f05f7
// | /**
// |  * Largest `n` for which `n!` is computed exactly (35 660 digits); larger
// |  * arguments report overflow instead of allocating unbounded memory.
// |  */
// | const MAX_EXACT_FACTORIAL = 10_000;
pub const MAX_EXACT_FACTORIAL: f64 = 10000f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=fa5c7f1916562fa1eebac5a9de1d3e5bb6c23f537383927873c082dbb0cc054d
// | /** The number of subdivisions for numerical integration (Simpson's rule). */
// | const INTEGRATION_SUBDIVISIONS = 1000;
pub const INTEGRATION_SUBDIVISIONS: f64 = 1000f64;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// | Evaluates a mathematical function with the given arguments.
// |  *
// |  * # Supported Functions
// |  *
// |  * ## Trigonometric (arguments in radians)
// |  * - `sin(x)` - Sine
// |  * - `cos(x)` - Cosine
// |  * - `tan(x)` - Tangent
// |  * - `asin(x)` - Arc sine
// |  * - `acos(x)` - Arc cosine
// |  * - `atan(x)` - Arc tangent
// |  * - `sinh(x)` - Hyperbolic sine
// |  * - `cosh(x)` - Hyperbolic cosine
// |  * - `tanh(x)` - Hyperbolic tangent
// |  *
// |  * ## Exponential and Logarithmic
// |  * - `exp(x)` - e^x
// |  * - `ln(x)` - Natural logarithm
// |  * - `log(x)` - Base-10 logarithm
// |  * - `log2(x)` - Base-2 logarithm
// |  * - `pow(base, exp)` - Power function
// |  *
// |  * ## Other
// |  * - `sqrt(x)` - Square root
// |  * - `abs(x)` - Absolute value
// |  * - `floor(x)` - Floor
// |  * - `ceil(x)` - Ceiling
// |  * - `round(x)` - Round to nearest
// |  * - `factorial(n)` - Factorial (n must be non-negative integer)
// |  *
// |  * ## Constants
// |  * - `pi()` - π ≈ 3.14159...
// |  * - `e()` - Euler's number ≈ 2.71828...
// |  *
// |  *
// |  * Throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {Decimal[]} args
// |  * @returns {Decimal}
// |  */
// | export function evaluateFunction(name, args) {
// |   const nameLower = name.toLowerCase();
// |
// |   switch (nameLower) {
// |     // Constants
// |     case 'pi': {
// |       checkArgCount(nameLower, args, 0);
// |       return checkedDecimal(PI);
// |     }
// |     case 'e': {
// |       checkArgCount(nameLower, args, 0);
// |       return checkedDecimal(E);
// |     }
// |
// |     // Trigonometric functions
// |     case 'sin': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.sin(x));
// |     }
// |     case 'cos': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.cos(x));
// |     }
// |     case 'tan': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       const result = libm.tan(x);
// |       if (!Number.isFinite(result)) {
// |         return err(CalculatorError.domain('tan is undefined at this value'));
// |       }
// |       return checkedDecimal(result);
// |     }
// |     case 'asin': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (!(x >= -1.0 && x <= 1.0)) {
// |         return err(CalculatorError.domain('asin argument must be in [-1, 1]'));
// |       }
// |       return checkedDecimal(libm.asin(x));
// |     }
// |     case 'acos': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (!(x >= -1.0 && x <= 1.0)) {
// |         return err(CalculatorError.domain('acos argument must be in [-1, 1]'));
// |       }
// |       return checkedDecimal(libm.acos(x));
// |     }
// |     case 'atan': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.atan(x));
// |     }
// |     case 'atan2': {
// |       checkArgCount(nameLower, args, 2);
// |       const y = args[0].toF64();
// |       const x = args[1].toF64();
// |       return checkedDecimal(libm.atan2(y, x));
// |     }
// |
// |     // Hyperbolic functions
// |     case 'sinh': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.sinh(x));
// |     }
// |     case 'cosh': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.cosh(x));
// |     }
// |     case 'tanh': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.tanh(x));
// |     }
// |
// |     // Exponential and logarithmic
// |     case 'exp': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       const result = libm.exp(x);
// |       if (result === Infinity || result === -Infinity) {
// |         return err(CalculatorError.Overflow);
// |       }
// |       return checkedDecimal(result);
// |     }
// |     case 'ln': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (x <= 0.0) {
// |         return err(CalculatorError.domain('ln argument must be positive'));
// |       }
// |       return checkedDecimal(libm.log(x));
// |     }
// |     case 'log': {
// |       // log(x) is log base 10, log(x, base) is log base `base`
// |       if (args.length === 0 || args.length > 2) {
// |         return err(CalculatorError.invalidArgs(nameLower, 'expected 1 or 2 arguments'));
// |       }
// |       const x = args[0].toF64();
// |       if (x <= 0.0) {
// |         return err(CalculatorError.domain('log argument must be positive'));
// |       }
// |       if (args.length === 2) {
// |         const base = args[1].toF64();
// |         if (base <= 0.0 || base === 1.0) {
// |           return err(CalculatorError.domain('log base must be positive and not 1'));
// |         }
// |         return checkedDecimal(libm.log(x) / libm.log(base));
// |       }
// |       return checkedDecimal(libm.log10(x));
// |     }
// |     case 'log2': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (x <= 0.0) {
// |         return err(CalculatorError.domain('log2 argument must be positive'));
// |       }
// |       return checkedDecimal(libm.log2(x));
// |     }
// |     case 'log10': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (x <= 0.0) {
// |         return err(CalculatorError.domain('log10 argument must be positive'));
// |       }
// |       return checkedDecimal(libm.log10(x));
// |     }
// |     case 'pow': {
// |       checkArgCount(nameLower, args, 2);
// |       const base = args[0].toF64();
// |       const exp = args[1].toF64();
// |       const result = libm.pow(base, exp);
// |       if (Number.isNaN(result)) {
// |         return err(
// |           CalculatorError.domain('pow result is undefined (e.g., negative base with fractional exponent)'),
// |         );
// |       }
// |       if (result === Infinity || result === -Infinity) {
// |         return err(CalculatorError.Overflow);
// |       }
// |       return checkedDecimal(result);
// |     }
// |
// |     // Other mathematical functions
// |     case 'sqrt': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       if (x < 0.0) {
// |         return err(CalculatorError.domain('sqrt argument must be non-negative'));
// |       }
// |       return checkedDecimal(Math.sqrt(x));
// |     }
// |     case 'cbrt': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(libm.cbrt(x));
// |     }
// |     case 'abs': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(Math.abs(x));
// |     }
// |     case 'floor': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(Math.floor(x));
// |     }
// |     case 'ceil': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(Math.ceil(x));
// |     }
// |     case 'round': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(round(x));
// |     }
// |     case 'trunc': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(Math.trunc(x));
// |     }
// |     case 'sign':
// |     case 'signum': {
// |       checkArgCount(nameLower, args, 1);
// |       const x = args[0].toF64();
// |       return checkedDecimal(signum(x));
// |     }
// |     case 'min': {
// |       if (args.length < 2) {
// |         return err(CalculatorError.invalidArgs(nameLower, 'expected at least 2 arguments'));
// |       }
// |       const min = args.map((arg) => arg.toF64()).reduce((acc, x) => fmin(acc, x), Infinity);
// |       return checkedDecimal(min);
// |     }
// |     case 'max': {
// |       if (args.length < 2) {
// |         return err(CalculatorError.invalidArgs(nameLower, 'expected at least 2 arguments'));
// |       }
// |       const max = args.map((arg) => arg.toF64()).reduce((acc, x) => fmax(acc, x), -Infinity);
// |       return checkedDecimal(max);
// |     }
// |     case 'factorial': {
// |       checkArgCount(nameLower, args, 1);
// |       const n = args[0].toF64();
// |       if (n < 0.0 || n !== Math.floor(n)) {
// |         return err(CalculatorError.domain('factorial argument must be a non-negative integer'));
// |       }
// |       const nInt = U64.fromF64(n);
// |       if (nInt > 170n) {
// |         return err(CalculatorError.Overflow);
// |       }
// |       const result = factorial(Number(nInt));
// |       return checkedDecimal(result);
// |     }
// |     case 'mod':
// |     case 'modulo': {
// |       checkArgCount(nameLower, args, 2);
// |       const result = args[0].checkedRem(args[1]);
// |       if (result === null) {
// |         return err(CalculatorError.domain('modulo by zero'));
// |       }
// |       return result;
// |     }
// |
// |     // Conversion functions
// |     case 'deg':
// |     case 'degrees': {
// |       checkArgCount(nameLower, args, 1);
// |       const radians = args[0].toF64();
// |       return checkedDecimal(toDegrees(radians));
// |     }
// |     case 'rad':
// |     case 'radians': {
// |       checkArgCount(nameLower, args, 1);
// |       const degrees = args[0].toF64();
// |       return checkedDecimal(toRadians(degrees));
// |     }
// |
// |     default:
// |       return err(CalculatorError.unknownFunction(name));
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const MATH_FUNCTIONS = new Set([
// |   'sin',
// |   'cos',
// |   'tan',
// |   'asin',
// |   'acos',
// |   'atan',
// |   'atan2',
// |   'sinh',
// |   'cosh',
// |   'tanh',
// |   'exp',
// |   'ln',
// |   'log',
// |   'log2',
// |   'log10',
// |   'pow',
// |   'sqrt',
// |   'cbrt',
// |   'abs',
// |   'floor',
// |   'ceil',
// |   'round',
// |   'trunc',
// |   'sign',
// |   'signum',
// |   'min',
// |   'max',
// |   'integrate',
// |   'factorial',
// |   'mod',
// |   'modulo',
// |   'uncertainty',
// |   'pi',
// |   'e',
// |   'deg',
// |   'degrees',
// |   'rad',
// |   'radians',
// |   'percent',
// |   'as_percent',
// |   'as_fraction',
// |   'as_reciprocal',
// |   'as_multiplier',
// |   'as_decimal',
// |   'percent_of',
// | ]);

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * Returns true if the given name is a known math function.
// |  * @param {string} name
// |  * @returns {boolean}
// |  */
// | export function isMathFunction(name) {
// |   const nameLower = name.toLowerCase();
// |   return MATH_FUNCTIONS.has(nameLower);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Evaluates a function on already-evaluated argument values.
// |  *
// |  * Functions with an exact integer definition are computed with arbitrary
// |  * precision (e.g. `30!` = `265252859812191058636308480000000`); everything
// |  * else goes through {@link evaluateFunction} on `Decimal` arguments.
// |  *
// |  * Throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {Value[]} args
// |  * @returns {Value}
// |  */
// | export function evaluateFunctionValues(name, args) {
// |   if (eqIgnoreAsciiCase(name, 'uncertainty')) {
// |     if (args.length !== 2) {
// |       return err(CalculatorError.invalidArgs(name, 'expected 2 arguments'));
// |     }
// |     return args[0].withUncertainty(args[1]);
// |   }
// |   const ratio = evaluateRatioFunction(name, args);
// |   if (ratio !== null) {
// |     return ratio;
// |   }
// |   if (args.some((arg) => arg.unit.isOpaqueCompound())) {
// |     return err(CalculatorError.invalidArgs(name, 'functions of compound units are not supported'));
// |   }
// |   const dimensional = dimensionalFunction(name, args);
// |   if (dimensional !== null) {
// |     return dimensional;
// |   }
// |   const exact = evaluateExactFunction(name, args);
// |   if (exact !== null) {
// |     return exact;
// |   }
// |   const decimals = args.map((arg) => {
// |     const decimal = arg.asDecimal();
// |     if (decimal === null) {
// |       return err(CalculatorError.invalidArgs(name, 'expected numeric argument'));
// |     }
// |     return decimal;
// |   });
// |   return Value.number(evaluateFunction(name, decimals));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Exact big-integer evaluation; `None` (`null`) means "not applicable, use
// |  * `Decimal`". The `Some(Err)` case throws `Err(CalculatorError)`.
// |  * @param {string} name
// |  * @param {Value[]} args
// |  * @returns {Value | null}
// |  */
// | function evaluateExactFunction(name, args) {
// |   const nameLower = name.toLowerCase();
// |   if (!(nameLower === 'factorial' || nameLower === 'fact') || args.length !== 1) {
// |     return null;
// |   }
// |   const rational = args[0].toRational();
// |   if (rational === null || !rational.isInteger()) {
// |     return null;
// |   }
// |   const n = rational;
// |   if (n.isNegative()) {
// |     return err(CalculatorError.domain('factorial argument must be a non-negative integer'));
// |   }
// |   const small = U32.tryFrom(n.numer());
// |   if (small === null || small > BigInt(MAX_EXACT_FACTORIAL)) {
// |     return err(CalculatorError.Overflow);
// |   }
// |   let product = 1n;
// |   for (let k = 2n; k <= small; k += 1n) {
// |     product *= k;
// |   }
// |   return Value.rational(Rational.fromBigint(product));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Converts a function result to `Decimal`, reporting overflow instead of
// |  * silently returning zero when the value is outside `Decimal`'s range
// |  * (previously `30!` or `exp(70)` evaluated to `0`).
// |  *
// |  * Throws `Err(CalculatorError)`.
// |  * @param {number} value
// |  * @returns {Decimal}
// |  */
// | function checkedDecimal(value) {
// |   if (Number.isNaN(value)) {
// |     return err(CalculatorError.domain('result is undefined'));
// |   }
// |   const decimal = Decimal.tryFromF64(value);
// |   if (decimal === null) {
// |     return err(CalculatorError.Overflow);
// |   }
// |   return decimal;
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * Checks that the function received the expected number of arguments.
// |  *
// |  * Throws `Err(CalculatorError)`.
// |  * @param {string} funcName
// |  * @param {Decimal[]} args
// |  * @param {number} expected
// |  */
// | function checkArgCount(funcName, args, expected) {
// |   if (args.length !== expected) {
// |     err(
// |       CalculatorError.invalidArgs(funcName, `expected ${expected} argument(s), got ${args.length}`),
// |     );
// |   }
// | }

// meta-language:translated JavaScript function_declaration items=2 sha256=ab9b4acc3812512079eff0afc3426ffe2c63c027a5e09683fef5f83641551610
// | /**
// |  * Computes factorial of n.
// |  * @param {number} n
// |  * @returns {number}
// |  */
// | function factorial(n) {
// |   if (n <= 1) {
// |     return 1.0;
// |   }
// |   let acc = 1.0;
// |   for (let x = 2; x <= n; x += 1) {
// |     acc *= x;
// |   }
// |   return acc;
// | }
pub fn factorial(n: f64) -> f64 {
    if (n <= (1f64)) {
        1f64
    } else {
        {
            let acc = 1f64;
            {
                let x = 2f64;
                {
                    let acc_2 = crate::ml_factorial_loop1(n, acc, x);
                    acc_2
                }
            }
        }
    }
}

pub fn ml_factorial_loop1(mut n: f64, mut acc: f64, mut x: f64) -> f64 {
    loop {
        return if (x <= n) {
            {
                let acc_2 = (acc * x);
                {
                    let x_2 = (x + 1f64);
                    {
                        (acc, x) = (acc_2, x_2);
                        continue;
                    }
                }
            }
        } else {
            acc
        };
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Performs numerical integration using Simpson's rule.
// |  *
// |  * Integrates the function `f` from `a` to `b` using Simpson's rule
// |  * with `INTEGRATION_SUBDIVISIONS` intervals.
// |  * @param {(x: number) => number} f
// |  * @param {number} a
// |  * @param {number} b
// |  * @returns {number}
// |  */
// | export function integrate(f, a, b) {
// |   const n = INTEGRATION_SUBDIVISIONS;
// |   const h = (b - a) / n;
// |
// |   let sum = f(a) + f(b);
// |
// |   for (let i = 1; i < n; i += 1) {
// |     const x = libm.fma(i, h, a);
// |     if (i % 2 === 0) {
// |       sum = libm.fma(2.0, f(x), sum);
// |     } else {
// |       sum = libm.fma(4.0, f(x), sum);
// |     }
// |   }
// |
// |   return (sum * h) / 3.0;
// | }
