// meta-language:carried JavaScript export_statement (unsupported)
// | /** Parser for calculator expressions. */
// | export class ExpressionParser {
// |   /**
// |    * @param {{
// |    *   numberGrammar: NumberGrammar,
// |    *   datetimeGrammar: DateTimeGrammar,
// |    *   currencyDb: CurrencyDatabase,
// |    *   currentDateContext: DateTime | null,
// |    *   localOffsetSeconds: number | null,
// |    * }} fields
// |    */
// |   constructor({ numberGrammar, datetimeGrammar, currencyDb, currentDateContext, localOffsetSeconds }) {
// |     this.numberGrammar = numberGrammar;
// |     this.datetimeGrammar = datetimeGrammar;
// |     this._currencyDb = currencyDb;
// |     /** Current date context for historical currency conversions (set by AtTime expressions). */
// |     this.currentDateContext = currentDateContext;
// |     /**
// |      * The user's local timezone offset in seconds east of UTC, when known.
// |      *
// |      * When set, `now` and bare (timezone-less) times such as `12:30` are
// |      * interpreted in this local timezone instead of UTC. Explicit timezones
// |      * (e.g. `12:30 UTC`) are always honored regardless of this setting.
// |      */
// |     this._localOffsetSeconds = localOffsetSeconds;
// |   }
// |
// |   /** Creates a new expression parser. @returns {ExpressionParser} */
// |   static new() {
// |     return new ExpressionParser({
// |       numberGrammar: NumberGrammar.new(),
// |       datetimeGrammar: DateTimeGrammar.new(),
// |       currencyDb: CurrencyDatabase.new(),
// |       currentDateContext: null,
// |       localOffsetSeconds: null,
// |     });
// |   }
// |
// |   /** `Default::default()`. @returns {ExpressionParser} */
// |   static default() {
// |     return ExpressionParser.new();
// |   }
// |
// |   /** Returns a reference to the currency database. */
// |   currencyDb() {
// |     return this._currencyDb;
// |   }
// |
// |   /** Returns a mutable reference to the currency database. */
// |   currencyDbMut() {
// |     return this._currencyDb;
// |   }
// |
// |   /**
// |    * Parses and evaluates an expression, returning the result, steps, and lino representation.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {string} input
// |    * @returns {[Value, string[], string]}
// |    */
// |   parseAndEvaluate(input) {
// |     input = trim(input);
// |     if (input === '') {
// |       return err(CalculatorError.EmptyInput);
// |     }
// |
// |     this._currencyDb.clearLastUsedRate();
// |     const result = this.datetimeGrammar.tryParseDatetimeSubtraction(input, this._localOffsetSeconds);
// |     if (result !== null) {
// |       return result;
// |     }
// |
// |     const expr = this.parse(input);
// |     const lino = expr.toLino();
// |     const [value, steps] = this.evaluateWithSteps(expr);
// |
// |     return [value, steps, lino];
// |   }
// |
// |   /** @param {string} input @returns {Expression} */
// |   parseTokenized(input) {
// |     return this.parseLexer(input, Lexer.new(input));
// |   }
// |
// |   /** @param {string} input @returns {Expression} */
// |   parseTokenizedDecimalCommas(input) {
// |     return this.parseLexer(input, Lexer.withDecimalCommas(input));
// |   }
// |
// |   /**
// |    * @param {string} input
// |    * @param {Lexer} lexer
// |    * @returns {Expression}
// |    */
// |   parseLexer(input, lexer) {
// |     const tokens = lexer.tokenize();
// |     const parser = TokenParser.new(tokens, this.numberGrammar, input);
// |     const expr = parser.parseCompleteExpression();
// |     if (this._localOffsetSeconds !== null) {
// |       expr.applyLocalOffset(this._localOffsetSeconds);
// |     }
// |     return expr;
// |   }
// |
// |   /**
// |    * Evaluates an expression. Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @returns {Value}
// |    */
// |   evaluate(expr) {
// |     return kelvinResult(expr, this.evaluateExpr(expr));
// |   }
// |
// |   /**
// |    * Evaluates an expression with step-by-step tracking.
// |    *
// |    * Returns the final `Value` alongside the human-readable list of steps
// |    * the evaluator took. Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @returns {[Value, string[]]}
// |    */
// |   evaluateWithSteps(expr) {
// |     /** @type {string[]} */
// |     const steps = [];
// |
// |     steps.push(`Input expression: ${expr}`);
// |
// |     const result = kelvinResult(expr, this.evaluateExprWithSteps(expr, steps));
// |
// |     steps.push(`Final result: ${result.toDisplayString()}`);
// |
// |     return [result, steps];
// |   }
// |
// |   /**
// |    * `Until` evaluation shared by the silent evaluators.
// |    * @param {Value} targetVal
// |    * @returns {Value}
// |    */
// |   untilValue(targetVal) {
// |     const now = this.currentNow();
// |     if (targetVal.kind.tag === 'DateTime') {
// |       const seconds = targetVal.kind._0.signedSubtractSeconds(now);
// |       return Value.duration(seconds);
// |     }
// |     return err(CalculatorError.InvalidOperation(
// |       'until requires a datetime expression',
// |     ));
// |   }
// |
// |   /**
// |    * Evaluates `value` with `time` as the currency date context.
// |    * @param {Value} timeVal
// |    * @param {() => Value} evaluate
// |    * @returns {Value}
// |    */
// |   withDateContext(timeVal, evaluate) {
// |     // Extract the DateTime for use in currency conversions
// |     const dateContext = timeVal.kind.tag === 'DateTime' ? timeVal.kind._0 : null;
// |
// |     // Set the date context for this evaluation
// |     const oldContext = this.currentDateContext;
// |     this.currentDateContext = dateContext;
// |
// |     // Evaluate the value expression with the date context, then restore
// |     // the old context
// |     try {
// |       return evaluate();
// |     } finally {
// |       this.currentDateContext = oldContext;
// |     }
// |   }
// |
// |   /**
// |    * Evaluates an expression without step tracking.
// |    *
// |    * This is the silent counterpart of `evaluateWithSteps`. Throws
// |    * `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @returns {Value}
// |    */
// |   evaluateExpr(expr) {
// |     switch (expr.tag) {
// |       case 'Number':
// |         // Convert to Rational for exact arithmetic
// |         return Value.rationalWithUnit(Rational.fromDecimal(expr.value), expr.unit);
// |       case 'DateTime':
// |         return Value.datetime(expr._0);
// |       case 'Now':
// |       case 'Today':
// |         return Value.datetime(this.currentDate(expr));
// |       case 'Until':
// |         return this.untilValue(this.evaluateExpr(expr._0));
// |       case 'Binary': {
// |         const leftVal = this.evaluateExpr(expr.left);
// |         const rightVal = this.evaluateExpr(expr.right);
// |         return this.applyBinaryOp(leftVal, expr.op, rightVal);
// |       }
// |       case 'Negate':
// |         return this.evaluateExpr(expr._0).negate();
// |       case 'Group':
// |         return this.evaluateExpr(expr._0);
// |       case 'AtTime': {
// |         // Evaluate the time expression to get a DateTime
// |         const timeVal = this.evaluateExpr(expr.time);
// |         return this.withDateContext(timeVal, () => this.evaluateExpr(expr.value));
// |       }
// |       case 'FunctionCall': {
// |         // Special handling for integrate(expr, var, lower, upper)
// |         if (expr.name.toLowerCase() === 'integrate') {
// |           return this.evaluateIntegrate(expr.args);
// |         }
// |         const argValues = expr.args.map((arg) => this.evaluateExpr(arg));
// |         return evaluateFunctionValues(expr.name, argValues);
// |       }
// |       case 'Variable':
// |         // Variables should not appear in direct evaluation
// |         // They are only used in integration contexts
// |         return err(CalculatorError.eval(`undefined variable: ${expr._0}`));
// |       case 'Power': {
// |         const baseVal = this.evaluateExpr(expr.base);
// |         const expVal = this.evaluateExpr(expr.exponent);
// |         return evaluatePower(baseVal, expVal);
// |       }
// |       case 'IndefiniteIntegral':
// |         // Indefinite integrals return a symbolic result
// |         return evaluateIndefiniteIntegral(expr.integrand, expr.variable);
// |       case 'UnitConversion': {
// |         const val = this.evaluateExpr(expr.value);
// |         return val.convertToUnitAtDate(expr.targetUnit, this._currencyDb, this.currentDateContext);
// |       }
// |       case 'Equality': {
// |         if (expressionContainsVariable(expr.left) || expressionContainsVariable(expr.right)) {
// |           return solveEquation(expr.left, expr.right);
// |         }
// |         const leftVal = this.evaluateExpr(expr.left);
// |         const rightVal = this.evaluateExpr(expr.right);
// |         return Value.boolean(leftVal.equals(rightVal));
// |       }
// |       case 'Comparison': {
// |         const leftVal = this.evaluateExpr(expr.left);
// |         const rightVal = this.evaluateExpr(expr.right);
// |         return this.evaluateComparisonValues(leftVal, expr.op, rightVal);
// |       }
// |       default:
// |         return unreachable(`unknown expression ${expr.tag}`);
// |     }
// |   }
// |
// |   /**
// |    * Evaluates an expression, pushing human-readable steps into `steps`.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @param {string[]} steps
// |    * @returns {Value}
// |    */
// |   evaluateExprWithSteps(expr, steps) {
// |     switch (expr.tag) {
// |       case 'Number': {
// |         // Convert to Rational for exact arithmetic
// |         const val = Value.rationalWithUnit(Rational.fromDecimal(expr.value), expr.unit);
// |         steps.push(`Literal value: ${val.toDisplayString()}`);
// |         return val;
// |       }
// |       case 'DateTime': {
// |         const dt = expr._0;
// |         steps.push(`DateTime value: ${dt}`);
// |         const utcEquivalent = dt.utcEquivalentDisplay();
// |         if (utcEquivalent !== null) {
// |           steps.push(`UTC equivalent: ${utcEquivalent}`);
// |         }
// |         const dtVal = Value.datetime(dt);
// |         // For standalone datetime, show time from now
// |         const now = this.currentNow();
// |         const seconds = dt.signedSubtractSeconds(now);
// |         if (seconds > 0n) {
// |           steps.push(`Time until: ${Value.duration(seconds).toDisplayString()}`);
// |         } else if (seconds < 0n) {
// |           steps.push(`Time since: ${Value.duration(0n - seconds).toDisplayString()} ago`);
// |         }
// |         return dtVal;
// |       }
// |       case 'Now':
// |       case 'Today': {
// |         const date = this.currentDate(expr);
// |         const description = expr.tag === 'Now' ? 'Current time' : "Today's date";
// |         steps.push(`${description}: ${date}`);
// |         return Value.datetime(date);
// |       }
// |       case 'Until': {
// |         const targetVal = this.evaluateExprWithSteps(expr._0, steps);
// |         const now = this.currentNow();
// |         if (targetVal.kind.tag !== 'DateTime') {
// |           return err(CalculatorError.InvalidOperation(
// |             'until requires a datetime expression',
// |           ));
// |         }
// |         const seconds = targetVal.kind._0.signedSubtractSeconds(now);
// |         const duration = Value.duration(seconds);
// |         if (seconds >= 0n) {
// |           steps.push(`Time until ${targetVal.toDisplayString()}: ${duration.toDisplayString()}`);
// |         } else {
// |           steps.push(
// |             `Time since ${targetVal.toDisplayString()}: ${Value.duration(0n - seconds).toDisplayString()} ago`,
// |           );
// |         }
// |         return duration;
// |       }
// |       case 'Binary': {
// |         const leftVal = this.evaluateExprWithSteps(expr.left, steps);
// |         const rightVal = this.evaluateExprWithSteps(expr.right, steps);
// |
// |         steps.push(`Compute: ${leftVal.toDisplayString()} ${expr.op} ${rightVal.toDisplayString()}`);
// |
// |         // Clear any previous rate tracking before the operation
// |         this._currencyDb.clearLastUsedRate();
// |
// |         const result = this.applyBinaryOp(leftVal, expr.op, rightVal);
// |
// |         // If a currency conversion was used, add rate info to steps.
// |         pushRateSteps(this._currencyDb, steps);
// |
// |         steps.push(`= ${result.toDisplayString()}`);
// |         return result;
// |       }
// |       case 'Negate': {
// |         const val = this.evaluateExprWithSteps(expr._0, steps);
// |         const result = val.negate();
// |         steps.push(`Negate: -${val} = ${result}`);
// |         return result;
// |       }
// |       case 'Group':
// |         steps.push('Evaluate grouped expression:');
// |         return this.evaluateExprWithSteps(expr._0, steps);
// |       case 'AtTime': {
// |         const timeVal = this.evaluateExprWithSteps(expr.time, steps);
// |         steps.push(`At time: ${timeVal.toDisplayString()}`);
// |         return this.withDateContext(timeVal, () => this.evaluateExprWithSteps(expr.value, steps));
// |       }
// |       case 'FunctionCall': {
// |         // Special handling for integrate(expr, var, lower, upper)
// |         if (expr.name.toLowerCase() === 'integrate') {
// |           steps.push(`Numerical integration: ${expr.name}(...)`);
// |           const result = this.evaluateIntegrate(expr.args);
// |           steps.push(`= ${result.toDisplayString()}`);
// |           return result;
// |         }
// |
// |         /** @type {Value[]} */
// |         const argValues = [];
// |         /** @type {string[]} */
// |         const argDisplay = [];
// |         for (const arg of expr.args) {
// |           const val = this.evaluateExprWithSteps(arg, steps);
// |           argDisplay.push(val.toDisplayString());
// |           argValues.push(val);
// |         }
// |
// |         steps.push(`Call function: ${expr.name}(${argDisplay.join(', ')})`);
// |         const val = evaluateFunctionValues(expr.name, argValues);
// |         steps.push(`= ${val.toDisplayString()}`);
// |         return val;
// |       }
// |       case 'Variable':
// |         return err(CalculatorError.eval(`undefined variable: ${expr._0}`));
// |       case 'Power': {
// |         const baseVal = this.evaluateExprWithSteps(expr.base, steps);
// |         const expVal = this.evaluateExprWithSteps(expr.exponent, steps);
// |
// |         steps.push(`Compute: ${baseVal.toDisplayString()} ^ ${expVal.toDisplayString()}`);
// |
// |         const val = evaluatePower(baseVal, expVal);
// |         steps.push(`= ${val.toDisplayString()}`);
// |         return val;
// |       }
// |       case 'IndefiniteIntegral': {
// |         steps.push(`Indefinite integral: ∫ ${expr.integrand} d${expr.variable}`);
// |         const result = evaluateIndefiniteIntegral(expr.integrand, expr.variable);
// |         steps.push(`= ${result.toDisplayString()}`);
// |         return result;
// |       }
// |       case 'UnitConversion': {
// |         const val = this.evaluateExprWithSteps(expr.value, steps);
// |         steps.push(`Convert: ${val.toDisplayString()} to ${expr.targetUnit.conversionTargetName()}`);
// |
// |         // Clear any previous rate tracking before the conversion
// |         this._currencyDb.clearLastUsedRate();
// |
// |         const result = val.convertToUnitAtDate(expr.targetUnit, this._currencyDb, this.currentDateContext);
// |
// |         // If a currency conversion was used, add rate info to steps.
// |         pushRateSteps(this._currencyDb, steps);
// |
// |         steps.push(`= ${result.toDisplayString()}`);
// |         return result;
// |       }
// |       case 'Equality': {
// |         const { left, right } = expr;
// |         if (expressionContainsVariable(left) || expressionContainsVariable(right)) {
// |           let result;
// |           const linear = tryResult(() => linearEquation.solve(left, right));
// |           if (linear.ok) {
// |             steps.push('Solve linear equation:');
// |             steps.push(...linear.value.derivationSteps());
// |             result = linear.value.toValue();
// |           } else {
// |             steps.push('Solve polynomial equation:');
// |             const solution = polynomialEquation.solve(left, right);
// |             steps.push(...solution.derivationSteps());
// |             result = solution.toValue();
// |           }
// |           steps.push(`Solution: ${result.toDisplayString()}`);
// |           return result;
// |         }
// |
// |         steps.push('Check equality:');
// |         const leftVal = this.evaluateExprWithSteps(left, steps);
// |         const rightVal = this.evaluateExprWithSteps(right, steps);
// |         steps.push(`Compare: ${leftVal.toDisplayString()} = ${rightVal.toDisplayString()}`);
// |         const result = Value.boolean(leftVal.equals(rightVal));
// |         steps.push(`= ${result.toDisplayString()}`);
// |         return result;
// |       }
// |       case 'Comparison': {
// |         const leftVal = this.evaluateExprWithSteps(expr.left, steps);
// |         const rightVal = this.evaluateExprWithSteps(expr.right, steps);
// |         const operator = expr.op === ComparisonOp.Compare ? 'vs' : expr.op.symbol();
// |         steps.push(`Compare: ${leftVal.toDisplayString()} ${operator} ${rightVal.toDisplayString()}`);
// |         this._currencyDb.clearLastUsedRate();
// |         const result = this.evaluateComparisonValues(leftVal, expr.op, rightVal);
// |         pushRateSteps(this._currencyDb, steps);
// |         steps.push(`= ${result.toDisplayString()}`);
// |         return result;
// |       }
// |       default:
// |         return unreachable(`unknown expression ${expr.tag}`);
// |     }
// |   }
// |
// |   /**
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Value} left
// |    * @param {ComparisonOp} op
// |    * @param {Value} right
// |    * @returns {Value}
// |    */
// |   evaluateComparisonValues(left, op, right) {
// |     if (op === ComparisonOp.Equal) {
// |       const ordering = tryResult(() => this.compareValues(left, right));
// |       return Value.boolean(ordering.ok ? ordering.value === 0 : left.equals(right));
// |     }
// |
// |     if (op === ComparisonOp.NotEqual) {
// |       const ordering = tryResult(() => this.compareValues(left, right));
// |       return Value.boolean(ordering.ok ? ordering.value !== 0 : !left.equals(right));
// |     }
// |
// |     const ordering = this.compareValues(left, right);
// |     switch (op.tag) {
// |       case 'Less':
// |         return Value.boolean(ordering === -1);
// |       case 'LessOrEqual':
// |         return Value.boolean(ordering === -1 || ordering === 0);
// |       case 'Greater':
// |         return Value.boolean(ordering === 1);
// |       case 'GreaterOrEqual':
// |         return Value.boolean(ordering === 1 || ordering === 0);
// |       case 'Compare':
// |         return Value.comparisonResult(
// |           left.toDisplayString(),
// |           orderingSymbol(ordering),
// |           right.toDisplayString(),
// |         );
// |       default:
// |         return unreachable('handled before ordering comparison');
// |     }
// |   }
// |
// |   /**
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Value} left
// |    * @param {Value} right
// |    * @returns {Ordering}
// |    */
// |   compareValues(left, right) {
// |     const leftSeconds = durationSecondsForComparison(left);
// |     const rightSeconds = durationSecondsForComparison(right);
// |     if (leftSeconds !== null && rightSeconds !== null) {
// |       // `partial_cmp`: None for NaN.
// |       if (Number.isNaN(leftSeconds) || Number.isNaN(rightSeconds)) {
// |         return err(CalculatorError.InvalidOperation(
// |           `cannot order ${left.typeName()} and ${right.typeName()}`,
// |         ));
// |       }
// |       if (leftSeconds < rightSeconds) {
// |         return -1;
// |       }
// |       return leftSeconds > rightSeconds ? 1 : 0;
// |     }
// |
// |     [left, right] = this.normalizeComparisonValues(left, right);
// |
// |     const leftRational = left.toRational();
// |     const rightRational = right.toRational();
// |     if (leftRational !== null && rightRational !== null) {
// |       return leftRational.cmp(rightRational);
// |     }
// |
// |     if (left.kind.tag === 'DateTime' && right.kind.tag === 'DateTime') {
// |       return left.kind._0.cmp(right.kind._0);
// |     }
// |     if (left.kind.tag === 'Duration' && right.kind.tag === 'Duration') {
// |       const a = left.kind.seconds;
// |       const b = right.kind.seconds;
// |       if (a < b) {
// |         return -1;
// |       }
// |       return a > b ? 1 : 0;
// |     }
// |     return err(CalculatorError.InvalidOperation(
// |       `cannot order ${left.typeName()} and ${right.typeName()}`,
// |     ));
// |   }
// |
// |   /**
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Value} left
// |    * @param {Value} right
// |    * @returns {[Value, Value]}
// |    */
// |   normalizeComparisonValues(left, right) {
// |     if (eq(left.unit, right.unit)) {
// |       return [left, right];
// |     }
// |
// |     if (left.unit.tag === 'None' || right.unit.tag === 'None') {
// |       return err(CalculatorError.InvalidOperation(
// |         `cannot compare ${left.toDisplayString()} with ${right.toDisplayString()}`,
// |       ));
// |     }
// |
// |     const dateContext = this.currentDateContext;
// |     const converted = tryResult(
// |       () => right.convertToUnitAtDate(left.unit, this._currencyDb, dateContext),
// |     );
// |     if (!converted.ok) {
// |       return err(CalculatorError.InvalidOperation(
// |         `cannot compare ${left.toDisplayString()} with ${right.toDisplayString()}`,
// |       ));
// |     }
// |
// |     return [left, converted.value];
// |   }
// |
// |   /**
// |    * Applies a binary operator to two already-evaluated values.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Value} left
// |    * @param {BinaryOp} op
// |    * @param {Value} right
// |    * @returns {Value}
// |    */
// |   applyBinaryOp(left, op, right) {
// |     switch (op) {
// |       case BinaryOp.Add:
// |         return left.addAtDate(right, this._currencyDb, this.currentDateContext);
// |       case BinaryOp.Subtract:
// |         return left.subtractAtDate(right, this._currencyDb, this.currentDateContext);
// |       case BinaryOp.Multiply:
// |         return left.multiply(right);
// |       case BinaryOp.Divide:
// |         return left.divide(right);
// |       case BinaryOp.Modulo:
// |         return left.modulo(right);
// |       default:
// |         return unreachable(`unknown operator ${op.tag}`);
// |     }
// |   }
// |
// |   /**
// |    * Evaluates an integrate function call: integrate(expr, var, lower, upper).
// |    *
// |    * Uses numerical integration (Simpson's rule) to compute the definite integral.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Expression[]} args
// |    * @returns {Value}
// |    */
// |   evaluateIntegrate(args) {
// |     if (args.length !== 4) {
// |       return err(CalculatorError.invalidArgs(
// |         'integrate',
// |         'expected 4 arguments: integrate(expr, var, lower, upper)',
// |       ));
// |     }
// |
// |     // Second argument must be a variable name
// |     if (args[1].tag !== 'Variable') {
// |       return err(CalculatorError.invalidArgs(
// |         'integrate',
// |         'second argument must be a variable name (e.g., x)',
// |       ));
// |     }
// |     const varName = args[1]._0;
// |
// |     // Evaluate lower and upper bounds
// |     const lowerVal = this.evaluateExpr(args[2]);
// |     const lower = lowerVal.asDecimal()
// |       ?? err(CalculatorError.invalidArgs('integrate', 'lower bound must be numeric'));
// |
// |     const upperVal = this.evaluateExpr(args[3]);
// |     const upper = upperVal.asDecimal()
// |       ?? err(CalculatorError.invalidArgs('integrate', 'upper bound must be numeric'));
// |
// |     const a = lower.toF64();
// |     const b = upper.toF64();
// |
// |     // The expression to integrate
// |     const integrand = args[0];
// |
// |     // Numerical integration using Simpson's rule
// |     const n = 1000; // Number of subdivisions
// |     const h = (b - a) / n;
// |
// |     let sum = 0.0;
// |
// |     // f(a) + f(b)
// |     sum += this.evaluateAt(integrand, varName, a).toF64();
// |     sum += this.evaluateAt(integrand, varName, b).toF64();
// |
// |     // 4 * sum of odd terms
// |     for (let i = 1; i < n; i += 2) {
// |       const x = fma(i, h, a);
// |       sum = fma(4.0, this.evaluateAt(integrand, varName, x).toF64(), sum);
// |     }
// |
// |     // 2 * sum of even terms
// |     for (let i = 2; i < n; i += 2) {
// |       const x = fma(i, h, a);
// |       sum = fma(2.0, this.evaluateAt(integrand, varName, x).toF64(), sum);
// |     }
// |
// |     const result = sum * h / 3.0;
// |
// |     if (Number.isNaN(result)) {
// |       return err(CalculatorError.domain('integration result is undefined'));
// |     }
// |     if (!Number.isFinite(result)) {
// |       return err(CalculatorError.Overflow);
// |     }
// |
// |     return Value.number(Decimal.fromF64(result));
// |   }
// |
// |   /**
// |    * Evaluates an expression at a specific numeric value of `varName`.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @param {string} varName
// |    * @param {number} value
// |    * @returns {Decimal}
// |    */
// |   evaluateAt(expr, varName, value) {
// |     const val = this.evaluateExprWithVar(expr, varName, Decimal.fromF64(value));
// |     return val.asDecimal()
// |       ?? err(CalculatorError.InvalidOperation('expected numeric result in integration'));
// |   }
// |
// |   /**
// |    * Evaluates an expression with a variable substitution.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @param {string} varName
// |    * @param {Decimal} varValue
// |    * @returns {Value}
// |    */
// |   evaluateExprWithVar(expr, varName, varValue) {
// |     /** @param {Expression} e */
// |     const recurse = (e) => this.evaluateExprWithVar(e, varName, varValue);
// |     switch (expr.tag) {
// |       case 'Number':
// |         return Value.rationalWithUnit(Rational.fromDecimal(expr.value), expr.unit);
// |       case 'DateTime':
// |         return Value.datetime(expr._0);
// |       case 'Now':
// |       case 'Today':
// |         return Value.datetime(this.currentDate(expr));
// |       case 'Until':
// |         return this.untilValue(recurse(expr._0));
// |       case 'Binary': {
// |         const leftVal = recurse(expr.left);
// |         const rightVal = recurse(expr.right);
// |         return this.applyBinaryOp(leftVal, expr.op, rightVal);
// |       }
// |       case 'Negate':
// |         return recurse(expr._0).negate();
// |       case 'Group':
// |         return recurse(expr._0);
// |       case 'AtTime':
// |         recurse(expr.time);
// |         return recurse(expr.value);
// |       case 'FunctionCall': {
// |         // Nested integrate not supported
// |         if (expr.name.toLowerCase() === 'integrate') {
// |           return err(CalculatorError.invalidArgs(
// |             'integrate',
// |             'nested integration is not supported',
// |           ));
// |         }
// |         // Evaluate all arguments with variable substitution
// |         const argValues = expr.args.map(recurse);
// |         return evaluateFunctionValues(expr.name, argValues);
// |       }
// |       case 'Variable':
// |         if (expr._0 === varName) {
// |           // Keep as Decimal for integration (numerical computation)
// |           return Value.number(varValue);
// |         }
// |         return err(CalculatorError.eval(`undefined variable: ${expr._0}`));
// |       case 'Power': {
// |         const baseVal = recurse(expr.base);
// |         const expVal = recurse(expr.exponent);
// |         return evaluatePower(baseVal, expVal);
// |       }
// |       case 'IndefiniteIntegral':
// |         return err(CalculatorError.invalidArgs(
// |           'nested integration',
// |           'nested indefinite integrals are not supported',
// |         ));
// |       case 'UnitConversion': {
// |         const val = recurse(expr.value);
// |         return val.convertToUnitAtDate(expr.targetUnit, this._currencyDb, this.currentDateContext);
// |       }
// |       case 'Equality': {
// |         const leftVal = recurse(expr.left);
// |         const rightVal = recurse(expr.right);
// |         return Value.boolean(leftVal.equals(rightVal));
// |       }
// |       case 'Comparison': {
// |         const leftVal = recurse(expr.left);
// |         const rightVal = recurse(expr.right);
// |         return this.evaluateComparisonValues(leftVal, expr.op, rightVal);
// |       }
// |       default:
// |         return unreachable(`unknown expression ${expr.tag}`);
// |     }
// |   }
// |
// |   // --- expression_parser_locale.rs ---
// |
// |   /**
// |    * Parses an expression string into an Expression AST.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {string} input
// |    * @returns {Expression}
// |    */
// |   parse(input) {
// |     return locale.parse(this, input);
// |   }
// |
// |   /**
// |    * Parses the primary numeric convention and a decimal-comma alternative.
// |    * Throws `Err(CalculatorError)`.
// |    * @param {string} input
// |    * @returns {Expression[]}
// |    */
// |   parseInterpretations(input) {
// |     return locale.parseInterpretations(this, input);
// |   }
// |
// |   // --- expression_parser_timezone.rs ---
// |
// |   /** @param {number | null} offsetSeconds */
// |   setLocalOffsetSeconds(offsetSeconds) {
// |     timezone.setLocalOffsetSeconds(this, offsetSeconds);
// |   }
// |
// |   /** @returns {number | null} */
// |   localOffsetSeconds() {
// |     return timezone.localOffsetSeconds(this);
// |   }
// |
// |   /** @returns {DateTime} */
// |   currentNow() {
// |     return timezone.currentNow(this);
// |   }
// |
// |   /** @param {Expression} expression @returns {DateTime} */
// |   currentDate(expression) {
// |     return timezone.currentDate(this, expression);
// |   }
// | }
