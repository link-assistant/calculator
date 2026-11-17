// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ffc260df15788813350b613598d4fe4be62ac98e20f5e114ddd51f49f27a7962 bytes=15712

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | //! Linear-equation helpers for the expression evaluator.

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=e0430524213e7e0aba35ba83d0c013edef8a31223066a098c64d5c30727092b5
// | import { clone, eq, err } from '../rust/core.js';
use crate::rust::core_::{clone, eq, err};
// meta-language:translated JavaScript import_statement items=1 sha256=6a58e9adb3b15ffddb375f0e6d3ccd15d27e918ab78e0d0474c5b280ad9f79ac
// | import { eqIgnoreAsciiCase } from '../rust/str.js';
use crate::rust::str_::eqIgnoreAsciiCase;
// meta-language:translated JavaScript import_statement items=1 sha256=da5833820f84db3f2f11ceaf4f23afda2567135bae29426b6f9d440067c23195
// | import { Rational, Unit, Value } from '../types/index.js';
use crate::types::index::{Rational, Unit, Value};

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../types/index.js').Expression} Expression */

// meta-language:carried JavaScript class_declaration (unsupported)
// | class LinearTerm {
// |   /** @param {{ variable: string, coefficient: Rational }} fields */
// |   constructor({ variable, coefficient }) {
// |     this.variable = variable;
// |     this.coefficient = coefficient;
// |   }
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | class LinearForm {
// |   /** @param {{ terms: LinearTerm[], constant: Rational }} fields */
// |   constructor({ terms, constant }) {
// |     this.terms = terms;
// |     this.constant = constant;
// |   }
// |
// |   /** @param {Rational} value */
// |   static constant(value) {
// |     return new LinearForm({
// |       terms: [],
// |       constant: value,
// |     });
// |   }
// |
// |   /** @param {string} name */
// |   static variable(name) {
// |     return new LinearForm({
// |       terms: [
// |         new LinearTerm({
// |           variable: name,
// |           coefficient: Rational.one(),
// |         }),
// |       ],
// |       constant: Rational.zero(),
// |     });
// |   }
// |
// |   /**
// |    * @param {Expression} expr
// |    * @returns {LinearForm}
// |    */
// |   static fromExpression(expr) {
// |     switch (expr.tag) {
// |       case 'Number': {
// |         if (!eq(expr.unit, Unit.None)) {
// |           return err(LinearForm.unsupportedEquation());
// |         }
// |         return LinearForm.constant(Rational.fromDecimal(expr.value));
// |       }
// |       case 'Variable':
// |         return LinearForm.variable(expr._0);
// |       case 'Binary': {
// |         const op = expr.op;
// |         const relative = !expr.left.isPercentage() && expr.right.isPercentage();
// |         const left = LinearForm.fromExpression(expr.left);
// |         let right = LinearForm.fromExpression(expr.right);
// |         right =
// |           relative && (op.tag === 'Add' || op.tag === 'Subtract')
// |             ? clone(left).multiply(right)
// |             : right;
// |         switch (op.tag) {
// |           case 'Add':
// |             return left.add(right);
// |           case 'Subtract':
// |             return left.subtract(right);
// |           case 'Multiply':
// |             return left.multiply(right);
// |           case 'Divide':
// |             return left.divide(right);
// |           default:
// |             // BinaryOp::Modulo
// |             return err(LinearForm.unsupportedEquation());
// |         }
// |       }
// |       case 'FunctionCall': {
// |         const { name, args } = expr;
// |         if (eqIgnoreAsciiCase(name, 'percent') && args.length === 1) {
// |           return LinearForm.fromExpression(args[0]).divide(
// |             LinearForm.constant(Rational.fromInteger(100)),
// |           );
// |         }
// |         if (
// |           (eqIgnoreAsciiCase(name, 'as_percent') || eqIgnoreAsciiCase(name, 'percent_of')) &&
// |           args.length === 1
// |         ) {
// |           return LinearForm.fromExpression(args[0]);
// |         }
// |         return err(LinearForm.unsupportedEquation());
// |       }
// |       case 'Negate':
// |         return LinearForm.fromExpression(expr._0).negate();
// |       case 'Group':
// |         return LinearForm.fromExpression(expr._0);
// |       default:
// |         // DateTime, Now, Today, Until, AtTime, Power, IndefiniteIntegral,
// |         // UnitConversion, Equality, Comparison
// |         return err(LinearForm.unsupportedEquation());
// |     }
// |   }
// |
// |   /** @param {LinearForm} other @returns {LinearForm} */
// |   add(other) {
// |     for (const term of other.terms) {
// |       this.addTerm(term.variable, term.coefficient);
// |     }
// |     this.constant = this.constant.add(other.constant);
// |     return this;
// |   }
// |
// |   /** @param {LinearForm} other @returns {LinearForm} */
// |   subtract(other) {
// |     for (const term of other.terms) {
// |       this.addTerm(term.variable, term.coefficient.neg());
// |     }
// |     this.constant = this.constant.sub(other.constant);
// |     return this;
// |   }
// |
// |   /** @param {LinearForm} other @returns {LinearForm} */
// |   multiply(other) {
// |     const selfHas = this.hasVariableTerms();
// |     const otherHas = other.hasVariableTerms();
// |     if (selfHas && otherHas) {
// |       return err(CalculatorError.InvalidOperation('equation is not linear'));
// |     }
// |     if (selfHas) {
// |       return this.scale(other.constant);
// |     }
// |     if (otherHas) {
// |       return other.scale(this.constant);
// |     }
// |     return LinearForm.constant(this.constant.mul(other.constant));
// |   }
// |
// |   /** @param {LinearForm} other @returns {LinearForm} */
// |   divide(other) {
// |     if (other.hasVariableTerms()) {
// |       return err(CalculatorError.InvalidOperation('equation has a variable denominator'));
// |     }
// |     return this.divideByRational(other.constant);
// |   }
// |
// |   /** @returns {LinearForm} */
// |   negate() {
// |     return this.scale(Rational.fromInteger(-1));
// |   }
// |
// |   /** @param {Rational} scale @returns {LinearForm} */
// |   scale(scale) {
// |     this.constant = this.constant.mul(scale);
// |     for (const term of this.terms) {
// |       term.coefficient = term.coefficient.mul(scale);
// |     }
// |     this.removeZeroTerms();
// |     return this;
// |   }
// |
// |   /** @param {Rational} denominator @returns {LinearForm} */
// |   divideByRational(denominator) {
// |     if (denominator.isZero()) {
// |       return err(CalculatorError.DivisionByZero);
// |     }
// |     this.constant = this.constant.div(denominator);
// |     for (const term of this.terms) {
// |       term.coefficient = term.coefficient.div(denominator);
// |     }
// |     this.removeZeroTerms();
// |     return this;
// |   }
// |
// |   /** @param {string} variable @param {Rational} coefficient */
// |   addTerm(variable, coefficient) {
// |     if (coefficient.isZero()) {
// |       return;
// |     }
// |
// |     const index = this.terms.findIndex((term) => term.variable === variable);
// |     if (index !== -1) {
// |       const newCoefficient = this.terms[index].coefficient.add(coefficient);
// |       if (newCoefficient.isZero()) {
// |         this.terms.splice(index, 1);
// |       } else {
// |         this.terms[index].coefficient = newCoefficient;
// |       }
// |       return;
// |     }
// |
// |     this.terms.push(
// |       new LinearTerm({
// |         variable,
// |         coefficient,
// |       }),
// |     );
// |   }
// |
// |   removeZeroTerms() {
// |     this.terms = this.terms.filter((term) => !term.coefficient.isZero());
// |   }
// |
// |   hasVariableTerms() {
// |     return this.terms.length !== 0;
// |   }
// |
// |   /** @param {string} variable @returns {Rational} */
// |   coefficientOf(variable) {
// |     const term = this.terms.find((t) => t.variable === variable);
// |     return term === undefined ? Rational.zero() : term.coefficient;
// |   }
// |
// |   /** @param {string[]} variables */
// |   pushVariableOrder(variables) {
// |     for (const term of this.terms) {
// |       if (!variables.some((variable) => variable === term.variable)) {
// |         variables.push(term.variable);
// |       }
// |     }
// |   }
// |
// |   static unsupportedEquation() {
// |     return CalculatorError.InvalidOperation(
// |       'only linear equations with unitless numbers are supported',
// |     );
// |   }
// |
// |   formatTermsFirst() {
// |     return this.format(false);
// |   }
// |
// |   formatSolutionExpression() {
// |     return this.format(true);
// |   }
// |
// |   /** @param {boolean} constantFirst @returns {string} */
// |   format(constantFirst) {
// |     /** @type {[boolean, string][]} */
// |     const parts = [];
// |
// |     if (constantFirst) {
// |       LinearForm.pushConstantPart(parts, this.constant);
// |     }
// |
// |     for (const term of this.terms) {
// |       LinearForm.pushVariablePart(parts, term.coefficient, term.variable);
// |     }
// |
// |     if (!constantFirst) {
// |       LinearForm.pushConstantPart(parts, this.constant);
// |     }
// |
// |     return LinearForm.joinSignedParts(parts);
// |   }
// |
// |   /** @param {[boolean, string][]} parts @param {Rational} value */
// |   static pushConstantPart(parts, value) {
// |     if (value.isZero()) {
// |       return;
// |     }
// |     parts.push([value.isNegative(), value.abs().toDisplayString()]);
// |   }
// |
// |   /** @param {[boolean, string][]} parts @param {Rational} coefficient @param {string} variable */
// |   static pushVariablePart(parts, coefficient, variable) {
// |     if (coefficient.isZero()) {
// |       return;
// |     }
// |     parts.push([
// |       coefficient.isNegative(),
// |       LinearForm.formatVariableBody(coefficient.abs(), variable),
// |     ]);
// |   }
// |
// |   /** @param {[boolean, string][]} parts @returns {string} */
// |   static joinSignedParts(parts) {
// |     let result = '';
// |
// |     parts.forEach(([isNegative, body], index) => {
// |       if (index === 0) {
// |         if (isNegative) {
// |           result += '-';
// |         }
// |       } else if (isNegative) {
// |         result += ' - ';
// |       } else {
// |         result += ' + ';
// |       }
// |       result += body;
// |     });
// |
// |     return result === '' ? '0' : result;
// |   }
// |
// |   /** @param {Rational} coefficient @param {string} variable @returns {string} */
// |   static formatVariableTerm(coefficient, variable) {
// |     if (coefficient.isZero()) {
// |       return '0';
// |     }
// |     return LinearForm.joinSignedParts([
// |       [coefficient.isNegative(), LinearForm.formatVariableBody(coefficient.abs(), variable)],
// |     ]);
// |   }
// |
// |   /** @param {Rational} coefficientAbs @param {string} variable @returns {string} */
// |   static formatVariableBody(coefficientAbs, variable) {
// |     if (coefficientAbs.equals(Rational.one())) {
// |       return variable;
// |     } else if (variable === '*') {
// |       return `${coefficientAbs.toDisplayString()}*(*)`;
// |     } else {
// |       return `${coefficientAbs.toDisplayString()}*${variable}`;
// |     }
// |   }
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | class VariableMoveDetail {
// |   /**
// |    * @param {{ variable: string, leftCoefficient: Rational, rightCoefficient: Rational,
// |    *   movedCoefficient: Rational }} fields
// |    */
// |   constructor({ variable, leftCoefficient, rightCoefficient, movedCoefficient }) {
// |     this.variable = variable;
// |     this.leftCoefficient = leftCoefficient;
// |     this.rightCoefficient = rightCoefficient;
// |     this.movedCoefficient = movedCoefficient;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | export class LinearEquationSolution {
// |   /**
// |    * @param {{ variable: string, coefficient: Rational, isolatedExpression: LinearForm,
// |    *   valueExpression: LinearForm, leftForm: LinearForm, rightForm: LinearForm,
// |    *   variableMoveDetails: VariableMoveDetail[] }} fields
// |    */
// |   constructor({
// |     variable,
// |     coefficient,
// |     isolatedExpression,
// |     valueExpression,
// |     leftForm,
// |     rightForm,
// |     variableMoveDetails,
// |   }) {
// |     this.variable = variable;
// |     this.coefficient = coefficient;
// |     this.isolatedExpression = isolatedExpression;
// |     this.valueExpression = valueExpression;
// |     this.leftForm = leftForm;
// |     this.rightForm = rightForm;
// |     this.variableMoveDetails = variableMoveDetails;
// |   }
// |
// |   /** @returns {Value} */
// |   toValue() {
// |     if (this.valueExpression.terms.length === 0) {
// |       return Value.equationSolution(this.variable, this.valueExpression.constant);
// |     } else {
// |       return Value.symbolicEquationSolution(
// |         this.variable,
// |         this.valueExpression.formatSolutionExpression(),
// |       );
// |     }
// |   }
// |
// |   /** @returns {string[]} */
// |   derivationSteps() {
// |     const steps = [
// |       `Original equation: ${this.leftForm.formatTermsFirst()} = ${this.rightForm.formatTermsFirst()}`,
// |       `Linear form: ${this.leftForm.formatTermsFirst()} = ${this.rightForm.formatTermsFirst()}`,
// |       `Choose target variable: ${this.variable}`,
// |       `Combine target coefficients: ${this.leftForm
// |         .coefficientOf(this.variable)
// |         .toDisplayString()} - ${this.rightForm
// |         .coefficientOf(this.variable)
// |         .toDisplayString()} = ${this.coefficient.toDisplayString()}`,
// |       `Move non-target variable terms to the right: ${this.formatVariableMoveDetails()}`,
// |       `Move constants to the right: ${this.rightForm.constant.toDisplayString()} - ${this.leftForm.constant.toDisplayString()} = ${this.isolatedExpression.constant.toDisplayString()}`,
// |       `Right side after moving terms: ${this.isolatedExpression.formatSolutionExpression()}`,
// |       `Isolate variable term: ${LinearForm.formatVariableTerm(this.coefficient, this.variable)} = ${this.isolatedExpression.formatSolutionExpression()}`,
// |       `Divide both sides by ${this.coefficient.toDisplayString()}: ${this.variable} = ${this.valueExpression.formatSolutionExpression()}`,
// |     ];
// |
// |     steps.push(
// |       `Solve for ${this.variable}: ${this.variable} = ${this.valueExpression.formatSolutionExpression()}`,
// |     );
// |
// |     return steps;
// |   }
// |
// |   /** @returns {string} */
// |   formatVariableMoveDetails() {
// |     if (this.variableMoveDetails.length === 0) {
// |       return 'no other variables to move';
// |     }
// |
// |     return this.variableMoveDetails
// |       .map(
// |         (detail) =>
// |           `${detail.variable}: ${detail.rightCoefficient.toDisplayString()} - ${detail.leftCoefficient.toDisplayString()} = ${detail.movedCoefficient.toDisplayString()}`,
// |       )
// |       .join('; ');
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @param {Expression} left
// |  * @param {Expression} right
// |  * @returns {LinearEquationSolution}
// |  */
// | export function solve(left, right) {
// |   const leftForm = LinearForm.fromExpression(left);
// |   const rightForm = LinearForm.fromExpression(right);
// |   const variables = collectVariableOrder(leftForm, rightForm);
// |   const variable = selectTargetVariable(leftForm, rightForm, variables);
// |   if (variable === null) {
// |     return err(CalculatorError.InvalidOperation('linear equation has no unique solution'));
// |   }
// |
// |   const coefficient = leftForm.coefficientOf(variable).sub(rightForm.coefficientOf(variable));
// |   if (coefficient.isZero()) {
// |     return err(CalculatorError.InvalidOperation('linear equation has no unique solution'));
// |   }
// |
// |   const isolatedExpression = isolateRightSide(leftForm, rightForm, variables, variable);
// |   const valueExpression = clone(isolatedExpression).divideByRational(coefficient);
// |   const variableMoveDetails = buildVariableMoveDetails(leftForm, rightForm, variables, variable);
// |
// |   return new LinearEquationSolution({
// |     variable,
// |     coefficient,
// |     isolatedExpression,
// |     valueExpression,
// |     leftForm,
// |     rightForm,
// |     variableMoveDetails,
// |   });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {LinearForm} left @param {LinearForm} right @returns {string[]} */
// | function collectVariableOrder(left, right) {
// |   /** @type {string[]} */
// |   const variables = [];
// |   left.pushVariableOrder(variables);
// |   right.pushVariableOrder(variables);
// |   return variables;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {LinearForm} left
// |  * @param {LinearForm} right
// |  * @param {string[]} variables
// |  * @returns {string | null}
// |  */
// | function selectTargetVariable(left, right, variables) {
// |   for (const preferred of ['?', '*']) {
// |     if (
// |       variables.some((variable) => variable === preferred) &&
// |       !combinedTargetCoefficient(left, right, preferred).isZero()
// |     ) {
// |       return preferred;
// |     }
// |   }
// |
// |   return (
// |     variables.find((variable) => !combinedTargetCoefficient(left, right, variable).isZero()) ??
// |     null
// |   );
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {LinearForm} left @param {LinearForm} right @param {string} variable */
// | function combinedTargetCoefficient(left, right, variable) {
// |   return left.coefficientOf(variable).sub(right.coefficientOf(variable));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {LinearForm} left
// |  * @param {LinearForm} right
// |  * @param {string[]} variables
// |  * @param {string} targetVariable
// |  * @returns {LinearForm}
// |  */
// | function isolateRightSide(left, right, variables, targetVariable) {
// |   const isolated = LinearForm.constant(right.constant.sub(left.constant));
// |
// |   for (const variable of variables) {
// |     if (variable === targetVariable) {
// |       continue;
// |     }
// |
// |     const coefficient = right.coefficientOf(variable).sub(left.coefficientOf(variable));
// |     isolated.addTerm(variable, coefficient);
// |   }
// |
// |   return isolated;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {LinearForm} left
// |  * @param {LinearForm} right
// |  * @param {string[]} variables
// |  * @param {string} targetVariable
// |  * @returns {VariableMoveDetail[]}
// |  */
// | function buildVariableMoveDetails(left, right, variables, targetVariable) {
// |   return variables
// |     .filter((variable) => variable !== targetVariable)
// |     .map((variable) => {
// |       const leftCoefficient = left.coefficientOf(variable);
// |       const rightCoefficient = right.coefficientOf(variable);
// |       const movedCoefficient = rightCoefficient.sub(leftCoefficient);
// |
// |       return new VariableMoveDetail({
// |         variable,
// |         leftCoefficient,
// |         rightCoefficient,
// |         movedCoefficient,
// |       });
// |     });
// | }
