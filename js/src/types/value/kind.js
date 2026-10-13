/**
 * Value kinds (port of `src/types/value/kind.rs`).
 */

/** Display format for an exact dimensionless ratio. */
export class RatioFormat {
  /** @param {'Fraction' | 'Reciprocal' | 'Multiplier'} tag */
  constructor(tag) {
    this.tag = tag;
    Object.freeze(this);
  }

  /** A reduced fraction, such as `1/5`. */
  static Fraction = new RatioFormat('Fraction');
  /** Odds with a numerator of one, such as `1/2.5`. */
  static Reciprocal = new RatioFormat('Reciprocal');
  /** A multiplier, such as `4x`. */
  static Multiplier = new RatioFormat('Multiplier');

  /** serde: the variant name. @returns {string} */
  toJSON() {
    return this.tag;
  }
}

/**
 * Different kinds of values the calculator can work with.
 *
 * Tuple variants store their payload as `_0`; struct variants keep their
 * field names.
 */
export class ValueKind {
  /** @param {string} tag @param {object} [fields] */
  constructor(tag, fields) {
    /** @type {'Number' | 'Rational' | 'Percent' | 'Ratio' | 'Uncertainty' | 'DateTime' | 'Duration' | 'Boolean' | 'Comparison' | 'EquationSolution' | 'EquationSolutions' | 'SymbolicEquationSolution'} */
    this.tag = /** @type {any} */ (tag);
    const f = /** @type {any} */ (fields ?? {});
    // Payload fields (`undefined` where the variant has none).
    /** @type {any} */ this._0 = f._0;
    /** @type {any} */ this.value = f.value;
    /** @type {any} */ this.format = f.format;
    /** @type {any} */ this.uncertainty = f.uncertainty;
    /** @type {any} */ this.seconds = f.seconds;
    /** @type {any} */ this.left = f.left;
    /** @type {any} */ this.relation = f.relation;
    /** @type {any} */ this.right = f.right;
    /** @type {any} */ this.variable = f.variable;
    /** @type {any} */ this.values = f.values;
    /** @type {any} */ this.expression = f.expression;
    Object.freeze(this);
  }

  /**
   * A decimal number (for compatibility and complex operations).
   * @param {import('../decimal.js').Decimal} n
   */
  static Number(n) {
    return new ValueKind('Number', { _0: n });
  }

  /**
   * A rational number for exact fractional arithmetic.
   * @param {import('../rational.js').Rational} r
   */
  static Rational(r) {
    return new ValueKind('Rational', { _0: r });
  }

  /**
   * A percentage stored as its exact fraction (`10%` stores `1/10`).
   * @param {import('../rational.js').Rational} r
   */
  static Percent(r) {
    return new ValueKind('Percent', { _0: r });
  }

  /**
   * An exact dimensionless ratio with an explicit display format: `value` is
   * the numeric fraction, `format` the requested presentation.
   * @param {{ value: import('../rational.js').Rational, format: RatioFormat }} fields
   */
  static Ratio({ value, format }) {
    return new ValueKind('Ratio', { value, format });
  }

  /**
   * A measured value with a non-negative absolute uncertainty: `value` is the
   * central value, `uncertainty` the absolute error bound.
   * @param {{ value: import('../rational.js').Rational, uncertainty: import('../rational.js').Rational }} fields
   */
  static Uncertainty({ value, uncertainty }) {
    return new ValueKind('Uncertainty', { value, uncertainty });
  }

  /**
   * A date and/or time.
   * @param {import('../datetime.js').DateTime} dt
   */
  static DateTime(dt) {
    return new ValueKind('DateTime', { _0: dt });
  }

  /**
   * A duration (difference between two datetimes), in seconds (an `i64`).
   * @param {{ seconds: bigint }} fields
   */
  static Duration({ seconds }) {
    return new ValueKind('Duration', { seconds });
  }

  /**
   * A boolean value.
   * @param {boolean} b
   */
  static Boolean(b) {
    return new ValueKind('Boolean', { _0: b });
  }

  /**
   * A generic comparison result such as `1 < 2`: display strings for the left
   * and right values, and the relation symbol (`<`, `>`, or `=`).
   * @param {{ left: string, relation: string, right: string }} fields
   */
  static Comparison({ left, relation, right }) {
    return new ValueKind('Comparison', { left, relation, right });
  }

  /**
   * A solved single-variable equation.
   * @param {{ variable: string, value: import('../rational.js').Rational }} fields
   */
  static EquationSolution({ variable, value }) {
    return new ValueKind('EquationSolution', { variable, value });
  }

  /**
   * A solved equation with multiple exact values for the same variable.
   * @param {{ variable: string, values: import('../rational.js').Rational[] }} fields
   */
  static EquationSolutions({ variable, values }) {
    return new ValueKind('EquationSolutions', { variable, values });
  }

  /**
   * A solved linear equation whose result still contains other variables.
   * @param {{ variable: string, expression: string }} fields
   */
  static SymbolicEquationSolution({ variable, expression }) {
    return new ValueKind('SymbolicEquationSolution', { variable, expression });
  }

  /**
   * serde's externally tagged representation.
   * @returns {unknown}
   */
  toJSON() {
    switch (this.tag) {
      case 'Number':
      case 'Rational':
      case 'Percent':
      case 'DateTime':
      case 'Boolean':
        return { [this.tag]: this._0 };
      case 'Ratio':
        return { Ratio: { value: this.value, format: this.format } };
      case 'Uncertainty':
        return { Uncertainty: { value: this.value, uncertainty: this.uncertainty } };
      case 'Duration':
        // serde writes the i64; JSON numbers lose precision beyond 2^53.
        return { Duration: { seconds: Number(this.seconds) } };
      case 'Comparison':
        return { Comparison: { left: this.left, relation: this.relation, right: this.right } };
      case 'EquationSolution':
        return { EquationSolution: { variable: this.variable, value: this.value } };
      case 'EquationSolutions':
        return { EquationSolutions: { variable: this.variable, values: this.values } };
      default:
        return { SymbolicEquationSolution: { variable: this.variable, expression: this.expression } };
    }
  }
}
