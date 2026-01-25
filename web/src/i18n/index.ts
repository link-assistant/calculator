import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { linoToJson } from 'lino-objects-codec';

// Import .lino files as raw strings
import enLino from './locales/en.lino?raw';
import ruLino from './locales/ru.lino?raw';
import zhLino from './locales/zh.lino?raw';
import hiLino from './locales/hi.lino?raw';
import arLino from './locales/ar.lino?raw';
import deLino from './locales/de.lino?raw';
import frLino from './locales/fr.lino?raw';

// Parse Links Notation files to JSON objects
const en = linoToJson({ lino: enLino }) as Record<string, unknown>;
const ru = linoToJson({ lino: ruLino }) as Record<string, unknown>;
const zh = linoToJson({ lino: zhLino }) as Record<string, unknown>;
const hi = linoToJson({ lino: hiLino }) as Record<string, unknown>;
const ar = linoToJson({ lino: arLino }) as Record<string, unknown>;
const de = linoToJson({ lino: deLino }) as Record<string, unknown>;
const fr = linoToJson({ lino: frLino }) as Record<string, unknown>;

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
