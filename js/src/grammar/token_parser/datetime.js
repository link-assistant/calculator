/**
 * Datetime recognition over tokens (part of the port of
 * `src/grammar/token_parser.rs`, split out to keep files under 1000 lines).
 */

import { CalculatorError } from '../../error.js';
import { err, ok } from '../../rust/core.js';
import { isAsciiDigit, replace } from '../../rust/str.js';
import { DateTime, Expression } from '../../types/index.js';
/** @typedef {import('../token_parser.js').TokenParser} TokenParser */
/** @typedef {import('../lexer.js').TokenKind} TokenKind */

/**
 * Whether `s` is an English ordinal suffix (st, nd, rd, th).
 * @param {string} s
 */
export function isOrdinalSuffix(s) {
  const lower = s.toLowerCase();
  return lower === 'st' || lower === 'nd' || lower === 'rd' || lower === 'th';
}

/** @param {string} s */
function allAsciiDigits(s) {
  return [...s].every(isAsciiDigit);
}

/**
 * `[x, y, z].join(" ").replace(" , ", ", ").replace(" : ", ":")`.
 * @param {string[]} parts
 */
function joinDatetimeParts(parts) {
  return replace(replace(parts.join(' '), ' , ', ', '), ' : ', ':');
}

/**
 * Tries to parse the remaining tokens after "until" as a datetime expression.
 * Handles cases like "until 11:59pm EST January 26th" where the datetime
 * starts with a number rather than a month name.
 * @returns {Expression}
 * @param {TokenParser} self
 */
export function tryParseUntilTarget(self) {
  /** @type {string[]} */
  const parts = [];

  while (!self.isAtEnd()) {
    const kind = /** @type {TokenKind} */ (self.currentKind());
    if (kind.tag === 'Number' || kind.tag === 'DateLiteral') {
      parts.push(kind._0);
      self.advance();
    } else if (kind.tag === 'Identifier') {
      const id = kind._0;
      if (isOrdinalSuffix(id) && parts.length > 0 && allAsciiDigits(parts[parts.length - 1])) {
        parts[parts.length - 1] += id;
        self.advance();
        continue;
      }
      parts.push(id);
      self.advance();
    } else if (kind.tag === 'Of') {
      parts.push('of');
      self.advance();
    } else if (kind.tag === 'Comma') {
      parts.push(',');
      self.advance();
    } else if (kind.tag === 'Colon') {
      parts.push(':');
      self.advance();
    } else {
      break;
    }
  }

  if (parts.length === 0) {
    return err(CalculatorError.parse(
      'until requires a datetime expression',
    ));
  }

  const datetimeStr = joinDatetimeParts(parts);
  return Expression.DateTime(DateTime.parse(datetimeStr));
}

/**
 * Tries to parse a time/datetime expression that starts with a number followed by a colon,
 * e.g. "11:59pm EST on Monday, January 26th".
 * The `hour_str` is the number already consumed, and the current position is at the Colon.
 * @param {string} hourStr
 * @returns {Expression}
 * @param {TokenParser} self
 */
export function tryParseTimeStartingWithNumber(self, hourStr) {
  const parts = [hourStr];

  while (!self.isAtEnd()) {
    const kind = /** @type {TokenKind} */ (self.currentKind());
    if (kind.tag === 'Number' || kind.tag === 'DateLiteral') {
      parts.push(kind._0);
      self.advance();
    } else if (kind.tag === 'Identifier') {
      const id = kind._0;
      // Attach ordinal suffixes directly to preceding number
      if (isOrdinalSuffix(id) && parts.length > 0 && allAsciiDigits(parts[parts.length - 1])) {
        parts[parts.length - 1] += id;
        self.advance();
        continue;
      }
      parts.push(id);
      self.advance();
    } else if (kind.tag === 'Comma') {
      parts.push(',');
      self.advance();
    } else if (kind.tag === 'Colon') {
      parts.push(':');
      self.advance();
    } else if (kind.tag === 'At') {
      const savePos = self.pos;
      self.advance();
      const tz = self.currentKind();
      if (tz !== null && tz.tag === 'Identifier' && DateTime.parseTzAbbreviation(tz._0) !== null) {
        parts.push(tz._0);
        self.advance();
        continue;
      }
      self.pos = savePos;
      break;
    } else {
      break;
    }
  }

  const datetimeStr = joinDatetimeParts(parts);
  return Expression.DateTime(DateTime.parse(datetimeStr));
}

/**
 * Tries to parse "N AM/PM [TZ]" as a time expression.
 *
 * Called when the parser has consumed a number token and sees AM/PM next.
 * Handles patterns like "6 PM", "6 PM GMT", "6 PM MSK".
 * The timezone identifier is only consumed if it is a recognized timezone abbreviation.
 * @param {string} hourStr
 * @returns {Expression}
 * @param {TokenParser} self
 */
export function tryParseTimeWithAmpm(self, hourStr) {
  // Consume the AM/PM identifier
  const kind = self.currentKind();
  if (kind === null || kind.tag !== 'Identifier') {
    return err(CalculatorError.parse('Expected AM/PM'));
  }
  const ampm = kind._0;
  self.advance();

  // Check if the next identifier is a recognized timezone abbreviation
  let datetimeStr = `${hourStr}:00 ${ampm}`;
  const tz = self.currentKind();
  if (tz !== null && tz.tag === 'Identifier' && DateTime.parseTzAbbreviation(tz._0) !== null) {
    datetimeStr = `${hourStr}:00 ${ampm} ${tz._0}`;
    self.advance(); // consume timezone token
  }

  return Expression.DateTime(DateTime.parse(datetimeStr));
}

/**
 * Tries to parse Russian "N по <TZ>" as "N:00 <TZ>".
 * @param {string} hourStr
 * @returns {Expression}
 * @param {TokenParser} self
 */
export function tryParseRussianTimeByTimezone(self, hourStr) {
  self.advance(); // consume "по"

  const kind = self.currentKind();
  if (kind === null || kind.tag !== 'Identifier') {
    return err(CalculatorError.parse('Expected timezone after по'));
  }
  const tz = kind._0;
  if (DateTime.parseTzAbbreviation(tz) === null) {
    return err(CalculatorError.parse(`Unknown timezone: ${tz}`));
  }
  self.advance();

  const datetimeStr = `${hourStr}:00 ${tz}`;
  return Expression.DateTime(DateTime.parse(datetimeStr));
}

/**
 * @param {string} first
 * @returns {Expression}
 * @param {TokenParser} self
 */
export function tryParseDatetimeFromTokens(self, first) {
  // Collect tokens that might be part of a datetime
  const parts = [first];
  // Track token positions so we can backtrack to any prefix
  const tokenPositions = [self.pos]; // position before each token was consumed

  // Look for patterns like: Jan 22, 2026 or Jan 27, 8:59am UTC
  // Also handles: Monday, January 26th, 2026
  // Collect: numbers, identifiers, colons, commas
  while (!self.isAtEnd()) {
    const kind = /** @type {TokenKind} */ (self.currentKind());
    if (kind.tag === 'Number') {
      tokenPositions.push(self.pos);
      parts.push(kind._0);
      self.advance();
    } else if (kind.tag === 'Identifier') {
      // Handle ordinal suffixes: if previous part is a number and
      // self is "st", "nd", "rd", or "th", attach without space
      const id = kind._0;
      if (isOrdinalSuffix(id) && parts.length > 0 && allAsciiDigits(parts[parts.length - 1])) {
        // Append to previous number part (ordinal suffix)
        tokenPositions.push(self.pos);
        parts[parts.length - 1] += id;
        self.advance();
        continue;
      }
      tokenPositions.push(self.pos);
      parts.push(id);
      self.advance();
    } else if (kind.tag === 'Of') {
      tokenPositions.push(self.pos);
      parts.push('of');
      self.advance();
    } else if (kind.tag === 'Comma') {
      tokenPositions.push(self.pos);
      parts.push(',');
      self.advance();
    } else if (kind.tag === 'Colon') {
      tokenPositions.push(self.pos);
      parts.push(':');
      self.advance();
    } else {
      break;
    }
  }

  // Try the full collected string first, then progressively shorter prefixes.
  // This handles cases like "17 февраля 2027 - 6 months" where the datetime
  // is "17 февраля 2027" but greedily collecting too many tokens would fail.
  const endPos = self.pos;
  for (let len = parts.length; len >= 1; len--) {
    const datetimeStr = joinDatetimeParts(parts.slice(0, len));
    const dt = ok(() => DateTime.parse(datetimeStr));
    if (dt !== null) {
      // Restore position to just after the tokens we actually consumed
      // token_positions[len - 1] is the position *before* consuming parts[len-1]
      // so the position after consuming parts[len-1] is:
      // - for the last element: end_pos (we already advanced past all)
      // - for shorter prefix: token_positions[len] (position before parts[len])
      if (len < parts.length) {
        self.pos = tokenPositions[len];
      } else {
        self.pos = endPos;
      }
      return Expression.DateTime(dt);
    }
  }

  // No prefix worked — restore original position and report error
  if (parts.length === 0) {
    return err(CalculatorError.parse('empty datetime'));
  }
  // Return error with the full string for better error messages
  const datetimeStr = joinDatetimeParts(parts);
  return err(CalculatorError.InvalidDateTime(
    `Could not parse '${datetimeStr}' as a date or time`,
  ));
}
