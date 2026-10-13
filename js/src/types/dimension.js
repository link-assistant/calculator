/**
 * Physical dimensions as vectors of integer exponents (issue #235).
 *
 * Every physical unit has a dimension such as `length` (metre),
 * `length·time^-1` (speed) or `mass·length^2·time^-2` (energy).
 * Multiplication adds exponents, division subtracts them and powers scale
 * them, so `10 m × 10 m` has dimension `length^2`. Addition requires equal
 * dimensions.
 */

import { eq } from '../rust/core.js';

/** Names of the base dimensions, in vector order. */
export const BASE_DIMENSIONS = Object.freeze([
  'length',
  'mass',
  'time',
  'current',
  'temperature',
  'amount',
  'luminosity',
  'information',
  'angle',
  'currency',
]);

/**
 * A dimension: the exponent of each base dimension.
 *
 * `pub struct Dimension(pub [i32; 10])`, `Copy`: instances and their
 * exponent arrays are frozen. Compare with `eq` from `rust/core.js`
 * (`#[derive(PartialEq)]`). The `i32` exponents are numbers; their
 * arithmetic wraps to 32 bits as in a release build.
 */
export class Dimension {
  /** @param {readonly number[]} exponents ten `i32` exponents */
  constructor(exponents) {
    /** @type {readonly number[]} */
    this._0 = Object.freeze([...exponents]);
    Object.freeze(this);
  }

  /** The dimension of plain numbers. */
  static NONE = new Dimension(Array(10).fill(0));

  /**
   * `#[derive(Default)]`: every exponent zero.
   * @returns {Dimension}
   */
  static default() {
    return new Dimension(Array(10).fill(0));
  }

  /**
   * Returns the base dimension with the given name (`length`, `mass`, …).
   * @param {string} name
   * @returns {Dimension | null}
   */
  static base(name) {
    const index = BASE_DIMENSIONS.indexOf(name);
    if (index === -1) {
      return null;
    }
    const exponents = Array(10).fill(0);
    exponents[index] = 1;
    return new Dimension(exponents);
  }

  /**
   * Whether this is the dimension of plain numbers.
   * @returns {boolean}
   */
  isNone() {
    return eq(this, Dimension.NONE);
  }

  /**
   * The dimension of a product.
   * @param {Dimension} other
   * @returns {Dimension}
   */
  mul(other) {
    const exponents = [...this._0];
    for (let index = 0; index < exponents.length; index += 1) {
      exponents[index] = (exponents[index] + other._0[index]) | 0;
    }
    return new Dimension(exponents);
  }

  /**
   * The dimension of a quotient.
   * @param {Dimension} other
   * @returns {Dimension}
   */
  div(other) {
    return this.mul(other.pow(-1));
  }

  /**
   * The dimension raised to an integer power.
   * @param {number} power an `i32`
   * @returns {Dimension}
   */
  pow(power) {
    return new Dimension(this._0.map((exponent) => Math.imul(exponent, power)));
  }

  /**
   * The `n`-th root, if every exponent stays an integer.
   * @param {number} n an `i32`
   * @returns {Dimension | null}
   */
  root(n) {
    if (n === 0 || this._0.some((exponent) => exponent % n !== 0)) {
      return null;
    }
    return new Dimension(this._0.map((exponent) => (exponent / n) | 0));
  }

  /**
   * `impl Display`: `dimensionless`, or the non-zero exponents such as
   * `length·time^-1`.
   * @returns {string}
   */
  toString() {
    if (this.isNone()) {
      return 'dimensionless';
    }
    let f = '';
    let first = true;
    for (let index = 0; index < BASE_DIMENSIONS.length; index += 1) {
      const name = BASE_DIMENSIONS[index];
      const exponent = this._0[index];
      if (exponent === 0) {
        continue;
      }
      if (!first) {
        f += '·';
      }
      first = false;
      if (exponent === 1) {
        f += name;
      } else {
        f += `${name}^${exponent}`;
      }
    }
    return f;
  }
}
