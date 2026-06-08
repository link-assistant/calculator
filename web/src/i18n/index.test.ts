import { describe, it, expect, vi, beforeEach } from 'vitest';
import { encodePreferences, decodePreferences, loadPreferences, savePreferences, parseLinoToI18n, type Preferences } from './index';
import arLino from './locales/ar.lino?raw';
import deLino from './locales/de.lino?raw';
import enLino from './locales/en.lino?raw';
import frLino from './locales/fr.lino?raw';
import hiLino from './locales/hi.lino?raw';
import ruLino from './locales/ru.lino?raw';
import zhLino from './locales/zh.lino?raw';

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

const LOCALES = {
  ar: arLino,
  de: deLino,
  en: enLino,
  fr: frLino,
  hi: hiLino,
  ru: ruLino,
  zh: zhLino,
} as const;

const REQUIRED_USER_OUTPUT_KEYS = [
  'errors.unknownError',
  'result.notations',
  'result.plot',
  'issueReport.alternativeInterpretations',
  'issueReport.reproductionSteps',
  'issueReport.reproductionOpen',
  'issueReport.reproductionEnter',
  'issueReport.reproductionRunCalculation',
  'issueReport.reproductionUseCalculator',
  'issueReport.reproductionClickReportIssue',
  'steps.input',
  'steps.utcEquivalent',
  'steps.timeUntil',
  'steps.timeSince',
  'steps.currentTime',
  'steps.timeUntilTarget',
  'steps.timeSinceTarget',
  'steps.convert',
  'steps.indefiniteIntegral',
  'steps.computedSymbolicResult',
  'steps.solveLinearEquation',
  'steps.solution',
  'steps.checkEquality',
  'steps.compare',
] as const;

function flattenTranslations(
  translations: ReturnType<typeof parseLinoToI18n>,
  prefix = ''
): Map<string, string> {
  const flattened = new Map<string, string>();

  Object.entries(translations).forEach(([section, values]) => {
    Object.entries(values).forEach(([key, value]) => {
      flattened.set(prefix ? `${prefix}.${section}.${key}` : `${section}.${key}`, value);
    });
  });

  return flattened;
}

function interpolationKeys(value: string): string[] {
  return Array.from(value.matchAll(/\{\{([^}]+)\}\}/g), match => match[1]).sort();
}

describe('i18n preferences', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('encodePreferences', () => {
    it('should encode theme preference', () => {
      const prefs: Preferences = { theme: 'dark', language: null, currency: null };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('preferences');
      expect(encoded).toContain('theme');
      expect(encoded).toContain('dark');
    });

    it('should encode language preference', () => {
      const prefs: Preferences = { theme: 'system', language: 'en', currency: null };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('language');
      expect(encoded).toContain('en');
    });

    it('should encode both theme and language', () => {
      const prefs: Preferences = { theme: 'light', language: 'ru', currency: null };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('theme');
      expect(encoded).toContain('light');
      expect(encoded).toContain('language');
      expect(encoded).toContain('ru');
    });

    it('should encode currency preference', () => {
      const prefs: Preferences = { theme: 'system', language: null, currency: 'EUR' };
      const encoded = encodePreferences(prefs);

      expect(encoded).toContain('currency');
      expect(encoded).toContain('EUR');
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

    it('should decode currency preference', () => {
      const encoded = '(preferences (currency: USD))';
      const prefs = decodePreferences(encoded);

      expect(prefs.currency).toBe('USD');
    });

    it('should return defaults for invalid input', () => {
      const prefs = decodePreferences('invalid');

      expect(prefs.theme).toBe('system');
      expect(prefs.language).toBeNull();
      expect(prefs.currency).toBeNull();
    });

    it('should handle malformed input gracefully', () => {
      const prefs = decodePreferences('');

      expect(prefs.theme).toBe('system');
      expect(prefs.language).toBeNull();
      expect(prefs.currency).toBeNull();
    });
  });

  describe('round-trip preferences', () => {
    const testCases: Preferences[] = [
      { theme: 'light', language: 'en', currency: 'USD' },
      { theme: 'dark', language: 'ru', currency: 'EUR' },
      { theme: 'system', language: 'zh', currency: null },
      { theme: 'dark', language: null, currency: 'BTC' },
    ];

    testCases.forEach((prefs) => {
      it(`should round-trip: theme=${prefs.theme}, language=${prefs.language}, currency=${prefs.currency}`, () => {
        const encoded = encodePreferences(prefs);
        const decoded = decodePreferences(encoded);

        expect(decoded.theme).toBe(prefs.theme);
        if (prefs.language) {
          expect(decoded.language).toBe(prefs.language);
        }
        if (prefs.currency) {
          expect(decoded.currency).toBe(prefs.currency);
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
      expect(prefs.currency).toBeNull();
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
      const prefs: Preferences = { theme: 'dark', language: 'en', currency: 'USD' };

      savePreferences(prefs);

      expect(localStorageMock.setItem).toHaveBeenCalled();
      const [key, value] = localStorageMock.setItem.mock.calls[0];
      expect(key).toBe('link-calculator-preferences');
      expect(value).toContain('preferences');
    });
  });

  describe('parseLinoToI18n', () => {
    it('should parse simple indented Links Notation to i18n format', () => {
      const lino = `app:
  title 'My App'
  subtitle 'A great app'`;

      const result = parseLinoToI18n(lino);

      expect(result).toEqual({
        app: {
          title: 'My App',
          subtitle: 'A great app',
        },
      });
    });

    it('should parse multiple sections', () => {
      const lino = `app:
  title 'My App'
footer:
  text Footer`;

      const result = parseLinoToI18n(lino);

      expect(result).toEqual({
        app: {
          title: 'My App',
        },
        footer: {
          text: 'Footer',
        },
      });
    });

    it('should handle values without quotes', () => {
      const lino = `settings:
  theme Theme
  language Language`;

      const result = parseLinoToI18n(lino);

      expect(result).toEqual({
        settings: {
          theme: 'Theme',
          language: 'Language',
        },
      });
    });

    it('should handle values with special characters', () => {
      const lino = `errors:
  message 'Something went wrong!'
  loading 'Loading...'`;

      const result = parseLinoToI18n(lino);

      expect(result).toEqual({
        errors: {
          message: 'Something went wrong!',
          loading: 'Loading...',
        },
      });
    });

    it('should handle empty input', () => {
      const result = parseLinoToI18n('');

      expect(result).toEqual({});
    });
  });

  describe('translation coverage', () => {
    it('should keep every supported locale in key parity with English', () => {
      const parsedLocales = Object.fromEntries(
        Object.entries(LOCALES).map(([locale, content]) => [
          locale,
          flattenTranslations(parseLinoToI18n(content)),
        ])
      );
      const englishKeys = Array.from(parsedLocales.en.keys()).sort();

      Object.entries(parsedLocales).forEach(([locale, translations]) => {
        const localeKeys = Array.from(translations.keys()).sort();
        const missingKeys = englishKeys.filter(key => !translations.has(key));
        const extraKeys = localeKeys.filter(key => !parsedLocales.en.has(key));

        expect(missingKeys, `${locale} is missing translation keys`).toEqual([]);
        expect(extraKeys, `${locale} has translation keys not present in English`).toEqual([]);
      });
    });

    it('should include all user-facing issue-report and calculation-step keys', () => {
      Object.entries(LOCALES).forEach(([locale, content]) => {
        const translations = flattenTranslations(parseLinoToI18n(content));
        const missingKeys = REQUIRED_USER_OUTPUT_KEYS.filter(key => !translations.has(key));

        expect(missingKeys, `${locale} is missing required user-output keys`).toEqual([]);
      });
    });

    it('should keep interpolation placeholders consistent with English', () => {
      const parsedLocales = Object.fromEntries(
        Object.entries(LOCALES).map(([locale, content]) => [
          locale,
          flattenTranslations(parseLinoToI18n(content)),
        ])
      );

      parsedLocales.en.forEach((englishValue, key) => {
        const englishInterpolationKeys = interpolationKeys(englishValue);

        Object.entries(parsedLocales).forEach(([locale, translations]) => {
          expect(
            interpolationKeys(translations.get(key) || ''),
            `${locale}.${key} interpolation placeholders differ from English`
          ).toEqual(englishInterpolationKeys);
        });
      });
    });
  });
});
