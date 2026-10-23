// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=05a39a3f13785cb6dd5e6ab19a7c9e4ebc4a02cd5036cd40a6d54c2135b9d1b8 bytes=31176

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Expression types for the calculator grammar (port of
// |  * `src/types/expression.rs`).
// |  *
// |  * `Expression` is not frozen: `applyLocalOffset` mutates the tree in place,
// |  * as `apply_local_offset(&mut self)` does.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=6a58e9adb3b15ffddb375f0e6d3ccd15d27e918ab78e0d0474c5b280ad9f79ac
// | import { eqIgnoreAsciiCase } from '../rust/str.js';
use crate::rust::str_::eqIgnoreAsciiCase;
// meta-language:translated JavaScript import_statement items=1 sha256=72afb9d4279eefbf0545cea7a2d8eac41039782a18efb99de071625aaa658e87
// | import { eq } from '../rust/core.js';
use crate::rust::core_::eq;
// meta-language:translated JavaScript import_statement items=1 sha256=4a34e46992370c42b7a43e2a0a14b26893d1efc9e061cb8ea770b18223976640
// | import { Unit } from './unit.js';
use crate::types::unit::Unit;

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A binary operation. */
// | export class BinaryOp {
// |   /** @param {'Add' | 'Subtract' | 'Multiply' | 'Divide' | 'Modulo'} tag */
// |   constructor(tag) {
// |     this.tag = tag;
// |     Object.freeze(this);
// |   }
// |
// |   static Add = new BinaryOp('Add');
// |   static Subtract = new BinaryOp('Subtract');
// |   static Multiply = new BinaryOp('Multiply');
// |   static Divide = new BinaryOp('Divide');
// |   static Modulo = new BinaryOp('Modulo');
// |
// |   /** Returns the symbol for the operation. @returns {string} */
// |   symbol() {
// |     switch (this.tag) {
// |       case 'Add':
// |         return '+';
// |       case 'Subtract':
// |         return '-';
// |       case 'Multiply':
// |         return '*';
// |       case 'Divide':
// |         return '/';
// |       default:
// |         return '%';
// |     }
// |   }
// |
// |   /** Returns the precedence of the operation (higher = binds tighter). @returns {number} */
// |   precedence() {
// |     return this.tag === 'Add' || this.tag === 'Subtract' ? 1 : 2;
// |   }
// |
// |   /** `impl Display`. */
// |   toString() {
// |     return this.symbol();
// |   }
// |
// |   /** serde: the variant name. */
// |   toJSON() {
// |     return this.tag;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A comparison operation. */
// | export class ComparisonOp {
// |   /** @param {'Equal' | 'Less' | 'LessOrEqual' | 'Greater' | 'GreaterOrEqual' | 'NotEqual' | 'Compare'} tag */
// |   constructor(tag) {
// |     this.tag = tag;
// |     Object.freeze(this);
// |   }
// |
// |   /** Equal (`==`). */
// |   static Equal = new ComparisonOp('Equal');
// |   /** Strictly less than (`<`). */
// |   static Less = new ComparisonOp('Less');
// |   /** Less than or equal (`<=`). */
// |   static LessOrEqual = new ComparisonOp('LessOrEqual');
// |   /** Strictly greater than (`>`). */
// |   static Greater = new ComparisonOp('Greater');
// |   /** Greater than or equal (`>=`). */
// |   static GreaterOrEqual = new ComparisonOp('GreaterOrEqual');
// |   /** Not equal (`!=`). */
// |   static NotEqual = new ComparisonOp('NotEqual');
// |   /** Generic comparison (`compare a and b`, `a vs b`). */
// |   static Compare = new ComparisonOp('Compare');
// |
// |   /** Returns the symbol for the comparison operation. @returns {string} */
// |   symbol() {
// |     switch (this.tag) {
// |       case 'Equal':
// |         return '==';
// |       case 'Less':
// |         return '<';
// |       case 'LessOrEqual':
// |         return '<=';
// |       case 'Greater':
// |         return '>';
// |       case 'GreaterOrEqual':
// |         return '>=';
// |       case 'NotEqual':
// |         return '!=';
// |       default:
// |         return 'compare';
// |     }
// |   }
// |
// |   /** `impl Display`. */
// |   toString() {
// |     return this.symbol();
// |   }
// |
// |   /** serde: the variant name. */
// |   toJSON() {
// |     return this.tag;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** Function names rendered as `\name\left(...\right)` in LaTeX. */
// | const LATEX_NAMED_FUNCTIONS = new Set([
// |   'sin', 'cos', 'tan', 'cot', 'sec', 'csc', 'sinh', 'cosh', 'tanh', 'coth', 'sech', 'csch',
// |   'arcsin', 'arccos', 'arctan', 'ln', 'log', 'exp',
// | ]);

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * An expression in the calculator grammar.
// |  *
// |  * Tuple variants store their payload as `_0`; struct variants keep their
// |  * field names in camelCase (`alternativeUnits`, `targetUnit`).
// |  */
// | export class Expression {
// |   /** @param {string} tag @param {object} [fields] */
// |   constructor(tag, fields) {
// |     /** @type {'Number' | 'DateTime' | 'Now' | 'Today' | 'Until' | 'Binary' | 'Negate' | 'Group' | 'AtTime' | 'FunctionCall' | 'Variable' | 'Power' | 'IndefiniteIntegral' | 'UnitConversion' | 'Equality' | 'Comparison'} */
// |     this.tag = /** @type {any} */ (tag);
// |     const f = /** @type {any} */ (fields ?? {});
// |     // Payload fields (`undefined` where the variant has none).
// |     /** @type {any} */ this._0 = f._0;
// |     /** @type {any} */ this.value = f.value;
// |     /** @type {any} */ this.unit = f.unit;
// |     /** @type {Unit[]} */ this.alternativeUnits = f.alternativeUnits;
// |     /** @type {any} */ this.left = f.left;
// |     /** @type {any} */ this.op = f.op;
// |     /** @type {any} */ this.right = f.right;
// |     /** @type {any} */ this.time = f.time;
// |     /** @type {any} */ this.name = f.name;
// |     /** @type {any} */ this.args = f.args;
// |     /** @type {any} */ this.base = f.base;
// |     /** @type {any} */ this.exponent = f.exponent;
// |     /** @type {any} */ this.integrand = f.integrand;
// |     /** @type {any} */ this.variable = f.variable;
// |     /** @type {any} */ this.targetUnit = f.targetUnit;
// |   }
// |
// |   // --- variants ---
// |
// |   /**
// |    * A literal number.
// |    * @param {{ value: import('./decimal.js').Decimal, unit: Unit, alternativeUnits: Unit[] }} fields
// |    */
// |   static Number({ value, unit, alternativeUnits }) {
// |     return new Expression('Number', { value, unit, alternativeUnits });
// |   }
// |
// |   /** A literal datetime. @param {import('./datetime.js').DateTime} dt */
// |   static DateTime(dt) {
// |     return new Expression('DateTime', { _0: dt });
// |   }
// |
// |   /** The current time ("now"). */
// |   static Now() {
// |     return new Expression('Now');
// |   }
// |
// |   /** The current calendar date ("today"). */
// |   static Today() {
// |     return new Expression('Today');
// |   }
// |
// |   /** "until <datetime>". @param {Expression} inner */
// |   static Until(inner) {
// |     return new Expression('Until', { _0: inner });
// |   }
// |
// |   /** A binary operation. @param {{ left: Expression, op: BinaryOp, right: Expression }} fields */
// |   static Binary({ left, op, right }) {
// |     return new Expression('Binary', { left, op, right });
// |   }
// |
// |   /** A unary negation. @param {Expression} inner */
// |   static Negate(inner) {
// |     return new Expression('Negate', { _0: inner });
// |   }
// |
// |   /** A grouped expression (parenthesized). @param {Expression} inner */
// |   static Group(inner) {
// |     return new Expression('Group', { _0: inner });
// |   }
// |
// |   /** A temporal context for a value. @param {{ value: Expression, time: Expression }} fields */
// |   static AtTime({ value, time }) {
// |     return new Expression('AtTime', { value, time });
// |   }
// |
// |   /** A function call. @param {{ name: string, args: Expression[] }} fields */
// |   static FunctionCall({ name, args }) {
// |     return new Expression('FunctionCall', { name, args });
// |   }
// |
// |   /** A variable reference. @param {string} name */
// |   static Variable(name) {
// |     return new Expression('Variable', { _0: name });
// |   }
// |
// |   /** Power/exponentiation (x^y). @param {{ base: Expression, exponent: Expression }} fields */
// |   static Power({ base, exponent }) {
// |     return new Expression('Power', { base, exponent });
// |   }
// |
// |   /** Indefinite integral. @param {{ integrand: Expression, variable: string }} fields */
// |   static IndefiniteIntegral({ integrand, variable }) {
// |     return new Expression('IndefiniteIntegral', { integrand, variable });
// |   }
// |
// |   /** Unit conversion ("741 KB as MB"). @param {{ value: Expression, targetUnit: Unit }} fields */
// |   static UnitConversion({ value, targetUnit }) {
// |     return new Expression('UnitConversion', { value, targetUnit });
// |   }
// |
// |   /** Equality check. @param {{ left: Expression, right: Expression }} fields */
// |   static Equality({ left, right }) {
// |     return new Expression('Equality', { left, right });
// |   }
// |
// |   /** Value comparison. @param {{ left: Expression, op: ComparisonOp, right: Expression }} fields */
// |   static Comparison({ left, op, right }) {
// |     return new Expression('Comparison', { left, op, right });
// |   }
// |
// |   // --- constructors ---
// |
// |   /** Creates a number expression. @param {import('./decimal.js').Decimal} value */
// |   static number(value) {
// |     return Expression.Number({ value, unit: Unit.None, alternativeUnits: [] });
// |   }
// |
// |   /** Creates a number expression with a unit. @param {import('./decimal.js').Decimal} value @param {Unit} unit */
// |   static numberWithUnit(value, unit) {
// |     return Expression.Number({ value, unit, alternativeUnits: [] });
// |   }
// |
// |   /**
// |    * Creates a number expression with a unit and alternative unit interpretations.
// |    * @param {import('./decimal.js').Decimal} value @param {Unit} unit @param {Unit[]} alternativeUnits
// |    */
// |   static numberWithUnitAlternatives(value, unit, alternativeUnits) {
// |     return Expression.Number({ value, unit, alternativeUnits });
// |   }
// |
// |   /** Creates a currency expression. @param {import('./decimal.js').Decimal} amount @param {string} code */
// |   static currency(amount, code) {
// |     return Expression.Number({ value: amount, unit: Unit.currency(code), alternativeUnits: [] });
// |   }
// |
// |   /** Creates a binary expression. @param {Expression} left @param {BinaryOp} op @param {Expression} right */
// |   static binary(left, op, right) {
// |     return Expression.Binary({ left, op, right });
// |   }
// |
// |   /** Creates a negation expression. @param {Expression} expr */
// |   static negate(expr) {
// |     return Expression.Negate(expr);
// |   }
// |
// |   /** Creates a grouped expression. @param {Expression} expr */
// |   static group(expr) {
// |     return Expression.Group(expr);
// |   }
// |
// |   /** Creates an at-time expression. @param {Expression} value @param {Expression} time */
// |   static atTime(value, time) {
// |     return Expression.AtTime({ value, time });
// |   }
// |
// |   /** Creates a function call expression. @param {string} name @param {Expression[]} args */
// |   static functionCall(name, args) {
// |     return Expression.FunctionCall({ name, args });
// |   }
// |
// |   /** Creates a variable expression. @param {string} name */
// |   static variable(name) {
// |     return Expression.Variable(name);
// |   }
// |
// |   /** Creates a power expression. @param {Expression} base @param {Expression} exponent */
// |   static power(base, exponent) {
// |     return Expression.Power({ base, exponent });
// |   }
// |
// |   /** Creates an indefinite integral expression. @param {Expression} integrand @param {string} variable */
// |   static indefiniteIntegral(integrand, variable) {
// |     return Expression.IndefiniteIntegral({ integrand, variable });
// |   }
// |
// |   /** Creates a unit conversion expression. @param {Expression} value @param {Unit} targetUnit */
// |   static unitConversion(value, targetUnit) {
// |     return Expression.UnitConversion({ value, targetUnit });
// |   }
// |
// |   /** Creates an equality check expression. @param {Expression} left @param {Expression} right */
// |   static equality(left, right) {
// |     return Expression.Equality({ left, right });
// |   }
// |
// |   /** Creates a comparison expression. @param {Expression} left @param {ComparisonOp} op @param {Expression} right */
// |   static comparison(left, op, right) {
// |     return Expression.Comparison({ left, op, right });
// |   }
// |
// |   /**
// |    * Converts the expression to links notation format.
// |    *
// |    * Links notation wraps all compound expressions in parentheses:
// |    * - Atomic values (numbers, variables) don't need parentheses
// |    * - All other expressions are wrapped in outer `()`
// |    * - Function calls use space-separated args: `(func (arg1 arg2 arg3))`
// |    * - Power uses `^` operator with spaces: `(x ^ 2)`
// |    * @returns {string}
// |    */
// |   toLino() {
// |     return this.toLinoInternal(null);
// |   }
// |
// |   /**
// |    * Recursively re-anchors timezone-less datetime literals in this expression
// |    * to the user's local timezone, given by `offsetSeconds` (seconds east of
// |    * UTC). `now` is resolved at evaluation time, so it is left untouched.
// |    * @param {number} offsetSeconds an `i32`
// |    */
// |   applyLocalOffset(offsetSeconds) {
// |     switch (this.tag) {
// |       case 'DateTime':
// |         this._0 = this._0.reinterpretNaiveAsLocal(offsetSeconds);
// |         break;
// |       case 'Until':
// |       case 'Negate':
// |       case 'Group':
// |         this._0.applyLocalOffset(offsetSeconds);
// |         break;
// |       case 'Binary':
// |       case 'Equality':
// |       case 'Comparison':
// |         this.left.applyLocalOffset(offsetSeconds);
// |         this.right.applyLocalOffset(offsetSeconds);
// |         break;
// |       case 'Power':
// |         this.base.applyLocalOffset(offsetSeconds);
// |         this.exponent.applyLocalOffset(offsetSeconds);
// |         break;
// |       case 'AtTime':
// |         this.value.applyLocalOffset(offsetSeconds);
// |         this.time.applyLocalOffset(offsetSeconds);
// |         break;
// |       case 'FunctionCall':
// |         for (const arg of this.args) {
// |           arg.applyLocalOffset(offsetSeconds);
// |         }
// |         break;
// |       case 'IndefiniteIntegral':
// |         this.integrand.applyLocalOffset(offsetSeconds);
// |         break;
// |       case 'UnitConversion':
// |         this.value.applyLocalOffset(offsetSeconds);
// |         break;
// |       default:
// |         break;
// |     }
// |   }
// |
// |   /**
// |    * Internal helper for `toLino`. `parentOp` is the parent operator (if any)
// |    * to determine if this subexpression needs parentheses.
// |    * @param {BinaryOp | null} parentOp
// |    * @returns {string}
// |    */
// |   toLinoInternal(parentOp) {
// |     switch (this.tag) {
// |       case 'Number': {
// |         const numStr = this.value.toString();
// |         return this.unit.tag === 'None' ? numStr : `(${numStr} ${this.unit})`;
// |       }
// |       case 'DateTime':
// |         return `(${this._0})`;
// |       case 'Now':
// |         return '(now)';
// |       case 'Today':
// |         return '(today)';
// |       case 'Until':
// |         return `(until ${this._0.toLinoInternal(null)})`;
// |       case 'Binary': {
// |         const context =
// |           this.op === BinaryOp.Multiply && this.left.isPercentage() && this.right.isPercentage()
// |             ? null
// |             : this.op;
// |         const leftStr = this.left.toLinoInternal(context);
// |         const rightStr = this.right.toLinoInternal(context);
// |         return `(${leftStr} ${this.op} ${rightStr})`;
// |       }
// |       case 'Negate': {
// |         const innerStr = this._0.toLinoInternal(null);
// |         // Only add parens if the inner expression needs them
// |         return this._0.needsParensForUnary() ? `(-(${innerStr}))` : `(-${innerStr})`;
// |       }
// |       case 'Group': {
// |         // Groups are explicitly requested by the user; if the inner expression
// |         // already has its own parentheses, don't double-wrap
// |         const innerStr = this._0.toLinoInternal(null);
// |         return innerStr.startsWith('(') && innerStr.endsWith(')') ? innerStr : `(${innerStr})`;
// |       }
// |       case 'AtTime':
// |         return `(${this.value.toLinoInternal(null)} at ${this.time.toLinoInternal(null)})`;
// |       case 'FunctionCall': {
// |         const { name, args } = this;
// |         if (eqIgnoreAsciiCase(name, 'percent') && args.length === 1) {
// |           if (parentOp === BinaryOp.Multiply || parentOp === BinaryOp.Divide) {
// |             return `(${args[0].toLinoInternal(null)} / 100)`;
// |           }
// |           return `(${args[0].toLinoInternal(null)}%)`;
// |         }
// |         if (eqIgnoreAsciiCase(name, 'percent_of') && args.length === 1) {
// |           return args[0].toLinoInternal(null);
// |         }
// |         if (args.length === 0) {
// |           return `(${name})`;
// |         }
// |         const argsStr = args.map((/** @type {Expression} */ a) => a.toLinoInternal(null)).join(' ');
// |         return `(${name} (${argsStr}))`;
// |       }
// |       case 'Variable':
// |         return this._0;
// |       case 'Power':
// |         return `(${this.base.toLinoInternal(null)} ^ ${this.exponent.toLinoInternal(null)})`;
// |       case 'IndefiniteIntegral':
// |         return `(integrate (${this.integrand.toLinoInternal(null)} * (differential of (${this.variable}))))`;
// |       case 'UnitConversion':
// |         return `(${this.value.toLinoInternal(null)} as ${this.targetUnit.conversionTargetName()})`;
// |       case 'Equality':
// |         return `(${this.left.toLinoInternal(null)} = ${this.right.toLinoInternal(null)})`;
// |       default: {
// |         // Comparison
// |         const leftStr = this.left.toLinoInternal(null);
// |         const rightStr = this.right.toLinoInternal(null);
// |         if (this.op === ComparisonOp.Compare) {
// |           return `(compare ${leftStr} ${rightStr})`;
// |         }
// |         return `(${leftStr} ${this.op} ${rightStr})`;
// |       }
// |     }
// |   }
// |
// |   /**
// |    * Generates alternative links notation interpretations for this expression.
// |    *
// |    * Returns `null` if there is only one natural interpretation, otherwise the
// |    * default interpretation first, followed by alternatives.
// |    * @returns {string[] | null}
// |    */
// |   alternativeLino() {
// |     const defaultLino = this.toLino();
// |     /** @type {string[]} */
// |     let alternatives = [];
// |     this.collectAlternatives(alternatives);
// |     if (alternatives.length === 0) {
// |       return null;
// |     }
// |     // Remove the default interpretation, then consecutive duplicates (`Vec::dedup`)
// |     alternatives = alternatives
// |       .filter((a) => a !== defaultLino)
// |       .filter((a, index, all) => index === 0 || all[index - 1] !== a);
// |     if (alternatives.length === 0) {
// |       return null;
// |     }
// |     return [defaultLino, ...alternatives];
// |   }
// |
// |   /**
// |    * Collects alternative lino representations based on expression structure.
// |    * @param {string[]} alternatives
// |    */
// |   collectAlternatives(alternatives) {
// |     switch (this.tag) {
// |       case 'Number':
// |         // For numbers with ambiguous units, one alternative per interpretation
// |         for (const altUnit of this.alternativeUnits) {
// |           const numStr = this.value.toString();
// |           alternatives.push(altUnit.tag === 'None' ? numStr : `(${numStr} ${altUnit})`);
// |         }
// |         break;
// |       case 'Binary': {
// |         const alt = this.toLinoAlternativeGrouping();
// |         if (alt !== null) {
// |           alternatives.push(alt);
// |         }
// |         // Also collect alternatives from children (e.g., ambiguous units in operands)
// |         this.left.collectAlternatives(alternatives);
// |         this.right.collectAlternatives(alternatives);
// |         break;
// |       }
// |       case 'FunctionCall':
// |         for (const arg of this.args) {
// |           arg.collectAlternatives(alternatives);
// |         }
// |         break;
// |       case 'Equality':
// |       case 'Comparison':
// |         this.left.collectAlternatives(alternatives);
// |         this.right.collectAlternatives(alternatives);
// |         break;
// |       default:
// |         break;
// |     }
// |   }
// |
// |   /** @param {BinaryOp} parentOp @param {BinaryOp} childOp */
// |   static canRegroupRightChild(parentOp, childOp) {
// |     return (
// |       childOp.precedence() !== parentOp.precedence() ||
// |       (parentOp === BinaryOp.Divide && childOp === BinaryOp.Multiply)
// |     );
// |   }
// |
// |   /** @param {BinaryOp} childOp @param {BinaryOp} parentOp */
// |   static canRegroupLeftChild(childOp, parentOp) {
// |     return (
// |       childOp.precedence() !== parentOp.precedence() ||
// |       (childOp === BinaryOp.Divide && parentOp === BinaryOp.Multiply)
// |     );
// |   }
// |
// |   /**
// |    * Generates one alternative grouped lino notation, if this binary node has one.
// |    * @returns {string | null}
// |    */
// |   toLinoAlternativeGrouping() {
// |     if (this.tag !== 'Binary') {
// |       return null;
// |     }
// |     const { left, op, right } = this;
// |     if (right.tag === 'Binary' && Expression.canRegroupRightChild(op, right.op)) {
// |       // Default: `(left op (rl rop rr))`; alternative: `((left op rl) rop rr)`
// |       const newLeft = Expression.binary(left, op, right.left);
// |       return Expression.binary(newLeft, right.op, right.right).toLino();
// |     }
// |     if (left.tag === 'Binary' && Expression.canRegroupLeftChild(left.op, op)) {
// |       // Default: `((ll lop lr) op right)`; alternative: `(ll lop (lr op right))`
// |       const newRight = Expression.binary(left.right, op, right);
// |       return Expression.binary(left.left, left.op, newRight).toLino();
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * Whether this expression retains percentage display during evaluation.
// |    * @returns {boolean}
// |    */
// |   isPercentage() {
// |     switch (this.tag) {
// |       case 'FunctionCall':
// |         if (eqIgnoreAsciiCase(this.name, 'percent_of') && this.args.length === 1) {
// |           return this.args[0].isPercentage();
// |         }
// |         return eqIgnoreAsciiCase(this.name, 'percent') || eqIgnoreAsciiCase(this.name, 'as_percent');
// |       case 'Group':
// |       case 'Negate':
// |         return this._0.isPercentage();
// |       case 'Binary':
// |         if (this.op === BinaryOp.Add || this.op === BinaryOp.Subtract) {
// |           return this.left.isPercentage();
// |         }
// |         if (this.op === BinaryOp.Multiply) {
// |           return this.left.isPercentage() && this.right.isPercentage();
// |         }
// |         return false;
// |       default:
// |         return false;
// |     }
// |   }
// |
// |   /** Whether this expression needs parentheses when used as a unary operand. */
// |   needsParensForUnary() {
// |     return ['Binary', 'AtTime', 'UnitConversion', 'Comparison'].includes(this.tag);
// |   }
// |
// |   /** Whether this expression needs parentheses when used as a power base. */
// |   needsParensForPower() {
// |     return ['Binary', 'Negate', 'AtTime', 'Power', 'UnitConversion'].includes(this.tag);
// |   }
// |
// |   /**
// |    * Returns true if this expression will evaluate to a DateTime value.
// |    * @returns {boolean}
// |    */
// |   evaluatesToDatetime() {
// |     switch (this.tag) {
// |       case 'DateTime':
// |       case 'Now':
// |       case 'Today':
// |         return true;
// |       case 'Group':
// |         return this._0.evaluatesToDatetime();
// |       default:
// |         return false;
// |     }
// |   }
// |
// |   /**
// |    * Returns true if this expression contains a live time reference.
// |    * @returns {boolean}
// |    */
// |   containsLiveTime() {
// |     switch (this.tag) {
// |       case 'DateTime':
// |         return this._0.isLiveTime();
// |       case 'Now':
// |       case 'Today':
// |         return true;
// |       case 'Until':
// |       case 'Negate':
// |       case 'Group':
// |         return this._0.containsLiveTime();
// |       case 'Binary':
// |       case 'Equality':
// |       case 'Comparison':
// |         return this.left.containsLiveTime() || this.right.containsLiveTime();
// |       case 'AtTime':
// |         return this.value.containsLiveTime() || this.time.containsLiveTime();
// |       case 'FunctionCall':
// |         return this.args.some((/** @type {Expression} */ a) => a.containsLiveTime());
// |       case 'Power':
// |         return this.base.containsLiveTime() || this.exponent.containsLiveTime();
// |       case 'UnitConversion':
// |         return this.value.containsLiveTime();
// |       case 'IndefiniteIntegral':
// |         return this.integrand.containsLiveTime();
// |       default:
// |         return false;
// |     }
// |   }
// |
// |   /**
// |    * Collects all (uppercase) currency codes referenced in this expression,
// |    * including target units in `UnitConversion` nodes.
// |    * @returns {Set<string>}
// |    */
// |   collectCurrencies() {
// |     /** @type {Set<string>} */
// |     const currencies = new Set();
// |     this.collectCurrenciesInner(currencies);
// |     return currencies;
// |   }
// |
// |   /** @param {Set<string>} currencies */
// |   collectCurrenciesInner(currencies) {
// |     switch (this.tag) {
// |       case 'Number':
// |         if (this.unit.tag === 'Currency') {
// |           currencies.add(this.unit._0.toUpperCase());
// |         }
// |         break;
// |       case 'Binary':
// |       case 'Equality':
// |       case 'Comparison':
// |         this.left.collectCurrenciesInner(currencies);
// |         this.right.collectCurrenciesInner(currencies);
// |         break;
// |       case 'Power':
// |         this.base.collectCurrenciesInner(currencies);
// |         this.exponent.collectCurrenciesInner(currencies);
// |         break;
// |       case 'Negate':
// |       case 'Group':
// |       case 'Until':
// |         this._0.collectCurrenciesInner(currencies);
// |         break;
// |       case 'AtTime':
// |         this.value.collectCurrenciesInner(currencies);
// |         this.time.collectCurrenciesInner(currencies);
// |         break;
// |       case 'FunctionCall':
// |         for (const arg of this.args) {
// |           arg.collectCurrenciesInner(currencies);
// |         }
// |         break;
// |       case 'IndefiniteIntegral':
// |         this.integrand.collectCurrenciesInner(currencies);
// |         break;
// |       case 'UnitConversion':
// |         this.value.collectCurrenciesInner(currencies);
// |         if (this.targetUnit.tag === 'Currency') {
// |           currencies.add(this.targetUnit._0.toUpperCase());
// |         }
// |         break;
// |       default:
// |         break;
// |     }
// |   }
// |
// |   /**
// |    * Returns the depth of the expression tree.
// |    * @returns {number}
// |    */
// |   depth() {
// |     switch (this.tag) {
// |       case 'Number':
// |       case 'DateTime':
// |       case 'Variable':
// |       case 'Now':
// |       case 'Today':
// |         return 1;
// |       case 'Binary':
// |       case 'Comparison':
// |       case 'Equality':
// |         return 1 + Math.max(this.left.depth(), this.right.depth());
// |       case 'Power':
// |         return 1 + Math.max(this.base.depth(), this.exponent.depth());
// |       case 'Negate':
// |       case 'Group':
// |       case 'Until':
// |         return 1 + this._0.depth();
// |       case 'AtTime':
// |         return 1 + Math.max(this.value.depth(), this.time.depth());
// |       case 'FunctionCall':
// |         return 1 + this.args.reduce((/** @type {number} */ m, /** @type {Expression} */ a) => Math.max(m, a.depth()), 0);
// |       case 'IndefiniteIntegral':
// |         return 1 + this.integrand.depth();
// |       default:
// |         // UnitConversion
// |         return 1 + this.value.depth();
// |     }
// |   }
// |
// |   /**
// |    * Converts the expression to a LaTeX representation.
// |    * @returns {string}
// |    */
// |   toLatex() {
// |     const joinArgs = () => this.args.map((/** @type {Expression} */ a) => a.toLatex()).join(', ');
// |     switch (this.tag) {
// |       case 'Number': {
// |         const numStr = this.value.toString();
// |         return this.unit.tag === 'None' ? numStr : `${numStr} \\text{${this.unit}}`;
// |       }
// |       case 'DateTime':
// |         return `\\text{${this._0}}`;
// |       case 'Now':
// |         return '\\text{now}';
// |       case 'Today':
// |         return '\\text{today}';
// |       case 'Until':
// |         return `\\text{until } ${this._0.toLatex()}`;
// |       case 'Binary': {
// |         const leftStr = this.left.toLatex();
// |         const rightStr = this.right.toLatex();
// |         switch (this.op.tag) {
// |           case 'Add':
// |             return `${leftStr} + ${rightStr}`;
// |           case 'Subtract':
// |             return `${leftStr} - ${rightStr}`;
// |           case 'Multiply':
// |             return `${leftStr} \\cdot ${rightStr}`;
// |           case 'Divide':
// |             return `\\frac{${leftStr}}{${rightStr}}`;
// |           default:
// |             return `${leftStr} \\bmod ${rightStr}`;
// |         }
// |       }
// |       case 'Negate':
// |         return `-${this._0.toLatex()}`;
// |       case 'Group':
// |         return `\\left(${this._0.toLatex()} \\right)`;
// |       case 'AtTime':
// |         return `${this.value.toLatex()} \\text{ at } ${this.time.toLatex()}`;
// |       case 'FunctionCall': {
// |         const { name, args } = this;
// |         const nameLower = name.toLowerCase();
// |         if (nameLower === 'percent' && args.length === 1) {
// |           return `${args[0].toLatex()}\\%`;
// |         }
// |         if (nameLower === 'percent_of' && args.length === 1) {
// |           return args[0].toLatex();
// |         }
// |         if (LATEX_NAMED_FUNCTIONS.has(nameLower)) {
// |           const inner = args.length === 1 ? args[0].toLatex() : joinArgs();
// |           return `\\${nameLower}\\left(${inner} \\right)`;
// |         }
// |         if (nameLower === 'sqrt') {
// |           return `\\sqrt{${args.length === 1 ? args[0].toLatex() : joinArgs()}}`;
// |         }
// |         if (nameLower === 'abs') {
// |           return `\\left| ${args.length === 1 ? args[0].toLatex() : joinArgs()} \\right|`;
// |         }
// |         if (nameLower === 'factorial' && args.length === 1) {
// |           // Render as n! in LaTeX; wrap in parentheses if compound expression
// |           if (args[0].tag === 'Number' || args[0].tag === 'Variable') {
// |             return `${args[0].toLatex()}!`;
// |           }
// |           return `\\left(${args[0].toLatex()}\\right)!`;
// |         }
// |         if (nameLower === 'pi') {
// |           return '\\pi';
// |         }
// |         if (nameLower === 'e') {
// |           return 'e';
// |         }
// |         if (nameLower === 'integrate') {
// |           if (args.length === 4) {
// |             return `\\int_{${args[2].toLatex()}}^{${args[3].toLatex()}} ${args[0].toLatex()} \\, d${args[1].toLatex()}`;
// |           }
// |           return `\\text{integrate}(${joinArgs()})`;
// |         }
// |         return `\\text{${name}}(${joinArgs()})`;
// |       }
// |       case 'Variable':
// |         return this._0;
// |       case 'Power': {
// |         const baseLatex = this.base.toLatex();
// |         const expLatex = this.exponent.toLatex();
// |         // Wrap base in parentheses if it's complex
// |         if (this.base.tag === 'Number' || this.base.tag === 'Variable') {
// |           return `${baseLatex}^{${expLatex}}`;
// |         }
// |         return `\\left(${baseLatex}\\right)^{${expLatex}}`;
// |       }
// |       case 'IndefiniteIntegral':
// |         return `\\int ${this.integrand.toLatex()} \\, d${this.variable}`;
// |       case 'UnitConversion':
// |         return `${this.value.toLatex()} \\to \\text{${this.targetUnit.conversionTargetName()}}`;
// |       case 'Equality':
// |         return `${this.left.toLatex()} = ${this.right.toLatex()}`;
// |       default: {
// |         // Comparison
// |         const latexOp = {
// |           Less: '<',
// |           Equal: '=',
// |           LessOrEqual: '\\le',
// |           Greater: '>',
// |           GreaterOrEqual: '\\ge',
// |           NotEqual: '\\ne',
// |           Compare: '\\operatorname{compare}',
// |         }[/** @type {ComparisonOp} */ (this.op).tag];
// |         if (this.op === ComparisonOp.Compare) {
// |           return `${latexOp}\\left(${this.left.toLatex()}, ${this.right.toLatex()}\\right)`;
// |         }
// |         return `${this.left.toLatex()} ${latexOp} ${this.right.toLatex()}`;
// |       }
// |     }
// |   }
// |
// |   /** `impl Display`. @returns {string} */
// |   toString() {
// |     switch (this.tag) {
// |       case 'Number':
// |         return this.unit.tag === 'None' ? `${this.value}` : `${this.value} ${this.unit}`;
// |       case 'DateTime':
// |         return `${this._0}`;
// |       case 'Now':
// |         return 'now';
// |       case 'Today':
// |         return 'today';
// |       case 'Until':
// |         return `until ${this._0}`;
// |       case 'Binary':
// |         return `${this.left} ${this.op} ${this.right}`;
// |       case 'Negate':
// |         return `-${this._0}`;
// |       case 'Group':
// |         return `(${this._0})`;
// |       case 'AtTime':
// |         return `${this.value} at ${this.time}`;
// |       case 'FunctionCall':
// |         return `${this.name}(${this.args.map(String).join(', ')})`;
// |       case 'Variable':
// |         return this._0;
// |       case 'Power':
// |         return `${this.base}^${this.exponent}`;
// |       case 'IndefiniteIntegral':
// |         return `integrate ${this.integrand} d${this.variable}`;
// |       case 'UnitConversion':
// |         return `${this.value} as ${this.targetUnit.conversionTargetName()}`;
// |       case 'Equality':
// |         return `${this.left} = ${this.right}`;
// |       default:
// |         if (this.op === ComparisonOp.Compare) {
// |           return `compare ${this.left} and ${this.right}`;
// |         }
// |         return `${this.left} ${this.op} ${this.right}`;
// |     }
// |   }
// |
// |   /** Derived `PartialEq`. @param {Expression} other @returns {boolean} */
// |   equals(other) {
// |     if (!(other instanceof Expression) || this.tag !== other.tag) {
// |       return false;
// |     }
// |     const keys = Object.keys(this);
// |     return keys.length === Object.keys(other).length && keys.every((key) => eq(this[key], other[key]));
// |   }
// |
// |   /** serde's externally tagged representation (`alternative_units` skipped when empty). */
// |   toJSON() {
// |     switch (this.tag) {
// |       case 'Now':
// |       case 'Today':
// |         return this.tag;
// |       case 'DateTime':
// |       case 'Until':
// |       case 'Negate':
// |       case 'Group':
// |       case 'Variable':
// |         return { [this.tag]: this._0 };
// |       case 'Number': {
// |         /** @type {Record<string, unknown>} */
// |         const body = { value: this.value, unit: this.unit };
// |         if (this.alternativeUnits.length > 0) {
// |           body.alternative_units = this.alternativeUnits;
// |         }
// |         return { Number: body };
// |       }
// |       case 'UnitConversion':
// |         return { UnitConversion: { value: this.value, target_unit: this.targetUnit } };
// |       default: {
// |         /** @type {Record<string, unknown>} */
// |         const body = {};
// |         for (const [key, value] of Object.entries(this)) {
// |           if (key !== 'tag') {
// |             body[key] = value;
// |           }
// |         }
// |         return { [this.tag]: body };
// |       }
// |     }
// |   }
// | }
