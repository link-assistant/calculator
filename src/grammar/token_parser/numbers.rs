use crate::grammar::TokenKind;
use crate::types::Decimal;

use super::TokenParser;

impl TokenParser<'_> {
    /// German abbreviations for million and billion are numeric scales.
    pub(super) fn consume_word_scale(&mut self) -> Option<Decimal> {
        let TokenKind::Identifier(word) = self.current_kind()? else {
            return None;
        };
        let multiplier = match word.as_str() {
            "Mio" => 1_000_000,
            "Mrd" => 1_000_000_000,
            _ => return None,
        };
        self.advance();
        Some(Decimal::new(multiplier))
    }
}
