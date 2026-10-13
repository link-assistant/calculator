// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abbc52f6d0002f46afd125e89285c31199e672bcb755532a08b160596ed10c33 bytes=2064

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Local-timezone handling for `ExpressionParser` (port of
// |  * `src/grammar/expression_parser_timezone.rs`).
// |  *
// |  * When the user's local timezone offset is configured, `now` and bare
// |  * (timezone-less) times such as `12:30` are interpreted in that timezone
// |  * instead of UTC. Inputs with an explicit timezone (e.g. `12:30 UTC`) are
// |  * always honored regardless of this setting.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=3dde822c7bf5a7351b778b31b2cdfeec4b3eada50f45b200f645dce350e1e450
// | import { unreachable } from '../rust/core.js';
use crate::rust::core_::unreachable;
// meta-language:translated JavaScript import_statement items=1 sha256=d8e51f5c7f1c927e493110f4c8027bb41f6445ef704a9dffb13615589b040d67
// | import { DateTime } from '../types/index.js';
use crate::types::index::DateTime;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./expression_parser.js').ExpressionParser} ExpressionParser */
// | /** @typedef {import('../types/expression.js').Expression} Expression */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Sets the user's local timezone offset in seconds east of UTC.
// |  *
// |  * Pass `null` to fall back to the default UTC interpretation.
// |  * @param {ExpressionParser} self
// |  * @param {number | null} offsetSeconds
// |  */
// | export function setLocalOffsetSeconds(self, offsetSeconds) {
// |   self._localOffsetSeconds = offsetSeconds;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Returns the configured local timezone offset in seconds east of UTC, if any.
// |  * @param {ExpressionParser} self
// |  * @returns {number | null}
// |  */
// | export function localOffsetSeconds(self) {
// |   return self._localOffsetSeconds;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Returns a `DateTime` representing the current instant, honoring the
// |  * configured local timezone offset when one is set.
// |  * @param {ExpressionParser} self
// |  * @returns {DateTime}
// |  */
// | export function currentNow(self) {
// |   const offset = self._localOffsetSeconds;
// |   if (offset !== null) {
// |     return DateTime.nowLocal(offset);
// |   }
// |   return DateTime.nowWithLabel('current UTC time', 0, 'UTC');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Resolves a dynamic current-date expression in the configured timezone.
// |  * @param {ExpressionParser} self
// |  * @param {Expression} expression
// |  * @returns {DateTime}
// |  */
// | export function currentDate(self, expression) {
// |   switch (expression.tag) {
// |     case 'Now':
// |       return self.currentNow();
// |     case 'Today':
// |       return DateTime.today(self._localOffsetSeconds ?? 0);
// |     default:
// |       return unreachable('current_date only accepts dynamic date expressions');
// |   }
// | }
