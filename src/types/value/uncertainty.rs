use crate::error::CalculatorError;
use crate::types::{Value, ValueKind};

impl Value {
    /// Creates a measurement with a non-negative absolute error in the same unit.
    pub fn with_uncertainty(&self, error: &Self) -> Result<Self, CalculatorError> {
        if self.unit != error.unit {
            return Err(CalculatorError::InvalidOperation(
                "uncertainty and central value must have matching units".into(),
            ));
        }
        let value = self
            .to_rational()
            .ok_or_else(|| CalculatorError::domain("central value must be numeric"))?;
        let uncertainty = error
            .to_rational()
            .ok_or_else(|| CalculatorError::domain("uncertainty must be numeric"))?;
        if uncertainty.is_negative() {
            return Err(CalculatorError::domain("uncertainty must be non-negative"));
        }
        Ok(Self {
            kind: ValueKind::Uncertainty { value, uncertainty },
            unit: self.unit.clone(),
        })
    }

    pub(super) fn negate_uncertainty(&self) -> Self {
        let ValueKind::Uncertainty { value, uncertainty } = &self.kind else {
            unreachable!()
        };
        Self {
            kind: ValueKind::Uncertainty {
                value: -value.clone(),
                uncertainty: uncertainty.clone(),
            },
            unit: self.unit.clone(),
        }
    }
}
