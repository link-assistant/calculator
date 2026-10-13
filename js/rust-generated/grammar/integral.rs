// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=51ab2d3ee549ba1274a40595ed37f3c910f1e1d2ceb98b17d7bd10b444071461 bytes=5320

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Symbolic integral evaluation module.
// |  *
// |  * This module handles symbolic integration for indefinite integrals,
// |  * computing results for known special cases like Si(x), Ci(x), etc.
// |  *
// |  * Port of `src/grammar/integral.rs`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from '../rust/core.js';
use crate::rust::core_::err;
// meta-language:translated JavaScript import_statement items=1 sha256=a2956f9b99d1ae00e39b539e3292a63af4da397cc8d21192cc6003308db3f487
// | import { fmtF64 } from '../rust/fmt.js';
use crate::rust::fmt::fmtF64;
// meta-language:translated JavaScript import_statement items=1 sha256=827d765e4f783ec09d9f789fb1048d58ee3712f2dc5d1257842287980428a177
// | import { replace } from '../rust/str.js';
use crate::rust::str_::replace;
// meta-language:translated JavaScript import_statement items=1 sha256=926f6ff83092b78f2d57dbd1083452b6edea1f872a548e0012b2426d31b74258
// | import { Expression } from '../types/index.js';
use crate::types::index::Expression;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Evaluates an indefinite integral.
// |  *
// |  * For known special integrals, returns symbolic result.
// |  * For others, returns an informational message.
// |  *
// |  * Always throws `Err(CalculatorError.SymbolicResult)`.
// |  * @param {Expression} integrand
// |  * @param {string} variable
// |  * @returns {import('../types/index.js').Value}
// |  */
// | export function evaluateIndefiniteIntegral(integrand, variable) {
// |   // Check for known special integrals
// |   const symbolicResult = trySymbolicIntegral(integrand, variable);
// |
// |   // Generate the links notation for the integral expression
// |   const integralExpr = Expression.indefiniteIntegral(integrand, variable);
// |   const lino = integralExpr.toLino();
// |
// |   if (symbolicResult !== null) {
// |     // Return a special value that indicates symbolic result
// |     // For now, we'll create an error with the symbolic result as a message
// |     // since the Value type doesn't support symbolic results yet
// |     const latexResult = symbolicResultToLatex(symbolicResult);
// |     return err(
// |       CalculatorError.SymbolicResult({
// |         expression: lino,
// |         result: symbolicResult,
// |         latexInput: `\\int ${integrand.toLatex()} \\, d${variable}`,
// |         latexResult,
// |       }),
// |     );
// |   }
// |   // For unknown integrals, provide a helpful message
// |   return err(
// |     CalculatorError.SymbolicResult({
// |       expression: lino,
// |       result:
// |         'Cannot compute symbolic result. Use definite integral with bounds: integrate(expr, var, lower, upper)',
// |       latexInput: `\\int ${integrand.toLatex()} \\, d${variable}`,
// |       latexResult: '\\text{Use definite integral with bounds}',
// |     }),
// |   );
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Tries to compute a symbolic integral for known special cases.
// |  * @param {Expression} integrand
// |  * @param {string} variable
// |  * @returns {string | null}
// |  */
// | export function trySymbolicIntegral(integrand, variable) {
// |   // Pattern: sin(x)/x -> Si(x) + C (Sine Integral)
// |   if (integrand.tag === 'Binary' && integrand.op.tag === 'Divide') {
// |     const { left, right } = integrand;
// |     if (left.tag === 'FunctionCall') {
// |       const { name, args } = left;
// |       if (name.toLowerCase() === 'sin' && args.length === 1) {
// |         if (args[0].tag === 'Variable') {
// |           const v = args[0]._0;
// |           if (right.tag === 'Variable') {
// |             const v2 = right._0;
// |             if (v === variable && v2 === variable) {
// |               return `Si(${variable}) + C`;
// |             }
// |           }
// |         }
// |       }
// |     }
// |   }
// |
// |   // Pattern: cos(x)/x -> Ci(x) + C (Cosine Integral)
// |   if (integrand.tag === 'Binary' && integrand.op.tag === 'Divide') {
// |     const { left, right } = integrand;
// |     if (left.tag === 'FunctionCall') {
// |       const { name, args } = left;
// |       if (name.toLowerCase() === 'cos' && args.length === 1) {
// |         if (args[0].tag === 'Variable') {
// |           const v = args[0]._0;
// |           if (right.tag === 'Variable') {
// |             const v2 = right._0;
// |             if (v === variable && v2 === variable) {
// |               return `Ci(${variable}) + C`;
// |             }
// |           }
// |         }
// |       }
// |     }
// |   }
// |
// |   // Pattern: x^n -> x^(n+1)/(n+1) + C
// |   if (integrand.tag === 'Power') {
// |     const { base, exponent } = integrand;
// |     if (base.tag === 'Variable') {
// |       const v = base._0;
// |       if (v === variable) {
// |         if (exponent.tag === 'Number') {
// |           const n = exponent.value.toF64();
// |           if (Math.abs(n - -1.0) > 1e-10) {
// |             // Not x^(-1)
// |             const newExp = n + 1.0;
// |             return `${variable}^${fmtF64(newExp)}/(${fmtF64(newExp)}) + C`;
// |           }
// |           // x^(-1) = 1/x -> ln|x| + C
// |           return `ln|${variable}| + C`;
// |         }
// |       }
// |     }
// |   }
// |
// |   // Pattern: just x -> x^2/2 + C
// |   if (integrand.tag === 'Variable') {
// |     const v = integrand._0;
// |     if (v === variable) {
// |       return `${variable}²/2 + C`;
// |     }
// |   }
// |
// |   // Pattern: constant -> constant * x + C
// |   if (integrand.tag === 'Number') {
// |     const { value } = integrand;
// |     return `${value.toString()} * ${variable} + C`;
// |   }
// |
// |   // Pattern: sin(x) -> -cos(x) + C
// |   if (integrand.tag === 'FunctionCall') {
// |     const { name, args } = integrand;
// |     if (args.length === 1) {
// |       if (args[0].tag === 'Variable') {
// |         const v = args[0]._0;
// |         if (v === variable) {
// |           switch (name.toLowerCase()) {
// |             case 'sin':
// |               return `-cos(${variable}) + C`;
// |             case 'cos':
// |               return `sin(${variable}) + C`;
// |             case 'exp':
// |               return `exp(${variable}) + C`;
// |             default:
// |           }
// |         }
// |       }
// |     }
// |   }
// |
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * Converts a symbolic result to LaTeX.
// |  * @param {string} result
// |  * @returns {string}
// |  */
// | export function symbolicResultToLatex(result) {
// |   // Basic conversions
// |   let latex = replace(result, 'Si(', '\\text{Si}(');
// |   latex = replace(latex, 'Ci(', '\\text{Ci}(');
// |   latex = replace(latex, 'ln|', '\\ln|');
// |   return replace(latex, '²', '^{2}');
// | }
