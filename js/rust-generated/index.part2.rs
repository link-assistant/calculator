// meta-language:carried JavaScript export_statement (unsupported)
// | /** The main calculator struct. */
// | export class Calculator {
// |   /** @param {{ parser: ExpressionParser }} [fields] */
// |   constructor(fields) {
// |     /** @type {ExpressionParser} */
// |     this._parser = fields === undefined ? ExpressionParser.new() : fields.parser;
// |   }
// |
// |   /** Creates a new Calculator instance. */
// |   static new() {
// |     return new Calculator({ parser: ExpressionParser.new() });
// |   }
// |
// |   /** `Default`. */
// |   static default() {
// |     return new Calculator({ parser: ExpressionParser.default() });
// |   }
// |
// |   /**
// |    * Plans a calculation without executing it, returning a JSON string.
// |    *
// |    * Parses the expression to determine:
// |    * - Links notation interpretation (default and alternatives)
// |    * - Which rate sources are required (ECB, CBR, Crypto)
// |    * - Which currency codes are referenced
// |    * - Whether the expression contains live time references
// |    *
// |    * The worker should call this first, fetch the required rate sources,
// |    * then call `execute()` to get the actual result.
// |    * @param {string} input
// |    * @returns {string}
// |    */
// |   plan(input) {
// |     const p = this.planInternal(input);
// |     return toJsonString(p);
// |   }
// |
// |   /**
// |    * Executes a calculation, returning a JSON string with the full result.
// |    *
// |    * This is the same as `calculate()` but named to clarify the plan→execute pipeline.
// |    * The worker should call `plan()` first to determine required rate sources,
// |    * fetch them, then call `execute()`.
// |    * @param {string} input
// |    * @returns {string}
// |    */
// |   execute(input) {
// |     const result = this.calculateInternal(input);
// |     return toJsonString(result);
// |   }
// |
// |   /**
// |    * Calculates the result of an expression, returning a JSON string.
// |    *
// |    * Kept for backwards compatibility. Equivalent to `execute()`.
// |    * @param {string} input
// |    * @returns {string}
// |    */
// |   calculate(input) {
// |     return this.execute(input);
// |   }
// |
// |   /**
// |    * Sets the user's local timezone offset, in minutes east of UTC.
// |    *
// |    * From the browser, pass `-new Date().getTimezoneOffset()` (note the sign:
// |    * `getTimezoneOffset()` returns minutes *behind* UTC, so it must be negated).
// |    * For example, UTC+5:30 (India) is `330`, and UTC-5 (US Eastern) is `-300`.
// |    *
// |    * Once set, bare times such as `12:30` and the `now` keyword are interpreted
// |    * in this local timezone instead of UTC. Inputs with an explicit timezone
// |    * (e.g. `12:30 UTC`, `now UTC`) are always honored regardless of this setting.
// |    * @param {number} offsetMinutes
// |    */
// |   setTimezoneOffset(offsetMinutes) {
// |     this._parser.setLocalOffsetSeconds(offsetMinutes * 60);
// |   }
// |
// |   /**
// |    * Clears any previously configured local timezone offset, restoring the
// |    * default UTC interpretation for `now` and bare times.
// |    */
// |   clearTimezoneOffset() {
// |     this._parser.setLocalOffsetSeconds(null);
// |   }
// |
// |   /**
// |    * Returns the version of the calculator.
// |    * @returns {string}
// |    */
// |   static version() {
// |     return VERSION;
// |   }
// |
// |   /**
// |    * Updates exchange rates from API response. Returns the number of rates updated.
// |    * Args: `base` (e.g., "USD"), `date` (e.g., "2026-01-25"), `ratesJson` (e.g., `{"eur": 0.92}`).
// |    * @param {string} base
// |    * @param {string} date
// |    * @param {string} ratesJson
// |    * @returns {number}
// |    */
// |   updateRatesFromApi(base, date, ratesJson) {
// |     const rates = parseRatesJson(ratesJson);
// |     if (rates === null) {
// |       return 0;
// |     }
// |
// |     const baseUpper = base.toUpperCase();
// |     const timestamp = clock.now().toRfc3339();
// |     let count = 0;
// |
// |     for (const [target, rate] of rates) {
// |       const targetUpper = target.toUpperCase();
// |
// |       if (baseUpper === targetUpper) {
// |         continue;
// |       } // Skip same currency
// |
// |       const info = ExchangeRateInfo.new(rate, currencyApi.API_SOURCE, date).withFetchedAt(timestamp);
// |
// |       const currencyDb = this._parser.currencyDbMut();
// |       currencyDb.setRateWithInfo(baseUpper, targetUpper, info);
// |       currencyDb.setHistoricalRateWithInfo(baseUpper, targetUpper, date, info);
// |
// |       count += 1;
// |     }
// |
// |     return count;
// |   }
// |
// |   /**
// |    * Updates RUB exchange rates from the Central Bank of Russia (cbr.ru) API response.
// |    * Returns the number of rates updated.
// |    *
// |    * The CBR rates format: `{"usd": 76.63, "eur": 90.58, "inr": 0.842, ...}`
// |    * where each value represents "1 CURRENCY = X RUB".
// |    *
// |    * These rates take priority over ECB/Frankfurter rates for RUB conversions,
// |    * since CBR provides official RUB rates directly (no cross-rate needed).
// |    *
// |    * Args: `date` (e.g., "2026-02-25"), `ratesJson` (currency_code → RUB amount).
// |    * @param {string} date
// |    * @param {string} ratesJson
// |    * @returns {number}
// |    */
// |   updateCbrRatesFromApi(date, ratesJson) {
// |     const rates = parseRatesJson(ratesJson);
// |     if (rates === null) {
// |       return 0;
// |     }
// |
// |     const timestamp = clock.now().toRfc3339();
// |     let count = 0;
// |
// |     for (const [currency, rubPerUnit] of rates) {
// |       const currencyUpper = currency.toUpperCase();
// |
// |       // Skip RUB itself
// |       if (currencyUpper === 'RUB') {
// |         continue;
// |       }
// |
// |       // Store: 1 CURRENCY = rub_per_unit RUB
// |       const info = ExchangeRateInfo.new(rubPerUnit, currencyApi.CBR_API_SOURCE, date).withFetchedAt(timestamp);
// |
// |       const currencyDb = this._parser.currencyDbMut();
// |       currencyDb.setRateWithInfo(currencyUpper, 'RUB', info);
// |       currencyDb.setHistoricalRateWithInfo(currencyUpper, 'RUB', date, info);
// |
// |       count += 1;
// |     }
// |
// |     return count;
// |   }
// |
// |   /**
// |    * Updates cryptocurrency exchange rates from API response.
// |    * Returns the number of rates updated.
// |    *
// |    * Args: `base` (fiat currency, e.g., "USD"), `date` (e.g., "2026-01-25"),
// |    * `ratesJson` (e.g., `{"TON": 5.42, "BTC": 95000.0}`).
// |    * @param {string} base
// |    * @param {string} date
// |    * @param {string} ratesJson
// |    * @returns {number}
// |    */
// |   updateCryptoRatesFromApi(base, date, ratesJson) {
// |     const rates = parseRatesJson(ratesJson);
// |     if (rates === null) {
// |       return 0;
// |     }
// |
// |     const baseUpper = base.toUpperCase();
// |     const timestamp = clock.now().toRfc3339();
// |     let count = 0;
// |
// |     for (const [ticker, price] of rates) {
// |       const tickerUpper = ticker.toUpperCase();
// |       // Store rate as: 1 ticker = price base_currency
// |       const info = ExchangeRateInfo.new(price, cryptoApi.COINGECKO_SOURCE, date).withFetchedAt(timestamp);
// |
// |       const currencyDb = this._parser.currencyDbMut();
// |       currencyDb.setRateWithInfo(tickerUpper, baseUpper, info);
// |       currencyDb.setHistoricalRateWithInfo(tickerUpper, baseUpper, date, info);
// |
// |       count += 1;
// |     }
// |
// |     return count;
// |   }
// |
// |   /**
// |    * Loads historical exchange rates from a consolidated .lino format.
// |    * Returns the number of rates loaded (0 if parsing failed).
// |    *
// |    * Supports both the new format (conversion/rates) and legacy format (rates/data).
// |    * Used by the web worker to populate historical CBR rate data from local .lino files.
// |    * @param {string} content
// |    * @returns {number}
// |    */
// |   loadRatesFromConsolidatedLino(content) {
// |     return ok(() => this.loadRatesFromConsolidatedLinoImpl(content)) ?? 0;
// |   }
// |
// |   /**
// |    * Internal planning method — parses expression and determines requirements.
// |    * @param {string} input
// |    * @returns {import('./plan.js').CalculationPlan}
// |    */
// |   planInternal(input) {
// |     input = trim(input);
// |     if (input === '') {
// |       return plan.emptyPlan();
// |     }
// |
// |     const parsed = tryResult(() => this._parser.parseInterpretations(input));
// |     if (parsed.ok) {
// |       const interpretations = parsed.value;
// |       if (interpretations.length > 0) {
// |         const p = plan.createPlan(input, interpretations[0]);
// |         p.alternativeLino = Calculator.combinedAlternativeLino(interpretations);
// |         return p;
// |       }
// |       return plan.errorPlan(input, 'Parse error: no parseable interpretation');
// |     }
// |     return plan.errorPlan(input, parsed.error.toString());
// |   }
// |
// |   /**
// |    * Internal calculation method that returns a proper Result type.
// |    * @param {string} input
// |    * @returns {CalculationResult}
// |    */
// |   calculateInternal(input) {
// |     // Try to parse the expression to generate alternative interpretations
// |     // and detect live time expressions before evaluation.
// |     const parsedInterpretations = ok(() => this._parser.parseInterpretations(input));
// |     const alternatives = parsedInterpretations === null
// |       ? null
// |       : Calculator.combinedAlternativeLino(parsedInterpretations);
// |     const first = parsedInterpretations === null ? undefined : parsedInterpretations[0];
// |     const isLiveTime = first !== undefined && first.containsLiveTime();
// |
// |     let result;
// |     const evaluated = tryResult(() => this._parser.parseAndEvaluate(input));
// |     if (evaluated.ok) {
// |       const [value, steps, lino] = evaluated.value;
// |       result = CalculationResult.successWithValue(value, lino, steps);
// |       // Set is_live_time for any datetime result so the frontend
// |       // auto-refreshes the "Time since/until" countdown display.
// |       const valueIsDatetime = value.kind.tag === 'DateTime';
// |       if (isLiveTime || valueIsDatetime) {
// |         result.isLiveTime = true;
// |       }
// |     } else if (evaluated.error.tag === 'SymbolicResult') {
// |       const e = evaluated.error;
// |       // Generate plot data for the integrand function
// |       const plotData = this.generatePlotDataForIntegral(input);
// |       result = CalculationResult.symbolic(e.expression, e.result, e.latexInput, e.latexResult, plotData);
// |     } else {
// |       result = CalculationResult.failureWithI18n(evaluated.error, input);
// |     }
// |
// |     // Attach alternative interpretations if available
// |     result.alternativeLino = alternatives;
// |
// |     return result;
// |   }
// |
// |   /**
// |    * @param {Expression[]} interpretations
// |    * @returns {string[] | null}
// |    */
// |   static combinedAlternativeLino(interpretations) {
// |     if (interpretations.length === 0) {
// |       return null;
// |     }
// |     const alternatives = [interpretations[0].toLino()];
// |
// |     for (const expr of interpretations) {
// |       const exprAlternatives = expr.alternativeLino();
// |       if (exprAlternatives !== null) {
// |         for (const lino of exprAlternatives) {
// |           Calculator.pushUniqueLino(alternatives, lino);
// |         }
// |       } else {
// |         Calculator.pushUniqueLino(alternatives, expr.toLino());
// |       }
// |     }
// |
// |     return alternatives.length > 1 ? alternatives : null;
// |   }
// |
// |   /**
// |    * @param {string[]} alternatives
// |    * @param {string} lino
// |    */
// |   static pushUniqueLino(alternatives, lino) {
// |     if (!alternatives.includes(lino)) {
// |       alternatives.push(lino);
// |     }
// |   }
// |
// |   /**
// |    * Generates plot data for an integral expression.
// |    * @param {string} input
// |    * @returns {PlotData | null}
// |    */
// |   generatePlotDataForIntegral(input) {
// |     // Try to parse and extract the integrand for plotting
// |     const expr = ok(() => this._parser.parse(input));
// |     if (expr === null) {
// |       return null;
// |     }
// |
// |     if (expr.tag === 'IndefiniteIntegral') {
// |       const { integrand, variable } = expr;
// |       // Generate plot points for the integrand
// |       const xValues = [];
// |       const yValues = [];
// |
// |       // Generate points from -10 to 10 with 200 steps
// |       const numPoints = 200;
// |       const xMin = -10.0;
// |       const xMax = 10.0;
// |       const step = (xMax - xMin) / numPoints;
// |
// |       for (let i = 0; i <= numPoints; i++) {
// |         const x = fma(i, step, xMin);
// |
// |         // Skip x = 0 for functions like sin(x)/x to avoid division issues
// |         if (Math.abs(x) < 1e-10) {
// |           // For sin(x)/x, the limit at x=0 is 1
// |           xValues.push(x);
// |           yValues.push(1.0);
// |           continue;
// |         }
// |
// |         // Try to evaluate the integrand at this point
// |         const yVal = ok(() => this.evaluateAtPoint(integrand, variable, x));
// |         if (yVal !== null && Number.isFinite(yVal)) {
// |           xValues.push(x);
// |           yValues.push(yVal);
// |         }
// |       }
// |
// |       if (xValues.length > 0) {
// |         return new PlotData({
// |           xValues,
// |           yValues,
// |           label: integrand.toString(),
// |           xLabel: variable,
// |           yLabel: `f(${variable})`,
// |         });
// |       }
// |     }
// |
// |     return null;
// |   }
// |
// |   /**
// |    * Evaluates an expression at a specific point. Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @param {string} variable
// |    * @param {number} value
// |    * @returns {number}
// |    */
// |   evaluateAtPoint(expr, variable, value) {
// |     const substituted = Calculator.substituteVariable(expr, variable, value);
// |     const result = this._parser.evaluate(substituted);
// |     const d = result.asDecimal();
// |     if (d === null) {
// |       return err(CalculatorError.eval('Expected numeric result'));
// |     }
// |     return d.toF64();
// |   }
// |
// |   /**
// |    * Substitutes a variable with a numeric value in an expression
// |    * (`src/substitution.rs`).
// |    * @param {Expression} expr
// |    * @param {string} variable
// |    * @param {number} value
// |    * @returns {Expression}
// |    */
// |   static substituteVariable(expr, variable, value) {
// |     return substituteVariable(expr, variable, value);
// |   }
// |
// |   /**
// |    * Parses an expression without evaluating it. Throws `Err(CalculatorError)`.
// |    * @param {string} input
// |    * @returns {Expression}
// |    */
// |   parse(input) {
// |     return this._parser.parse(input);
// |   }
// |
// |   /**
// |    * Evaluates a parsed expression. Throws `Err(CalculatorError)`.
// |    * @param {Expression} expr
// |    * @returns {Value}
// |    */
// |   evaluate(expr) {
// |     return this._parser.evaluate(expr);
// |   }
// |
// |   /**
// |    * Returns the underlying `ExpressionParser`.
// |    *
// |    * Lets downstream consumers reach the parser's lower-level methods —
// |    * `evaluateWithSteps`, `evaluateExpr`, `applyBinaryOp`, and friends —
// |    * without rebuilding the parser state (currency database, current date
// |    * context, …).
// |    * @returns {ExpressionParser}
// |    */
// |   parser() {
// |     return this._parser;
// |   }
// |
// |   /**
// |    * Returns the underlying `ExpressionParser` (mutable borrow in Rust).
// |    * @returns {ExpressionParser}
// |    */
// |   parserMut() {
// |     return this._parser;
// |   }
// |
// |   /**
// |    * Calculates an expression and returns every intermediate artefact.
// |    *
// |    * Unlike `calculateInternal`, which returns a UI-shaped `CalculationResult`
// |    * with display strings only, this method gives a downstream consumer
// |    * everything it needs to either render the result itself or replay the
// |    * calculation: the parsed AST, the structured value, the raw step list and
// |    * the Links Notation interpretation. Throws `Err(CalculatorError)`.
// |    * @param {string} input
// |    * @returns {[Expression, Value, string[], string]}
// |    */
// |   calculateWithValue(input) {
// |     input = trim(input);
// |     if (input === '') {
// |       return err(CalculatorError.EmptyInput);
// |     }
// |     const expr = this._parser.parse(input);
// |     const lino = expr.toLino();
// |     const [value, steps] = this._parser.evaluateWithSteps(expr);
// |     return [expr, value, steps, lino];
// |   }
// |
// |   /**
// |    * Loads a historical exchange rate from .lino format content.
// |    * Throws `Err(string)`.
// |    *
// |    * The .lino format for rates:
// |    * ```text
// |    * rate:
// |    *   from USD
// |    *   to EUR
// |    *   value 0.92
// |    *   date 2026-01-25
// |    *   source 'frankfurter.dev (ECB)'
// |    * ```
// |    * @param {string} content
// |    */
// |   loadRateFromLino(content) {
// |     let fromCurrency = null;
// |     let toCurrency = null;
// |     let value = null;
// |     let date = null;
// |     let source = null;
// |
// |     for (const raw of lines(content)) {
// |       const line = trim(raw);
// |       if (line === '' || line === 'rate:') {
// |         continue;
// |       }
// |
// |       let rest;
// |       if ((rest = stripPrefix(line, 'from ')) !== null) {
// |         fromCurrency = trim(rest).toUpperCase();
// |       } else if ((rest = stripPrefix(line, 'to ')) !== null) {
// |         toCurrency = trim(rest).toUpperCase();
// |       } else if ((rest = stripPrefix(line, 'value ')) !== null) {
// |         const r = rest;
// |         value = ok(() => parseF64(trim(r)));
// |       } else if ((rest = stripPrefix(line, 'date ')) !== null) {
// |         date = trim(rest);
// |       } else if ((rest = stripPrefix(line, 'source ')) !== null) {
// |         // Remove quotes from source
// |         source = unquoteSource(rest);
// |       }
// |     }
// |
// |     if (fromCurrency === null) return err("Missing 'from' currency");
// |     if (toCurrency === null) return err("Missing 'to' currency");
// |     if (value === null) return err("Missing 'value'");
// |     if (date === null) return err("Missing 'date'");
// |     const rateSource = source ?? 'unknown';
// |
// |     // Create ExchangeRateInfo and add to the database
// |     const rateInfo = ExchangeRateInfo.new(value, rateSource, date);
// |
// |     this._parser.currencyDbMut().setHistoricalRateWithInfo(fromCurrency, toCurrency, date, rateInfo);
// |   }
// |
// |   /**
// |    * Loads multiple historical exchange rates from a batch of .lino content.
// |    * Each rate should be separated by double newlines or start with "rate:".
// |    * @param {string[]} contents
// |    * @returns {number}
// |    */
// |   loadRatesBatch(contents) {
// |     let loaded = 0;
// |     for (const content of contents) {
// |       if (tryResult(() => this.loadRateFromLino(content)).ok) {
// |         loaded += 1;
// |       }
// |       // Silently skip invalid rate files
// |     }
// |     return loaded;
// |   }
// |
// |   /**
// |    * Throws `Err(string)` when no rate was loaded.
// |    * @param {string} content
// |    * @returns {number}
// |    */
// |   loadRatesFromConsolidatedLinoImpl(content) {
// |     let fromCurrency = null;
// |     let toCurrency = null;
// |     let source = null;
// |     let inDataSection = false;
// |     let loaded = 0;
// |
// |     for (const line of lines(content)) {
// |       const trimmed = trim(line);
// |
// |       // Skip empty lines
// |       if (trimmed === '') {
// |         continue;
// |       }
// |
// |       // Handle 'rates:' based on context:
// |       // - If we haven't parsed from_currency yet, it's a root marker (legacy format), skip it
// |       // - If we have parsed from_currency, it's the data section marker (new format)
// |       if (trimmed === 'rates:') {
// |         if (fromCurrency !== null) {
// |           // New format: 'rates:' marks the start of data section
// |           inDataSection = true;
// |         }
// |         // Either way, continue to next line (skip the 'rates:' line itself)
// |         continue;
// |       }
// |
// |       // Skip root marker for new format
// |       if (trimmed === 'conversion:') {
// |         continue;
// |       }
// |
// |       // Check for legacy data section marker
// |       if (trimmed === 'data:') {
// |         inDataSection = true;
// |         continue;
// |       }
// |
// |       if (inDataSection) {
// |         // Parse date and value: "2021-01-25 0.8234"
// |         const parts = splitWhitespace(trimmed);
// |         if (parts.length >= 2 && fromCurrency !== null && toCurrency !== null) {
// |           const date = parts[0];
// |           const value = ok(() => parseF64(parts[1]));
// |           if (value !== null) {
// |             const rateSource = source ?? 'unknown';
// |             const rateInfo = ExchangeRateInfo.new(value, rateSource, date);
// |             this._parser.currencyDbMut().setHistoricalRateWithInfo(fromCurrency, toCurrency, date, rateInfo);
// |             loaded += 1;
// |           }
// |         }
// |       } else {
// |         // Parse header section
// |         let rest;
// |         if ((rest = stripPrefix(trimmed, 'from ')) !== null) {
// |           fromCurrency = trim(rest).toUpperCase();
// |         } else if ((rest = stripPrefix(trimmed, 'to ')) !== null) {
// |           toCurrency = trim(rest).toUpperCase();
// |         } else if ((rest = stripPrefix(trimmed, 'source ')) !== null) {
// |           // Remove quotes from source
// |           source = unquoteSource(rest);
// |         }
// |       }
// |     }
// |
// |     if (loaded === 0) {
// |       return err('No rates loaded from consolidated file');
// |     }
// |     return loaded;
// |   }
// | }
