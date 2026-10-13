// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6a8c5451411daa69605f3ad986af0d7c3c3d2ef4a408ce90410c87667b603095 bytes=33106

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Link Calculator - A grammar-based expression parser and calculator
// |  * (port of `src/lib.rs`).
// |  *
// |  * This library provides a WebAssembly-compatible calculator that supports:
// |  * - `DateTime` parsing and arithmetic
// |  * - Decimal numbers with units
// |  * - Currency conversions with temporal awareness
// |  * - Links notation for expression representation
// |  *
// |  * @example
// |  * import { Calculator } from 'link-calculator';
// |  *
// |  * const calculator = Calculator.new();
// |  * const result = calculator.calculateInternal('2 + 3');
// |  * // result.success === true, result.result === '5'
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import * as clock from './clock.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as cryptoApi from './crypto_api.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as currencyApi from './currency_api.js';
// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from './error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=5eb94310fcdbc51e1a8ccf51962f50bde5c46fc6a127e8659f72d49343862a27
// | import { ExpressionParser } from './grammar/index.js';
use crate::grammar::index::ExpressionParser;
// meta-language:carried JavaScript import_statement (unsupported)
// | import * as plan from './plan.js';
// meta-language:translated JavaScript import_statement items=1 sha256=113157ff84ac83f0e135e5d70161ad73b7c07c492aa4b719f25ef0388191e017
// | import { fma } from './rust/libm.js';
use crate::rust::libm::fma;
// meta-language:translated JavaScript import_statement items=1 sha256=a2ae0815669f7054bba80f964dad6aba3d95e5ac76315e413af8dd9012011704
// | import { parseF64 } from './rust/num.js';
use crate::rust::num::parseF64;
// meta-language:translated JavaScript import_statement items=1 sha256=f993dbd7c68acedcfa1f0082be5d0debedd93a4cebe60ebd33148d5fde32bcef
// | import { lines, splitWhitespace, stripPrefix, trim, trimEndMatches, trimStartMatches } from './rust/str.js';
use crate::rust::str_::{lines, splitWhitespace, stripPrefix, trim, trimEndMatches, trimStartMatches};
// meta-language:translated JavaScript import_statement items=1 sha256=c085ac086553638de9c767811eb37074907b98805020b9552f1c47dee45e2372
// | import { err, ok, tryResult } from './rust/core.js';
use crate::rust::core_::{err, ok, tryResult};
// meta-language:translated JavaScript import_statement items=1 sha256=7bfeca1461172d58640336fa0ece3f12847523eebf8b5571b476207cb699a7d0
// | import { substituteVariable } from './substitution.js';
use crate::substitution::substituteVariable;
// meta-language:translated JavaScript import_statement items=1 sha256=4d9f7af2498a77de5c4678ac3eb54f8e8bc2d6c243079586410ba5dfa8c4d941
// | import { DateTimeResult, ExchangeRateInfo } from './types/index.js';
use crate::types::index::{DateTimeResult, ExchangeRateInfo};
// meta-language:translated JavaScript import_statement items=1 sha256=e4f71c7a9f459a39010e52dbb59561a265b917b9947578cd68a5180006f52401
// | import { generateIssueLink } from './utils.js';
use crate::utils::generateIssueLink;

// meta-language:carried JavaScript export_statement (unsupported)
// | export * as clock from './clock.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as cryptoApi from './crypto_api.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as currencyApi from './currency_api.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as error from './error.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as grammar from './grammar/index.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as lino from './lino/index.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as plan from './plan.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as types from './types/index.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export * as utils from './utils.js';

// meta-language:carried JavaScript export_statement (unsupported)
// | export { CalculationPlan, RateSource } from './plan.js';
// meta-language:carried JavaScript export_statement (unsupported)
// | export { generateIssueLink, truncate } from './utils.js';

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./types/index.js').Expression} Expression */
// | /** @typedef {import('./types/index.js').Value} Value */
// | /** @typedef {import('./error.js').ErrorInfo} ErrorInfo */

// meta-language:translated JavaScript export_statement items=1 sha256=057ebdfc02f1ebfa36ec28413d69987f1a5dd6325d7586262554ccc8d79f57b5
// | /** Package version (matches Cargo.toml version). */
// | export const VERSION = '0.24.0';
pub const VERSION: &str = "0.24.0";

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Data for plotting a function. */
// | export class PlotData {
// |   /**
// |    * @param {{ xValues: number[], yValues: number[], label: string, xLabel: string, yLabel: string }} fields
// |    */
// |   constructor({ xValues, yValues, label, xLabel, yLabel }) {
// |     /** X-axis values. */
// |     this.xValues = xValues;
// |     /** Y-axis values. */
// |     this.yValues = yValues;
// |     /** Label for the plot (e.g., "sin(x)/x"). */
// |     this.label = label;
// |     /** X-axis label. */
// |     this.xLabel = xLabel;
// |     /** Y-axis label. */
// |     this.yLabel = yLabel;
// |   }
// |
// |   /** serde's JSON. */
// |   toJSON() {
// |     return {
// |       x_values: this.xValues,
// |       y_values: this.yValues,
// |       label: this.label,
// |       x_label: this.xLabel,
// |       y_label: this.yLabel,
// |     };
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A single calculation step with i18n support. */
// | export class CalculationStep {
// |   /** @param {{ key: string, params: Map<string, string> | null, text: string }} fields */
// |   constructor({ key, params, text }) {
// |     /** The translation key for this step type. */
// |     this.key = key;
// |     /** Parameters for interpolation in the translated message. */
// |     this.params = params;
// |     /** The raw (English) text for fallback. */
// |     this.text = text;
// |   }
// |
// |   /**
// |    * Creates a new step with a translation key, params, and fallback text.
// |    * @param {string} key
// |    * @param {Map<string, string> | null} params
// |    * @param {string} text
// |    */
// |   static new(key, params, text) {
// |     return new CalculationStep({ key, params, text });
// |   }
// |
// |   /**
// |    * Creates a simple step with just text (no translation key).
// |    * @param {string} text
// |    */
// |   static textOnly(text) {
// |     return new CalculationStep({ key: '', params: null, text });
// |   }
// |
// |   /** serde's JSON (`params` skipped when `None`). */
// |   toJSON() {
// |     /** @type {Record<string, unknown>} */
// |     const json = { key: this.key };
// |     if (this.params !== null) {
// |       json.params = Object.fromEntries(this.params);
// |     }
// |     json.text = this.text;
// |     return json;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Repeating decimal notation formats. */
// | export class RepeatingDecimalFormats {
// |   /**
// |    * @param {{ vinculum: string, parenthesis: string, ellipsis: string, latex: string, fraction: string }} fields
// |    */
// |   constructor({ vinculum, parenthesis, ellipsis, latex, fraction }) {
// |     /** Vinculum notation with overline: 0.3̅ */
// |     this.vinculum = vinculum;
// |     /** Parenthesis notation: 0.(3) */
// |     this.parenthesis = parenthesis;
// |     /** Ellipsis notation: 0.333... */
// |     this.ellipsis = ellipsis;
// |     /** LaTeX notation: 0.\overline{3} */
// |     this.latex = latex;
// |     /** Fraction representation: 1/3 */
// |     this.fraction = fraction;
// |   }
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * @typedef {{ result: string, linoInterpretation: string, alternativeLino: string[] | null,
// |  *   steps: string[], stepsI18n: CalculationStep[] | null, success: boolean, error: string | null,
// |  *   errorInfo: ErrorInfo | null, issueLink: string | null, latexInput: string | null,
// |  *   latexResult: string | null, isSymbolic: boolean | null, plotData: PlotData | null,
// |  *   repeatingDecimal: RepeatingDecimalFormats | null, fraction: string | null,
// |  *   isLiveTime: boolean | null, datetimeResult: DateTimeResult | null }} CalculationResultFields
// |  */

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** The fields every constructor of `CalculationResult` leaves at their defaults. */
// | const RESULT_DEFAULTS = Object.freeze({
// |   alternativeLino: null,
// |   stepsI18n: null,
// |   success: true,
// |   error: null,
// |   errorInfo: null,
// |   issueLink: null,
// |   latexInput: null,
// |   latexResult: null,
// |   isSymbolic: null,
// |   plotData: null,
// |   repeatingDecimal: null,
// |   fraction: null,
// |   isLiveTime: null,
// |   datetimeResult: null,
// | });

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Result of a calculation operation. */
// | export class CalculationResult {
// |   /** @param {CalculationResultFields} fields */
// |   constructor(fields) {
// |     /** The computed value as a string. */
// |     this.result = fields.result;
// |     /** The input interpreted in links notation format. */
// |     this.linoInterpretation = fields.linoInterpretation;
// |     /**
// |      * Alternative links notation interpretations the user can switch between.
// |      * The first element is always the currently selected (default) interpretation.
// |      */
// |     this.alternativeLino = fields.alternativeLino;
// |     /** Step-by-step explanation of the calculation (raw text for backwards compatibility). */
// |     this.steps = fields.steps;
// |     /** Step-by-step explanation with i18n support. */
// |     this.stepsI18n = fields.stepsI18n;
// |     /** Whether the calculation was successful. */
// |     this.success = fields.success;
// |     /** Error message if calculation failed (raw text for backwards compatibility). */
// |     this.error = fields.error;
// |     /** Error information for i18n support. */
// |     this.errorInfo = fields.errorInfo;
// |     /** Link to create an issue for unrecognized input. */
// |     this.issueLink = fields.issueLink;
// |     /** LaTeX representation of the input (for rendering mathematical formulas). */
// |     this.latexInput = fields.latexInput;
// |     /** LaTeX representation of the result (for rendering mathematical formulas). */
// |     this.latexResult = fields.latexResult;
// |     /** Whether this is a symbolic result (e.g., indefinite integral). */
// |     this.isSymbolic = fields.isSymbolic;
// |     /** Plot data points for graphing (x, y pairs). */
// |     this.plotData = fields.plotData;
// |     /** Repeating decimal notations (if the result is a repeating decimal). */
// |     this.repeatingDecimal = fields.repeatingDecimal;
// |     /** Fraction representation of the result (if applicable). */
// |     this.fraction = fields.fraction;
// |     /**
// |      * Whether the result represents a live (auto-updating) time expression.
// |      * When `true`, the frontend should periodically re-calculate the expression.
// |      */
// |     this.isLiveTime = fields.isLiveTime;
// |     /** Structured datetime metadata for browser-local and UTC conversion display. */
// |     this.datetimeResult = fields.datetimeResult;
// |   }
// |
// |   /**
// |    * Creates a successful calculation result.
// |    * @param {string} result
// |    * @param {string} lino
// |    * @param {string[]} steps
// |    */
// |   static success(result, lino, steps) {
// |     return new CalculationResult({ ...RESULT_DEFAULTS, result, linoInterpretation: lino, steps });
// |   }
// |
// |   /**
// |    * Creates a successful calculation result with rational value information.
// |    * @param {Value} value
// |    * @param {string} lino
// |    * @param {string[]} steps
// |    */
// |   static successWithValue(value, lino, steps) {
// |     const result = value.toDisplayString();
// |
// |     // Extract repeating decimal and fraction info if available
// |     let repeatingDecimal = null;
// |     let fraction = null;
// |     const rational = value.asRational();
// |     if (rational !== null) {
// |       fraction = !rational.isInteger() ? rational.toFractionString() : null;
// |
// |       const rd = rational.toRepeatingDecimalNotation();
// |       repeatingDecimal = rd === null
// |         ? null
// |         : new RepeatingDecimalFormats({
// |           vinculum: rd.toVinculumNotation(),
// |           parenthesis: rd.toParenthesisNotation(),
// |           ellipsis: rd.toEllipsisNotation(),
// |           latex: rd.toLatex(),
// |           fraction: rational.toFractionString(),
// |         });
// |     }
// |     const datetimeResult = value.kind.tag === 'DateTime'
// |       ? DateTimeResult.fromDatetime(value.kind._0)
// |       : null;
// |
// |     return new CalculationResult({
// |       ...RESULT_DEFAULTS,
// |       result,
// |       linoInterpretation: lino,
// |       steps,
// |       repeatingDecimal,
// |       fraction,
// |       datetimeResult,
// |     });
// |   }
// |
// |   /**
// |    * Creates a successful calculation result with i18n step info.
// |    * @param {string} result
// |    * @param {string} lino
// |    * @param {string[]} steps
// |    * @param {CalculationStep[]} stepsI18n
// |    */
// |   static successWithI18n(result, lino, steps, stepsI18n) {
// |     return new CalculationResult({ ...RESULT_DEFAULTS, result, linoInterpretation: lino, steps, stepsI18n });
// |   }
// |
// |   /**
// |    * Creates a successful calculation result with LaTeX formatting.
// |    * @param {string} result
// |    * @param {string} lino
// |    * @param {string[]} steps
// |    * @param {string | null} latexInput
// |    * @param {string | null} latexResult
// |    */
// |   static successWithLatex(result, lino, steps, latexInput, latexResult) {
// |     return new CalculationResult({
// |       ...RESULT_DEFAULTS,
// |       result,
// |       linoInterpretation: lino,
// |       steps,
// |       latexInput,
// |       latexResult,
// |     });
// |   }
// |
// |   /**
// |    * Creates a symbolic result (e.g., for indefinite integrals).
// |    * @param {string} expression
// |    * @param {string} result
// |    * @param {string} latexInput
// |    * @param {string} latexResult
// |    * @param {PlotData | null} plotData
// |    */
// |   static symbolic(expression, result, latexInput, latexResult, plotData) {
// |     return new CalculationResult({
// |       ...RESULT_DEFAULTS,
// |       result,
// |       linoInterpretation: expression,
// |       steps: [`Input: ${expression}`, 'Computed symbolic result'],
// |       latexInput,
// |       latexResult,
// |       isSymbolic: true,
// |       plotData,
// |     });
// |   }
// |
// |   /**
// |    * Creates a failed calculation result.
// |    * @param {string} error
// |    * @param {string} input
// |    */
// |   static failure(error, input) {
// |     const issueLink = generateIssueLink(input, error);
// |     return new CalculationResult({
// |       ...RESULT_DEFAULTS,
// |       result: '',
// |       linoInterpretation: '',
// |       steps: [],
// |       success: false,
// |       error,
// |       issueLink,
// |     });
// |   }
// |
// |   /**
// |    * Creates a failed calculation result with i18n error info.
// |    * @param {CalculatorError} error
// |    * @param {string} input
// |    */
// |   static failureWithI18n(error, input) {
// |     const errorString = error.toString();
// |     const issueLink = generateIssueLink(input, errorString);
// |     return new CalculationResult({
// |       ...RESULT_DEFAULTS,
// |       result: '',
// |       linoInterpretation: '',
// |       steps: [],
// |       success: false,
// |       error: errorString,
// |       errorInfo: error.toErrorInfo(),
// |       issueLink,
// |     });
// |   }
// |
// |   /**
// |    * serde's JSON (field names and order, `skip_serializing_if = "Option::is_none"`).
// |    * @returns {Record<string, unknown>}
// |    */
// |   toJSON() {
// |     /** @type {Record<string, unknown>} */
// |     const json = {
// |       result: this.result,
// |       lino_interpretation: this.linoInterpretation,
// |     };
// |     /** @param {string} key @param {unknown} value */
// |     const optional = (key, value) => {
// |       if (value !== null) {
// |         json[key] = value;
// |       }
// |     };
// |     optional('alternative_lino', this.alternativeLino);
// |     json.steps = this.steps;
// |     optional('steps_i18n', this.stepsI18n);
// |     json.success = this.success;
// |     json.error = this.error;
// |     optional('error_info', this.errorInfo);
// |     json.issue_link = this.issueLink;
// |     optional('latex_input', this.latexInput);
// |     optional('latex_result', this.latexResult);
// |     optional('is_symbolic', this.isSymbolic);
// |     optional('plot_data', this.plotData);
// |     optional('repeating_decimal', this.repeatingDecimal);
// |     optional('fraction', this.fraction);
// |     optional('is_live_time', this.isLiveTime);
// |     optional('datetime_result', this.datetimeResult);
// |     return json;
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::from_str::<HashMap<String, f64>>(json)`: an object of numbers,
// |  * or `null` when the text is not one.
// |  * @param {string} json
// |  * @returns {Map<string, number> | null}
// |  */
// | function parseRatesJson(json) {
// |   let parsed;
// |   try {
// |     parsed = JSON.parse(json);
// |   } catch {
// |     return null;
// |   }
// |   if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
// |     return null;
// |   }
// |   const rates = new Map();
// |   for (const [key, value] of Object.entries(parsed)) {
// |     if (typeof value !== 'number' || !Number.isFinite(value)) {
// |       return null;
// |     }
// |     rates.set(key, value);
// |   }
// |   return rates;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `serde_json::to_string(value)`.
// |  * @param {unknown} value
// |  */
// | function toJsonString(value) {
// |   return JSON.stringify(value);
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * Removes the quotes around a `source` value.
// |  * @param {string} rest
// |  */
// | function unquoteSource(rest) {
// |   let src = trim(rest);
// |   src = trimEndMatches(trimStartMatches(src, "'"), "'");
// |   src = trimEndMatches(trimStartMatches(src, '"'), '"');
// |   return src;
// | }

