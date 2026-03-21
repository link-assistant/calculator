export interface PlotData {
  x_values: number[];
  y_values: number[];
  label: string;
  x_label: string;
  y_label: string;
}

/**
 * Different notation formats for repeating decimals.
 */
export interface RepeatingDecimalFormats {
  /** Vinculum notation with overline: 0.3̅ */
  vinculum: string;
  /** Parenthesis notation: 0.(3) */
  parenthesis: string;
  /** Ellipsis notation: 0.333... */
  ellipsis: string;
  /** LaTeX notation: 0.\overline{3} */
  latex: string;
  /** Fraction representation: 1/3 */
  fraction: string;
}

/**
 * Error information for i18n support.
 * Contains the translation key and optional interpolation parameters.
 */
export interface ErrorInfo {
  /** The translation key for this error (e.g., "errors.divisionByZero"). */
  key: string;
  /** Parameters for interpolation in the translated message. */
  params?: Record<string, string>;
}

/**
 * A single calculation step with i18n support.
 */
export interface CalculationStep {
  /** The translation key for this step type. */
  key: string;
  /** Parameters for interpolation in the translated message. */
  params?: Record<string, string>;
  /** The raw (English) text for fallback. */
  text: string;
}

/**
 * A calculation plan returned by calculator.plan() before execution.
 *
 * Contains the parsed interpretation and data requirements so the worker
 * can fetch only the needed rate sources before executing.
 */
export interface CalculationPlan {
  /** The input expression, trimmed. */
  expression: string;
  /** Links notation interpretation (default). */
  lino_interpretation: string;
  /** Alternative interpretations, if any. First is the default. */
  alternative_lino?: string[];
  /** Rate sources required for execution (e.g., ["ecb", "cbr", "crypto"]). */
  required_sources: string[];
  /** Currency codes found in the expression. */
  currencies: string[];
  /** Whether the expression contains live time references. */
  is_live_time: boolean;
  /** Whether parsing succeeded. */
  success: boolean;
  /** Error message if parsing failed. */
  error?: string;
}

export interface CalculationResult {
  result: string;
  lino_interpretation: string;
  /** Alternative links notation interpretations the user can switch between. */
  alternative_lino?: string[];
  /** Step-by-step explanation (raw text for backwards compatibility). */
  steps: string[];
  /** Step-by-step explanation with i18n support. */
  steps_i18n?: CalculationStep[];
  success: boolean;
  /** Error message (raw text for backwards compatibility). */
  error?: string;
  /** Error information for i18n support. */
  error_info?: ErrorInfo;
  issue_link?: string;
  latex_input?: string;
  latex_result?: string;
  is_symbolic?: boolean;
  plot_data?: PlotData;
  /** Repeating decimal notations (if result is a repeating decimal). */
  repeating_decimal?: RepeatingDecimalFormats;
  /** Fraction representation (if applicable). */
  fraction?: string;
  /** Whether the result represents a live (auto-updating) time expression.
   * When true, the frontend should periodically re-calculate to keep the time current. */
  is_live_time?: boolean;
}
