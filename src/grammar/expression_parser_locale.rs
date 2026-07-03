//! Locale-aware parser fallback for number input.

use crate::error::CalculatorError;
use crate::grammar::{locale_numbers, ExpressionParser};
use crate::types::Expression;

impl ExpressionParser {
    /// Parses an expression string into an Expression AST.
    pub fn parse(&self, input: &str) -> Result<Expression, CalculatorError> {
        let interpretations = self.parse_interpretations(input)?;
        interpretations
            .into_iter()
            .next()
            .ok_or_else(|| CalculatorError::parse("No parseable interpretation"))
    }

    /// Parses an expression into every supported locale interpretation.
    ///
    /// The ordinary grammar is tried first and wins when it succeeds. If the
    /// ordinary grammar rejects the input, common locale number conventions are
    /// normalized to the grammar's canonical decimal-dot format and tried in a
    /// stable order.
    pub fn parse_interpretations(&self, input: &str) -> Result<Vec<Expression>, CalculatorError> {
        match self.parse_tokenized(input) {
            Ok(expr) => Ok(vec![expr]),
            Err(first_error) => {
                let mut interpretations = Vec::new();
                let mut linos = Vec::new();

                for variant in locale_numbers::variants(input) {
                    if let Ok(expr) = self.parse_tokenized(&variant) {
                        let lino = expr.to_lino();
                        if !linos.contains(&lino) {
                            linos.push(lino);
                            interpretations.push(expr);
                        }
                    }
                }

                if interpretations.is_empty() {
                    Err(first_error)
                } else {
                    Ok(interpretations)
                }
            }
        }
    }
}
