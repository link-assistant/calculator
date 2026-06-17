//! Expression parser that combines all grammars.

use crate::error::CalculatorError;
use crate::grammar::linear_equation;
use crate::grammar::polynomial_equation;
use crate::grammar::token_parser::TokenParser;
use crate::grammar::{
    evaluate_function, evaluate_indefinite_integral, DateTimeGrammar, Lexer, NumberGrammar,
};
use crate::types::{
    BinaryOp, ComparisonOp, CurrencyDatabase, DateTime, Decimal, Expression, Rational, Unit, Value,
    ValueKind,
};
use std::cmp::Ordering;

// Local-timezone handling for `now` and bare times lives in a child module so it
// can access `ExpressionParser`'s private fields while keeping this file small.
#[path = "expression_parser_timezone.rs"]
mod timezone;

/// Evaluates a power expression, using exact rational arithmetic when possible.
///
/// When both base and exponent are rational and the exponent is an integer
/// that fits in i32, the computation is exact (arbitrary precision).
/// Otherwise, falls back to f64 computation.
///
/// This function is exposed so downstream consumers can reproduce the
/// exact-versus-floating-point fallback used inside the evaluator.
pub fn evaluate_power(base_val: &Value, exp_val: &Value) -> Result<Value, CalculatorError> {
    // Try exact rational exponentiation first
    if let (Some(base_rat), Some(exp_rat)) = (base_val.to_rational(), exp_val.to_rational()) {
        if exp_rat.is_integer() {
            // Check exponent fits in i32 (reasonable range for exact computation)
            let exp_i128 = exp_rat.numer();
            if let Ok(exp_i32) = i32::try_from(exp_i128) {
                // Guard against absurdly large exponents that would consume too much memory
                if exp_i32.abs() <= 1_000_000 {
                    if exp_i32 < 0 && base_rat.is_zero() {
                        return Err(CalculatorError::domain(
                            "division by zero (negative exponent with zero base)",
                        ));
                    }
                    let result = base_rat.pow_i32(exp_i32);
                    return Ok(Value::rational(result));
                }
            }
        }
    }

    // Fallback to f64 for non-integer exponents or very large exponents
    let base_dec = base_val
        .as_decimal()
        .ok_or_else(|| CalculatorError::InvalidOperation("power base must be numeric".into()))?;
    let exp_dec = exp_val.as_decimal().ok_or_else(|| {
        CalculatorError::InvalidOperation("power exponent must be numeric".into())
    })?;

    let base_f64 = base_dec.to_f64();
    let exp_f64 = exp_dec.to_f64();
    let result = base_f64.powf(exp_f64);

    if result.is_nan() {
        return Err(CalculatorError::domain("power result is undefined"));
    }
    if result.is_infinite() {
        return Err(CalculatorError::Overflow);
    }

    Ok(Value::number(Decimal::from_f64(result)))
}

/// Parser for calculator expressions.
#[derive(Debug, Default)]
pub struct ExpressionParser {
    number_grammar: NumberGrammar,
    datetime_grammar: DateTimeGrammar,
    currency_db: CurrencyDatabase,
    /// Current date context for historical currency conversions (set by AtTime expressions).
    current_date_context: Option<DateTime>,
    /// The user's local timezone offset in seconds east of UTC, when known.
    ///
    /// When set, `now` and bare (timezone-less) times such as `12:30` are
    /// interpreted in this local timezone instead of UTC. Explicit timezones
    /// (e.g. `12:30 UTC`) are always honored regardless of this setting.
    local_offset_seconds: Option<i32>,
}

impl ExpressionParser {
    /// Creates a new expression parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            number_grammar: NumberGrammar::new(),
            datetime_grammar: DateTimeGrammar::new(),
            currency_db: CurrencyDatabase::new(),
            current_date_context: None,
            local_offset_seconds: None,
        }
    }

    /// Returns a reference to the currency database.
    pub fn currency_db(&self) -> &CurrencyDatabase {
        &self.currency_db
    }

    /// Returns a mutable reference to the currency database.
    pub fn currency_db_mut(&mut self) -> &mut CurrencyDatabase {
        &mut self.currency_db
    }

    /// Parses and evaluates an expression, returning the result, steps, and lino representation.
    pub fn parse_and_evaluate(
        &mut self,
        input: &str,
    ) -> Result<(Value, Vec<String>, String), CalculatorError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CalculatorError::EmptyInput);
        }

        // Clear any previous rate tracking
        self.currency_db.clear_last_used_rate();

        // Try datetime subtraction pattern first: "(datetime) - (datetime)"
        if let Some(result) = self
            .datetime_grammar
            .try_parse_datetime_subtraction(input, self.local_offset_seconds)
        {
            return Ok(result);
        }

        // Parse the expression
        let expr = self.parse(input)?;

        // Generate links notation representation
        let lino = expr.to_lino();

        // Evaluate with step tracking
        let (value, steps) = self.evaluate_with_steps(&expr)?;

        Ok((value, steps, lino))
    }

    /// Parses an expression string into an Expression AST.
    pub fn parse(&self, input: &str) -> Result<Expression, CalculatorError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = TokenParser::new(&tokens, &self.number_grammar, input);
        let mut expr = parser.parse_complete_expression()?;
        // Re-anchor timezone-less datetime literals to the user's local timezone
        // when it is known, so bare times like `12:30` mean local time.
        if let Some(offset) = self.local_offset_seconds {
            expr.apply_local_offset(offset);
        }
        Ok(expr)
    }

    /// Evaluates an expression.
    pub fn evaluate(&mut self, expr: &Expression) -> Result<Value, CalculatorError> {
        self.evaluate_expr(expr)
    }

    fn expression_contains_variable(expr: &Expression) -> bool {
        match expr {
            Expression::Variable(_) => true,
            Expression::Until(inner) | Expression::Negate(inner) | Expression::Group(inner) => {
                Self::expression_contains_variable(inner)
            }
            Expression::Binary { left, right, .. }
            | Expression::Power {
                base: left,
                exponent: right,
            } => {
                Self::expression_contains_variable(left)
                    || Self::expression_contains_variable(right)
            }
            Expression::Equality { left, right } | Expression::Comparison { left, right, .. } => {
                Self::expression_contains_variable(left)
                    || Self::expression_contains_variable(right)
            }
            Expression::AtTime { value, time } => {
                Self::expression_contains_variable(value)
                    || Self::expression_contains_variable(time)
            }
            Expression::FunctionCall { args, .. } => {
                args.iter().any(Self::expression_contains_variable)
            }
            Expression::IndefiniteIntegral { integrand, .. } => {
                Self::expression_contains_variable(integrand)
            }
            Expression::UnitConversion { value, .. } => Self::expression_contains_variable(value),
            Expression::Number { .. } | Expression::DateTime(_) | Expression::Now => false,
        }
    }

    fn solve_equation(left: &Expression, right: &Expression) -> Result<Value, CalculatorError> {
        if let Ok(solution) = linear_equation::solve(left, right) {
            return Ok(solution.to_value());
        }

        Ok(polynomial_equation::solve(left, right)?.to_value())
    }

    /// Evaluates an expression with step-by-step tracking.
    ///
    /// Returns the final [`Value`] alongside the human-readable list of steps
    /// the evaluator took. Downstream consumers can drive this directly after
    /// they have an [`Expression`] in hand (for example from [`Self::parse`])
    /// without having to round-trip through [`Self::parse_and_evaluate`].
    pub fn evaluate_with_steps(
        &mut self,
        expr: &Expression,
    ) -> Result<(Value, Vec<String>), CalculatorError> {
        let mut steps = Vec::new();

        steps.push(format!("Input expression: {expr}"));

        let result = self.evaluate_expr_with_steps(expr, &mut steps)?;

        steps.push(format!("Final result: {}", result.to_display_string()));

        Ok((result, steps))
    }

    /// Evaluates an expression without step tracking.
    ///
    /// This is the silent counterpart of [`Self::evaluate_with_steps`]. It is
    /// exposed so downstream consumers can reuse the calculator's evaluator
    /// when reconstructing computations from a pre-parsed AST.
    pub fn evaluate_expr(&mut self, expr: &Expression) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit, .. } => {
                // Convert to Rational for exact arithmetic
                let rational = Rational::from_decimal(*value);
                Ok(Value::rational_with_unit(rational, unit.clone()))
            }
            Expression::DateTime(dt) => Ok(Value::datetime(dt.clone())),
            Expression::Now => Ok(Value::datetime(self.current_now())),
            Expression::Until(target) => {
                let target_val = self.evaluate_expr(target)?;
                let now = self.current_now();
                match &target_val.kind {
                    ValueKind::DateTime(target_dt) => {
                        let seconds = target_dt.signed_subtract_seconds(&now);
                        Ok(Value::duration(seconds))
                    }
                    _ => Err(CalculatorError::InvalidOperation(
                        "until requires a datetime expression".into(),
                    )),
                }
            }
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.apply_binary_op(&left_val, *op, &right_val)
            }
            Expression::Negate(inner) => {
                let val = self.evaluate_expr(inner)?;
                Ok(val.negate())
            }
            Expression::Group(inner) => self.evaluate_expr(inner),
            Expression::AtTime { value, time } => {
                // Evaluate the time expression to get a DateTime
                let time_val = self.evaluate_expr(time)?;

                // Extract the DateTime for use in currency conversions
                let date_context = match &time_val.kind {
                    ValueKind::DateTime(dt) => Some(dt.clone()),
                    _ => None,
                };

                // Set the date context for this evaluation
                let old_context = self.current_date_context.take();
                self.current_date_context = date_context;

                // Evaluate the value expression with the date context
                let result = self.evaluate_expr(value);

                // Restore the old context
                self.current_date_context = old_context;

                result
            }
            Expression::FunctionCall { name, args } => {
                let name_lower = name.to_lowercase();

                // Special handling for integrate(expr, var, lower, upper)
                if name_lower == "integrate" {
                    return self.evaluate_integrate(args);
                }

                // Evaluate all arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    let val = self.evaluate_expr(arg)?;
                    // Extract the decimal value
                    let decimal = val.as_decimal().ok_or_else(|| {
                        CalculatorError::invalid_args(name, "expected numeric argument")
                    })?;
                    arg_values.push(decimal);
                }

                // Call the function
                let result = evaluate_function(name, &arg_values)?;
                Ok(Value::number(result))
            }
            Expression::Variable(name) => {
                // Variables should not appear in direct evaluation
                // They are only used in integration contexts
                Err(CalculatorError::eval(format!("undefined variable: {name}")))
            }
            Expression::Power { base, exponent } => {
                let base_val = self.evaluate_expr(base)?;
                let exp_val = self.evaluate_expr(exponent)?;
                evaluate_power(&base_val, &exp_val)
            }
            Expression::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                // Indefinite integrals return a symbolic result
                // For now, we return an error directing users to use definite integrals for numeric results
                // or display the symbolic representation
                evaluate_indefinite_integral(integrand, variable)
            }
            Expression::UnitConversion { value, target_unit } => {
                let val = self.evaluate_expr(value)?;
                val.convert_to_unit_at_date(
                    target_unit,
                    &mut self.currency_db,
                    self.current_date_context.as_ref(),
                )
            }
            Expression::Equality { left, right } => {
                if Self::expression_contains_variable(left)
                    || Self::expression_contains_variable(right)
                {
                    return Self::solve_equation(left, right);
                }

                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                Ok(Value::boolean(left_val == right_val))
            }
            Expression::Comparison { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.evaluate_comparison_values(&left_val, *op, &right_val)
            }
        }
    }

    /// Evaluates an expression, pushing human-readable steps into `steps`.
    ///
    /// The same evaluator [`Self::evaluate_with_steps`] uses internally, but
    /// the caller owns the step buffer. Useful when interleaving step output
    /// from several sub-evaluations.
    pub fn evaluate_expr_with_steps(
        &mut self,
        expr: &Expression,
        steps: &mut Vec<String>,
    ) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit, .. } => {
                // Convert to Rational for exact arithmetic
                let rational = Rational::from_decimal(*value);
                let val = Value::rational_with_unit(rational, unit.clone());
                steps.push(format!("Literal value: {}", val.to_display_string()));
                Ok(val)
            }
            Expression::DateTime(dt) => {
                steps.push(format!("DateTime value: {dt}"));
                if let Some(utc_equivalent) = dt.utc_equivalent_display() {
                    steps.push(format!("UTC equivalent: {utc_equivalent}"));
                }
                let dt_val = Value::datetime(dt.clone());
                // For standalone datetime, show time from now
                let now = self.current_now();
                let seconds = dt.signed_subtract_seconds(&now);
                if seconds > 0 {
                    steps.push(format!(
                        "Time until: {}",
                        Value::duration(seconds).to_display_string()
                    ));
                } else if seconds < 0 {
                    steps.push(format!(
                        "Time since: {} ago",
                        Value::duration(-seconds).to_display_string()
                    ));
                }
                Ok(dt_val)
            }
            Expression::Now => {
                let now = self.current_now();
                steps.push(format!("Current time: {now}"));
                Ok(Value::datetime(now))
            }
            Expression::Until(target) => {
                let target_val = self.evaluate_expr_with_steps(target, steps)?;
                let now = self.current_now();
                match &target_val.kind {
                    ValueKind::DateTime(target_dt) => {
                        let seconds = target_dt.signed_subtract_seconds(&now);
                        let duration = Value::duration(seconds);
                        if seconds >= 0 {
                            steps.push(format!(
                                "Time until {}: {}",
                                target_val.to_display_string(),
                                duration.to_display_string()
                            ));
                        } else {
                            steps.push(format!(
                                "Time since {}: {} ago",
                                target_val.to_display_string(),
                                Value::duration(-seconds).to_display_string()
                            ));
                        }
                        Ok(duration)
                    }
                    _ => Err(CalculatorError::InvalidOperation(
                        "until requires a datetime expression".into(),
                    )),
                }
            }
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expr_with_steps(left, steps)?;
                let right_val = self.evaluate_expr_with_steps(right, steps)?;

                steps.push(format!(
                    "Compute: {} {} {}",
                    left_val.to_display_string(),
                    op,
                    right_val.to_display_string()
                ));

                // Clear any previous rate tracking before the operation
                self.currency_db.clear_last_used_rate();

                let result = self.apply_binary_op(&left_val, *op, &right_val)?;

                // If a currency conversion was used, add rate info to steps.
                // For cross-rate (triangulated) conversions there may be multiple entries.
                for (from, to, rate_info) in self.currency_db.get_last_used_rates() {
                    steps.push(format!(
                        "Exchange rate: {}",
                        rate_info.format_for_display(from, to)
                    ));
                }

                steps.push(format!("= {}", result.to_display_string()));

                Ok(result)
            }
            Expression::Negate(inner) => {
                let val = self.evaluate_expr_with_steps(inner, steps)?;
                let result = val.negate();
                steps.push(format!("Negate: -{val} = {result}"));
                Ok(result)
            }
            Expression::Group(inner) => {
                steps.push("Evaluate grouped expression:".to_string());
                self.evaluate_expr_with_steps(inner, steps)
            }
            Expression::AtTime { value, time } => {
                let time_val = self.evaluate_expr_with_steps(time, steps)?;
                steps.push(format!("At time: {}", time_val.to_display_string()));

                // Extract the DateTime for use in currency conversions
                let date_context = match &time_val.kind {
                    ValueKind::DateTime(dt) => Some(dt.clone()),
                    _ => None,
                };

                // Set the date context for this evaluation
                let old_context = self.current_date_context.take();
                self.current_date_context = date_context;

                // Evaluate the value expression with the date context
                let result = self.evaluate_expr_with_steps(value, steps);

                // Restore the old context
                self.current_date_context = old_context;

                result
            }
            Expression::FunctionCall { name, args } => {
                let name_lower = name.to_lowercase();

                // Special handling for integrate(expr, var, lower, upper)
                if name_lower == "integrate" {
                    steps.push(format!("Numerical integration: {}(...)", name));
                    let result = self.evaluate_integrate(args)?;
                    steps.push(format!("= {}", result.to_display_string()));
                    return Ok(result);
                }

                let mut arg_values = Vec::new();
                let mut arg_display = Vec::new();
                for arg in args {
                    let val = self.evaluate_expr_with_steps(arg, steps)?;
                    arg_display.push(val.to_display_string());
                    let decimal = val.as_decimal().ok_or_else(|| {
                        CalculatorError::invalid_args(name, "expected numeric argument")
                    })?;
                    arg_values.push(decimal);
                }

                steps.push(format!(
                    "Call function: {}({})",
                    name,
                    arg_display.join(", ")
                ));
                let result = evaluate_function(name, &arg_values)?;
                let val = Value::number(result);
                steps.push(format!("= {}", val.to_display_string()));
                Ok(val)
            }
            Expression::Variable(name) => {
                Err(CalculatorError::eval(format!("undefined variable: {name}")))
            }
            Expression::Power { base, exponent } => {
                let base_val = self.evaluate_expr_with_steps(base, steps)?;
                let exp_val = self.evaluate_expr_with_steps(exponent, steps)?;

                steps.push(format!(
                    "Compute: {} ^ {}",
                    base_val.to_display_string(),
                    exp_val.to_display_string()
                ));

                let val = evaluate_power(&base_val, &exp_val)?;
                steps.push(format!("= {}", val.to_display_string()));
                Ok(val)
            }
            Expression::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                steps.push(format!(
                    "Indefinite integral: ∫ {} d{}",
                    integrand, variable
                ));
                let result = evaluate_indefinite_integral(integrand, variable)?;
                steps.push(format!("= {}", result.to_display_string()));
                Ok(result)
            }
            Expression::UnitConversion { value, target_unit } => {
                let val = self.evaluate_expr_with_steps(value, steps)?;
                steps.push(format!(
                    "Convert: {} to {}",
                    val.to_display_string(),
                    target_unit.display_name()
                ));

                // Clear any previous rate tracking before the conversion
                self.currency_db.clear_last_used_rate();

                let result = val.convert_to_unit_at_date(
                    target_unit,
                    &mut self.currency_db,
                    self.current_date_context.as_ref(),
                )?;

                // If a currency conversion was used, add rate info to steps.
                // For cross-rate (triangulated) conversions there may be multiple entries.
                for (from, to, rate_info) in self.currency_db.get_last_used_rates() {
                    steps.push(format!(
                        "Exchange rate: {}",
                        rate_info.format_for_display(from, to)
                    ));
                }

                steps.push(format!("= {}", result.to_display_string()));
                Ok(result)
            }
            Expression::Equality { left, right } => {
                if Self::expression_contains_variable(left)
                    || Self::expression_contains_variable(right)
                {
                    let result = if let Ok(solution) = linear_equation::solve(left, right) {
                        steps.push("Solve linear equation:".to_string());
                        steps.extend(solution.derivation_steps());
                        solution.to_value()
                    } else {
                        steps.push("Solve polynomial equation:".to_string());
                        let solution = polynomial_equation::solve(left, right)?;
                        steps.extend(solution.derivation_steps());
                        solution.to_value()
                    };
                    steps.push(format!("Solution: {}", result.to_display_string()));
                    return Ok(result);
                }

                steps.push("Check equality:".to_string());
                let left_val = self.evaluate_expr_with_steps(left, steps)?;
                let right_val = self.evaluate_expr_with_steps(right, steps)?;
                steps.push(format!(
                    "Compare: {} = {}",
                    left_val.to_display_string(),
                    right_val.to_display_string()
                ));
                let result = Value::boolean(left_val == right_val);
                steps.push(format!("= {}", result.to_display_string()));
                Ok(result)
            }
            Expression::Comparison { left, op, right } => {
                let left_val = self.evaluate_expr_with_steps(left, steps)?;
                let right_val = self.evaluate_expr_with_steps(right, steps)?;
                let operator = if *op == ComparisonOp::Compare {
                    "vs"
                } else {
                    op.symbol()
                };
                steps.push(format!(
                    "Compare: {} {} {}",
                    left_val.to_display_string(),
                    operator,
                    right_val.to_display_string()
                ));
                self.currency_db.clear_last_used_rate();
                let result = self.evaluate_comparison_values(&left_val, *op, &right_val)?;
                for (from, to, rate_info) in self.currency_db.get_last_used_rates() {
                    steps.push(format!(
                        "Exchange rate: {}",
                        rate_info.format_for_display(from, to)
                    ));
                }
                steps.push(format!("= {}", result.to_display_string()));
                Ok(result)
            }
        }
    }

    fn evaluate_comparison_values(
        &mut self,
        left: &Value,
        op: ComparisonOp,
        right: &Value,
    ) -> Result<Value, CalculatorError> {
        if op == ComparisonOp::Equal {
            return Ok(Value::boolean(
                self.compare_values(left, right)
                    .map_or_else(|_| left == right, |ordering| ordering == Ordering::Equal),
            ));
        }

        if op == ComparisonOp::NotEqual {
            return Ok(Value::boolean(
                self.compare_values(left, right)
                    .map_or_else(|_| left != right, |ordering| ordering != Ordering::Equal),
            ));
        }

        let ordering = self.compare_values(left, right)?;
        let result = match op {
            ComparisonOp::Less => Value::boolean(ordering == Ordering::Less),
            ComparisonOp::LessOrEqual => {
                Value::boolean(matches!(ordering, Ordering::Less | Ordering::Equal))
            }
            ComparisonOp::Greater => Value::boolean(ordering == Ordering::Greater),
            ComparisonOp::GreaterOrEqual => {
                Value::boolean(matches!(ordering, Ordering::Greater | Ordering::Equal))
            }
            ComparisonOp::Compare => Value::comparison_result(
                left.to_display_string(),
                Self::ordering_symbol(ordering),
                right.to_display_string(),
            ),
            ComparisonOp::Equal => unreachable!("handled before ordering comparison"),
            ComparisonOp::NotEqual => unreachable!("handled before ordering comparison"),
        };

        Ok(result)
    }

    fn compare_values(&mut self, left: &Value, right: &Value) -> Result<Ordering, CalculatorError> {
        if let (Some(left_seconds), Some(right_seconds)) = (
            Self::duration_seconds_for_comparison(left),
            Self::duration_seconds_for_comparison(right),
        ) {
            return left_seconds.partial_cmp(&right_seconds).ok_or_else(|| {
                CalculatorError::InvalidOperation(format!(
                    "cannot order {} and {}",
                    left.type_name(),
                    right.type_name()
                ))
            });
        }

        let (left, right) = self.normalize_comparison_values(left, right)?;

        if let (Some(left_rational), Some(right_rational)) =
            (left.to_rational(), right.to_rational())
        {
            return Ok(left_rational.cmp(&right_rational));
        }

        match (&left.kind, &right.kind) {
            (ValueKind::DateTime(left_dt), ValueKind::DateTime(right_dt)) => {
                Ok(left_dt.cmp(right_dt))
            }
            (
                ValueKind::Duration {
                    seconds: left_seconds,
                },
                ValueKind::Duration {
                    seconds: right_seconds,
                },
            ) => Ok(left_seconds.cmp(right_seconds)),
            _ => Err(CalculatorError::InvalidOperation(format!(
                "cannot order {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    fn normalize_comparison_values(
        &mut self,
        left: &Value,
        right: &Value,
    ) -> Result<(Value, Value), CalculatorError> {
        if left.unit == right.unit {
            return Ok((left.clone(), right.clone()));
        }

        if left.unit == Unit::None || right.unit == Unit::None {
            return Err(CalculatorError::InvalidOperation(format!(
                "cannot compare {} with {}",
                left.to_display_string(),
                right.to_display_string()
            )));
        }

        let date_context = self.current_date_context.clone();
        let right_converted = right
            .convert_to_unit_at_date(&left.unit, &mut self.currency_db, date_context.as_ref())
            .map_err(|_| {
                CalculatorError::InvalidOperation(format!(
                    "cannot compare {} with {}",
                    left.to_display_string(),
                    right.to_display_string()
                ))
            })?;

        Ok((left.clone(), right_converted))
    }

    fn duration_seconds_for_comparison(value: &Value) -> Option<f64> {
        match (&value.kind, &value.unit) {
            (ValueKind::Duration { seconds }, Unit::None) => Some(*seconds as f64),
            (ValueKind::Number(decimal), Unit::Duration(unit)) => {
                Some(unit.to_secs(decimal.to_f64()))
            }
            (ValueKind::Rational(rational), Unit::Duration(unit)) => {
                Some(unit.to_secs(rational.to_f64()))
            }
            _ => None,
        }
    }

    const fn ordering_symbol(ordering: Ordering) -> &'static str {
        match ordering {
            Ordering::Less => "<",
            Ordering::Equal => "=",
            Ordering::Greater => ">",
        }
    }

    /// Applies a binary operator to two already-evaluated values.
    ///
    /// Exposes the same routing the evaluator uses internally so callers can
    /// reuse currency-aware add/subtract or unit-aware multiply/divide
    /// outside of a full expression.
    pub fn apply_binary_op(
        &mut self,
        left: &Value,
        op: BinaryOp,
        right: &Value,
    ) -> Result<Value, CalculatorError> {
        match op {
            BinaryOp::Add => left.add_at_date(
                right,
                &mut self.currency_db,
                self.current_date_context.as_ref(),
            ),
            BinaryOp::Subtract => left.subtract_at_date(
                right,
                &mut self.currency_db,
                self.current_date_context.as_ref(),
            ),
            BinaryOp::Multiply => left.multiply(right),
            BinaryOp::Divide => left.divide(right),
            BinaryOp::Modulo => left.modulo(right),
        }
    }

    /// Evaluates an integrate function call: integrate(expr, var, lower, upper).
    ///
    /// Uses numerical integration (Simpson's rule) to compute the definite integral.
    /// Exposed so downstream consumers can reuse the same integrator when
    /// reconstructing or composing their own evaluators.
    #[allow(clippy::many_single_char_names)]
    pub fn evaluate_integrate(&mut self, args: &[Expression]) -> Result<Value, CalculatorError> {
        if args.len() != 4 {
            return Err(CalculatorError::invalid_args(
                "integrate",
                "expected 4 arguments: integrate(expr, var, lower, upper)",
            ));
        }

        // Second argument must be a variable name
        let var_name = match &args[1] {
            Expression::Variable(name) => name.clone(),
            _ => {
                return Err(CalculatorError::invalid_args(
                    "integrate",
                    "second argument must be a variable name (e.g., x)",
                ))
            }
        };

        // Evaluate lower and upper bounds
        let lower_val = self.evaluate_expr(&args[2])?;
        let lower = lower_val.as_decimal().ok_or_else(|| {
            CalculatorError::invalid_args("integrate", "lower bound must be numeric")
        })?;

        let upper_val = self.evaluate_expr(&args[3])?;
        let upper = upper_val.as_decimal().ok_or_else(|| {
            CalculatorError::invalid_args("integrate", "upper bound must be numeric")
        })?;

        let a = lower.to_f64();
        let b = upper.to_f64();

        // The expression to integrate
        let integrand = &args[0];

        // Numerical integration using Simpson's rule
        let n = 1000_usize; // Number of subdivisions
        let h = (b - a) / (n as f64);

        let mut sum = 0.0;

        // f(a) + f(b)
        sum += self.evaluate_at(integrand, &var_name, a)?.to_f64();
        sum += self.evaluate_at(integrand, &var_name, b)?.to_f64();

        // 4 * sum of odd terms
        for i in (1..n).step_by(2) {
            let x = (i as f64).mul_add(h, a);
            sum = 4.0_f64.mul_add(self.evaluate_at(integrand, &var_name, x)?.to_f64(), sum);
        }

        // 2 * sum of even terms
        for i in (2..n).step_by(2) {
            let x = (i as f64).mul_add(h, a);
            sum = 2.0_f64.mul_add(self.evaluate_at(integrand, &var_name, x)?.to_f64(), sum);
        }

        let result = sum * h / 3.0;

        if result.is_nan() {
            return Err(CalculatorError::domain("integration result is undefined"));
        }
        if result.is_infinite() {
            return Err(CalculatorError::Overflow);
        }

        Ok(Value::number(Decimal::from_f64(result)))
    }

    /// Evaluates an expression at a specific numeric value of `var_name`.
    ///
    /// Convenience wrapper around [`Self::evaluate_expr_with_var`] that
    /// coerces the result to a [`Decimal`]. Exposed for downstream consumers
    /// (e.g. custom integrators or plotters).
    pub fn evaluate_at(
        &mut self,
        expr: &Expression,
        var_name: &str,
        value: f64,
    ) -> Result<Decimal, CalculatorError> {
        let val = self.evaluate_expr_with_var(expr, var_name, Decimal::from_f64(value))?;
        val.as_decimal().ok_or_else(|| {
            CalculatorError::InvalidOperation("expected numeric result in integration".into())
        })
    }

    /// Evaluates an expression with a variable substitution.
    ///
    /// Replaces every occurrence of `var_name` in `expr` with `var_value`
    /// while evaluating. Exposed so callers can implement their own numeric
    /// integration, plotting, or symbolic substitution.
    pub fn evaluate_expr_with_var(
        &mut self,
        expr: &Expression,
        var_name: &str,
        var_value: Decimal,
    ) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit, .. } => {
                let rational = Rational::from_decimal(*value);
                Ok(Value::rational_with_unit(rational, unit.clone()))
            }
            Expression::DateTime(dt) => Ok(Value::datetime(dt.clone())),
            Expression::Now => Ok(Value::datetime(self.current_now())),
            Expression::Until(target) => {
                let target_val = self.evaluate_expr_with_var(target, var_name, var_value)?;
                let now = self.current_now();
                match &target_val.kind {
                    ValueKind::DateTime(target_dt) => {
                        let seconds = target_dt.signed_subtract_seconds(&now);
                        Ok(Value::duration(seconds))
                    }
                    _ => Err(CalculatorError::InvalidOperation(
                        "until requires a datetime expression".into(),
                    )),
                }
            }
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate_expr_with_var(left, var_name, var_value)?;
                let right_val = self.evaluate_expr_with_var(right, var_name, var_value)?;
                self.apply_binary_op(&left_val, *op, &right_val)
            }
            Expression::Negate(inner) => {
                let val = self.evaluate_expr_with_var(inner, var_name, var_value)?;
                Ok(val.negate())
            }
            Expression::Group(inner) => self.evaluate_expr_with_var(inner, var_name, var_value),
            Expression::AtTime { value, time } => {
                let _time_val = self.evaluate_expr_with_var(time, var_name, var_value)?;
                self.evaluate_expr_with_var(value, var_name, var_value)
            }
            Expression::FunctionCall { name, args } => {
                let name_lower = name.to_lowercase();

                // Nested integrate not supported
                if name_lower == "integrate" {
                    return Err(CalculatorError::invalid_args(
                        "integrate",
                        "nested integration is not supported",
                    ));
                }

                // Evaluate all arguments with variable substitution
                let mut arg_values = Vec::new();
                for arg in args {
                    let val = self.evaluate_expr_with_var(arg, var_name, var_value)?;
                    let decimal = val.as_decimal().ok_or_else(|| {
                        CalculatorError::invalid_args(name, "expected numeric argument")
                    })?;
                    arg_values.push(decimal);
                }

                let result = evaluate_function(name, &arg_values)?;
                Ok(Value::number(result))
            }
            Expression::Variable(name) => {
                if name == var_name {
                    // Keep as Decimal for integration (numerical computation)
                    Ok(Value::number(var_value))
                } else {
                    Err(CalculatorError::eval(format!("undefined variable: {name}")))
                }
            }
            Expression::Power { base, exponent } => {
                let base_val = self.evaluate_expr_with_var(base, var_name, var_value)?;
                let exp_val = self.evaluate_expr_with_var(exponent, var_name, var_value)?;
                evaluate_power(&base_val, &exp_val)
            }
            Expression::IndefiniteIntegral { .. } => Err(CalculatorError::invalid_args(
                "nested integration",
                "nested indefinite integrals are not supported",
            )),
            Expression::UnitConversion { value, target_unit } => {
                let val = self.evaluate_expr_with_var(value, var_name, var_value)?;
                val.convert_to_unit_at_date(
                    target_unit,
                    &mut self.currency_db,
                    self.current_date_context.as_ref(),
                )
            }
            Expression::Equality { left, right } => {
                let left_val = self.evaluate_expr_with_var(left, var_name, var_value)?;
                let right_val = self.evaluate_expr_with_var(right, var_name, var_value)?;
                Ok(Value::boolean(left_val == right_val))
            }
            Expression::Comparison { left, op, right } => {
                let left_val = self.evaluate_expr_with_var(left, var_name, var_value)?;
                let right_val = self.evaluate_expr_with_var(right, var_name, var_value)?;
                self.evaluate_comparison_values(&left_val, *op, &right_val)
            }
        }
    }
}
