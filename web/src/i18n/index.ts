import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { Parser } from 'links-notation';

// Import .lino files as raw text using Vite's ?raw suffix
import enLino from './locales/en.lino?raw';
import ruLino from './locales/ru.lino?raw';
import zhLino from './locales/zh.lino?raw';
import hiLino from './locales/hi.lino?raw';
import arLino from './locales/ar.lino?raw';
import deLino from './locales/de.lino?raw';
import frLino from './locales/fr.lino?raw';

const STORAGE_KEY = 'link-calculator-preferences';

export interface Preferences {
  theme: 'light' | 'dark' | 'system';
  language: string | null;
}

// Links Notation helpers for encoding/decoding preferences
// Using a simple regex-based approach for reliability

export function encodePreferences(prefs: Preferences): string {
  // Encode as Links Notation: (preferences (theme: dark) (language: en))
  const parts: string[] = [];
  if (prefs.theme) {
    parts.push(`(theme: ${prefs.theme})`);
  }
  if (prefs.language) {
    parts.push(`(language: ${prefs.language})`);
  }
  return `(preferences ${parts.join(' ')})`;
}

export function decodePreferences(lino: string): Preferences {
  const prefs: Preferences = { theme: 'system', language: null };

  if (!lino) return prefs;

  // Use regex extraction from Links Notation format
  const themeMatch = lino.match(/\(theme:\s*(light|dark|system)\)/);
  const langMatch = lino.match(/\(language:\s*(\w+)\)/);

  if (themeMatch) {
    prefs.theme = themeMatch[1] as 'light' | 'dark' | 'system';
  }
  if (langMatch) {
    prefs.language = langMatch[1];
  }

  return prefs;
}

export function loadPreferences(): Preferences {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return decodePreferences(stored);
    }
  } catch {
    // localStorage not available
  }
  return { theme: 'system', language: null };
}

export function savePreferences(prefs: Preferences): void {
  try {
    const encoded = encodePreferences(prefs);
    localStorage.setItem(STORAGE_KEY, encoded);
  } catch {
    // localStorage not available
  }
}

export const SUPPORTED_LANGUAGES = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский' },
  { code: 'zh', name: 'Chinese', nativeName: '中文' },
  { code: 'hi', name: 'Hindi', nativeName: 'हिन्दी' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية' },
  { code: 'de', name: 'German', nativeName: 'Deutsch' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
] as const;

// Parser for Links Notation
const parser = new Parser();

/**
 * i18n translations type - nested object with section keys and string values
 */
interface I18nTranslations {
  [key: string]: { [key: string]: string };
}

/**
 * Convert parsed Links Notation to i18n JSON format.
 * The .lino format uses indented links where each section is:
 *   sectionName:
 *     key value
 *     key2 value2
 *
 * This highly readable format is parsed by links-notation parser
 * and converted to i18next-compatible JSON structure.
 */
function linksToI18nJson(links: ReturnType<typeof parser.parse>): I18nTranslations {
  const result: I18nTranslations = {};

  for (const link of links) {
    if (!link.id) continue;

    const sectionKey = link.id;

    if (link.values && link.values.length > 0) {
      // This is a nested object with key-value pairs
      const nestedObj: { [key: string]: string } = {};

      for (const valueLink of link.values) {
        if (valueLink.values && valueLink.values.length >= 2) {
          // This is a doublet (key value)
          const nestedKey = valueLink.values[0].id || '';
          const nestedValue = valueLink.values[1].id || '';
          nestedObj[nestedKey] = nestedValue;
        }
      }

      result[sectionKey] = nestedObj;
    }
  }

  return result;
}

/**
 * Parse a Links Notation string to i18n JSON format.
 * Supports the indented format for human-readable locale files.
 */
export function parseLinoToI18n(linoContent: string): I18nTranslations {
  const links = parser.parse(linoContent);
  return linksToI18nJson(links);
}

// Parse all locale files from Links Notation to JSON
const en = parseLinoToI18n(enLino);
const ru = parseLinoToI18n(ruLino);
const zh = parseLinoToI18n(zhLino);
const hi = parseLinoToI18n(hiLino);
const ar = parseLinoToI18n(arLino);
const de = parseLinoToI18n(deLino);
const fr = parseLinoToI18n(frLino);

const resources = {
  en: { translation: en },
  ru: { translation: ru },
  zh: { translation: zh },
  hi: { translation: hi },
  ar: { translation: ar },
  de: { translation: de },
  fr: { translation: fr },
};

// Load saved preferences
const savedPrefs = loadPreferences();

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    // If we have a saved language preference, use it
    lng: savedPrefs.language || undefined,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      lookupLocalStorage: 'i18nextLng',
      caches: [],
    },
  });

export default i18n;
