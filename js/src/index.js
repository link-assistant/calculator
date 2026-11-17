/**
 * Link Calculator - A grammar-based expression parser and calculator
 * (port of `src/lib.rs`).
 *
 * This library provides a WebAssembly-compatible calculator that supports:
 * - `DateTime` parsing and arithmetic
 * - Decimal numbers with units
 * - Currency conversions with temporal awareness
 * - Links notation for expression representation
 *
 * @example
 * import { Calculator } from 'link-calculator';
 *
 * const calculator = Calculator.new();
 * const result = calculator.calculateInternal('2 + 3');
 * // result.success === true, result.result === '5'
 */

import * as clock from './clock.js';
import * as cryptoApi from './crypto_api.js';
import * as currencyApi from './currency_api.js';
import { CalculatorError } from './error.js';
import { ExpressionParser } from './grammar/index.js';
import * as plan from './plan.js';
import { fma } from './rust/libm.js';
import { parseF64 } from './rust/num.js';
import { lines, splitWhitespace, stripPrefix, trim, trimEndMatches, trimStartMatches } from './rust/str.js';
import { err, ok, tryResult } from './rust/core.js';
import { substituteVariable } from './substitution.js';
import { DateTimeResult, ExchangeRateInfo } from './types/index.js';
import { generateIssueLink } from './utils.js';

export * as clock from './clock.js';
export * as cryptoApi from './crypto_api.js';
export * as currencyApi from './currency_api.js';
export * as error from './error.js';
export * as grammar from './grammar/index.js';
export * as lino from './lino/index.js';
export * as plan from './plan.js';
export * as types from './types/index.js';
export * as utils from './utils.js';

export { CalculationPlan, RateSource } from './plan.js';
export { generateIssueLink, truncate } from './utils.js';

/** @typedef {import('./types/index.js').Expression} Expression */
/** @typedef {import('./types/index.js').Value} Value */
/** @typedef {import('./error.js').ErrorInfo} ErrorInfo */

/** Package version (matches Cargo.toml version). */
export const VERSION = '0.24.0';

/** Data for plotting a function. */
export class PlotData {
  /**
   * @param {{ xValues: number[], yValues: number[], label: string, xLabel: string, yLabel: string }} fields
   */
  constructor({ xValues, yValues, label, xLabel, yLabel }) {
    /** X-axis values. */
    this.xValues = xValues;
    /** Y-axis values. */
    this.yValues = yValues;
    /** Label for the plot (e.g., "sin(x)/x"). */
    this.label = label;
    /** X-axis label. */
    this.xLabel = xLabel;
    /** Y-axis label. */
    this.yLabel = yLabel;
  }

  /** serde's JSON. */
  toJSON() {
    return {
      x_values: this.xValues,
      y_values: this.yValues,
      label: this.label,
      x_label: this.xLabel,
      y_label: this.yLabel,
    };
  }
}

/** A single calculation step with i18n support. */
export class CalculationStep {
  /** @param {{ key: string, params: Map<string, string> | null, text: string }} fields */
  constructor({ key, params, text }) {
    /** The translation key for this step type. */
    this.key = key;
    /** Parameters for interpolation in the translated message. */
    this.params = params;
    /** The raw (English) text for fallback. */
    this.text = text;
  }

  /**
   * Creates a new step with a translation key, params, and fallback text.
   * @param {string} key
   * @param {Map<string, string> | null} params
   * @param {string} text
   */
  static new(key, params, text) {
    return new CalculationStep({ key, params, text });
  }

  /**
   * Creates a simple step with just text (no translation key).
   * @param {string} text
   */
  static textOnly(text) {
    return new CalculationStep({ key: '', params: null, text });
  }

  /** serde's JSON (`params` skipped when `None`). */
  toJSON() {
    /** @type {Record<string, unknown>} */
    const json = { key: this.key };
    if (this.params !== null) {
      json.params = Object.fromEntries(this.params);
    }
    json.text = this.text;
    return json;
  }
}

/** Repeating decimal notation formats. */
export class RepeatingDecimalFormats {
  /**
   * @param {{ vinculum: string, parenthesis: string, ellipsis: string, latex: string, fraction: string }} fields
   */
  constructor({ vinculum, parenthesis, ellipsis, latex, fraction }) {
    /** Vinculum notation with overline: 0.3̅ */
    this.vinculum = vinculum;
    /** Parenthesis notation: 0.(3) */
    this.parenthesis = parenthesis;
    /** Ellipsis notation: 0.333... */
    this.ellipsis = ellipsis;
    /** LaTeX notation: 0.\overline{3} */
    this.latex = latex;
    /** Fraction representation: 1/3 */
    this.fraction = fraction;
  }
}

/**
 * @typedef {{ result: string, linoInterpretation: string, alternativeLino: string[] | null,
 *   steps: string[], stepsI18n: CalculationStep[] | null, success: boolean, error: string | null,
 *   errorInfo: ErrorInfo | null, issueLink: string | null, latexInput: string | null,
 *   latexResult: string | null, isSymbolic: boolean | null, plotData: PlotData | null,
 *   repeatingDecimal: RepeatingDecimalFormats | null, fraction: string | null,
 *   isLiveTime: boolean | null, datetimeResult: DateTimeResult | null }} CalculationResultFields
 */

/** The fields every constructor of `CalculationResult` leaves at their defaults. */
const RESULT_DEFAULTS = Object.freeze({
  alternativeLino: null,
  stepsI18n: null,
  success: true,
  error: null,
  errorInfo: null,
  issueLink: null,
  latexInput: null,
  latexResult: null,
  isSymbolic: null,
  plotData: null,
  repeatingDecimal: null,
  fraction: null,
  isLiveTime: null,
  datetimeResult: null,
});

/** Result of a calculation operation. */
export class CalculationResult {
  /** @param {CalculationResultFields} fields */
  constructor(fields) {
    /** The computed value as a string. */
    this.result = fields.result;
    /** The input interpreted in links notation format. */
    this.linoInterpretation = fields.linoInterpretation;
    /**
     * Alternative links notation interpretations the user can switch between.
     * The first element is always the currently selected (default) interpretation.
     */
    this.alternativeLino = fields.alternativeLino;
    /** Step-by-step explanation of the calculation (raw text for backwards compatibility). */
    this.steps = fields.steps;
    /** Step-by-step explanation with i18n support. */
    this.stepsI18n = fields.stepsI18n;
    /** Whether the calculation was successful. */
    this.success = fields.success;
    /** Error message if calculation failed (raw text for backwards compatibility). */
    this.error = fields.error;
    /** Error information for i18n support. */
    this.errorInfo = fields.errorInfo;
    /** Link to create an issue for unrecognized input. */
    this.issueLink = fields.issueLink;
    /** LaTeX representation of the input (for rendering mathematical formulas). */
    this.latexInput = fields.latexInput;
    /** LaTeX representation of the result (for rendering mathematical formulas). */
    this.latexResult = fields.latexResult;
    /** Whether this is a symbolic result (e.g., indefinite integral). */
    this.isSymbolic = fields.isSymbolic;
    /** Plot data points for graphing (x, y pairs). */
    this.plotData = fields.plotData;
    /** Repeating decimal notations (if the result is a repeating decimal). */
    this.repeatingDecimal = fields.repeatingDecimal;
    /** Fraction representation of the result (if applicable). */
    this.fraction = fields.fraction;
    /**
     * Whether the result represents a live (auto-updating) time expression.
     * When `true`, the frontend should periodically re-calculate the expression.
     */
    this.isLiveTime = fields.isLiveTime;
    /** Structured datetime metadata for browser-local and UTC conversion display. */
    this.datetimeResult = fields.datetimeResult;
  }

  /**
   * Creates a successful calculation result.
   * @param {string} result
   * @param {string} lino
   * @param {string[]} steps
   */
  static success(result, lino, steps) {
    return new CalculationResult({ ...RESULT_DEFAULTS, result, linoInterpretation: lino, steps });
  }

  /**
   * Creates a successful calculation result with rational value information.
   * @param {Value} value
   * @param {string} lino
   * @param {string[]} steps
   */
  static successWithValue(value, lino, steps) {
    const result = value.toDisplayString();

    // Extract repeating decimal and fraction info if available
    let repeatingDecimal = null;
    let fraction = null;
    const rational = value.asRational();
    if (rational !== null) {
      fraction = !rational.isInteger() ? rational.toFractionString() : null;

      const rd = rational.toRepeatingDecimalNotation();
      repeatingDecimal = rd === null
        ? null
        : new RepeatingDecimalFormats({
          vinculum: rd.toVinculumNotation(),
          parenthesis: rd.toParenthesisNotation(),
          ellipsis: rd.toEllipsisNotation(),
          latex: rd.toLatex(),
          fraction: rational.toFractionString(),
        });
    }
    const datetimeResult = value.kind.tag === 'DateTime'
      ? DateTimeResult.fromDatetime(value.kind._0)
      : null;

    return new CalculationResult({
      ...RESULT_DEFAULTS,
      result,
      linoInterpretation: lino,
      steps,
      repeatingDecimal,
      fraction,
      datetimeResult,
    });
  }

  /**
   * Creates a successful calculation result with i18n step info.
   * @param {string} result
   * @param {string} lino
   * @param {string[]} steps
   * @param {CalculationStep[]} stepsI18n
   */
  static successWithI18n(result, lino, steps, stepsI18n) {
    return new CalculationResult({ ...RESULT_DEFAULTS, result, linoInterpretation: lino, steps, stepsI18n });
  }

  /**
   * Creates a successful calculation result with LaTeX formatting.
   * @param {string} result
   * @param {string} lino
   * @param {string[]} steps
   * @param {string | null} latexInput
   * @param {string | null} latexResult
   */
  static successWithLatex(result, lino, steps, latexInput, latexResult) {
    return new CalculationResult({
      ...RESULT_DEFAULTS,
      result,
      linoInterpretation: lino,
      steps,
      latexInput,
      latexResult,
    });
  }

  /**
   * Creates a symbolic result (e.g., for indefinite integrals).
   * @param {string} expression
   * @param {string} result
   * @param {string} latexInput
   * @param {string} latexResult
   * @param {PlotData | null} plotData
   */
  static symbolic(expression, result, latexInput, latexResult, plotData) {
    return new CalculationResult({
      ...RESULT_DEFAULTS,
      result,
      linoInterpretation: expression,
      steps: [`Input: ${expression}`, 'Computed symbolic result'],
      latexInput,
      latexResult,
      isSymbolic: true,
      plotData,
    });
  }

  /**
   * Creates a failed calculation result.
   * @param {string} error
   * @param {string} input
   */
  static failure(error, input) {
    const issueLink = generateIssueLink(input, error);
    return new CalculationResult({
      ...RESULT_DEFAULTS,
      result: '',
      linoInterpretation: '',
      steps: [],
      success: false,
      error,
      issueLink,
    });
  }

  /**
   * Creates a failed calculation result with i18n error info.
   * @param {CalculatorError} error
   * @param {string} input
   */
  static failureWithI18n(error, input) {
    const errorString = error.toString();
    const issueLink = generateIssueLink(input, errorString);
    return new CalculationResult({
      ...RESULT_DEFAULTS,
      result: '',
      linoInterpretation: '',
      steps: [],
      success: false,
      error: errorString,
      errorInfo: error.toErrorInfo(),
      issueLink,
    });
  }

  /**
   * serde's JSON (field names and order, `skip_serializing_if = "Option::is_none"`).
   * @returns {Record<string, unknown>}
   */
  toJSON() {
    /** @type {Record<string, unknown>} */
    const json = {
      result: this.result,
      lino_interpretation: this.linoInterpretation,
    };
    /** @param {string} key @param {unknown} value */
    const optional = (key, value) => {
      if (value !== null) {
        json[key] = value;
      }
    };
    optional('alternative_lino', this.alternativeLino);
    json.steps = this.steps;
    optional('steps_i18n', this.stepsI18n);
    json.success = this.success;
    json.error = this.error;
    optional('error_info', this.errorInfo);
    json.issue_link = this.issueLink;
    optional('latex_input', this.latexInput);
    optional('latex_result', this.latexResult);
    optional('is_symbolic', this.isSymbolic);
    optional('plot_data', this.plotData);
    optional('repeating_decimal', this.repeatingDecimal);
    optional('fraction', this.fraction);
    optional('is_live_time', this.isLiveTime);
    optional('datetime_result', this.datetimeResult);
    return json;
  }
}

/**
 * `serde_json::from_str::<HashMap<String, f64>>(json)`: an object of numbers,
 * or `null` when the text is not one.
 * @param {string} json
 * @returns {Map<string, number> | null}
 */
function parseRatesJson(json) {
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch {
    return null;
  }
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return null;
  }
  const rates = new Map();
  for (const [key, value] of Object.entries(parsed)) {
    if (typeof value !== 'number' || !Number.isFinite(value)) {
      return null;
    }
    rates.set(key, value);
  }
  return rates;
}

/**
 * `serde_json::to_string(value)`.
 * @param {unknown} value
 */
function toJsonString(value) {
  return JSON.stringify(value);
}

/**
 * Removes the quotes around a `source` value.
 * @param {string} rest
 */
function unquoteSource(rest) {
  let src = trim(rest);
  src = trimEndMatches(trimStartMatches(src, "'"), "'");
  src = trimEndMatches(trimStartMatches(src, '"'), '"');
  return src;
}

/** The main calculator struct. */
export class Calculator {
  /** @param {{ parser: ExpressionParser }} [fields] */
  constructor(fields) {
    /** @type {ExpressionParser} */
    this._parser = fields === undefined ? ExpressionParser.new() : fields.parser;
  }

  /** Creates a new Calculator instance. */
  static new() {
    return new Calculator({ parser: ExpressionParser.new() });
  }

  /** `Default`. */
  static default() {
    return new Calculator({ parser: ExpressionParser.default() });
  }

  /**
   * Plans a calculation without executing it, returning a JSON string.
   *
   * Parses the expression to determine:
   * - Links notation interpretation (default and alternatives)
   * - Which rate sources are required (ECB, CBR, Crypto)
   * - Which currency codes are referenced
   * - Whether the expression contains live time references
   *
   * The worker should call this first, fetch the required rate sources,
   * then call `execute()` to get the actual result.
   * @param {string} input
   * @returns {string}
   */
  plan(input) {
    const p = this.planInternal(input);
    return toJsonString(p);
  }

  /**
   * Executes a calculation, returning a JSON string with the full result.
   *
   * This is the same as `calculate()` but named to clarify the plan→execute pipeline.
   * The worker should call `plan()` first to determine required rate sources,
   * fetch them, then call `execute()`.
   * @param {string} input
   * @returns {string}
   */
  execute(input) {
    const result = this.calculateInternal(input);
    return toJsonString(result);
  }

  /**
   * Calculates the result of an expression, returning a JSON string.
   *
   * Kept for backwards compatibility. Equivalent to `execute()`.
   * @param {string} input
   * @returns {string}
   */
  calculate(input) {
    return this.execute(input);
  }

  /**
   * Sets the user's local timezone offset, in minutes east of UTC.
   *
   * From the browser, pass `-new Date().getTimezoneOffset()` (note the sign:
   * `getTimezoneOffset()` returns minutes *behind* UTC, so it must be negated).
   * For example, UTC+5:30 (India) is `330`, and UTC-5 (US Eastern) is `-300`.
   *
   * Once set, bare times such as `12:30` and the `now` keyword are interpreted
   * in this local timezone instead of UTC. Inputs with an explicit timezone
   * (e.g. `12:30 UTC`, `now UTC`) are always honored regardless of this setting.
   * @param {number} offsetMinutes
   */
  setTimezoneOffset(offsetMinutes) {
    this._parser.setLocalOffsetSeconds(offsetMinutes * 60);
  }

  /**
   * Clears any previously configured local timezone offset, restoring the
   * default UTC interpretation for `now` and bare times.
   */
  clearTimezoneOffset() {
    this._parser.setLocalOffsetSeconds(null);
  }

  /**
   * Returns the version of the calculator.
   * @returns {string}
   */
  static version() {
    return VERSION;
  }

  /**
   * Updates exchange rates from API response. Returns the number of rates updated.
   * Args: `base` (e.g., "USD"), `date` (e.g., "2026-01-25"), `ratesJson` (e.g., `{"eur": 0.92}`).
   * @param {string} base
   * @param {string} date
   * @param {string} ratesJson
   * @returns {number}
   */
  updateRatesFromApi(base, date, ratesJson) {
    const rates = parseRatesJson(ratesJson);
    if (rates === null) {
      return 0;
    }

    const baseUpper = base.toUpperCase();
    const timestamp = clock.now().toRfc3339();
    let count = 0;

    for (const [target, rate] of rates) {
      const targetUpper = target.toUpperCase();

      if (baseUpper === targetUpper) {
        continue;
      } // Skip same currency

      const info = ExchangeRateInfo.new(rate, currencyApi.API_SOURCE, date).withFetchedAt(timestamp);

      const currencyDb = this._parser.currencyDbMut();
      currencyDb.setRateWithInfo(baseUpper, targetUpper, info);
      currencyDb.setHistoricalRateWithInfo(baseUpper, targetUpper, date, info);

      count += 1;
    }

    return count;
  }

  /**
   * Updates RUB exchange rates from the Central Bank of Russia (cbr.ru) API response.
   * Returns the number of rates updated.
   *
   * The CBR rates format: `{"usd": 76.63, "eur": 90.58, "inr": 0.842, ...}`
   * where each value represents "1 CURRENCY = X RUB".
   *
   * These rates take priority over ECB/Frankfurter rates for RUB conversions,
   * since CBR provides official RUB rates directly (no cross-rate needed).
   *
   * Args: `date` (e.g., "2026-02-25"), `ratesJson` (currency_code → RUB amount).
   * @param {string} date
   * @param {string} ratesJson
   * @returns {number}
   */
  updateCbrRatesFromApi(date, ratesJson) {
    const rates = parseRatesJson(ratesJson);
    if (rates === null) {
      return 0;
    }

    const timestamp = clock.now().toRfc3339();
    let count = 0;

    for (const [currency, rubPerUnit] of rates) {
      const currencyUpper = currency.toUpperCase();

      // Skip RUB itself
      if (currencyUpper === 'RUB') {
        continue;
      }

      // Store: 1 CURRENCY = rub_per_unit RUB
      const info = ExchangeRateInfo.new(rubPerUnit, currencyApi.CBR_API_SOURCE, date).withFetchedAt(timestamp);

      const currencyDb = this._parser.currencyDbMut();
      currencyDb.setRateWithInfo(currencyUpper, 'RUB', info);
      currencyDb.setHistoricalRateWithInfo(currencyUpper, 'RUB', date, info);

      count += 1;
    }

    return count;
  }

  /**
   * Updates cryptocurrency exchange rates from API response.
   * Returns the number of rates updated.
   *
   * Args: `base` (fiat currency, e.g., "USD"), `date` (e.g., "2026-01-25"),
   * `ratesJson` (e.g., `{"TON": 5.42, "BTC": 95000.0}`).
   * @param {string} base
   * @param {string} date
   * @param {string} ratesJson
   * @returns {number}
   */
  updateCryptoRatesFromApi(base, date, ratesJson) {
    const rates = parseRatesJson(ratesJson);
    if (rates === null) {
      return 0;
    }

    const baseUpper = base.toUpperCase();
    const timestamp = clock.now().toRfc3339();
    let count = 0;

    for (const [ticker, price] of rates) {
      const tickerUpper = ticker.toUpperCase();
      // Store rate as: 1 ticker = price base_currency
      const info = ExchangeRateInfo.new(price, cryptoApi.COINGECKO_SOURCE, date).withFetchedAt(timestamp);

      const currencyDb = this._parser.currencyDbMut();
      currencyDb.setRateWithInfo(tickerUpper, baseUpper, info);
      currencyDb.setHistoricalRateWithInfo(tickerUpper, baseUpper, date, info);

      count += 1;
    }

    return count;
  }

  /**
   * Loads historical exchange rates from a consolidated .lino format.
   * Returns the number of rates loaded (0 if parsing failed).
   *
   * Supports both the new format (conversion/rates) and legacy format (rates/data).
   * Used by the web worker to populate historical CBR rate data from local .lino files.
   * @param {string} content
   * @returns {number}
   */
  loadRatesFromConsolidatedLino(content) {
    return ok(() => this.loadRatesFromConsolidatedLinoImpl(content)) ?? 0;
  }

  /**
   * Internal planning method — parses expression and determines requirements.
   * @param {string} input
   * @returns {import('./plan.js').CalculationPlan}
   */
  planInternal(input) {
    input = trim(input);
    if (input === '') {
      return plan.emptyPlan();
    }

    const parsed = tryResult(() => this._parser.parseInterpretations(input));
    if (parsed.ok) {
      const interpretations = parsed.value;
      if (interpretations.length > 0) {
        const p = plan.createPlan(input, interpretations[0]);
        p.alternativeLino = Calculator.combinedAlternativeLino(interpretations);
        return p;
      }
      return plan.errorPlan(input, 'Parse error: no parseable interpretation');
    }
    return plan.errorPlan(input, parsed.error.toString());
  }

  /**
   * Internal calculation method that returns a proper Result type.
   * @param {string} input
   * @returns {CalculationResult}
   */
  calculateInternal(input) {
    // Try to parse the expression to generate alternative interpretations
    // and detect live time expressions before evaluation.
    const parsedInterpretations = ok(() => this._parser.parseInterpretations(input));
    const alternatives = parsedInterpretations === null
      ? null
      : Calculator.combinedAlternativeLino(parsedInterpretations);
    const first = parsedInterpretations === null ? undefined : parsedInterpretations[0];
    const isLiveTime = first !== undefined && first.containsLiveTime();

    let result;
    const evaluated = tryResult(() => this._parser.parseAndEvaluate(input));
    if (evaluated.ok) {
      const [value, steps, lino] = evaluated.value;
      result = CalculationResult.successWithValue(value, lino, steps);
      // Set is_live_time for any datetime result so the frontend
      // auto-refreshes the "Time since/until" countdown display.
      const valueIsDatetime = value.kind.tag === 'DateTime';
      if (isLiveTime || valueIsDatetime) {
        result.isLiveTime = true;
      }
    } else if (evaluated.error.tag === 'SymbolicResult') {
      const e = evaluated.error;
      // Generate plot data for the integrand function
      const plotData = this.generatePlotDataForIntegral(input);
      result = CalculationResult.symbolic(e.expression, e.result, e.latexInput, e.latexResult, plotData);
    } else {
      result = CalculationResult.failureWithI18n(evaluated.error, input);
    }

    // Attach alternative interpretations if available
    result.alternativeLino = alternatives;

    return result;
  }

  /**
   * @param {Expression[]} interpretations
   * @returns {string[] | null}
   */
  static combinedAlternativeLino(interpretations) {
    if (interpretations.length === 0) {
      return null;
    }
    const alternatives = [interpretations[0].toLino()];

    for (const expr of interpretations) {
      const exprAlternatives = expr.alternativeLino();
      if (exprAlternatives !== null) {
        for (const lino of exprAlternatives) {
          Calculator.pushUniqueLino(alternatives, lino);
        }
      } else {
        Calculator.pushUniqueLino(alternatives, expr.toLino());
      }
    }

    return alternatives.length > 1 ? alternatives : null;
  }

  /**
   * @param {string[]} alternatives
   * @param {string} lino
   */
  static pushUniqueLino(alternatives, lino) {
    if (!alternatives.includes(lino)) {
      alternatives.push(lino);
    }
  }

  /**
   * Generates plot data for an integral expression.
   * @param {string} input
   * @returns {PlotData | null}
   */
  generatePlotDataForIntegral(input) {
    // Try to parse and extract the integrand for plotting
    const expr = ok(() => this._parser.parse(input));
    if (expr === null) {
      return null;
    }

    if (expr.tag === 'IndefiniteIntegral') {
      const { integrand, variable } = expr;
      // Generate plot points for the integrand
      const xValues = [];
      const yValues = [];

      // Generate points from -10 to 10 with 200 steps
      const numPoints = 200;
      const xMin = -10.0;
      const xMax = 10.0;
      const step = (xMax - xMin) / numPoints;

      for (let i = 0; i <= numPoints; i++) {
        const x = fma(i, step, xMin);

        // Skip x = 0 for functions like sin(x)/x to avoid division issues
        if (Math.abs(x) < 1e-10) {
          // For sin(x)/x, the limit at x=0 is 1
          xValues.push(x);
          yValues.push(1.0);
          continue;
        }

        // Try to evaluate the integrand at this point
        const yVal = ok(() => this.evaluateAtPoint(integrand, variable, x));
        if (yVal !== null && Number.isFinite(yVal)) {
          xValues.push(x);
          yValues.push(yVal);
        }
      }

      if (xValues.length > 0) {
        return new PlotData({
          xValues,
          yValues,
          label: integrand.toString(),
          xLabel: variable,
          yLabel: `f(${variable})`,
        });
      }
    }

    return null;
  }

  /**
   * Evaluates an expression at a specific point. Throws `Err(CalculatorError)`.
   * @param {Expression} expr
   * @param {string} variable
   * @param {number} value
   * @returns {number}
   */
  evaluateAtPoint(expr, variable, value) {
    const substituted = Calculator.substituteVariable(expr, variable, value);
    const result = this._parser.evaluate(substituted);
    const d = result.asDecimal();
    if (d === null) {
      return err(CalculatorError.eval('Expected numeric result'));
    }
    return d.toF64();
  }

  /**
   * Substitutes a variable with a numeric value in an expression
   * (`src/substitution.rs`).
   * @param {Expression} expr
   * @param {string} variable
   * @param {number} value
   * @returns {Expression}
   */
  static substituteVariable(expr, variable, value) {
    return substituteVariable(expr, variable, value);
  }

  /**
   * Parses an expression without evaluating it. Throws `Err(CalculatorError)`.
   * @param {string} input
   * @returns {Expression}
   */
  parse(input) {
    return this._parser.parse(input);
  }

  /**
   * Evaluates a parsed expression. Throws `Err(CalculatorError)`.
   * @param {Expression} expr
   * @returns {Value}
   */
  evaluate(expr) {
    return this._parser.evaluate(expr);
  }

  /**
   * Returns the underlying `ExpressionParser`.
   *
   * Lets downstream consumers reach the parser's lower-level methods —
   * `evaluateWithSteps`, `evaluateExpr`, `applyBinaryOp`, and friends —
   * without rebuilding the parser state (currency database, current date
   * context, …).
   * @returns {ExpressionParser}
   */
  parser() {
    return this._parser;
  }

  /**
   * Returns the underlying `ExpressionParser` (mutable borrow in Rust).
   * @returns {ExpressionParser}
   */
  parserMut() {
    return this._parser;
  }

  /**
   * Calculates an expression and returns every intermediate artefact.
   *
   * Unlike `calculateInternal`, which returns a UI-shaped `CalculationResult`
   * with display strings only, this method gives a downstream consumer
   * everything it needs to either render the result itself or replay the
   * calculation: the parsed AST, the structured value, the raw step list and
   * the Links Notation interpretation. Throws `Err(CalculatorError)`.
   * @param {string} input
   * @returns {[Expression, Value, string[], string]}
   */
  calculateWithValue(input) {
    input = trim(input);
    if (input === '') {
      return err(CalculatorError.EmptyInput);
    }
    const expr = this._parser.parse(input);
    const lino = expr.toLino();
    const [value, steps] = this._parser.evaluateWithSteps(expr);
    return [expr, value, steps, lino];
  }

  /**
   * Loads a historical exchange rate from .lino format content.
   * Throws `Err(string)`.
   *
   * The .lino format for rates:
   * ```text
   * rate:
   *   from USD
   *   to EUR
   *   value 0.92
   *   date 2026-01-25
   *   source 'frankfurter.dev (ECB)'
   * ```
   * @param {string} content
   */
  loadRateFromLino(content) {
    let fromCurrency = null;
    let toCurrency = null;
    let value = null;
    let date = null;
    let source = null;

    for (const raw of lines(content)) {
      const line = trim(raw);
      if (line === '' || line === 'rate:') {
        continue;
      }

      let rest;
      if ((rest = stripPrefix(line, 'from ')) !== null) {
        fromCurrency = trim(rest).toUpperCase();
      } else if ((rest = stripPrefix(line, 'to ')) !== null) {
        toCurrency = trim(rest).toUpperCase();
      } else if ((rest = stripPrefix(line, 'value ')) !== null) {
        const r = rest;
        value = ok(() => parseF64(trim(r)));
      } else if ((rest = stripPrefix(line, 'date ')) !== null) {
        date = trim(rest);
      } else if ((rest = stripPrefix(line, 'source ')) !== null) {
        // Remove quotes from source
        source = unquoteSource(rest);
      }
    }

    if (fromCurrency === null) return err("Missing 'from' currency");
    if (toCurrency === null) return err("Missing 'to' currency");
    if (value === null) return err("Missing 'value'");
    if (date === null) return err("Missing 'date'");
    const rateSource = source ?? 'unknown';

    // Create ExchangeRateInfo and add to the database
    const rateInfo = ExchangeRateInfo.new(value, rateSource, date);

    this._parser.currencyDbMut().setHistoricalRateWithInfo(fromCurrency, toCurrency, date, rateInfo);
  }

  /**
   * Loads multiple historical exchange rates from a batch of .lino content.
   * Each rate should be separated by double newlines or start with "rate:".
   * @param {string[]} contents
   * @returns {number}
   */
  loadRatesBatch(contents) {
    let loaded = 0;
    for (const content of contents) {
      if (tryResult(() => this.loadRateFromLino(content)).ok) {
        loaded += 1;
      }
      // Silently skip invalid rate files
    }
    return loaded;
  }

  /**
   * Throws `Err(string)` when no rate was loaded.
   * @param {string} content
   * @returns {number}
   */
  loadRatesFromConsolidatedLinoImpl(content) {
    let fromCurrency = null;
    let toCurrency = null;
    let source = null;
    let inDataSection = false;
    let loaded = 0;

    for (const line of lines(content)) {
      const trimmed = trim(line);

      // Skip empty lines
      if (trimmed === '') {
        continue;
      }

      // Handle 'rates:' based on context:
      // - If we haven't parsed from_currency yet, it's a root marker (legacy format), skip it
      // - If we have parsed from_currency, it's the data section marker (new format)
      if (trimmed === 'rates:') {
        if (fromCurrency !== null) {
          // New format: 'rates:' marks the start of data section
          inDataSection = true;
        }
        // Either way, continue to next line (skip the 'rates:' line itself)
        continue;
      }

      // Skip root marker for new format
      if (trimmed === 'conversion:') {
        continue;
      }

      // Check for legacy data section marker
      if (trimmed === 'data:') {
        inDataSection = true;
        continue;
      }

      if (inDataSection) {
        // Parse date and value: "2021-01-25 0.8234"
        const parts = splitWhitespace(trimmed);
        if (parts.length >= 2 && fromCurrency !== null && toCurrency !== null) {
          const date = parts[0];
          const value = ok(() => parseF64(parts[1]));
          if (value !== null) {
            const rateSource = source ?? 'unknown';
            const rateInfo = ExchangeRateInfo.new(value, rateSource, date);
            this._parser.currencyDbMut().setHistoricalRateWithInfo(fromCurrency, toCurrency, date, rateInfo);
            loaded += 1;
          }
        }
      } else {
        // Parse header section
        let rest;
        if ((rest = stripPrefix(trimmed, 'from ')) !== null) {
          fromCurrency = trim(rest).toUpperCase();
        } else if ((rest = stripPrefix(trimmed, 'to ')) !== null) {
          toCurrency = trim(rest).toUpperCase();
        } else if ((rest = stripPrefix(trimmed, 'source ')) !== null) {
          // Remove quotes from source
          source = unquoteSource(rest);
        }
      }
    }

    if (loaded === 0) {
      return err('No rates loaded from consolidated file');
    }
    return loaded;
  }
}
