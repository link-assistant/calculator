// Utility functions for the worker, extracted for testability.

/**
 * Check if a calculation result failed due to missing exchange rates.
 */
export function isMissingRatesError(result: { success: boolean; error?: string; error_info?: { key: string } }): boolean {
  if (result.success) return false;
  // Check error_info key for currency-related errors
  if (result.error_info?.key === 'errors.currencyConversion' ||
      result.error_info?.key === 'errors.noExchangeRate' ||
      result.error_info?.key === 'errors.unknownCurrency') {
    return true;
  }
  // Fallback: check error message text
  if (result.error && (
    result.error.includes('No exchange rate available') ||
    result.error.includes('Cannot convert')
  )) {
    return true;
  }
  return false;
}
