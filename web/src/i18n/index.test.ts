import { describe, it, expect, vi, beforeEach } from 'vitest';
import { encodePreferences, decodePreferences, loadPreferences, savePreferences, type Preferences } from './index';

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

describe('i18n preferences', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('encodePreferences', () => {
    it('should encode theme preference', () => {
      const prefs: Preferences = { theme: 'dark', language: null };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('preferences');
      expect(encoded).toContain('theme');
      expect(encoded).toContain('dark');
    });

    it('should encode language preference', () => {
      const prefs: Preferences = { theme: 'system', language: 'en' };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('language');
      expect(encoded).toContain('en');
    });

    it('should encode both theme and language', () => {
      const prefs: Preferences = { theme: 'light', language: 'ru' };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('theme');
      expect(encoded).toContain('light');
      expect(encoded).toContain('language');
      expect(encoded).toContain('ru');
    });
  });

  describe('decodePreferences', () => {
    it('should decode theme preference', () => {
      const encoded = '(preferences (theme: dark))';
      const prefs = decodePreferences(encoded);

      expect(prefs.theme).toBe('dark');
    });

    it('should decode language preference', () => {
      const encoded = '(preferences (language: en))';
      const prefs = decodePreferences(encoded);

      expect(prefs.language).toBe('en');
    });

    it('should return defaults for invalid input', () => {
      const prefs = decodePreferences('invalid');

      expect(prefs.theme).toBe('system');
      expect(prefs.language).toBeNull();
    });

    it('should handle malformed input gracefully', () => {
      const prefs = decodePreferences('');

      expect(prefs.theme).toBe('system');
      expect(prefs.language).toBeNull();
    });
  });

  describe('round-trip preferences', () => {
    const testCases: Preferences[] = [
      { theme: 'light', language: 'en' },
      { theme: 'dark', language: 'ru' },
      { theme: 'system', language: 'zh' },
      { theme: 'dark', language: null },
    ];

    testCases.forEach((prefs) => {
      it(`should round-trip: theme=${prefs.theme}, language=${prefs.language}`, () => {
        const encoded = encodePreferences(prefs);
        const decoded = decodePreferences(encoded);

        expect(decoded.theme).toBe(prefs.theme);
        if (prefs.language) {
          expect(decoded.language).toBe(prefs.language);
        }
      });
    });
  });

  describe('loadPreferences', () => {
    it('should return defaults when localStorage is empty', () => {
      localStorageMock.getItem.mockReturnValue(null);

      const prefs = loadPreferences();

      expect(prefs.theme).toBe('system');
      expect(prefs.language).toBeNull();
    });

    it('should load preferences from localStorage', () => {
      const stored = '(preferences (theme: dark) (language: en))';
      localStorageMock.getItem.mockReturnValue(stored);

      const prefs = loadPreferences();

      expect(prefs.theme).toBe('dark');
      expect(prefs.language).toBe('en');
    });
  });

  describe('savePreferences', () => {
    it('should save preferences to localStorage', () => {
      const prefs: Preferences = { theme: 'dark', language: 'en' };

      savePreferences(prefs);

      expect(localStorageMock.setItem).toHaveBeenCalled();
      const [key, value] = localStorageMock.setItem.mock.calls[0];
      expect(key).toBe('link-calculator-preferences');
      expect(value).toContain('preferences');
    });
  });
});
