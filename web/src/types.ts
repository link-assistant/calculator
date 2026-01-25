export interface PlotData {
  x_values: number[];
  y_values: number[];
  label: string;
  x_label: string;
  y_label: string;
}

export interface CalculationResult {
  result: string;
  lino_interpretation: string;
  steps: string[];
  success: boolean;
  error?: string;
  issue_link?: string;
  latex_input?: string;
  latex_result?: string;
  is_symbolic?: boolean;
  plot_data?: PlotData;
}
