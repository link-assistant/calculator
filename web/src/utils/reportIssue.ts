import type { CalculationResult } from '../types';

export interface PageState {
  expression: string;
  result: CalculationResult | null;
  wasmReady: boolean;
  version: string;
  theme: string;
  language: string;
  url: string;
  userAgent: string;
  timestamp: string;
}

/**
 * Generate a markdown report of the current page state for issue reporting
 */
export function generateIssueReport(state: PageState): string {
  const sections: string[] = [];

  sections.push('## Environment');
  sections.push('');
  sections.push(`- **Version**: ${state.version || 'Unknown'}`);
  sections.push(`- **URL**: ${state.url}`);
  sections.push(`- **User Agent**: ${state.userAgent}`);
  sections.push(`- **Theme**: ${state.theme}`);
  sections.push(`- **Language**: ${state.language}`);
  sections.push(`- **WASM Ready**: ${state.wasmReady ? 'Yes' : 'No'}`);
  sections.push(`- **Timestamp**: ${state.timestamp}`);

  if (state.expression) {
    sections.push('');
    sections.push('## Input');
    sections.push('');
    sections.push('```');
    sections.push(state.expression);
    sections.push('```');
  }

  if (state.result) {
    sections.push('');
    sections.push('## Result');
    sections.push('');

    if (state.result.success) {
      sections.push(`**Result**: ${state.result.result}`);

      if (state.result.lino_interpretation) {
        sections.push('');
        sections.push('**Links Notation**:');
        sections.push('```');
        sections.push(state.result.lino_interpretation);
        sections.push('```');
      }

      if (state.result.steps && state.result.steps.length > 0) {
        sections.push('');
        sections.push('**Steps**:');
        state.result.steps.forEach((step, i) => {
          sections.push(`${i + 1}. ${step}`);
        });
      }
    } else {
      sections.push(`**Error**: ${state.result.error || 'Unknown error'}`);
    }
  }

  sections.push('');
  sections.push('## Description');
  sections.push('');
  sections.push('<!-- Please describe the issue you encountered -->');
  sections.push('');

  return sections.join('\n');
}

/**
 * Generate a GitHub issue URL with prefilled content
 */
export function generateIssueUrl(state: PageState): string {
  const report = generateIssueReport(state);
  const title = state.expression
    ? `Issue with expression: ${state.expression.slice(0, 50)}${state.expression.length > 50 ? '...' : ''}`
    : 'Issue report';

  const baseUrl = 'https://github.com/link-assistant/calculator/issues/new';
  const params = new URLSearchParams({
    title,
    body: report,
    labels: 'bug',
  });

  return `${baseUrl}?${params.toString()}`;
}
