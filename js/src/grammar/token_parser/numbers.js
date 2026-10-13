/**
 * Numeric word scales (port of `src/grammar/token_parser/numbers.rs`).
 */

import { Decimal } from '../../types/index.js';

/** @typedef {import('../token_parser.js').TokenParser} TokenParser */

/**
 * German abbreviations for million and billion are numeric scales.
 * @param {TokenParser} self
 * @returns {Decimal | null}
 */
export function consumeWordScale(self) {
  const kind = self.currentKind();
  if (kind === null || kind.tag !== 'Identifier') {
    return null;
  }
  let multiplier;
  if (kind._0 === 'Mio') {
    multiplier = 1_000_000;
  } else if (kind._0 === 'Mrd') {
    multiplier = 1_000_000_000;
  } else {
    return null;
  }
  self.advance();
  return Decimal.new(multiplier);
}
