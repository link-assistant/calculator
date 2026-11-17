//! Polynomial-equation helpers for the expression evaluator.

import { CalculatorError } from '../error.js';
import { clone, eq, err } from '../rust/core.js';
import { I128, U128, U32 } from '../rust/num.js';
import { eqIgnoreAsciiCase } from '../rust/str.js';
import { Rational, Unit, Value } from '../types/index.js';

/** @typedef {import('../types/index.js').Expression} Expression */

const MAX_POLYNOMIAL_DEGREE = 32;
const ROOT_SEARCH_BOUND = 100n;
const MAX_FACTORABLE_COEFFICIENT_ABS = 1_000_000_000n;

/**
 * `BTreeMap<u32, Rational>` iteration: the entries sorted by degree.
 * @param {Map<number, Rational>} terms
 * @returns {[number, Rational][]}
 */
function sortedTerms(terms) {
  return [...terms].sort(([a], [b]) => a - b);
}

/**
 * `BTreeSet<Rational>`: a `Map` keyed by the reduced fraction.
 * @param {Map<string, Rational>} set
 * @param {Rational} value
 */
function insertRational(set, value) {
  set.set(`${value.numerBigint()}/${value.denomBigint()}`, value);
}

/**
 * `BTreeSet<Rational>` iteration: the values in ascending order.
 * @param {Map<string, Rational>} set
 * @returns {Rational[]}
 */
function sortedRationals(set) {
  return [...set.values()].sort((a, b) => a.cmp(b));
}

class PolynomialForm {
  /** @param {{ variable: string | null, terms: Map<number, Rational> }} fields */
  constructor({ variable, terms }) {
    this.variable = variable;
    this.terms = terms;
  }

  /** @param {Rational} value */
  static constant(value) {
    /** @type {Map<number, Rational>} */
    const terms = new Map();
    if (!value.isZero()) {
      terms.set(0, value);
    }
    return new PolynomialForm({
      variable: null,
      terms,
    });
  }

  /** @param {string} name */
  static variable(name) {
    /** @type {Map<number, Rational>} */
    const terms = new Map();
    terms.set(1, Rational.one());
    return new PolynomialForm({
      variable: name,
      terms,
    });
  }

  /**
   * @param {Expression} expr
   * @returns {PolynomialForm}
   */
  static fromExpression(expr) {
    switch (expr.tag) {
      case 'Number': {
        if (!eq(expr.unit, Unit.None)) {
          return err(PolynomialForm.unsupportedEquation());
        }
        return PolynomialForm.constant(Rational.fromDecimal(expr.value));
      }
      case 'Variable':
        return PolynomialForm.variable(expr._0);
      case 'Binary': {
        const op = expr.op;
        const relative = !expr.left.isPercentage() && expr.right.isPercentage();
        const left = PolynomialForm.fromExpression(expr.left);
        let right = PolynomialForm.fromExpression(expr.right);
        right =
          relative && (op.tag === 'Add' || op.tag === 'Subtract')
            ? clone(left).multiply(right)
            : right;
        switch (op.tag) {
          case 'Add':
            return left.add(right);
          case 'Subtract':
            return left.subtract(right);
          case 'Multiply':
            return left.multiply(right);
          case 'Divide':
            return left.divide(right);
          default:
            // BinaryOp::Modulo
            return err(PolynomialForm.unsupportedEquation());
        }
      }
      case 'FunctionCall': {
        const { name, args } = expr;
        if (eqIgnoreAsciiCase(name, 'percent') && args.length === 1) {
          return PolynomialForm.fromExpression(args[0]).divide(
            PolynomialForm.constant(Rational.fromInteger(100)),
          );
        }
        if (
          (eqIgnoreAsciiCase(name, 'as_percent') || eqIgnoreAsciiCase(name, 'percent_of')) &&
          args.length === 1
        ) {
          return PolynomialForm.fromExpression(args[0]);
        }
        return err(PolynomialForm.unsupportedEquation());
      }
      case 'Negate':
        return PolynomialForm.fromExpression(expr._0).negate();
      case 'Group':
        return PolynomialForm.fromExpression(expr._0);
      case 'Power': {
        const base = PolynomialForm.fromExpression(expr.base);
        const exponent = PolynomialForm.fromExpression(
          expr.exponent,
        ).asNonNegativeIntegerExponent();
        return base.pow(exponent);
      }
      default:
        // DateTime, Now, Today, Until, AtTime, IndefiniteIntegral,
        // UnitConversion, Equality, Comparison
        return err(PolynomialForm.unsupportedEquation());
    }
  }

  /** @param {PolynomialForm} other @returns {PolynomialForm} */
  add(other) {
    const variable = mergeVariableNames(this.variable, other.variable);
    const result = new PolynomialForm({
      variable,
      terms: this.terms,
    });

    for (const [degree, coefficient] of sortedTerms(other.terms)) {
      result.addTerm(degree, coefficient);
    }

    return result;
  }

  /** @param {PolynomialForm} other @returns {PolynomialForm} */
  subtract(other) {
    return this.add(other.negate());
  }

  /** @param {PolynomialForm} other @returns {PolynomialForm} */
  multiply(other) {
    const variable = mergeVariableNames(this.variable, other.variable);
    const result = new PolynomialForm({
      variable,
      terms: new Map(),
    });

    for (const [leftDegree, leftCoefficient] of sortedTerms(this.terms)) {
      for (const [rightDegree, rightCoefficient] of sortedTerms(other.terms)) {
        const degree = U32.checkedAdd(BigInt(leftDegree), BigInt(rightDegree));
        if (degree === null || degree > BigInt(MAX_POLYNOMIAL_DEGREE)) {
          return err(PolynomialForm.unsupportedEquation());
        }
        result.addTerm(Number(degree), leftCoefficient.mul(rightCoefficient));
      }
    }

    return result;
  }

  /** @param {PolynomialForm} other @returns {PolynomialForm} */
  divide(other) {
    if (other.hasVariableTerms()) {
      return err(
        CalculatorError.InvalidOperation('polynomial equation has a variable denominator'),
      );
    }

    const denominator = other.coefficient(0);
    if (denominator.isZero()) {
      return err(CalculatorError.DivisionByZero);
    }

    return this.divideByRational(denominator);
  }

  /** @param {number} exponent @returns {PolynomialForm} */
  pow(exponent) {
    if (exponent === 0) {
      return PolynomialForm.constant(Rational.one());
    }

    let result = PolynomialForm.constant(Rational.one());
    for (let i = 0; i < exponent; i += 1) {
      result = result.multiply(clone(this));
    }

    return result;
  }

  /** @returns {PolynomialForm} */
  negate() {
    return this.scale(Rational.fromInteger(-1));
  }

  /** @param {Rational} scale @returns {PolynomialForm} */
  scale(scale) {
    for (const [degree, coefficient] of this.terms) {
      this.terms.set(degree, coefficient.mul(scale));
    }
    this.removeZeroTerms();
    return this;
  }

  /** @param {Rational} denominator @returns {PolynomialForm} */
  divideByRational(denominator) {
    if (denominator.isZero()) {
      return err(CalculatorError.DivisionByZero);
    }

    for (const [degree, coefficient] of this.terms) {
      this.terms.set(degree, coefficient.div(denominator));
    }
    this.removeZeroTerms();
    return this;
  }

  /** @param {number} degree @param {Rational} coefficient */
  addTerm(degree, coefficient) {
    if (coefficient.isZero()) {
      return;
    }

    const newCoefficient = this.coefficient(degree).add(coefficient);
    if (newCoefficient.isZero()) {
      this.terms.delete(degree);
    } else {
      this.terms.set(degree, newCoefficient);
    }
  }

  removeZeroTerms() {
    for (const [degree, coefficient] of [...this.terms]) {
      if (coefficient.isZero()) {
        this.terms.delete(degree);
      }
    }
    if (!this.hasVariableTerms()) {
      this.variable = null;
    }
  }

  hasVariableTerms() {
    return [...this.terms.keys()].some((degree) => degree > 0);
  }

  /** @param {number} degree @returns {Rational} */
  coefficient(degree) {
    return this.terms.get(degree) ?? Rational.zero();
  }

  /** @returns {number} */
  degree() {
    return this.terms.size === 0 ? 0 : Math.max(...this.terms.keys());
  }

  /** @returns {number} */
  asNonNegativeIntegerExponent() {
    if (this.hasVariableTerms()) {
      return err(PolynomialForm.unsupportedEquation());
    }

    const exponent = this.coefficient(0);
    if (!exponent.isInteger()) {
      return err(PolynomialForm.unsupportedEquation());
    }

    const numer = exponent.numer();
    if (numer < 0n || numer > BigInt(MAX_POLYNOMIAL_DEGREE)) {
      return err(PolynomialForm.unsupportedEquation());
    }

    const result = U32.tryFrom(numer);
    if (result === null) {
      return err(PolynomialForm.unsupportedEquation());
    }
    return Number(result);
  }

  /** @param {Rational} value @returns {Rational} */
  evaluateAt(value) {
    let result = Rational.zero();

    for (let degree = this.degree(); degree >= 0; degree -= 1) {
      result = result.mul(value).add(this.coefficient(degree));
    }

    return result;
  }

  /** @returns {string} */
  format() {
    const variable = this.variable;
    if (variable === null) {
      return this.coefficient(0).toDisplayString();
    }

    /** @type {[boolean, string][]} */
    const parts = sortedTerms(this.terms)
      .reverse()
      .map(([degree, coefficient]) => {
        const coefficientAbs = coefficient.abs();
        return [
          coefficient.isNegative(),
          PolynomialForm.formatTermBody(coefficientAbs, degree, variable),
        ];
      });

    return joinSignedParts(parts);
  }

  /** @param {Rational} coefficientAbs @param {number} degree @param {string} variable */
  static formatTermBody(coefficientAbs, degree, variable) {
    if (degree === 0) {
      return coefficientAbs.toDisplayString();
    }

    const variablePower = formatVariablePower(variable, degree);
    if (coefficientAbs.equals(Rational.one())) {
      return variablePower;
    } else {
      return `${coefficientAbs.toDisplayString()}*${variablePower}`;
    }
  }

  static unsupportedEquation() {
    return CalculatorError.InvalidOperation(
      'only polynomial equations with one variable, unitless numbers, nonnegative integer powers, and real rational roots are supported',
    );
  }
}

export class PolynomialEquationSolution {
  /**
   * @param {{ variable: string, polynomial: PolynomialForm, degree: number, roots: Rational[],
   *   originalLeft: string, originalRight: string }} fields
   */
  constructor({ variable, polynomial, degree, roots, originalLeft, originalRight }) {
    this.variable = variable;
    this.polynomial = polynomial;
    this.degree = degree;
    this.roots = roots;
    this.originalLeft = originalLeft;
    this.originalRight = originalRight;
  }

  /** @returns {Value} */
  toValue() {
    if (this.roots.length === 1) {
      return Value.equationSolution(this.variable, this.roots[0]);
    } else {
      return Value.equationSolutions(this.variable, [...this.roots]);
    }
  }

  /** @returns {string[]} */
  derivationSteps() {
    const steps = [
      `Original equation: ${this.originalLeft} = ${this.originalRight}`,
      `Move all terms to the left: ${this.polynomial.format()} = 0`,
      `Polynomial form: ${this.polynomial.format()} = 0`,
      `Polynomial degree: ${this.degree}`,
      `Choose target variable: ${this.variable}`,
      'Find real rational roots: test exact rational candidates and keep the values that make the polynomial equal 0',
    ];

    for (const root of this.roots) {
      steps.push(
        `Verify root ${root.toDisplayString()}: substituting ${this.variable} = ${root.toDisplayString()} gives 0`,
      );
    }

    steps.push(`Factor roots: ${formatRootFactors(this.variable, this.roots)}`);
    steps.push(`Solutions: ${formatSolutions(this.variable, this.roots)}`);

    return steps;
  }
}

/**
 * @param {Expression} left
 * @param {Expression} right
 * @returns {PolynomialEquationSolution}
 */
export function solve(left, right) {
  const leftForm = PolynomialForm.fromExpression(left);
  const rightForm = PolynomialForm.fromExpression(right);
  const polynomial = leftForm.subtract(rightForm);
  const variable = polynomial.variable;
  if (variable === null) {
    return err(CalculatorError.InvalidOperation('polynomial equation has no variable'));
  }
  const degree = polynomial.degree();

  if (degree <= 1) {
    return err(CalculatorError.InvalidOperation('polynomial equation is linear'));
  }

  const roots = findRealRationalRoots(polynomial);
  if (roots.length === 0) {
    return err(
      CalculatorError.InvalidOperation(
        'polynomial equation has no supported real rational solutions',
      ),
    );
  }

  return new PolynomialEquationSolution({
    variable,
    polynomial,
    degree,
    roots,
    originalLeft: left.toString(),
    originalRight: right.toString(),
  });
}

/**
 * @param {string | null} left
 * @param {string | null} right
 * @returns {string | null}
 */
function mergeVariableNames(left, right) {
  if (left !== null && right !== null && left !== right) {
    return err(
      CalculatorError.InvalidOperation(
        'polynomial equations with multiple variables are not supported',
      ),
    );
  }
  return left ?? right;
}

/** @param {PolynomialForm} polynomial @returns {Rational[]} */
function findRealRationalRoots(polynomial) {
  const binomial = binomialPowerRoots(polynomial);
  if (binomial !== null) {
    return binomial;
  }

  /** @type {Map<string, Rational>} */
  const roots = new Map();

  if (polynomial.degree() === 2) {
    for (const root of quadraticRoots(polynomial)) {
      insertRational(roots, root);
    }
  }

  for (const candidate of rationalRootCandidates(polynomial)) {
    if (polynomial.evaluateAt(candidate).isZero()) {
      insertRational(roots, candidate);
    }
  }

  return sortedRationals(roots);
}

/** @param {PolynomialForm} polynomial @returns {Rational[] | null} */
function binomialPowerRoots(polynomial) {
  const degree = polynomial.degree();
  if (degree < 2) {
    return null;
  }

  const hasOnlyHighAndConstantTerms = [...polynomial.terms.keys()].every(
    (termDegree) => termDegree === 0 || termDegree === degree,
  );
  if (!hasOnlyHighAndConstantTerms) {
    return null;
  }

  const leading = polynomial.coefficient(degree);
  if (leading.isZero()) {
    return null;
  }

  const target = polynomial.coefficient(0).neg().div(leading);
  if (target.isZero()) {
    return [Rational.zero()];
  }

  if (target.isNegative() && degree % 2 === 0) {
    return [];
  }

  const root = rationalNthRoot(target, degree);
  if (root === null) {
    return null;
  }
  if (degree % 2 === 0) {
    const negativeRoot = root.neg();
    return [negativeRoot, root];
  } else {
    return [root];
  }
}

/** @param {PolynomialForm} polynomial @returns {Rational[]} */
function quadraticRoots(polynomial) {
  const a = polynomial.coefficient(2);
  const b = polynomial.coefficient(1);
  const c = polynomial.coefficient(0);
  if (a.isZero()) {
    return [];
  }

  const four = Rational.fromInteger(4);
  const two = Rational.fromInteger(2);
  const discriminant = b.mul(b).sub(four.mul(a).mul(c));
  const squareRoot = rationalSquareRoot(discriminant);
  if (squareRoot === null) {
    return [];
  }

  const denominator = two.mul(a);
  const first = b.neg().sub(squareRoot).div(denominator);
  const second = b.neg().add(squareRoot).div(denominator);

  switch (first.cmp(second)) {
    case 0:
      return [first];
    case -1:
      return [first, second];
    default:
      return [second, first];
  }
}

/** @param {Rational} value @returns {Rational | null} */
function rationalSquareRoot(value) {
  return rationalNthRoot(value, 2);
}

/** @param {Rational} value @param {number} degree @returns {Rational | null} */
function rationalNthRoot(value, degree) {
  if (value.isNegative()) {
    if (degree % 2 === 0) {
      return null;
    }

    const root = rationalNthRoot(value.abs(), degree);
    return root === null ? null : root.neg();
  }

  const numerator = U128.tryFrom(value.numer());
  if (numerator === null) {
    return null;
  }
  const denominator = U128.tryFrom(value.denom());
  if (denominator === null) {
    return null;
  }
  const numeratorRoot = integerNthRoot(numerator, degree);
  if (numeratorRoot === null) {
    return null;
  }
  const denominatorRoot = integerNthRoot(denominator, degree);
  if (denominatorRoot === null) {
    return null;
  }

  const numer = I128.tryFrom(numeratorRoot);
  const denom = I128.tryFrom(denominatorRoot);
  if (numer === null || denom === null) {
    return null;
  }
  return Rational.new(numer, denom);
}

/** @param {bigint} value @param {number} degree @returns {bigint | null} */
function integerNthRoot(value, degree) {
  let low = 0n;
  let high = value;

  while (low <= high) {
    const midpoint = low + (high - low) / 2n;
    const power = checkedPowU128(midpoint, degree);
    if (power !== null && power === value) {
      return midpoint;
    } else if (power !== null && power < value) {
      low = midpoint + 1n;
    } else {
      if (midpoint === 0n) {
        break;
      }
      high = midpoint - 1n;
    }
  }

  return null;
}

/** @param {bigint} base @param {number} exponent @returns {bigint | null} */
function checkedPowU128(base, exponent) {
  /** @type {bigint | null} */
  let result = 1n;

  for (let i = 0; i < exponent; i += 1) {
    result = U128.checkedMul(result, base);
    if (result === null) {
      return null;
    }
  }

  return result;
}

/** @param {PolynomialForm} polynomial @returns {Rational[]} */
function rationalRootCandidates(polynomial) {
  /** @type {Map<string, Rational>} */
  const candidates = new Map();
  for (let n = -ROOT_SEARCH_BOUND; n <= ROOT_SEARCH_BOUND; n += 1n) {
    insertRational(candidates, Rational.fromBigint(n));
  }

  const exactCandidates = rationalRootTheoremCandidates(polynomial);
  if (exactCandidates !== null) {
    for (const candidate of exactCandidates) {
      insertRational(candidates, candidate);
    }
  }

  return sortedRationals(candidates);
}

/** @param {PolynomialForm} polynomial @returns {Rational[] | null} */
function rationalRootTheoremCandidates(polynomial) {
  if (polynomial.terms.size === 0) {
    return null;
  }
  const lowestDegree = Math.min(...polynomial.terms.keys());
  const highestDegree = polynomial.degree();
  /** @type {bigint | null} */
  let denominatorLcm = 1n;

  for (const [, coefficient] of sortedTerms(polynomial.terms)) {
    denominatorLcm = checkedLcm(denominatorLcm, coefficient.denom());
    if (denominatorLcm === null) {
      return null;
    }
  }

  const constant = scaledIntegerCoefficient(polynomial, lowestDegree, denominatorLcm);
  if (constant === null) {
    return null;
  }
  const leading = scaledIntegerCoefficient(polynomial, highestDegree, denominatorLcm);
  if (leading === null) {
    return null;
  }
  const numeratorFactors = positiveDivisors(constant);
  if (numeratorFactors === null) {
    return null;
  }
  const denominatorFactors = positiveDivisors(leading);
  if (denominatorFactors === null) {
    return null;
  }
  /** @type {Map<string, Rational>} */
  const candidates = new Map();

  for (const numerator of numeratorFactors) {
    for (const denominator of denominatorFactors) {
      insertRational(candidates, Rational.new(numerator, denominator));
      insertRational(candidates, Rational.new(-numerator, denominator));
    }
  }

  if (lowestDegree > 0) {
    insertRational(candidates, Rational.zero());
  }

  return sortedRationals(candidates);
}

/**
 * @param {PolynomialForm} polynomial
 * @param {number} degree
 * @param {bigint} denominatorLcm
 * @returns {bigint | null}
 */
function scaledIntegerCoefficient(polynomial, degree, denominatorLcm) {
  const coefficient = polynomial.coefficient(degree);
  const factor = I128.checkedDiv(denominatorLcm, coefficient.denom());
  if (factor === null) {
    return null;
  }
  return I128.checkedMul(coefficient.numer(), factor);
}

/** @param {bigint} left @param {bigint} right @returns {bigint | null} */
function checkedLcm(left, right) {
  const gcd = gcdI128(left, right);
  const quotient = I128.checkedDiv(left, gcd);
  if (quotient === null) {
    return null;
  }
  const product = I128.checkedMul(quotient, right);
  if (product === null) {
    return null;
  }
  return I128.checkedAbs(product);
}

/** @param {bigint} left @param {bigint} right @returns {bigint} */
function gcdI128(left, right) {
  left = left < 0n ? -left : left;
  right = right < 0n ? -right : right;

  while (right !== 0n) {
    const remainder = left % right;
    left = right;
    right = remainder;
  }

  return left;
}

/** @param {bigint} value @returns {bigint[] | null} */
function positiveDivisors(value) {
  const abs = I128.checkedAbs(value);
  if (abs === null) {
    return null;
  }
  if (abs === 0n || abs > MAX_FACTORABLE_COEFFICIENT_ABS) {
    return null;
  }

  /** @type {bigint[]} */
  const divisors = [];
  let candidate = 1n;
  while (candidate <= abs / candidate) {
    if (abs % candidate === 0n) {
      divisors.push(candidate);
      const paired = abs / candidate;
      if (paired !== candidate) {
        divisors.push(paired);
      }
    }
    candidate += 1n;
  }

  return divisors;
}

/** @param {string} variable @param {Rational[]} roots @returns {string} */
function formatSolutions(variable, roots) {
  return roots.map((root) => `${variable} = ${root.toDisplayString()}`).join(' or ');
}

/** @param {string} variable @param {Rational[]} roots @returns {string} */
function formatRootFactors(variable, roots) {
  return roots.map((root) => formatRootFactor(variable, root)).join(' * ');
}

/** @param {string} variable @param {Rational} root @returns {string} */
function formatRootFactor(variable, root) {
  const v = formatVariablePower(variable, 1);
  if (root.isZero()) {
    return v;
  }

  if (root.isNegative()) {
    return `(${v} + ${root.abs().toDisplayString()})`;
  } else {
    return `(${v} - ${root.toDisplayString()})`;
  }
}

/** @param {string} variable @param {number} degree @returns {string} */
function formatVariablePower(variable, degree) {
  const v = variable === '*' ? '(*)' : variable;

  if (degree === 1) {
    return v;
  } else {
    return `${v}^${degree}`;
  }
}

/** @param {[boolean, string][]} parts @returns {string} */
function joinSignedParts(parts) {
  let result = '';

  parts.forEach(([isNegative, body], index) => {
    if (index === 0) {
      if (isNegative) {
        result += '-';
      }
    } else if (isNegative) {
      result += ' - ';
    } else {
      result += ' + ';
    }
    result += body;
  });

  return result === '' ? '0' : result;
}
