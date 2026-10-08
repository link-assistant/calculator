//! Primary and alternative numeric interpretations, resolved during lexing.

use crate::error::CalculatorError;
use crate::grammar::ExpressionParser;
use crate::types::Expression;

impl ExpressionParser {
    /// Parses an expression string into an Expression AST.
    pub fn parse(&self, input: &str) -> Result<Expression, CalculatorError> {
        self.parse_tokenized(input)
    }

    /// Parses the primary numeric convention and a decimal-comma alternative.
    ///
    /// Thousands grouping wins for a comma before three digits. The decimal
    /// interpretation is still available to callers that display alternatives.
    /// Function arguments use commas as separators in both interpretations.
    pub fn parse_interpretations(&self, input: &str) -> Result<Vec<Expression>, CalculatorError> {
        let primary = self.parse_tokenized(input)?;
        let mut interpretations = vec![primary];
        if input.contains(',') {
            if let Ok(alternative) = self.parse_tokenized_decimal_commas(input) {
                if alternative.to_lino() != interpretations[0].to_lino() {
                    interpretations.push(alternative);
                }
            }
        }
        Ok(interpretations)
    }
}
