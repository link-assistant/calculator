// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3012f9b78dccf7e53cff54c142f8bc1e9042df35d39c904c632cb9e98ee3a65f bytes=710

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Numeric word scales (port of `src/grammar/token_parser/numbers.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=84a0d1adf2ae595e085753ab28284a0da725754b1f33567045b97ef4b5b952e8
// | import { Decimal } from '../../types/index.js';
use crate::types::index::Decimal;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('../token_parser.js').TokenParser} TokenParser */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * German abbreviations for million and billion are numeric scales.
// |  * @param {TokenParser} self
// |  * @returns {Decimal | null}
// |  */
// | export function consumeWordScale(self) {
// |   const kind = self.currentKind();
// |   if (kind === null || kind.tag !== 'Identifier') {
// |     return null;
// |   }
// |   let multiplier;
// |   if (kind._0 === 'Mio') {
// |     multiplier = 1_000_000;
// |   } else if (kind._0 === 'Mrd') {
// |     multiplier = 1_000_000_000;
// |   } else {
// |     return null;
// |   }
// |   self.advance();
// |   return Decimal.new(multiplier);
// | }
