import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import ru from './locales/ru.json';
import zh from './locales/zh.json';
import hi from './locales/hi.json';
import ar from './locales/ar.json';
import de from './locales/de.json';
import fr from './locales/fr.json';

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
