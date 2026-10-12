//! Units after a number and as conversion targets (issue #235).
//!
//! A unit expression is a product of unit names with optional integer powers:
//! `m^2`, `m²`, `s^-1`, adjacent `s−1`, joined by spaces, `*`, `·`, `⋅`, `/`
//! or `per`, with a parenthesized denominator (`kg / (m^-2 s^2)`). Only
//! dimensional units (data/units, mass, durations, data sizes) take part, so
//! `5 USD * 2` and `10 m / 2 s` stay arithmetic. A quantity followed by more
//! quantities of the same dimension is their sum: `5 feet 10 inches`, `6'2"`
//! or `1 h 30 min`.

use crate::grammar::{is_math_function, TokenKind};
use crate::types::{BinaryOp, Decimal, Expression, PhysicalUnit, Unit};

use super::TokenParser;

/// `(canonical symbol, power)` factors of a unit expression.
type Terms = Vec<(String, i32)>;

/// Largest power accepted in a unit expression, bounding `m^1000000`.
const MAX_UNIT_POWER: i32 = 1000;

impl TokenParser<'_> {
    /// Parses the unit after a number: a dimensional unit expression, a
    /// legacy unit with alternatives, or `None` when no unit follows.
    pub(super) fn parse_quantity_unit(&mut self, number_end: usize) -> Option<(Unit, Vec<Unit>)> {
        if let Some(unit) = self.inch_before_conversion() {
            return Some((unit, Vec::new()));
        }
        let TokenKind::Identifier(id) = self.current_kind()? else {
            return None;
        };
        let id = id.clone();
        // An unknown single-letter identifier is a conventional coefficient
        // (`2x` or `2 x`). Established short units (`2h`, `2m`) and
        // multi-letter custom units retain their meaning.
        let coefficient_variable =
            Self::is_single_letter_variable(&id) && !self.identifier_is_known_unit(&id);
        let ratio_keyword = Self::ratio_keyword(&id)
            && self.current().is_some_and(|token| token.start > number_end);
        // `min` and `sec` are units after a number unless called: `5 min`,
        // but `2 min(3, 4)` stays a function call.
        let function = is_math_function(&id) && !self.identifier_is_dimensional_unit(&id);
        if function || self.peek_is_left_paren() || coefficient_variable || ratio_keyword {
            return None;
        }

        let save = self.pos;
        if let Some(terms) = self.parse_unit_terms() {
            let compound = self.pos > save + 1 || terms.len() != 1 || terms[0].1 != 1;
            if compound {
                return Some((PhysicalUnit::from_terms(terms).into_unit(), Vec::new()));
            }
        }
        self.pos = save;

        let unit = self
            .number_grammar
            .parse_unit_with_alternatives(&id)
            .unwrap_or_else(|_| (Unit::Custom(id.clone()), Vec::new()));
        self.advance();
        Some(unit)
    }

    /// Whether `id` names a dimensional unit (`min`, `sec`), so a function
    /// name such as `pi` that is also a currency code stays a constant.
    fn identifier_is_dimensional_unit(&self, id: &str) -> bool {
        self.number_grammar
            .parse_unit_with_alternatives(id)
            .ok()
            .and_then(|(unit, _)| PhysicalUnit::from_unit(&unit))
            .is_some_and(|unit| !unit.is_empty())
    }

    /// Extends a quantity into a sum of quantities of the same dimension:
    /// `5 feet 10 inches`, `1 meter 20 cm`, `6'2"`, `1 h 30 min`.
    pub(super) fn parse_compound_quantity(
        &mut self,
        first: Expression,
    ) -> Result<Expression, crate::error::CalculatorError> {
        let Expression::Number { unit, .. } = &first else {
            return Ok(first);
        };
        let Some(dimension) = PhysicalUnit::from_unit(unit)
            .filter(|unit| !unit.is_empty())
            .map(|unit| unit.dimension())
        else {
            return Ok(first);
        };
        let mut sum = first;
        loop {
            let save = self.pos;
            let Some(TokenKind::Number(text)) = self.current_kind() else {
                break;
            };
            let text = text.clone();
            self.advance();
            let part_unit = self
                .parse_unit_terms()
                .filter(|terms| terms.len() == 1 && terms[0].1 == 1)
                .map(PhysicalUnit::from_terms)
                .filter(|part| part.dimension() == dimension);
            let Some(part_unit) = part_unit else {
                self.pos = save;
                break;
            };
            let value = self.number_grammar.parse_number(&text)?;
            let part = Expression::number_with_unit(value, part_unit.into_unit());
            sum = Expression::binary(sum, BinaryOp::Add, part);
        }
        Ok(sum)
    }

    /// Parses a conversion target as a unit expression (`m/s`, `°F`,
    /// `kg m^2 s^-2`); `None` leaves the target to the legacy unit names.
    pub(super) fn parse_physical_conversion_target(&mut self) -> Option<Unit> {
        let save = self.pos;
        if let Some(terms) = self.parse_unit_terms() {
            if self.conversion_target_ends() {
                return Some(PhysicalUnit::from_terms(terms).into_unit());
            }
        }
        self.pos = save;
        None
    }

    /// Whether a conversion target is complete: the input, a group or an
    /// `at` clause ends.
    fn conversion_target_ends(&self) -> bool {
        self.is_at_end()
            || matches!(self.current_kind(), Some(TokenKind::RightParen))
            || self.check_at()
    }

    /// `2in` or `2 in` followed by the end of input or a conversion keyword
    /// is two inches (`in` is otherwise the conversion keyword).
    fn inch_before_conversion(&mut self) -> Option<Unit> {
        if !self.check_in() {
            return None;
        }
        let next = self.tokens.get(self.pos + 1).map(|token| &token.kind);
        let ends = matches!(
            next,
            None | Some(
                TokenKind::Eof
                    | TokenKind::In
                    | TokenKind::As
                    | TokenKind::To
                    | TokenKind::RightParen
            )
        );
        if !ends {
            return None;
        }
        self.advance();
        PhysicalUnit::parse("in").map(PhysicalUnit::into_unit)
    }

    /// Parses `factor ((space | * | / | per) factor)*` into merged terms.
    fn parse_unit_terms(&mut self) -> Option<Terms> {
        let mut terms = self.parse_unit_factor()?;
        loop {
            let save = self.pos;
            let divide = match self.current_kind() {
                Some(TokenKind::Star) => Some(false),
                Some(TokenKind::Slash) => Some(true),
                Some(TokenKind::Identifier(word)) if word.eq_ignore_ascii_case("per") => Some(true),
                _ => None,
            };
            if divide.is_some() {
                self.advance();
            }
            let factor = if divide == Some(true) && self.check(&TokenKind::LeftParen) {
                self.parse_unit_group()
            } else {
                self.parse_unit_factor()
            };
            let Some(factor) = factor else {
                self.pos = save;
                break;
            };
            let sign = if divide == Some(true) { -1 } else { 1 };
            terms.extend(
                factor
                    .into_iter()
                    .map(|(symbol, power)| (symbol, power * sign)),
            );
        }
        Some(merge_terms(terms))
    }

    /// Parses `( unit expression )`.
    fn parse_unit_group(&mut self) -> Option<Terms> {
        let save = self.pos;
        self.advance();
        let terms = self.parse_unit_terms();
        if terms.is_some() && self.check(&TokenKind::RightParen) {
            self.advance();
            return terms;
        }
        self.pos = save;
        None
    }

    /// Parses a dimensional unit name with an optional integer power.
    fn parse_unit_factor(&mut self) -> Option<Terms> {
        let unit = self.parse_unit_name()?;
        let power = self.parse_unit_power()?;
        Some(
            unit.powi(power)
                .terms()
                .iter()
                .map(|term| (term.symbol.clone(), term.power))
                .collect(),
        )
    }

    /// Resolves the dimensional unit at the current token, including
    /// two-word names such as `light year`.
    fn parse_unit_name(&mut self) -> Option<PhysicalUnit> {
        let TokenKind::Identifier(id) = self.current_kind()? else {
            return None;
        };
        if let Some(TokenKind::Identifier(next)) = self.peek_kind() {
            if let Some(unit) = PhysicalUnit::parse(&format!("{id}_{next}")) {
                self.pos += 2;
                return Some(unit);
            }
        }
        if (is_math_function(id) && self.peek_is_left_paren()) || id.eq_ignore_ascii_case("per") {
            return None;
        }
        let (unit, _) = self.number_grammar.parse_unit_with_alternatives(id).ok()?;
        let unit = PhysicalUnit::from_unit(&unit).filter(|unit| !unit.is_empty())?;
        self.advance();
        Some(unit)
    }

    /// Parses `^n`, `^-n`, `^(-n)`, a superscript, or an adjacent `−n`;
    /// `Some(1)` when no power follows and `None` for an invalid power.
    fn parse_unit_power(&mut self) -> Option<i32> {
        let previous_end = self.tokens.get(self.pos.checked_sub(1)?)?.end;
        let save = self.pos;
        let power = match self.current_kind() {
            Some(TokenKind::Caret) => {
                self.advance();
                let parenthesized = self.check(&TokenKind::LeftParen);
                if parenthesized {
                    self.advance();
                }
                let power = self.parse_signed_integer();
                if parenthesized && power.is_some() {
                    self.check(&TokenKind::RightParen).then(|| self.advance())?;
                }
                power
            }
            Some(TokenKind::Superscript(text)) => {
                let power = text.replace('+', "").parse::<i32>().ok();
                self.advance();
                power
            }
            Some(TokenKind::Minus)
                if self
                    .current()
                    .is_some_and(|token| token.start == previous_end) =>
            {
                let minus_end = self.current()?.end;
                let adjacent = self.tokens.get(self.pos + 1).is_some_and(|token| {
                    token.start == minus_end && matches!(token.kind, TokenKind::Number(_))
                });
                if adjacent {
                    self.parse_signed_integer()
                } else {
                    return Some(1);
                }
            }
            _ => return Some(1),
        };
        match power {
            Some(power) if power != 0 && power.abs() <= MAX_UNIT_POWER => Some(power),
            _ => {
                self.pos = save;
                // `2 kg^x` is not a unit power; leave `^` to arithmetic.
                Some(1)
            }
        }
    }

    /// Parses an optionally negative integer literal.
    fn parse_signed_integer(&mut self) -> Option<i32> {
        let negative = self.check(&TokenKind::Minus);
        if negative {
            self.advance();
        }
        let TokenKind::Number(text) = self.current_kind()? else {
            return None;
        };
        let power = text.parse::<i32>().ok()?;
        self.advance();
        Some(if negative { -power } else { power })
    }
}

/// Adds the powers of equal symbols, keeping first-seen order.
fn merge_terms(terms: Terms) -> Terms {
    let mut merged: Terms = Vec::new();
    for (symbol, power) in terms {
        match merged.iter_mut().find(|(existing, _)| *existing == symbol) {
            Some((_, total)) => *total += power,
            None => merged.push((symbol, power)),
        }
    }
    merged.retain(|(_, power)| *power != 0);
    merged
}

/// `a / b unit` reads as `(a / b) unit` when `b` carries a unit, so
/// `1/0.5 kg` is `2 kg`.
pub(super) fn divide_into_quantity(left: Expression, right: Expression) -> Expression {
    match right {
        Expression::Number {
            value,
            unit,
            alternative_units,
        } if unit != Unit::None
            && matches!(
                left,
                Expression::Number {
                    unit: Unit::None,
                    ..
                }
            ) =>
        {
            let quotient = Expression::binary(left, BinaryOp::Divide, Expression::number(value));
            let unit_one =
                Expression::number_with_unit_alternatives(Decimal::new(1), unit, alternative_units);
            Expression::binary(quotient, BinaryOp::Multiply, unit_one)
        }
        right => Expression::binary(left, BinaryOp::Divide, right),
    }
}
