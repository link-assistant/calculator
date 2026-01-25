//! Expression parser that combines all grammars.

use crate::error::CalculatorError;
use crate::grammar::{
    evaluate_function, is_math_function, DateTimeGrammar, Lexer, NumberGrammar, Token, TokenKind,
};
use crate::types::{BinaryOp, CurrencyDatabase, Decimal, Expression, Unit, Value};

/// Parser for calculator expressions.
#[derive(Debug, Default)]
pub struct ExpressionParser {
    number_grammar: NumberGrammar,
    datetime_grammar: DateTimeGrammar,
    currency_db: CurrencyDatabase,
}

impl ExpressionParser {
    /// Creates a new expression parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            number_grammar: NumberGrammar::new(),
            datetime_grammar: DateTimeGrammar::new(),
            currency_db: CurrencyDatabase::new(),
        }
    }

    /// Parses and evaluates an expression, returning the result, steps, and lino representation.
    pub fn parse_and_evaluate(
        &self,
        input: &str,
    ) -> Result<(Value, Vec<String>, String), CalculatorError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CalculatorError::EmptyInput);
        }

        // Try datetime subtraction pattern first: "(datetime) - (datetime)"
        if let Some(result) = self.try_parse_datetime_subtraction(input) {
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

    /// Tries to parse a datetime subtraction expression like "(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)".
    fn try_parse_datetime_subtraction(&self, input: &str) -> Option<(Value, Vec<String>, String)> {
        // Look for pattern: (datetime) - (datetime)
        let input = input.trim();

        // Check if it starts with '(' and contains '-'
        if !input.starts_with('(') || !input.contains('-') {
            return None;
        }

        // Try to find the matching closing paren for the first datetime
        let mut paren_depth = 0;
        let mut first_end = None;

        for (i, ch) in input.char_indices() {
            match ch {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        first_end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let first_end = first_end?;

        // Extract first datetime (without parens)
        let first_dt_str = &input[1..first_end];

        // Find the minus sign
        let rest = input[first_end + 1..].trim();
        if !rest.starts_with('-') {
            return None;
        }

        let second_part = rest[1..].trim();
        if !second_part.starts_with('(') || !second_part.ends_with(')') {
            return None;
        }

        // Extract second datetime (without parens)
        let second_dt_str = &second_part[1..second_part.len() - 1];

        // Try to parse both as datetimes
        let Ok(dt1) = self.datetime_grammar.parse(first_dt_str) else {
            return None;
        };

        let Ok(dt2) = self.datetime_grammar.parse(second_dt_str) else {
            return None;
        };

        // Calculate the difference
        let diff = dt1.subtract(&dt2);
        #[allow(clippy::cast_possible_wrap)]
        let seconds = diff.as_secs() as i64;

        let value = Value::duration(seconds);

        let steps = vec![
            format!("Parse first datetime: {dt1}"),
            format!("Parse second datetime: {dt2}"),
            format!("Calculate difference: {dt1} - {dt2}"),
            format!("Result: {}", value.to_display_string()),
        ];

        let lino = format!("((({}) - ({})))", first_dt_str.trim(), second_dt_str.trim());

        Some((value, steps, lino))
    }

    /// Parses an expression string into an Expression AST.
    pub fn parse(&self, input: &str) -> Result<Expression, CalculatorError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = TokenParser::new(&tokens, &self.number_grammar, input);
        parser.parse_expression()
    }

    /// Evaluates an expression.
    pub fn evaluate(&self, expr: &Expression) -> Result<Value, CalculatorError> {
        self.evaluate_expr(expr)
    }

    /// Evaluates an expression with step-by-step tracking.
    fn evaluate_with_steps(
        &self,
        expr: &Expression,
    ) -> Result<(Value, Vec<String>), CalculatorError> {
        let mut steps = Vec::new();

        steps.push(format!("Input expression: {expr}"));

        let result = self.evaluate_expr_with_steps(expr, &mut steps)?;

        steps.push(format!("Final result: {}", result.to_display_string()));

        Ok((result, steps))
    }

    fn evaluate_expr(&self, expr: &Expression) -> Result<Value, CalculatorError> {
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
                // For now, just evaluate the value
                // In the future, this would use the time for currency conversion
                let _time_val = self.evaluate_expr(time)?;
                self.evaluate_expr(value)
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
                self.evaluate_indefinite_integral(integrand, variable)
            }
        }
    }

    fn evaluate_expr_with_steps(
        &self,
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

                let result = self.apply_binary_op(&left_val, *op, &right_val)?;
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
                self.evaluate_expr_with_steps(value, steps)
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
                let result = self.evaluate_indefinite_integral(integrand, variable)?;
                steps.push(format!("= {}", result.to_display_string()));
                Ok(result)
            }
        }
    }

    fn apply_binary_op(
        &self,
        left: &Value,
        op: BinaryOp,
        right: &Value,
    ) -> Result<Value, CalculatorError> {
        match op {
            BinaryOp::Add => left.add(right, &self.currency_db),
            BinaryOp::Subtract => left.subtract(right, &self.currency_db),
            BinaryOp::Multiply => left.multiply(right),
            BinaryOp::Divide => left.divide(right),
        }
    }

    /// Evaluates an integrate function call: integrate(expr, var, lower, upper).
    ///
    /// Uses numerical integration (Simpson's rule) to compute the definite integral.
    #[allow(clippy::many_single_char_names)]
    fn evaluate_integrate(&self, args: &[Expression]) -> Result<Value, CalculatorError> {
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
        &self,
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
        &self,
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

    /// Evaluates an indefinite integral.
    ///
    /// For known special integrals, returns symbolic result.
    /// For others, returns an informational message.
    fn evaluate_indefinite_integral(
        &self,
        integrand: &Expression,
        variable: &str,
    ) -> Result<Value, CalculatorError> {
        // Check for known special integrals
        let symbolic_result = self.try_symbolic_integral(integrand, variable);

        if let Some(result) = symbolic_result {
            // Return a special value that indicates symbolic result
            // For now, we'll create an error with the symbolic result as a message
            // since the Value type doesn't support symbolic results yet
            let latex_result = self.symbolic_result_to_latex(&result);
            Err(CalculatorError::SymbolicResult {
                expression: format!("∫ {} d{}", integrand, variable),
                result,
                latex_input: format!("\\int {} \\, d{}", integrand.to_latex(), variable),
                latex_result,
            })
        } else {
            // For unknown integrals, provide a helpful message
            Err(CalculatorError::SymbolicResult {
                expression: format!("∫ {} d{}", integrand, variable),
                result: "Cannot compute symbolic result. Use definite integral with bounds: integrate(expr, var, lower, upper)".to_string(),
                latex_input: format!("\\int {} \\, d{}", integrand.to_latex(), variable),
                latex_result: "\\text{Use definite integral with bounds}".to_string(),
            })
        }
    }

    /// Tries to compute a symbolic integral for known special cases.
    fn try_symbolic_integral(&self, integrand: &Expression, variable: &str) -> Option<String> {
        // Pattern: sin(x)/x -> Si(x) + C (Sine Integral)
        if let Expression::Binary {
            left,
            op: BinaryOp::Divide,
            right,
        } = integrand
        {
            if let Expression::FunctionCall { name, args } = left.as_ref() {
                if name.to_lowercase() == "sin" && args.len() == 1 {
                    if let Expression::Variable(v) = &args[0] {
                        if let Expression::Variable(v2) = right.as_ref() {
                            if v == variable && v2 == variable {
                                return Some(format!("Si({}) + C", variable));
                            }
                        }
                    }
                }
            }
        }

        // Pattern: cos(x)/x -> Ci(x) + C (Cosine Integral)
        if let Expression::Binary {
            left,
            op: BinaryOp::Divide,
            right,
        } = integrand
        {
            if let Expression::FunctionCall { name, args } = left.as_ref() {
                if name.to_lowercase() == "cos" && args.len() == 1 {
                    if let Expression::Variable(v) = &args[0] {
                        if let Expression::Variable(v2) = right.as_ref() {
                            if v == variable && v2 == variable {
                                return Some(format!("Ci({}) + C", variable));
                            }
                        }
                    }
                }
            }
        }

        // Pattern: x^n -> x^(n+1)/(n+1) + C
        if let Expression::Power { base, exponent } = integrand {
            if let Expression::Variable(v) = base.as_ref() {
                if v == variable {
                    if let Expression::Number { value, .. } = exponent.as_ref() {
                        let n = value.to_f64();
                        if (n - (-1.0)).abs() > 1e-10 {
                            // Not x^(-1)
                            let new_exp = n + 1.0;
                            return Some(format!("{}^{}/({}) + C", variable, new_exp, new_exp));
                        } else {
                            // x^(-1) = 1/x -> ln|x| + C
                            return Some(format!("ln|{}| + C", variable));
                        }
                    }
                }
            }
        }

        // Pattern: just x -> x^2/2 + C
        if let Expression::Variable(v) = integrand {
            if v == variable {
                return Some(format!("{}²/2 + C", variable));
            }
        }

        // Pattern: constant -> constant * x + C
        if let Expression::Number { value, .. } = integrand {
            return Some(format!("{} * {} + C", value, variable));
        }

        // Pattern: sin(x) -> -cos(x) + C
        if let Expression::FunctionCall { name, args } = integrand {
            if args.len() == 1 {
                if let Expression::Variable(v) = &args[0] {
                    if v == variable {
                        match name.to_lowercase().as_str() {
                            "sin" => return Some(format!("-cos({}) + C", variable)),
                            "cos" => return Some(format!("sin({}) + C", variable)),
                            "exp" => return Some(format!("exp({}) + C", variable)),
                            _ => {}
                        }
                    }
                }
            }
        }

        None
    }

    /// Converts a symbolic result to LaTeX.
    fn symbolic_result_to_latex(&self, result: &str) -> String {
        // Basic conversions
        result
            .replace("Si(", "\\text{Si}(")
            .replace("Ci(", "\\text{Ci}(")
            .replace("ln|", "\\ln|")
            .replace("²", "^{2}")
            .replace(" + C", " + C")
    }
}

/// Internal token-based parser.
struct TokenParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    number_grammar: &'a NumberGrammar,
    #[allow(dead_code)]
    original_input: &'a str,
}

impl<'a> TokenParser<'a> {
    const fn new(
        tokens: &'a [Token],
        number_grammar: &'a NumberGrammar,
        original_input: &'a str,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            number_grammar,
            original_input,
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, CalculatorError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_multiplicative()?;

        while let Some(op) = self.match_additive_op() {
            let right = self.parse_multiplicative()?;
            left = Expression::binary(left, op, right);
        }

        // Check for "at" keyword
        if self.check_at() {
            self.advance(); // consume "at"
            let time = self.parse_primary()?;
            left = Expression::at_time(left, time);
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_power()?;

        while let Some(op) = self.match_multiplicative_op() {
            let right = self.parse_power()?;
            left = Expression::binary(left, op, right);
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_unary()?;

        // Power is right-associative: 2^3^4 = 2^(3^4)
        if self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?; // Right-associative recursion
            left = Expression::power(left, right);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, CalculatorError> {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::negate(expr));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, CalculatorError> {
        // Parenthesized expression
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::group(expr));
        }

        // Number with optional unit
        if let Some(TokenKind::Number(n)) = self.current_kind() {
            let num_str = n.clone();
            self.advance();

            // Check for power operator after number (without unit)
            // This is handled in parse_power, so we just need to handle units here

            // Check for unit (identifier following number that is not a function)
            let unit = if let Some(TokenKind::Identifier(id)) = self.current_kind() {
                // Don't treat function names as units
                if !is_math_function(id) && !self.peek_is_left_paren() {
                    let unit = self
                        .number_grammar
                        .parse_unit(id)
                        .unwrap_or_else(|_| Unit::Custom(id.clone()));
                    self.advance();
                    unit
                } else {
                    Unit::None
                }
            } else {
                Unit::None
            };

            let value = self.number_grammar.parse_number(&num_str)?;
            return Ok(Expression::number_with_unit(value, unit));
        }

        // Standalone identifier (could be a function call, unit, variable, or datetime part)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();
            self.advance();

            // Check if this is a function call (identifier followed by left paren)
            if self.check(&TokenKind::LeftParen) {
                return self.parse_function_call(&id);
            }

            // Check for natural integration syntax: "integrate <expr> d<var>"
            if id.to_lowercase() == "integrate" {
                return self.parse_natural_integral();
            }

            // If it looks like a datetime start (month name), try to parse more
            if DateTimeGrammar::looks_like_datetime(&id) {
                return self.try_parse_datetime_from_tokens(&id);
            }

            // Check if this is a math constant (pi, e)
            if is_math_function(&id) {
                // It's a constant like pi() or e() used without parens
                // Treat it as a zero-argument function call
                return Ok(Expression::function_call(id, vec![]));
            }

            // Allow single-letter identifiers as variables (for use in integrate, etc.)
            // Variables will be validated at evaluation time
            if id.len() == 1 && id.chars().next().unwrap().is_ascii_alphabetic() {
                return Ok(Expression::variable(id));
            }

            // Otherwise it's probably just an identifier/unit (which is an error in expression context)
            return Err(CalculatorError::parse(format!(
                "Unexpected identifier: {id}"
            )));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token: {:?}",
            self.current()
        )))
    }

    fn parse_function_call(&mut self, name: &str) -> Result<Expression, CalculatorError> {
        // We're positioned at the left paren
        self.expect(&TokenKind::LeftParen)?;

        let mut args = Vec::new();

        // Check for empty argument list
        if !self.check(&TokenKind::RightParen) {
            // Parse first argument
            args.push(self.parse_expression()?);

            // Parse remaining arguments
            while self.check(&TokenKind::Comma) {
                self.advance(); // consume comma
                args.push(self.parse_expression()?);
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(Expression::function_call(name, args))
    }

    fn try_parse_datetime_from_tokens(
        &mut self,
        first: &str,
    ) -> Result<Expression, CalculatorError> {
        // Collect tokens that might be part of a datetime
        let mut parts = vec![first.to_string()];

        // Look for patterns like: Jan 22, 2026 or Jan 27, 8:59am UTC
        // Collect: numbers, identifiers, colons, commas
        while !self.is_at_end() {
            match self.current_kind() {
                Some(TokenKind::Number(n)) => {
                    parts.push(n.clone());
                    self.advance();
                }
                Some(TokenKind::Identifier(id)) => {
                    parts.push(id.clone());
                    self.advance();
                }
                Some(TokenKind::Comma) => {
                    parts.push(",".to_string());
                    self.advance();
                }
                Some(TokenKind::Colon) => {
                    parts.push(":".to_string());
                    self.advance();
                }
                _ => break,
            }
        }

        let datetime_str = parts.join(" ").replace(" , ", ", ").replace(" : ", ":");

        match crate::types::DateTime::parse(&datetime_str) {
            Ok(dt) => Ok(Expression::DateTime(dt)),
            Err(e) => Err(e),
        }
    }

    /// Parses natural integral notation: "integrate <expr> d<var>"
    /// Examples:
    /// - integrate sin(x)/x dx
    /// - integrate x^2 dx
    fn parse_natural_integral(&mut self) -> Result<Expression, CalculatorError> {
        // We've already consumed "integrate", now we need to find the integrand and d<var>
        // Strategy: collect tokens until we find "d<var>" pattern (identifier starting with 'd')

        let start_pos = self.pos;
        let mut integrand_end_pos = None;
        let mut var_name = None;

        // Scan forward to find the d<var> pattern
        let mut scan_pos = self.pos;
        while scan_pos < self.tokens.len() {
            if let TokenKind::Identifier(id) = &self.tokens[scan_pos].kind {
                // Check if this is a differential notation like "dx", "dy", "dt"
                let id_lower = id.to_lowercase();
                if id_lower.starts_with('d') && id_lower.len() == 2 {
                    let var_char = id_lower.chars().nth(1).unwrap();
                    if var_char.is_ascii_alphabetic() {
                        integrand_end_pos = Some(scan_pos);
                        var_name = Some(var_char.to_string());
                        break;
                    }
                }
            }
            scan_pos += 1;
        }

        // If we didn't find d<var>, return an error with helpful message
        let (end_pos, var) = match (integrand_end_pos, var_name) {
            (Some(end), Some(v)) => (end, v),
            _ => {
                return Err(CalculatorError::parse(
                    "Invalid integration syntax. Expected: integrate <expression> d<var> (e.g., integrate sin(x)/x dx)"
                ));
            }
        };

        // Reset position and parse the integrand expression
        // We need a sub-parser that only parses up to the d<var> token
        self.pos = start_pos;

        // Parse the integrand by parsing an expression and stopping at the d<var>
        let integrand = self.parse_integrand_until(end_pos)?;

        // Now consume the d<var> token
        self.pos = end_pos;
        self.advance();

        Ok(Expression::indefinite_integral(integrand, var))
    }

    /// Parse an integrand expression up to (but not including) the position `until_pos`.
    fn parse_integrand_until(&mut self, until_pos: usize) -> Result<Expression, CalculatorError> {
        // Save the tokens after until_pos temporarily
        let original_len = self.tokens.len();

        // We need to be careful - parse_expression will consume tokens
        // We'll parse and then check we didn't go past until_pos
        let result = self.parse_integrand_expression(until_pos)?;

        // Verify we stopped at the right place
        if self.pos > until_pos {
            self.pos = until_pos;
        }

        let _ = original_len; // Suppress unused warning
        Ok(result)
    }

    /// Parse integrand with awareness of the boundary.
    fn parse_integrand_expression(
        &mut self,
        boundary: usize,
    ) -> Result<Expression, CalculatorError> {
        self.parse_integrand_additive(boundary)
    }

    fn parse_integrand_additive(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_multiplicative(boundary)?;

        while self.pos < boundary {
            if let Some(op) = self.match_additive_op() {
                if self.pos >= boundary {
                    // Put the operator back
                    self.pos -= 1;
                    break;
                }
                let right = self.parse_integrand_multiplicative(boundary)?;
                left = Expression::binary(left, op, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_integrand_multiplicative(
        &mut self,
        boundary: usize,
    ) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_power(boundary)?;

        while self.pos < boundary {
            if let Some(op) = self.match_multiplicative_op() {
                if self.pos >= boundary {
                    // Put the operator back
                    self.pos -= 1;
                    break;
                }
                let right = self.parse_integrand_power(boundary)?;
                left = Expression::binary(left, op, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_integrand_power(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        let mut left = self.parse_integrand_unary(boundary)?;

        if self.pos < boundary && self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_integrand_power(boundary)?;
            left = Expression::power(left, right);
        }

        Ok(left)
    }

    fn parse_integrand_unary(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        if self.pos < boundary && self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_integrand_unary(boundary)?;
            return Ok(Expression::negate(expr));
        }

        self.parse_integrand_primary(boundary)
    }

    fn parse_integrand_primary(&mut self, boundary: usize) -> Result<Expression, CalculatorError> {
        if self.pos >= boundary {
            return Err(CalculatorError::parse("Unexpected end of integrand"));
        }

        // Parenthesized expression
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expression::group(expr));
        }

        // Number
        if let Some(TokenKind::Number(n)) = self.current_kind() {
            let num_str = n.clone();
            self.advance();
            let value = self.number_grammar.parse_number(&num_str)?;
            return Ok(Expression::number(value));
        }

        // Identifier (function call or variable)
        if let Some(TokenKind::Identifier(id)) = self.current_kind() {
            let id = id.clone();
            self.advance();

            // Check if this is a function call
            if self.pos < boundary && self.check(&TokenKind::LeftParen) {
                return self.parse_function_call(&id);
            }

            // Check if this is a math constant
            if is_math_function(&id) {
                return Ok(Expression::function_call(id, vec![]));
            }

            // Single-letter identifier is a variable
            if id.len() == 1 && id.chars().next().unwrap().is_ascii_alphabetic() {
                return Ok(Expression::variable(id));
            }

            // Multi-letter identifier could be an implicit variable in integration context
            return Ok(Expression::variable(id));
        }

        Err(CalculatorError::parse(format!(
            "Unexpected token in integrand: {:?}",
            self.current()
        )))
    }

    fn match_additive_op(&mut self) -> Option<BinaryOp> {
        if self.check(&TokenKind::Plus) {
            self.advance();
            Some(BinaryOp::Add)
        } else if self.check(&TokenKind::Minus) {
            self.advance();
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    fn match_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.check(&TokenKind::Star) {
            self.advance();
            Some(BinaryOp::Multiply)
        } else if self.check(&TokenKind::Slash) {
            self.advance();
            Some(BinaryOp::Divide)
        } else {
            None
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current_kind()
            .is_some_and(|k| std::mem::discriminant(k) == std::mem::discriminant(kind))
    }

    fn check_at(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::At))
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current().map(|t| &t.kind)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    fn peek_is_left_paren(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::LeftParen))
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Eof) | None)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), CalculatorError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(CalculatorError::unexpected_token(
                &format!("{:?}", self.current_kind()),
                &format!("{kind:?}"),
                self.pos,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_number() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("42").unwrap();
        assert!(matches!(expr, Expression::Number { .. }));
    }

    #[test]
    fn test_parse_addition() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("2 + 3").unwrap();
        assert!(matches!(expr, Expression::Binary { .. }));
    }

    #[test]
    fn test_parse_currency() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("100 USD").unwrap();
        if let Expression::Number { value, unit } = expr {
            assert_eq!(value, Decimal::new(100));
            assert_eq!(unit, Unit::currency("USD"));
        } else {
            panic!("Expected Number expression");
        }
    }

    #[test]
    fn test_parse_currency_subtraction() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("84 USD - 34 EUR").unwrap();
        assert!(matches!(expr, Expression::Binary { .. }));
    }

    #[test]
    fn test_evaluate_simple() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2 + 3").unwrap();
        assert_eq!(value.to_display_string(), "5");
    }

    #[test]
    fn test_evaluate_multiplication() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("4 * 5").unwrap();
        assert_eq!(value.to_display_string(), "20");
    }

    #[test]
    fn test_evaluate_precedence() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2 + 3 * 4").unwrap();
        assert_eq!(value.to_display_string(), "14"); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_evaluate_parentheses() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("(2 + 3) * 4").unwrap();
        assert_eq!(value.to_display_string(), "20"); // (2 + 3) * 4 = 20
    }

    #[test]
    fn test_evaluate_negation() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("-5 + 3").unwrap();
        assert_eq!(value.to_display_string(), "-2");
    }

    #[test]
    fn test_datetime_subtraction() {
        let parser = ExpressionParser::new();
        let result = parser.parse_and_evaluate("(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
        assert!(result.is_ok());
        let (value, _, _) = result.unwrap();
        // Should be approximately 1 day, 20 hours, 8 minutes
        assert!(value.to_display_string().contains("day"));
    }

    #[test]
    fn test_lino_representation() {
        let parser = ExpressionParser::new();
        let (_, _, lino) = parser.parse_and_evaluate("84 USD - 34 EUR").unwrap();
        assert!(lino.contains("84 USD"));
        assert!(lino.contains("34 EUR"));
    }

    // New tests for math functions
    #[test]
    fn test_parse_function_call() {
        let parser = ExpressionParser::new();
        let expr = parser.parse("sin(0)").unwrap();
        assert!(matches!(expr, Expression::FunctionCall { .. }));
    }

    #[test]
    fn test_evaluate_sin() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("sin(0)").unwrap();
        assert_eq!(value.to_display_string(), "0");
    }

    #[test]
    fn test_evaluate_sqrt() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("sqrt(16)").unwrap();
        assert_eq!(value.to_display_string(), "4");
    }

    #[test]
    fn test_evaluate_power() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2^3").unwrap();
        assert_eq!(value.to_display_string(), "8");
    }

    #[test]
    fn test_evaluate_pi() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("pi()").unwrap();
        let pi = value.as_decimal().unwrap().to_f64();
        assert!((pi - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("2 + sin(0) * 3").unwrap();
        assert_eq!(value.to_display_string(), "2");
    }

    #[test]
    fn test_evaluate_nested_functions() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("sqrt(abs(-16))").unwrap();
        assert_eq!(value.to_display_string(), "4");
    }

    #[test]
    fn test_evaluate_function_with_expression() {
        let parser = ExpressionParser::new();
        let (value, _, _) = parser.parse_and_evaluate("sqrt(4 + 12)").unwrap();
        assert_eq!(value.to_display_string(), "4");
    }
}
