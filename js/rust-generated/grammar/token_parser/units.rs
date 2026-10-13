// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=922b380bd9e136a720755633f0e8f36bba0b51ca3907a087c66a8a9ea156479b bytes=3775

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Conversion target units (port of `src/grammar/token_parser/units.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=234ac7040c5dcb58f4b62f9ddbe30f7246f7dc5022ce4a4eb5010dd9dd621c8e
// | import { eq, err } from '../../rust/core.js';
use crate::rust::core_::{eq, err};
// meta-language:translated JavaScript import_statement items=1 sha256=9e83504ba95da6c34de6bc560ec941b463401ec2b0afedfc2e5ac3bd5e26b0a4
// | import {
// |   CurrencyDatabase, DataSizeUnit, DateTime, DurationUnit, Expression, MassUnit, Unit,
// | } from '../../types/index.js';
use crate::types::index::{CurrencyDatabase, DataSizeUnit, DateTime, DurationUnit, Expression, MassUnit, Unit};
// meta-language:translated JavaScript import_statement items=1 sha256=9c729167abcd53b361ad55e2ea7f88090270e1ebd62749f3f06a2b472ad02fe2
// | import { NumberGrammar } from '../number_grammar.js';
use crate::grammar::number_grammar::NumberGrammar;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../token_parser.js').TokenParser} TokenParser */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses a unit name after the `as`, `in`, or `to` keyword.
// |  * Throws `Err(CalculatorError)`.
// |  * @param {TokenParser} self
// |  * @returns {Unit}
// |  */
// | export function parseUnitForConversion(self) {
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Identifier') {
// |     return err(CalculatorError.parse(
// |       "Expected a unit name after 'as'/'in'/'to' (e.g., 'MB', 'kg', 'USD', 'dollars')",
// |     ));
// |   }
// |
// |   const unitStr = kind._0;
// |   if (isNumberConversionTarget(unitStr)) {
// |     self.advance();
// |     return Unit.None;
// |   }
// |   const physical = self.parsePhysicalConversionTarget();
// |   if (physical !== null) {
// |     return physical;
// |   }
// |   self.advance();
// |
// |   let dataSize = DataSizeUnit.parse(unitStr);
// |   if (dataSize !== null) {
// |     return Unit.DataSize(dataSize);
// |   }
// |   let mass = MassUnit.parse(unitStr);
// |   if (mass !== null) {
// |     return Unit.Mass(mass);
// |   }
// |
// |   const lower = unitStr.toLowerCase();
// |   dataSize = DataSizeUnit.parse(lower);
// |   if (dataSize !== null) {
// |     return Unit.DataSize(dataSize);
// |   }
// |   mass = MassUnit.parse(lower);
// |   if (mass !== null) {
// |     return Unit.Mass(mass);
// |   }
// |
// |   // An uppercase ISO code such as "SEK" is a currency, not German "sek" (issue #224).
// |   if (NumberGrammar.isUppercaseCurrencyCode(unitStr)) {
// |     return Unit.currency(unitStr);
// |   }
// |
// |   const duration = DurationUnit.parse(unitStr);
// |   if (duration !== null) {
// |     return Unit.Duration(duration);
// |   }
// |
// |   // Timezone comes before currency because currency code parsing accepts
// |   // any 2-5 letter code.
// |   if (DateTime.parseTzAbbreviation(unitStr) !== null) {
// |     return Unit.Timezone(unitStr.toUpperCase());
// |   }
// |
// |   const currencyCode = CurrencyDatabase.parseCurrency(unitStr);
// |   if (currencyCode !== null) {
// |     return Unit.currency(currencyCode);
// |   }
// |
// |   return err(CalculatorError.parse(
// |     `Unknown unit '${unitStr}'. Supported conversions: `
// |     + 'data sizes (B, KB, MB, GB, KiB, MiB, GiB, ...), '
// |     + 'mass (g, kg, tons, lb, oz), '
// |     + 'currencies (USD, EUR, GBP, TON, BTC, ETH, ...) and natural language '
// |     + 'aliases (dollars, euros, bitcoin, toncoin, ...), '
// |     + 'timezones (UTC, GMT, EST, MSK, JST, ...), '
// |     + 'time durations (ms, seconds, minutes, hours, days, weeks, months, years), '
// |     + 'and number/unitless.',
// |   ));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Resolves unit ambiguity when a conversion target provides context.
// |  * @param {Expression} expr
// |  * @param {Unit} targetUnit
// |  * @returns {Expression}
// |  */
// | export function resolveUnitAmbiguityForConversion(expr, targetUnit) {
// |   if (expr.tag === 'Number') {
// |     const { value, unit, alternativeUnits } = expr;
// |     if (alternativeUnits.length === 0 || unit.isSameCategory(targetUnit)) {
// |       return expr;
// |     }
// |
// |     for (const alt of alternativeUnits) {
// |       if (alt.isSameCategory(targetUnit)) {
// |         const newAlternatives = alternativeUnits.filter((u) => !eq(u, alt));
// |         newAlternatives.push(unit);
// |         return Expression.numberWithUnitAlternatives(value, alt, newAlternatives);
// |       }
// |     }
// |   }
// |   return expr;
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const NUMBER_CONVERSION_TARGETS = new Set([
// |   'number', 'numbers', 'unitless', 'scalar', 'число', 'числа', 'числом', 'zahl', 'nombre',
// |   '数字', '数', 'संख्या', 'رقم', 'عدد',
// | ]);

// meta-language:carried JavaScript function_declaration (type)
// | /** @param {string} unitStr */
// | function isNumberConversionTarget(unitStr) {
// |   return NUMBER_CONVERSION_TARGETS.has(unitStr.toLowerCase());
// | }
