import type { CalculationResult } from '../types';
import type { TFunction } from 'i18next';

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
 * Translation labels for issue report sections.
 */
export interface IssueReportLabels {
  environment: string;
  version: string;
  url: string;
  userAgent: string;
  theme: string;
  language: string;
  wasmReady: string;
  timestamp: string;
  input: string;
  resultSection: string;
  resultLabel: string;
  linksNotation: string;
  stepsLabel: string;
  errorLabel: string;
  description: string;
  descriptionPlaceholder: string;
  yes: string;
  no: string;
  unknownVersion: string;
  unknownError: string;
}

/**
 * Get default English labels for issue reports.
 */
export function getDefaultLabels(): IssueReportLabels {
  return {
    environment: 'Environment',
    version: 'Version',
    url: 'URL',
    userAgent: 'User Agent',
    theme: 'Theme',
    language: 'Language',
    wasmReady: 'WASM Ready',
    timestamp: 'Timestamp',
    input: 'Input',
    resultSection: 'Result',
    resultLabel: 'Result',
    linksNotation: 'Links Notation',
    stepsLabel: 'Steps',
    errorLabel: 'Error',
    description: 'Description',
    descriptionPlaceholder: 'Please describe the issue you encountered',
    yes: 'Yes',
    no: 'No',
    unknownVersion: 'Unknown',
    unknownError: 'Unknown error',
  };
}

/**
 * Get labels from i18n translation function.
 */
export function getLabelsFromI18n(t: TFunction): IssueReportLabels {
  return {
    environment: t('issueReport.environment', 'Environment'),
    version: t('issueReport.version', 'Version'),
    url: t('issueReport.url', 'URL'),
    userAgent: t('issueReport.userAgent', 'User Agent'),
    theme: t('issueReport.theme', 'Theme'),
    language: t('issueReport.language', 'Language'),
    wasmReady: t('issueReport.wasmReady', 'WASM Ready'),
    timestamp: t('issueReport.timestamp', 'Timestamp'),
    input: t('issueReport.input', 'Input'),
    resultSection: t('issueReport.resultSection', 'Result'),
    resultLabel: t('issueReport.resultLabel', 'Result'),
    linksNotation: t('issueReport.linksNotation', 'Links Notation'),
    stepsLabel: t('issueReport.stepsLabel', 'Steps'),
    errorLabel: t('issueReport.errorLabel', 'Error'),
    description: t('issueReport.description', 'Description'),
    descriptionPlaceholder: t(
      'issueReport.descriptionPlaceholder',
      'Please describe the issue you encountered'
    ),
    yes: t('common.yes', 'Yes'),
    no: t('common.no', 'No'),
    unknownVersion: t('common.unknown', 'Unknown'),
    unknownError: t('errors.unknownError', 'Unknown error'),
  };
}

/**
 * Generate a markdown report of the current page state for issue reporting.
 * @param state - The current page state
 * @param labels - Translation labels (defaults to English if not provided)
 */
export function generateIssueReport(
  state: PageState,
  labels: IssueReportLabels = getDefaultLabels()
): string {
  const sections: string[] = [];

  sections.push(`## ${labels.environment}`);
  sections.push('');
  sections.push(`- **${labels.version}**: ${state.version || labels.unknownVersion}`);
  sections.push(`- **${labels.url}**: ${state.url}`);
  sections.push(`- **${labels.userAgent}**: ${state.userAgent}`);
  sections.push(`- **${labels.theme}**: ${state.theme}`);
  sections.push(`- **${labels.language}**: ${state.language}`);
  sections.push(`- **${labels.wasmReady}**: ${state.wasmReady ? labels.yes : labels.no}`);
  sections.push(`- **${labels.timestamp}**: ${state.timestamp}`);

  if (state.expression) {
    sections.push('');
    sections.push(`## ${labels.input}`);
    sections.push('');
    sections.push('```');
    sections.push(state.expression);
    sections.push('```');
  }

  if (state.result) {
    sections.push('');
    sections.push(`## ${labels.resultSection}`);
    sections.push('');

    if (state.result.success) {
      sections.push(`**${labels.resultLabel}**: ${state.result.result}`);

      if (state.result.lino_interpretation) {
        sections.push('');
        sections.push(`**${labels.linksNotation}**:`);
        sections.push('```');
        sections.push(state.result.lino_interpretation);
        sections.push('```');
      }

      if (state.result.steps && state.result.steps.length > 0) {
        sections.push('');
        sections.push(`**${labels.stepsLabel}**:`);
        state.result.steps.forEach((step, i) => {
          sections.push(`${i + 1}. ${step}`);
        });
      }
    } else {
      sections.push(`**${labels.errorLabel}**: ${state.result.error || labels.unknownError}`);
    }
  }

  sections.push('');
  sections.push(`## ${labels.description}`);
  sections.push('');
  sections.push(`<!-- ${labels.descriptionPlaceholder} -->`);
  sections.push('');

  return sections.join('\n');
}

/**
 * Generate issue title with i18n support.
 * @param expression - The expression that caused the issue
 * @param t - Translation function (optional, defaults to English)
 */
export function generateIssueTitle(expression: string, t?: TFunction): string {
  if (expression) {
    const truncated = expression.slice(0, 50);
    const ellipsis = expression.length > 50 ? '...' : '';
    if (t) {
      return t('issueReport.issueTitle', {
        expression: `${truncated}${ellipsis}`,
        defaultValue: `Issue with expression: ${truncated}${ellipsis}`,
      });
    }
    return `Issue with expression: ${truncated}${ellipsis}`;
  }
  if (t) {
    return t('issueReport.defaultIssueTitle', 'Issue report');
  }
  return 'Issue report';
}

/**
 * Generate a GitHub issue URL with prefilled content.
 * @param state - The current page state
 * @param t - Translation function (optional, for localized issue body)
 */
export function generateIssueUrl(state: PageState, t?: TFunction): string {
  const labels = t ? getLabelsFromI18n(t) : getDefaultLabels();
  const report = generateIssueReport(state, labels);
  const title = generateIssueTitle(state.expression, t);

  const baseUrl = 'https://github.com/link-assistant/calculator/issues/new';
  const params = new URLSearchParams({
    title,
    body: report,
    labels: 'bug',
  });

  return `${baseUrl}?${params.toString()}`;
}
