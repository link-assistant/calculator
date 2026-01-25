export interface CalculationResult {
  result: string;
  lino_interpretation: string;
  steps: string[];
  success: boolean;
  error?: string;
  issue_link?: string;
}
