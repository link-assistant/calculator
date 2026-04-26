import { describe, it, expect } from 'vitest';
import { generateIssueReport, generateIssueUrl, type PageState } from './reportIssue';

describe('reportIssue utilities', () => {
  const basePageState: PageState = {
    expression: '',
    result: null,
    wasmReady: true,
    version: '1.0.0',
    theme: 'light',
    language: 'en',
    url: 'https://example.com/calculator/',
    userAgent: 'Mozilla/5.0 Test Browser',
    timestamp: '2024-01-01T12:00:00.000Z',
  };

  describe('generateIssueReport', () => {
    it('should include environment section', () => {
      const report = generateIssueReport(basePageState);

      expect(report).toContain('## Environment');
      expect(report).toContain('**Version**: 1.0.0');
      expect(report).toContain('**Theme**: light');
      expect(report).toContain('**Language**: en');
      expect(report).toContain('**WASM Ready**: Yes');
    });

    it('should include input section when expression is provided', () => {
      const state: PageState = {
        ...basePageState,
        expression: '2 + 3',
      };

      const report = generateIssueReport(state);

      expect(report).toContain('## Input');
      expect(report).toContain('2 + 3');
    });

    it('should not include input section when expression is empty', () => {
      const report = generateIssueReport(basePageState);

      expect(report).not.toContain('## Input');
    });

    it('should include result section for successful result', () => {
      const state: PageState = {
        ...basePageState,
        expression: '2 + 3',
        result: {
          success: true,
          result: '5',
          lino_interpretation: '((2) + (3))',
          steps: ['Step 1', 'Step 2'],
        },
      };

      const report = generateIssueReport(state);

      expect(report).toContain('## Result');
      expect(report).toContain('**Result**: 5');
      expect(report).toContain('**Links Notation**');
      expect(report).toContain('((2) + (3))');
      expect(report).toContain('**Steps**');
      expect(report).toContain('Step 1');
      expect(report).toContain('Step 2');
    });

    it('should include error for failed result', () => {
      const state: PageState = {
        ...basePageState,
        expression: 'invalid',
        result: {
          success: false,
          result: '',
          lino_interpretation: '',
          steps: [],
          error: 'Parse error',
        },
      };

      const report = generateIssueReport(state);

      expect(report).toContain('**Error**: Parse error');
    });

    it('should include alternative interpretations when present', () => {
      const state: PageState = {
        ...basePageState,
        expression: '2 + 3 * 4',
        result: {
          success: true,
          result: '14',
          lino_interpretation: '(2 + (3 * 4))',
          steps: [],
          alternative_lino: ['(2 + (3 * 4))', '((2 + 3) * 4)'],
        },
      };

      const report = generateIssueReport(state);

      expect(report).toContain('**Alternative interpretations**');
      expect(report).toContain('(2 + (3 * 4))');
      expect(report).toContain('((2 + 3) * 4)');
    });

    it('should not include alternative interpretations section when only one exists', () => {
      const state: PageState = {
        ...basePageState,
        expression: 'cos(0)',
        result: {
          success: true,
          result: '1',
          lino_interpretation: '(cos (0))',
          steps: [],
          alternative_lino: ['(cos (0))'],
        },
      };

      const report = generateIssueReport(state);

      expect(report).not.toContain('**Alternative interpretations**');
    });

    it('should not include alternative interpretations section when absent', () => {
      const state: PageState = {
        ...basePageState,
        expression: 'cos(0)',
        result: {
          success: true,
          result: '1',
          lino_interpretation: '(cos (0))',
          steps: [],
        },
      };

      const report = generateIssueReport(state);

      expect(report).not.toContain('**Alternative interpretations**');
    });

    it('should include description placeholder', () => {
      const report = generateIssueReport(basePageState);

      expect(report).toContain('## Description');
      expect(report).toContain('<!-- Please describe the issue you encountered -->');
    });
  });

  describe('generateIssueUrl', () => {
    // Helper to decode URL with + signs converted to spaces (URLSearchParams encoding)
    const decodeUrl = (url: string) => decodeURIComponent(url.replace(/\+/g, ' '));

    it('should generate issue URL for a caller-supplied repository and report payload', () => {
      const url = generateIssueUrl({
        repository: 'link-assistant/meta-expression',
        input: 'moon orbits the Sun',
        result: {
          success: true,
          result: 'unknown',
          linksNotation: '(statement-1: statement self (moon orbits the Sun))',
          alternativeLinksNotations: [
            '(interpretation-1: interpretation statement-1 factual-claim)',
            '(interpretation-2: interpretation statement-1 needs-specific-relation)',
          ],
          steps: [
            'Interpreted moon as a noun candidate',
            'Interpreted orbits as a relation candidate',
          ],
        },
        reproductionSteps: [
          'Open the expression page',
          'Enter moon orbits the Sun',
          'Click Report Issue',
        ],
        environment: {
          version: '0.1.0',
          url: 'https://link-assistant.github.io/meta-expression/web/',
          userAgent: 'Mozilla/5.0 Test Browser',
          wasmReady: true,
        },
        issueLabels: ['bug', 'diagnostics'],
      });

      const parsed = new URL(url);
      const body = parsed.searchParams.get('body') || '';

      expect(`${parsed.origin}${parsed.pathname}`).toBe(
        'https://github.com/link-assistant/meta-expression/issues/new'
      );
      expect(parsed.searchParams.get('labels')).toBe('bug,diagnostics');
      expect(parsed.searchParams.get('title')).toBe(
        'Issue with expression: moon orbits the Sun'
      );
      expect(body).toContain('## Environment');
      expect(body).toContain('## Input');
      expect(body).toContain('## Result');
      expect(body).toContain('**Links Notation**');
      expect(body).toContain('**Alternative interpretations**');
      expect(body).toContain('**Steps**');
      expect(body).toContain('## Reproduction Steps');
      expect(body).toContain('moon orbits the Sun');
    });

    it('should include failed result diagnostics in generated issue URLs', () => {
      const url = generateIssueUrl({
        repository: 'link-assistant/meta-expression',
        input: 'moon orbits the Sun',
        result: {
          success: false,
          error: 'No Wikidata entity found for moon',
          linksNotation: '(statement-1: statement self (moon orbits the Sun))',
          steps: ['Parsed the statement before entity resolution failed'],
        },
        environment: {
          version: '0.1.0',
          url: 'https://link-assistant.github.io/meta-expression/web/',
          wasmReady: false,
        },
      });

      const parsed = new URL(url);
      const body = parsed.searchParams.get('body') || '';

      expect(parsed.searchParams.get('title')).toBe(
        'Issue with expression: moon orbits the Sun'
      );
      expect(body).toContain('**Error**: No Wikidata entity found for moon');
      expect(body).toContain('**Links Notation**');
      expect(body).toContain('Parsed the statement before entity resolution failed');
    });

    it('should generate valid GitHub issue URL', () => {
      const url = generateIssueUrl(basePageState);

      expect(url).toContain('https://github.com/link-assistant/calculator/issues/new');
      expect(url).toContain('title=');
      expect(url).toContain('body=');
      expect(url).toContain('labels=bug');
    });

    it('should include expression in title when provided', () => {
      const state: PageState = {
        ...basePageState,
        expression: '2 + 3',
      };

      const url = generateIssueUrl(state);

      expect(url).toContain('title=');
      // URL encoded version of "Issue with expression: 2 + 3"
      expect(decodeUrl(url)).toContain('Issue with expression: 2 + 3');
    });

    it('should truncate long expressions in title', () => {
      const longExpression = 'a'.repeat(100);
      const state: PageState = {
        ...basePageState,
        expression: longExpression,
      };

      const url = generateIssueUrl(state);
      const decodedUrl = decodeUrl(url);

      // Title should be truncated to 50 chars + "..."
      expect(decodedUrl).toContain('...');
    });

    it('should use generic title when no expression', () => {
      const url = generateIssueUrl(basePageState);

      expect(decodeUrl(url)).toContain('Issue report');
    });
  });
});
