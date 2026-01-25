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
}
