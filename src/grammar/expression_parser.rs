//! Expression parser that combines all grammars.

use crate::error::CalculatorError;
use crate::grammar::token_parser::TokenParser;
use crate::grammar::{
    evaluate_function, evaluate_indefinite_integral, DateTimeGrammar, Lexer, NumberGrammar,
};
use crate::types::{BinaryOp, CurrencyDatabase, DateTime, Decimal, Expression, Value, ValueKind};

/// Parser for calculator expressions.
#[derive(Debug, Default)]
pub struct ExpressionParser {
    number_grammar: NumberGrammar,
    datetime_grammar: DateTimeGrammar,
    currency_db: CurrencyDatabase,
    /// Current date context for historical currency conversions (set by AtTime expressions).
    current_date_context: Option<DateTime>,
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
        }
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
        if let Some(result) = self.datetime_grammar.try_parse_datetime_subtraction(input) {
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
        parser.parse_expression()
    }

    /// Evaluates an expression.
    pub fn evaluate(&mut self, expr: &Expression) -> Result<Value, CalculatorError> {
        self.evaluate_expr(expr)
    }

    /// Evaluates an expression with step-by-step tracking.
    fn evaluate_with_steps(
        &mut self,
        expr: &Expression,
    ) -> Result<(Value, Vec<String>), CalculatorError> {
        let mut steps = Vec::new();

        steps.push(format!("Input expression: {expr}"));

        let result = self.evaluate_expr_with_steps(expr, &mut steps)?;

        steps.push(format!("Final result: {}", result.to_display_string()));

        Ok((result, steps))
    }

    fn evaluate_expr(&mut self, expr: &Expression) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit } => Ok(Value::number_with_unit(*value, unit.clone())),
            Expression::DateTime(dt) => Ok(Value::datetime(dt.clone())),
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

                let base_dec = base_val.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation("power base must be numeric".into())
                })?;
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
            Expression::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                // Indefinite integrals return a symbolic result
                // For now, we return an error directing users to use definite integrals for numeric results
                // or display the symbolic representation
                evaluate_indefinite_integral(integrand, variable)
            }
        }
    }

    fn evaluate_expr_with_steps(
        &mut self,
        expr: &Expression,
        steps: &mut Vec<String>,
    ) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit } => {
                let val = Value::number_with_unit(*value, unit.clone());
                steps.push(format!("Literal value: {}", val.to_display_string()));
                Ok(val)
            }
            Expression::DateTime(dt) => {
                steps.push(format!("DateTime value: {dt}"));
                Ok(Value::datetime(dt.clone()))
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

                // If a currency conversion was used, add rate info to steps
                if let Some((from, to, rate_info)) = self.currency_db.get_last_used_rate() {
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

                let base_dec = base_val.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation("power base must be numeric".into())
                })?;
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

                let val = Value::number(Decimal::from_f64(result));
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
        }
    }

    fn apply_binary_op(
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
        }
    }

    /// Evaluates an integrate function call: integrate(expr, var, lower, upper).
    ///
    /// Uses numerical integration (Simpson's rule) to compute the definite integral.
    #[allow(clippy::many_single_char_names)]
    fn evaluate_integrate(&mut self, args: &[Expression]) -> Result<Value, CalculatorError> {
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
            sum += 4.0 * self.evaluate_at(integrand, &var_name, x)?.to_f64();
        }

        // 2 * sum of even terms
        for i in (2..n).step_by(2) {
            let x = (i as f64).mul_add(h, a);
            sum += 2.0 * self.evaluate_at(integrand, &var_name, x)?.to_f64();
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

    /// Evaluates an expression with a variable substitution.
    fn evaluate_at(
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
    fn evaluate_expr_with_var(
        &mut self,
        expr: &Expression,
        var_name: &str,
        var_value: Decimal,
    ) -> Result<Value, CalculatorError> {
        match expr {
            Expression::Number { value, unit } => Ok(Value::number_with_unit(*value, unit.clone())),
            Expression::DateTime(dt) => Ok(Value::datetime(dt.clone())),
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
                    Ok(Value::number(var_value))
                } else {
                    Err(CalculatorError::eval(format!("undefined variable: {name}")))
                }
            }
            Expression::Power { base, exponent } => {
                let base_val = self.evaluate_expr_with_var(base, var_name, var_value)?;
                let exp_val = self.evaluate_expr_with_var(exponent, var_name, var_value)?;

                let base_dec = base_val.as_decimal().ok_or_else(|| {
                    CalculatorError::InvalidOperation("power base must be numeric".into())
                })?;
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
            Expression::IndefiniteIntegral { .. } => Err(CalculatorError::invalid_args(
                "nested integration",
                "nested indefinite integrals are not supported",
            )),
        }
    }
}
