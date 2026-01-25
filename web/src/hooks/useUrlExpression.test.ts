import { describe, it, expect, beforeEach } from 'vitest';
import { encodeExpression, decodeExpression, generateShareUrl } from './useUrlExpression';

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
