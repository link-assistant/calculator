use crate::grammar::TokenKind;
use crate::types::{Decimal, Unit};

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

    /// Preserve conventional speed notation as an opaque compound unit.
    /// This is deliberately limited to distance/time, keeping arithmetic `/`
    /// and established mass/currency/duration units in their existing grammar.
    pub(super) fn consume_speed_unit(&mut self) -> Option<Unit> {
        let TokenKind::Identifier(distance) = self.current_kind()? else {
            return None;
        };
        if !matches!(distance.as_str(), "km" | "m" | "cm" | "mm") {
            return None;
        }
        let slash = self.tokens.get(self.pos + 1)?;
        let time = self.tokens.get(self.pos + 2)?;
        let TokenKind::Identifier(time_name) = &time.kind else {
            return None;
        };
        if !matches!(slash.kind, TokenKind::Slash)
            || !matches!(time_name.as_str(), "h" | "s" | "min")
            || self.current()?.end != slash.start
            || slash.end != time.start
        {
            return None;
        }
        let unit = Unit::Custom(format!("{distance}/{time_name}"));
        self.pos += 3;
        Some(unit)
    }
}
