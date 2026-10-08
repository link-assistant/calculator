use super::duration::format_duration;
use crate::types::{Unit, Value, ValueKind};

impl Value {
    /// Converts the value to a display string.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        match &self.kind {
            ValueKind::Number(n) => {
                let n_str = n.normalize().to_string();
                if self.unit == Unit::None {
                    n_str
                } else {
                    format!("{} {}", n_str, self.unit)
                }
            }
            ValueKind::Rational(r) => {
                let r_str = r.to_display_string();
                if self.unit == Unit::None {
                    r_str
                } else {
                    format!("{} {}", r_str, self.unit)
                }
            }
            ValueKind::Uncertainty { value, uncertainty } => {
                let suffix = if self.unit == Unit::None {
                    String::new()
                } else {
                    format!(" {}", self.unit)
                };
                format!(
                    "{} ± {}{suffix}",
                    value.to_display_string(),
                    uncertainty.to_display_string()
                )
            }
            ValueKind::DateTime(dt) => dt.to_string(),
            ValueKind::Duration { seconds } => format_duration(*seconds),
            ValueKind::Boolean(b) => b.to_string(),
            ValueKind::Comparison {
                left,
                relation,
                right,
            } => format!("{left} {relation} {right}"),
            ValueKind::EquationSolution { variable, value } => {
                format!("{variable} = {}", value.to_display_string())
            }
            ValueKind::EquationSolutions { variable, values } => values
                .iter()
                .map(|value| format!("{variable} = {}", value.to_display_string()))
                .collect::<Vec<_>>()
                .join(" or "),
            ValueKind::SymbolicEquationSolution {
                variable,
                expression,
            } => {
                format!("{variable} = {expression}")
            }
        }
    }
}
