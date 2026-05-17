use crate::error::CalculatorError;
use crate::grammar::TokenKind;
use crate::types::{DataSizeUnit, Expression, MassUnit, Unit};

use super::TokenParser;

impl TokenParser<'_> {
    /// Parses a unit name after the `as`, `in`, or `to` keyword.
    pub(super) fn parse_unit_for_conversion(&mut self) -> Result<Unit, CalculatorError> {
        let Some(TokenKind::Identifier(id)) = self.current_kind() else {
            return Err(CalculatorError::parse(
                "Expected a unit name after 'as'/'in'/'to' (e.g., 'MB', 'kg', 'USD', 'dollars')",
            ));
        };

        let unit_str = id.clone();
        self.advance();

        if let Some(data_size) = DataSizeUnit::parse(&unit_str) {
            return Ok(Unit::DataSize(data_size));
        }
        if let Some(mass) = MassUnit::parse(&unit_str) {
            return Ok(Unit::Mass(mass));
        }

        let lower = unit_str.to_lowercase();
        if let Some(data_size) = DataSizeUnit::parse(&lower) {
            return Ok(Unit::DataSize(data_size));
        }
        if let Some(mass) = MassUnit::parse(&lower) {
            return Ok(Unit::Mass(mass));
        }

        if let Some(duration) = crate::types::DurationUnit::parse(&unit_str) {
            return Ok(Unit::Duration(duration));
        }

        // Timezone comes before currency because currency code parsing accepts
        // any 2-5 letter code.
        if crate::types::DateTime::parse_tz_abbreviation(&unit_str).is_some() {
            return Ok(Unit::Timezone(unit_str.to_uppercase()));
        }

        if let Some(currency_code) = crate::types::CurrencyDatabase::parse_currency(&unit_str) {
            return Ok(Unit::currency(&currency_code));
        }

        Err(CalculatorError::parse(format!(
            "Unknown unit '{unit_str}'. Supported conversions: \
             data sizes (B, KB, MB, GB, KiB, MiB, GiB, ...), \
             mass (g, kg, tons, lb, oz), \
             currencies (USD, EUR, GBP, TON, BTC, ETH, ...) and natural language \
             aliases (dollars, euros, bitcoin, toncoin, ...), \
             timezones (UTC, GMT, EST, MSK, JST, ...), \
             time durations (ms, seconds, minutes, hours, days, weeks, months, years)."
        )))
    }

    /// Resolves unit ambiguity when a conversion target provides context.
    pub(super) fn resolve_unit_ambiguity_for_conversion(
        expr: Expression,
        target_unit: &Unit,
    ) -> Expression {
        if let Expression::Number {
            value,
            ref unit,
            ref alternative_units,
        } = expr
        {
            if alternative_units.is_empty() || unit.is_same_category(target_unit) {
                return expr;
            }

            for alt in alternative_units {
                if alt.is_same_category(target_unit) {
                    let mut new_alternatives: Vec<Unit> = alternative_units
                        .iter()
                        .filter(|u| *u != alt)
                        .cloned()
                        .collect();
                    new_alternatives.push(unit.clone());
                    return Expression::number_with_unit_alternatives(
                        value,
                        alt.clone(),
                        new_alternatives,
                    );
                }
            }
        }
        expr
    }
}
