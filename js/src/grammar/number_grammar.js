/**
 * Grammar for parsing numbers with optional units.
 */

import * as cryptoApi from '../crypto_api.js';
import { CalculatorError } from '../error.js';
import { Err, eq, err, ok } from '../rust/core.js';
import { isAsciiDigit, isAsciiUppercase, isLowercase, isUppercase, stripPrefix, trim, utf8Len } from '../rust/str.js';
import {
  CurrencyDatabase, DataSizeUnit, Decimal, Dimension, DurationUnit, MassUnit, PhysicalUnit, Unit,
} from '../types/index.js';

/**
 * Grammar for parsing numbers with optional units.
 */
export class NumberGrammar {
  /**
   * Creates a new number grammar.
   * @returns {NumberGrammar}
   */
  static new() {
    return new NumberGrammar();
  }

  /** @returns {NumberGrammar} */
  static default() {
    return new NumberGrammar();
  }

  /**
   * Parses a number string into a Decimal. Throws `Err(CalculatorError)`.
   * @param {string} s
   * @returns {Decimal}
   */
  parseNumber(s) {
    s = trim(s);

    // Handle negative numbers
    const stripped = stripPrefix(s, '-');
    const isNegative = stripped !== null;
    if (stripped !== null) {
      s = trim(stripped);
    }

    let decimal;
    try {
      decimal = s.includes('e') || s.includes('E') ? Decimal.fromScientific(s) : Decimal.fromStr(s);
    } catch (e) {
      if (e instanceof Err) {
        return err(CalculatorError.parse(`Invalid number: ${s}`));
      }
      throw e;
    }

    return isNegative ? decimal.neg() : decimal;
  }

  /**
   * Returns the decimal multiplier for an SI-style numeric suffix.
   *
   * This is used for compact number notation such as `19k RUB` and
   * `19к рублей`, where the suffix is attached to the number and the unit
   * follows as the next token. SI suffixes are case-sensitive where case
   * distinguishes multipliers, so `m` is milli and `M` is mega. `K`/`К`
   * are accepted as common user spellings of kilo.
   * @internal
   * @param {string} s
   * @returns {Decimal | null}
   */
  static siSuffixMultiplier(s) {
    let multiplier;
    switch (s) {
      // Submultiples
      case 'r': multiplier = '0.000000000000000000000000001'; break;
      case 'y': multiplier = '0.000000000000000000000001'; break;
      case 'z': multiplier = '0.000000000000000000001'; break;
      case 'a': multiplier = '0.000000000000000001'; break;
      case 'f': multiplier = '0.000000000000001'; break;
      case 'p': multiplier = '0.000000000001'; break;
      case 'n': multiplier = '0.000000001'; break;
      case 'µ': case 'μ': case 'u': multiplier = '0.000001'; break;
      case 'm': multiplier = '0.001'; break;
      case 'c': multiplier = '0.01'; break;
      case 'd': multiplier = '0.1'; break;
      // Multiples
      case 'da': multiplier = '10'; break;
      case 'h': multiplier = '100'; break;
      case 'k': case 'K': case 'к': case 'К': multiplier = '1000'; break;
      case 'M': case 'М': multiplier = '1000000'; break;
      case 'G': multiplier = '1000000000'; break;
      case 'T': multiplier = '1000000000000'; break;
      case 'P': multiplier = '1000000000000000'; break;
      case 'E': multiplier = '1000000000000000000'; break;
      case 'Z': multiplier = '1000000000000000000000'; break;
      case 'Y': multiplier = '1000000000000000000000000'; break;
      case 'R': multiplier = '1000000000000000000000000000'; break;
      default: return null;
    }

    return ok(() => Decimal.fromStr(multiplier));
  }

  /**
   * Parses a number with an optional unit. Throws `Err(CalculatorError)`.
   * @param {string} numberStr
   * @param {string | null} unitStr
   * @returns {[Decimal, Unit]}
   */
  parseNumberWithUnit(numberStr, unitStr) {
    const number = this.parseNumber(numberStr);

    const unit = unitStr !== null ? this.parseUnit(unitStr) : Unit.None;

    return [number, unit];
  }

  /**
   * Parses a unit string. Throws `Err(CalculatorError)`.
   * @param {string} s
   * @returns {Unit}
   */
  parseUnit(s) {
    const [unit, _alternatives] = this.parseUnitWithAlternatives(s);
    return unit;
  }

  /**
   * Parses a unit string, returning the primary unit and any alternative interpretations.
   *
   * Some identifiers are ambiguous — for example, "ton" can mean either a metric ton
   * (mass unit, 1000 kg) or Toncoin (TON cryptocurrency). This method returns all
   * valid interpretations so the caller can surface them as alternatives to the user.
   * @param {string} s
   * @returns {[Unit, Unit[]]}
   */
  parseUnitWithAlternatives(s) {
    s = trim(s);
    /** @type {Unit[]} */
    const alternatives = [];

    // Try to parse as data size unit first (before currency, to avoid conflicts)
    const dataSize = DataSizeUnit.parse(s);
    if (dataSize !== null) {
      return [Unit.DataSize(dataSize), alternatives];
    }

    // Try to parse as mass unit (before currency, to avoid "t" being treated as currency)
    const mass = MassUnit.parse(s);
    if (mass !== null) {
      const primary = Unit.Mass(mass);

      // Check if this identifier also matches a well-known cryptocurrency or fiat code.
      // Only flag ambiguity for known crypto tickers (e.g., "ton" → TON/Toncoin),
      // not for the generic catch-all that accepts any 2-5 letter string.
      const currencyCode = CurrencyDatabase.parseCurrency(s);
      if (currencyCode !== null) {
        if (NumberGrammar.isWellKnownCurrency(currencyCode)) {
          const currencyUnit = Unit.currency(currencyCode);
          if (!eq(currencyUnit, primary)) {
            alternatives.push(currencyUnit);
          }
        }
      }

      return [primary, alternatives];
    }

    // Physical units from data/units (issue #235). An uppercase symbol such
    // as `W`, `H` or `S` is watt, henry or siemens rather than the week,
    // hour or second the case-insensitive duration names would read.
    const physical = PhysicalUnit.parse(s);
    if (physical !== null) {
      if ([...s].some((c) => isUppercase(c)) && !eq(physical.dimension(), Dimension.base('time'))) {
        return NumberGrammar.physicalWithCurrency(s, physical.clone());
      }
    }

    // Try to parse as duration unit (before currency, to avoid e.g. "h" being treated as a currency)
    const dur = DurationUnit.parse(s);
    if (dur !== null) {
      const duration = Unit.Duration(dur);
      // "sek" is German for seconds but "SEK" is the Swedish krona (issue #224):
      // an uppercase ISO code means the currency, otherwise keep it as an alternative.
      const currencyCode = CurrencyDatabase.parseCurrency(s);
      if (currencyCode !== null) {
        if (NumberGrammar.isWellKnownCurrency(currencyCode)) {
          const currency = Unit.currency(currencyCode);
          if (NumberGrammar.isUppercaseCurrencyCode(s)) {
            alternatives.push(duration);
            return [currency, alternatives];
          }
          alternatives.push(currency);
        }
      }
      return [duration, alternatives];
    }

    if (physical !== null) {
      return NumberGrammar.physicalWithCurrency(s, physical);
    }

    // Try to parse as cryptocurrency or fiat currency alias
    const currencyCode = CurrencyDatabase.parseCurrency(s);
    if (currencyCode !== null) {
      const primary = Unit.currency(currencyCode);

      // Check if a case-insensitive mass unit match exists (ambiguity detection)
      const lowerMass = MassUnit.parse(s.toLowerCase());
      if (lowerMass !== null) {
        const massUnit = Unit.Mass(lowerMass);
        if (!eq(massUnit, primary)) {
          alternatives.push(massUnit);
        }
      }

      return [primary, alternatives];
    }

    // If nothing matches, treat as custom unit
    if (s === '') {
      return [Unit.None, alternatives];
    }
    return [Unit.Custom(s), alternatives];
  }

  /**
   * A physical unit that may also spell a well-known currency: the
   * uppercase spelling is the currency, any other spelling the unit.
   * @param {string} s
   * @param {PhysicalUnit} unit
   * @returns {[Unit, Unit[]]}
   */
  static physicalWithCurrency(s, unit) {
    const physical = unit.intoUnit();
    const parsed = CurrencyDatabase.parseCurrency(s);
    const code = parsed !== null && NumberGrammar.isWellKnownCurrency(parsed) ? parsed : null;
    if (code === null) {
      return [physical, []];
    }
    const currency = Unit.currency(code);
    if ([...s].every((ch) => !isLowercase(ch))) {
      return [currency, [physical]];
    }
    return [physical, [currency]];
  }

  /**
   * Returns `true` for an all-uppercase three-letter well-known currency code
   * such as `SEK`, which then wins over a same-spelled unit like German `sek`.
   * @internal
   * @param {string} s
   * @returns {boolean}
   */
  static isUppercaseCurrencyCode(s) {
    return utf8Len(s) === 3 && [...s].every((c) => isAsciiUppercase(c)) && NumberGrammar.isWellKnownCurrency(s);
  }

  /**
   * Checks if a currency code is a well-known fiat or crypto currency.
   *
   * Returns `true` for explicitly listed currencies (ISO 4217 fiat codes
   * and known crypto tickers like TON, BTC, ETH). Returns `false` for codes
   * that only match the generic "any 2-5 letter string" catch-all pattern
   * in `CurrencyDatabase::parse_currency`.
   * @param {string} code
   * @returns {boolean}
   */
  static isWellKnownCurrency(code) {
    const upper = code.toUpperCase();

    // Check if it's a known crypto ticker
    if (cryptoApi.coingeckoId(upper) !== null) {
      return true;
    }

    // Check well-known fiat codes (ISO 4217 major currencies)
    return WELL_KNOWN_FIAT.has(upper);
  }

  /**
   * Checks if a string looks like a number.
   * @param {string} s
   * @returns {boolean}
   */
  static looksLikeNumber(s) {
    s = trim(s);
    if (s === '') {
      return false;
    }

    s = stripPrefix(s, '-') ?? s;
    s = stripPrefix(s, '+') ?? s;

    // Must start with a digit or decimal point
    const first = [...s][0];
    return first !== undefined && (isAsciiDigit(first) || first === '.');
  }
}

/** The fiat codes of the `matches!` in `is_well_known_currency`. */
const WELL_KNOWN_FIAT = new Set([
  'USD', 'EUR', 'GBP', 'JPY', 'CHF', 'CNY', 'RUB', 'INR', 'KRW', 'CLF', 'AUD', 'CAD', 'NZD', 'SEK',
  'NOK', 'DKK', 'SGD', 'HKD', 'MXN', 'BRL', 'ZAR', 'TRY', 'PLN', 'CZK', 'HUF', 'RON', 'BGN', 'ISK',
  'IDR', 'MYR', 'PHP', 'THB', 'KES',
]);
