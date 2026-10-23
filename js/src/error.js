/**
 * Error types for the Link Calculator.
 */

import { debugStruct, debugTuple } from './rust/fmt.js';
import { debugStr } from './rust/str.js';

/**
 * The fields of the variants of {@link CalculatorError}: `_0` for the tuple
 * variants, the camelCase field names for the struct variants.
 * @typedef {object} CalculatorErrorFields
 * @property {string} [_0]
 * @property {string} [found]
 * @property {string} [expected]
 * @property {number} [position]
 * @property {string} [operation]
 * @property {string} [leftUnit]
 * @property {string} [rightUnit]
 * @property {string} [from]
 * @property {string} [to]
 * @property {string} [reason]
 * @property {string} [currency]
 * @property {string} [date]
 * @property {string} [function]
 * @property {string} [expression]
 * @property {string} [result]
 * @property {string} [latexInput]
 * @property {string} [latexResult]
 */

/**
 * Main error type for calculator operations.
 *
 * `#[derive(Error, Debug, Clone, PartialEq, Eq)]`: `toString()` is the
 * `#[error(...)]` message, `fmtDebug()` the derived `Debug`; compare with
 * `eq` from `rust/core.js`.
 */
export class CalculatorError {
  /**
   * @param {string} tag the variant name
   * @param {CalculatorErrorFields} [fields]
   */
  constructor(tag, fields) {
    this.tag = tag;
    Object.assign(this, fields);
    Object.freeze(this);
  }

  /**
   * Error parsing the input expression.
   * @param {string} value
   */
  static ParseError(value) {
    return new CalculatorError('ParseError', { _0: value });
  }

  /**
   * Error with an unexpected token during parsing.
   * @param {{ found: string, expected: string, position: number }} fields
   */
  static UnexpectedToken({ found, expected, position }) {
    return new CalculatorError('UnexpectedToken', { found, expected, position });
  }

  /**
   * Error when units don't match in an operation.
   * @param {{ operation: string, leftUnit: string, rightUnit: string }} fields
   */
  static UnitMismatch({ operation, leftUnit, rightUnit }) {
    return new CalculatorError('UnitMismatch', { operation, leftUnit, rightUnit });
  }

  /**
   * Error during evaluation.
   * @param {string} value
   */
  static EvaluationError(value) {
    return new CalculatorError('EvaluationError', { _0: value });
  }

  /** Division by zero error. */
  static DivisionByZero = new CalculatorError('DivisionByZero');

  /**
   * Invalid date/time format.
   * @param {string} value
   */
  static InvalidDateTime(value) {
    return new CalculatorError('InvalidDateTime', { _0: value });
  }

  /**
   * Invalid currency.
   * @param {string} value
   */
  static UnknownCurrency(value) {
    return new CalculatorError('UnknownCurrency', { _0: value });
  }

  /**
   * Currency conversion error.
   * @param {{ from: string, to: string, reason: string }} fields
   */
  static CurrencyConversion({ from, to, reason }) {
    return new CalculatorError('CurrencyConversion', { from, to, reason });
  }

  /**
   * No historical rate available.
   * @param {{ currency: string, date: string }} fields
   */
  static NoHistoricalRate({ currency, date }) {
    return new CalculatorError('NoHistoricalRate', { currency, date });
  }

  /** Overflow error. */
  static Overflow = new CalculatorError('Overflow');

  /**
   * Invalid operation.
   * @param {string} value
   */
  static InvalidOperation(value) {
    return new CalculatorError('InvalidOperation', { _0: value });
  }

  /** Empty input error. */
  static EmptyInput = new CalculatorError('EmptyInput');

  /**
   * Unknown function error.
   * @param {string} value
   */
  static UnknownFunction(value) {
    return new CalculatorError('UnknownFunction', { _0: value });
  }

  /**
   * Invalid function arguments error.
   * @param {{ function: string, reason: string }} fields
   */
  static InvalidFunctionArgs({ function: func, reason }) {
    return new CalculatorError('InvalidFunctionArgs', { function: func, reason });
  }

  /**
   * Domain error (e.g., sqrt of negative number, log of non-positive).
   * @param {string} value
   */
  static DomainError(value) {
    return new CalculatorError('DomainError', { _0: value });
  }

  /**
   * Symbolic result (for indefinite integrals and symbolic computation).
   * This is not really an error but a different type of result that needs special handling.
   *
   * - `expression`: The original expression in text form.
   * - `result`: The symbolic result.
   * - `latexInput`: LaTeX representation of the input.
   * - `latexResult`: LaTeX representation of the result.
   * @param {{ expression: string, result: string, latexInput: string, latexResult: string }} fields
   */
  static SymbolicResult({ expression, result, latexInput, latexResult }) {
    return new CalculatorError('SymbolicResult', { expression, result, latexInput, latexResult });
  }

  /**
   * The `#[error(...)]` message (`Display`).
   * @returns {string}
   */
  toString() {
    const f = /** @type {CalculatorErrorFields} */ (this);
    switch (this.tag) {
      case 'ParseError':
        return `Parse error: ${f._0}`;
      case 'UnexpectedToken':
        return `Unexpected token '${f.found}' at position ${f.position}, expected ${f.expected}`;
      case 'UnitMismatch':
        return `Unit mismatch: cannot ${f.operation} '${f.leftUnit}' and '${f.rightUnit}'`;
      case 'EvaluationError':
        return `Evaluation error: ${f._0}`;
      case 'DivisionByZero':
        return 'Division by zero';
      case 'InvalidDateTime':
        return `Invalid datetime format: ${f._0}`;
      case 'UnknownCurrency':
        return `Unknown currency: ${f._0}`;
      case 'CurrencyConversion':
        return `Cannot convert ${f.from} to ${f.to}: ${f.reason}`;
      case 'NoHistoricalRate':
        return `No exchange rate available for ${f.currency} on ${f.date}`;
      case 'Overflow':
        return 'Numeric overflow';
      case 'InvalidOperation':
        return `Invalid operation: ${f._0}`;
      case 'EmptyInput':
        return 'Empty input';
      case 'UnknownFunction':
        return `Unknown function: ${f._0}`;
      case 'InvalidFunctionArgs':
        return `Invalid arguments for function '${f.function}': ${f.reason}`;
      case 'DomainError':
        return `Domain error: ${f._0}`;
      case 'SymbolicResult':
        return `${f.result}`;
      default:
        throw new TypeError(`unknown CalculatorError variant ${this.tag}`);
    }
  }

  /**
   * The derived `Debug` (`{:?}`), e.g. `ParseError("x")` or
   * `UnexpectedToken { found: "+", expected: "number", position: 5 }`.
   * @returns {string}
   */
  fmtDebug() {
    const f = /** @type {CalculatorErrorFields} */ (this);
    const str = (/** @type {string | undefined} */ value) => debugStr(/** @type {string} */ (value));
    switch (this.tag) {
      case 'ParseError':
      case 'EvaluationError':
      case 'InvalidDateTime':
      case 'UnknownCurrency':
      case 'InvalidOperation':
      case 'UnknownFunction':
      case 'DomainError':
        return debugTuple(this.tag, [str(f._0)]);
      case 'UnexpectedToken':
        return debugStruct(this.tag, [
          ['found', str(f.found)],
          ['expected', str(f.expected)],
          ['position', String(f.position)],
        ]);
      case 'UnitMismatch':
        return debugStruct(this.tag, [
          ['operation', str(f.operation)],
          ['left_unit', str(f.leftUnit)],
          ['right_unit', str(f.rightUnit)],
        ]);
      case 'CurrencyConversion':
        return debugStruct(this.tag, [
          ['from', str(f.from)],
          ['to', str(f.to)],
          ['reason', str(f.reason)],
        ]);
      case 'NoHistoricalRate':
        return debugStruct(this.tag, [
          ['currency', str(f.currency)],
          ['date', str(f.date)],
        ]);
      case 'InvalidFunctionArgs':
        return debugStruct(this.tag, [
          ['function', str(f.function)],
          ['reason', str(f.reason)],
        ]);
      case 'SymbolicResult':
        return debugStruct(this.tag, [
          ['expression', str(f.expression)],
          ['result', str(f.result)],
          ['latex_input', str(f.latexInput)],
          ['latex_result', str(f.latexResult)],
        ]);
      default:
        return this.tag;
    }
  }

  /**
   * Returns the i18n error info for this error.
   *
   * This provides the translation key and any parameters needed
   * for the frontend to display a localized error message.
   * @returns {ErrorInfo}
   */
  toErrorInfo() {
    const f = /** @type {CalculatorErrorFields} */ (this);
    switch (this.tag) {
      case 'ParseError': {
        const msg = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('message', msg);
        return ErrorInfo.withParams('errors.parseError', params);
      }
      case 'UnexpectedToken': {
        const { found, expected, position } = f;
        const params = new Map();
        params.set('found', /** @type {string} */ (found));
        params.set('expected', /** @type {string} */ (expected));
        params.set('position', String(position));
        return ErrorInfo.withParams('errors.unexpectedToken', params);
      }
      case 'UnitMismatch': {
        const { operation, leftUnit, rightUnit } = f;
        const params = new Map();
        params.set('operation', /** @type {string} */ (operation));
        params.set('leftUnit', /** @type {string} */ (leftUnit));
        params.set('rightUnit', /** @type {string} */ (rightUnit));
        return ErrorInfo.withParams('errors.unitMismatch', params);
      }
      case 'EvaluationError': {
        const msg = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('message', msg);
        return ErrorInfo.withParams('errors.evaluationError', params);
      }
      case 'DivisionByZero':
        return ErrorInfo.new('errors.divisionByZero');
      case 'InvalidDateTime': {
        const format = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('format', format);
        return ErrorInfo.withParams('errors.invalidDateTime', params);
      }
      case 'UnknownCurrency': {
        const currency = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('currency', currency);
        return ErrorInfo.withParams('errors.unknownCurrency', params);
      }
      case 'CurrencyConversion': {
        const { from, to, reason } = f;
        const params = new Map();
        params.set('from', /** @type {string} */ (from));
        params.set('to', /** @type {string} */ (to));
        params.set('reason', /** @type {string} */ (reason));
        return ErrorInfo.withParams('errors.currencyConversion', params);
      }
      case 'NoHistoricalRate': {
        const { currency, date } = f;
        const params = new Map();
        params.set('currency', /** @type {string} */ (currency));
        params.set('date', /** @type {string} */ (date));
        return ErrorInfo.withParams('errors.noHistoricalRate', params);
      }
      case 'Overflow':
        return ErrorInfo.new('errors.overflow');
      case 'InvalidOperation': {
        const msg = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('message', msg);
        return ErrorInfo.withParams('errors.invalidOperation', params);
      }
      case 'EmptyInput':
        return ErrorInfo.new('errors.emptyInput');
      case 'UnknownFunction': {
        const name = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('name', name);
        return ErrorInfo.withParams('errors.unknownFunction', params);
      }
      case 'InvalidFunctionArgs': {
        const { function: func, reason } = f;
        const params = new Map();
        params.set('function', /** @type {string} */ (func));
        params.set('reason', /** @type {string} */ (reason));
        return ErrorInfo.withParams('errors.invalidFunctionArgs', params);
      }
      case 'DomainError': {
        const msg = /** @type {string} */ (f._0);
        const params = new Map();
        params.set('message', msg);
        return ErrorInfo.withParams('errors.domainError', params);
      }
      case 'SymbolicResult': {
        const { result } = f;
        // SymbolicResult is not really an error, but we provide info for consistency
        const params = new Map();
        params.set('result', /** @type {string} */ (result));
        return ErrorInfo.withParams('result.symbolic', params);
      }
      default:
        throw new TypeError(`unknown CalculatorError variant ${this.tag}`);
    }
  }

  /**
   * Creates a parse error with the given message.
   * @param {string} msg
   * @returns {CalculatorError}
   */
  static parse(msg) {
    return CalculatorError.ParseError(msg);
  }

  /**
   * Creates an unexpected token error.
   * @param {string} found
   * @param {string} expected
   * @param {number} position
   * @returns {CalculatorError}
   */
  static unexpectedToken(found, expected, position) {
    return CalculatorError.UnexpectedToken({
      found,
      expected,
      position,
    });
  }

  /**
   * Creates a unit mismatch error.
   * @param {string} operation
   * @param {string} left
   * @param {string} right
   * @returns {CalculatorError}
   */
  static unitMismatch(operation, left, right) {
    return CalculatorError.UnitMismatch({
      operation,
      leftUnit: left,
      rightUnit: right,
    });
  }

  /**
   * Creates an evaluation error with the given message.
   * @param {string} msg
   * @returns {CalculatorError}
   */
  static eval(msg) {
    return CalculatorError.EvaluationError(msg);
  }

  /**
   * Creates an unknown function error.
   * @param {string} name
   * @returns {CalculatorError}
   */
  static unknownFunction(name) {
    return CalculatorError.UnknownFunction(name);
  }

  /**
   * Creates an invalid function arguments error.
   * @param {string} func the Rust parameter `function` (a reserved word in JavaScript)
   * @param {string} reason
   * @returns {CalculatorError}
   */
  static invalidArgs(func, reason) {
    return CalculatorError.InvalidFunctionArgs({
      function: func,
      reason,
    });
  }

  /**
   * Creates a domain error.
   * @param {string} msg
   * @returns {CalculatorError}
   */
  static domain(msg) {
    return CalculatorError.DomainError(msg);
  }
}

/**
 * Error information for i18n support.
 *
 * This struct contains all the information needed to translate an error
 * on the frontend, including the error key and any interpolation parameters.
 *
 * `params` is a `HashMap` in Rust, so serde serializes its entries in an
 * unspecified (per-process random) order; the `Map` here keeps insertion
 * order, the order of the `insert` calls in `toErrorInfo`.
 */
export class ErrorInfo {
  /**
   * @param {{ key: string, params: Map<string, string> | null }} fields
   */
  constructor({ key, params }) {
    /** The translation key for this error (e.g., "errors.divisionByZero"). */
    this.key = key;
    /** Parameters for interpolation in the translated message. */
    this.params = params;
  }

  /**
   * Creates a new error info with just a key.
   * @param {string} key
   * @returns {ErrorInfo}
   */
  static new(key) {
    return new ErrorInfo({
      key,
      params: null,
    });
  }

  /**
   * Creates a new error info with key and parameters.
   * @param {string} key
   * @param {Map<string, string>} params
   * @returns {ErrorInfo}
   */
  static withParams(key, params) {
    return new ErrorInfo({
      key,
      params,
    });
  }

  /**
   * serde's JSON: `params` is skipped when `None`.
   * @returns {{ key: string, params?: Record<string, string> }}
   */
  toJSON() {
    /** @type {{ key: string, params?: Record<string, string> }} */
    const json = { key: this.key };
    if (this.params !== null) {
      json.params = Object.fromEntries(this.params);
    }
    return json;
  }
}
