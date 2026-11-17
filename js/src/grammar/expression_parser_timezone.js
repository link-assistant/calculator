/**
 * Local-timezone handling for `ExpressionParser` (port of
 * `src/grammar/expression_parser_timezone.rs`).
 *
 * When the user's local timezone offset is configured, `now` and bare
 * (timezone-less) times such as `12:30` are interpreted in that timezone
 * instead of UTC. Inputs with an explicit timezone (e.g. `12:30 UTC`) are
 * always honored regardless of this setting.
 */

import { unreachable } from '../rust/core.js';
import { DateTime } from '../types/index.js';

/** @typedef {import('./expression_parser.js').ExpressionParser} ExpressionParser */
/** @typedef {import('../types/expression.js').Expression} Expression */

/**
 * Sets the user's local timezone offset in seconds east of UTC.
 *
 * Pass `null` to fall back to the default UTC interpretation.
 * @param {ExpressionParser} self
 * @param {number | null} offsetSeconds
 */
export function setLocalOffsetSeconds(self, offsetSeconds) {
  self._localOffsetSeconds = offsetSeconds;
}

/**
 * Returns the configured local timezone offset in seconds east of UTC, if any.
 * @param {ExpressionParser} self
 * @returns {number | null}
 */
export function localOffsetSeconds(self) {
  return self._localOffsetSeconds;
}

/**
 * Returns a `DateTime` representing the current instant, honoring the
 * configured local timezone offset when one is set.
 * @param {ExpressionParser} self
 * @returns {DateTime}
 */
export function currentNow(self) {
  const offset = self._localOffsetSeconds;
  if (offset !== null) {
    return DateTime.nowLocal(offset);
  }
  return DateTime.nowWithLabel('current UTC time', 0, 'UTC');
}

/**
 * Resolves a dynamic current-date expression in the configured timezone.
 * @param {ExpressionParser} self
 * @param {Expression} expression
 * @returns {DateTime}
 */
export function currentDate(self, expression) {
  switch (expression.tag) {
    case 'Now':
      return self.currentNow();
    case 'Today':
      return DateTime.today(self._localOffsetSeconds ?? 0);
    default:
      return unreachable('current_date only accepts dynamic date expressions');
  }
}
