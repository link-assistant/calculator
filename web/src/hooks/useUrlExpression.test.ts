import { describe, it, expect, beforeEach } from 'vitest';
import { encodeExpression, decodeExpression, generateShareUrl, isLegacyFormat } from './useUrlExpression';

describe('URL Expression utilities', () => {
  describe('encodeExpression', () => {
    it('should return empty string for empty input', () => {
      expect(encodeExpression('')).toBe('');
      expect(encodeExpression('   ')).toBe('');
    });

    it('should encode a simple expression', () => {
      const encoded = encodeExpression('2 + 3');
      expect(encoded).toBeTruthy();
      expect(typeof encoded).toBe('string');
    });

    it('should encode expressions with special characters', () => {
      const encoded = encodeExpression('84 USD - 34 EUR');
      expect(encoded).toBeTruthy();
    });

    it('should encode expressions with quotes', () => {
      const encoded = encodeExpression('test "quoted" value');
      expect(encoded).toBeTruthy();
    });

    it('should use only URL-safe characters (a-zA-Z0-9, -, _)', () => {
      // Test with various expressions that might produce +, /, or = in standard base64
      const testCases = [
        '2 + 3',
        'hello world!',
        '100 * 1.5 / 2',
        '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)',
        'a very long expression with many different characters: 1234567890!@#$%^&*()',
        '你好世界', // Unicode characters
      ];

      testCases.forEach((expression) => {
        const encoded = encodeExpression(expression);
        // Should not contain +, /, or =
        expect(encoded).not.toMatch(/[+/=]/);
        // Should only contain URL-safe characters
        expect(encoded).toMatch(/^[a-zA-Z0-9_-]*$/);
      });
    });

    it('should not include padding characters', () => {
      const encoded = encodeExpression('test');
      expect(encoded).not.toContain('=');
    });
  });

  describe('decodeExpression', () => {
    it('should return empty string for empty input', () => {
      expect(decodeExpression('')).toBe('');
    });

    it('should decode an encoded expression', () => {
      const original = '2 + 3';
      const encoded = encodeExpression(original);
      const decoded = decodeExpression(encoded);
      expect(decoded).toBe(original);
    });

    it('should decode expressions with special characters', () => {
      const original = '84 USD - 34 EUR';
      const encoded = encodeExpression(original);
      const decoded = decodeExpression(encoded);
      expect(decoded).toBe(original);
    });

    it('should handle datetime expressions', () => {
      const original = '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)';
      const encoded = encodeExpression(original);
      const decoded = decodeExpression(encoded);
      expect(decoded).toBe(original);
    });

    it('should handle invalid encoded strings gracefully', () => {
      // Invalid base64 should return empty
      const result = decodeExpression('!!!invalid!!!');
      expect(result).toBe('');
    });

    it('should decode legacy base64 format with padding', () => {
      // Legacy format: btoa(encodeURIComponent('(expression "2 + 3")'))
      const legacyEncoded = btoa(encodeURIComponent('(expression "2 + 3")'));
      const decoded = decodeExpression(legacyEncoded);
      expect(decoded).toBe('2 + 3');
    });

    it('should decode new base64url format without padding', () => {
      const original = '2 + 3';
      const encoded = encodeExpression(original);
      // Verify it doesn't have padding
      expect(encoded).not.toContain('=');
      const decoded = decodeExpression(encoded);
      expect(decoded).toBe(original);
    });
  });

  describe('isLegacyFormat', () => {
    it('should detect legacy format with padding', () => {
      const legacyEncoded = btoa(encodeURIComponent('(expression "test")'));
      expect(isLegacyFormat(legacyEncoded)).toBe(true);
    });

    it('should detect new base64url format', () => {
      const newEncoded = encodeExpression('test');
      expect(isLegacyFormat(newEncoded)).toBe(false);
    });

    it('should detect legacy format with + character', () => {
      // Create a string that produces + in base64
      expect(isLegacyFormat('abc+def')).toBe(true);
    });

    it('should detect legacy format with / character', () => {
      expect(isLegacyFormat('abc/def')).toBe(true);
    });
  });

  describe('generateShareUrl', () => {
    beforeEach(() => {
      // Mock window.location
      Object.defineProperty(window, 'location', {
        value: {
          origin: 'https://example.com',
          pathname: '/calculator/',
        },
        writable: true,
      });
    });

    it('should generate URL without query param for empty expression', () => {
      const url = generateShareUrl('');
      expect(url).toBe('https://example.com/calculator/');
    });

    it('should generate URL with encoded expression', () => {
      const url = generateShareUrl('2 + 3');
      expect(url).toContain('https://example.com/calculator/?q=');
      expect(url).toContain('?q=');
    });

    it('should generate valid URL for complex expressions', () => {
      const url = generateShareUrl('84 USD - 34 EUR');
      expect(url).toContain('?q=');
      // URL should be valid
      expect(() => new URL(url)).not.toThrow();
    });
  });

  describe('round-trip encoding/decoding', () => {
    const testCases = [
      '2 + 3',
      '(2 + 3) * 4',
      '84 USD - 34 EUR',
      '100 * 1.5',
      '3.14 + 2.86',
      '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)',
      'some expression with spaces',
      '1+2*3/4-5',
    ];

    testCases.forEach((expression) => {
      it(`should round-trip: ${expression}`, () => {
        const encoded = encodeExpression(expression);
        const decoded = decodeExpression(encoded);
        expect(decoded).toBe(expression);
      });
    });
  });
});
