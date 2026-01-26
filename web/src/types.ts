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

export interface CalculationResult {
  result: string;
  lino_interpretation: string;
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
}
