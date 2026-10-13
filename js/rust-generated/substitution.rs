// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e47c7ad34d8feba9daa89542802c4ba1fd06a6f1e88e963b4104f32367e9b097 bytes=1849

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Variable substitution for plotting (port of `src/substitution.rs`, an
// |  * `impl Calculator` block; `Calculator.substituteVariable` delegates here).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=e679dc7f45b3574703b127df48809f052a30d68047b1cbade8d76a146f6675b8
// | import { Decimal, Expression } from './types/index.js';
use crate::types::index::{Decimal, Expression};

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Substitutes a variable with a numeric value in an expression.
// |  * @param {Expression} expr
// |  * @param {string} variable
// |  * @param {number} value
// |  * @returns {Expression}
// |  */
// | export function substituteVariable(expr, variable, value) {
// |   /** @param {Expression} e */
// |   const sub = (e) => substituteVariable(e, variable, value);
// |   switch (expr.tag) {
// |     case 'Variable':
// |       if (expr._0 === variable) {
// |         return Expression.number(Decimal.fromF64(value));
// |       }
// |       return expr;
// |     case 'Number':
// |     case 'DateTime':
// |     case 'Now':
// |     case 'Today':
// |       return expr;
// |     case 'Until':
// |       return Expression.Until(sub(expr._0));
// |     case 'Binary':
// |       return Expression.binary(sub(expr.left), expr.op, sub(expr.right));
// |     case 'Negate':
// |       return Expression.negate(sub(expr._0));
// |     case 'Group':
// |       return Expression.group(sub(expr._0));
// |     case 'Power':
// |       return Expression.power(sub(expr.base), sub(expr.exponent));
// |     case 'FunctionCall':
// |       return Expression.functionCall(
// |         expr.name,
// |         expr.args.map((/** @type {Expression} */ a) => sub(a)),
// |       );
// |     case 'AtTime':
// |       return Expression.atTime(sub(expr.value), sub(expr.time));
// |     case 'IndefiniteIntegral':
// |       return Expression.indefiniteIntegral(sub(expr.integrand), expr.variable);
// |     case 'UnitConversion':
// |       return Expression.unitConversion(sub(expr.value), expr.targetUnit);
// |     case 'Equality':
// |       return Expression.equality(sub(expr.left), sub(expr.right));
// |     case 'Comparison':
// |       return Expression.comparison(sub(expr.left), expr.op, sub(expr.right));
// |     default:
// |       return expr;
// |   }
// | }
