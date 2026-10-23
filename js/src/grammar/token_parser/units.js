/**
 * Conversion target units (port of `src/grammar/token_parser/units.rs`).
 */

import { CalculatorError } from '../../error.js';
import { eq, err } from '../../rust/core.js';
import {
  CurrencyDatabase, DataSizeUnit, DateTime, DurationUnit, Expression, MassUnit, Unit,
} from '../../types/index.js';
import { NumberGrammar } from '../number_grammar.js';

/** @typedef {import('../token_parser.js').TokenParser} TokenParser */

/**
 * Parses a unit name after the `as`, `in`, or `to` keyword.
 * Throws `Err(CalculatorError)`.
 * @param {TokenParser} self
 * @returns {Unit}
 */
export function parseUnitForConversion(self) {
  const kind = self.currentKind();
  if (kind === null || kind.tag !== 'Identifier') {
    return err(CalculatorError.parse(
      "Expected a unit name after 'as'/'in'/'to' (e.g., 'MB', 'kg', 'USD', 'dollars')",
    ));
  }

  const unitStr = kind._0;
  if (isNumberConversionTarget(unitStr)) {
    self.advance();
    return Unit.None;
  }
  const physical = self.parsePhysicalConversionTarget();
  if (physical !== null) {
    return physical;
  }
  self.advance();

  let dataSize = DataSizeUnit.parse(unitStr);
  if (dataSize !== null) {
    return Unit.DataSize(dataSize);
  }
  let mass = MassUnit.parse(unitStr);
  if (mass !== null) {
    return Unit.Mass(mass);
  }

  const lower = unitStr.toLowerCase();
  dataSize = DataSizeUnit.parse(lower);
  if (dataSize !== null) {
    return Unit.DataSize(dataSize);
  }
  mass = MassUnit.parse(lower);
  if (mass !== null) {
    return Unit.Mass(mass);
  }

  // An uppercase ISO code such as "SEK" is a currency, not German "sek" (issue #224).
  if (NumberGrammar.isUppercaseCurrencyCode(unitStr)) {
    return Unit.currency(unitStr);
  }

  const duration = DurationUnit.parse(unitStr);
  if (duration !== null) {
    return Unit.Duration(duration);
  }

  // Timezone comes before currency because currency code parsing accepts
  // any 2-5 letter code.
  if (DateTime.parseTzAbbreviation(unitStr) !== null) {
    return Unit.Timezone(unitStr.toUpperCase());
  }

  const currencyCode = CurrencyDatabase.parseCurrency(unitStr);
  if (currencyCode !== null) {
    return Unit.currency(currencyCode);
  }

  return err(CalculatorError.parse(
    `Unknown unit '${unitStr}'. Supported conversions: `
    + 'data sizes (B, KB, MB, GB, KiB, MiB, GiB, ...), '
    + 'mass (g, kg, tons, lb, oz), '
    + 'currencies (USD, EUR, GBP, TON, BTC, ETH, ...) and natural language '
    + 'aliases (dollars, euros, bitcoin, toncoin, ...), '
    + 'timezones (UTC, GMT, EST, MSK, JST, ...), '
    + 'time durations (ms, seconds, minutes, hours, days, weeks, months, years), '
    + 'and number/unitless.',
  ));
}

/**
 * Resolves unit ambiguity when a conversion target provides context.
 * @param {Expression} expr
 * @param {Unit} targetUnit
 * @returns {Expression}
 */
export function resolveUnitAmbiguityForConversion(expr, targetUnit) {
  if (expr.tag === 'Number') {
    const { value, unit, alternativeUnits } = expr;
    if (alternativeUnits.length === 0 || unit.isSameCategory(targetUnit)) {
      return expr;
    }

    for (const alt of alternativeUnits) {
      if (alt.isSameCategory(targetUnit)) {
        const newAlternatives = alternativeUnits.filter((u) => !eq(u, alt));
        newAlternatives.push(unit);
        return Expression.numberWithUnitAlternatives(value, alt, newAlternatives);
      }
    }
  }
  return expr;
}

const NUMBER_CONVERSION_TARGETS = new Set([
  'number', 'numbers', 'unitless', 'scalar', 'число', 'числа', 'числом', 'zahl', 'nombre',
  '数字', '数', 'संख्या', 'رقم', 'عدد',
]);

/** @param {string} unitStr */
function isNumberConversionTarget(unitStr) {
  return NUMBER_CONVERSION_TARGETS.has(unitStr.toLowerCase());
}
