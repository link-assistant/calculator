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

    it('should include description placeholder', () => {
      const report = generateIssueReport(basePageState);

      expect(report).toContain('## Description');
      expect(report).toContain('<!-- Please describe the issue you encountered -->');
    });
  });

  describe('generateIssueUrl', () => {
    // Helper to decode URL with + signs converted to spaces (URLSearchParams encoding)
    const decodeUrl = (url: string) => decodeURIComponent(url.replace(/\+/g, ' '));

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
