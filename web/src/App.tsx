import { useState, useEffect, useCallback, useRef, lazy, Suspense, KeyboardEvent, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { useTheme, useUrlExpression, useDelayedLoading } from './hooks';
import { SUPPORTED_LANGUAGES, loadPreferences, savePreferences } from './i18n';
import { generateIssueUrl, type PageState } from './utils/reportIssue';
import { AutoResizeTextarea, ColorCodedLino, RepeatingDecimalNotations, type AutoResizeTextareaRef } from './components';
import { getExamplesForDisplay } from './examples';
import type { CalculationResult, ErrorInfo } from './types';

// SVG Logo component for Link.Calculator branding
const LinkCalculatorLogo = ({ size = 24 }: { size?: number }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 100 100"
    width={size}
    height={size}
    style={{ verticalAlign: 'middle' }}
  >
    <rect x="10" y="10" width="80" height="80" rx="8" fill="currentColor" />
    <rect x="18" y="18" width="64" height="20" rx="4" fill="var(--primary-light, #e0e7ff)" />
    <g fill="var(--primary-light, #c7d2fe)" opacity="0.7">
      <rect x="18" y="44" width="14" height="14" rx="2" />
      <rect x="36" y="44" width="14" height="14" rx="2" />
      <rect x="54" y="44" width="14" height="14" rx="2" />
      <rect x="72" y="44" width="10" height="14" rx="2" />
      <rect x="18" y="62" width="14" height="14" rx="2" />
      <rect x="36" y="62" width="14" height="14" rx="2" />
      <rect x="54" y="62" width="14" height="14" rx="2" />
      <rect x="72" y="62" width="10" height="14" rx="2" />
    </g>
  </svg>
);

// Top 10 crypto currencies by market cap
const CRYPTO_CURRENCIES = [
  { code: 'BTC', name: 'Bitcoin' },
  { code: 'ETH', name: 'Ethereum' },
  { code: 'USDT', name: 'Tether' },
  { code: 'BNB', name: 'BNB' },
  { code: 'SOL', name: 'Solana' },
  { code: 'XRP', name: 'XRP' },
  { code: 'USDC', name: 'USD Coin' },
  { code: 'ADA', name: 'Cardano' },
  { code: 'DOGE', name: 'Dogecoin' },
  { code: 'AVAX', name: 'Avalanche' },
];

// Major fiat currencies
const FIAT_CURRENCIES = [
  { code: 'USD', name: 'US Dollar' },
  { code: 'EUR', name: 'Euro' },
  { code: 'GBP', name: 'British Pound' },
  { code: 'JPY', name: 'Japanese Yen' },
  { code: 'CNY', name: 'Chinese Yuan' },
  { code: 'INR', name: 'Indian Rupee' },
  { code: 'RUB', name: 'Russian Ruble' },
  { code: 'BRL', name: 'Brazilian Real' },
  { code: 'CHF', name: 'Swiss Franc' },
  { code: 'CAD', name: 'Canadian Dollar' },
  { code: 'AUD', name: 'Australian Dollar' },
  { code: 'KRW', name: 'Korean Won' },
];

// Lazy load the math and plot components for better initial bundle size
const MathRenderer = lazy(() => import('./components/MathRenderer'));
const FunctionPlot = lazy(() => import('./components/FunctionPlot'));

/**
 * Translates an error using i18n error info.
 * Falls back to the raw error message if translation key doesn't exist.
 */
function translateError(
  t: TFunction,
  errorInfo: ErrorInfo | undefined,
  fallbackError: string | undefined
): string {
  if (!errorInfo) {
    return fallbackError || t('errors.calculationFailed');
  }

  // Check if the translation key exists
  const translated = t(errorInfo.key, errorInfo.params || {});

  // If the translation key is not found, i18next returns the key itself
  // In that case, fall back to the raw error message
  if (translated === errorInfo.key) {
    return fallbackError || t('errors.calculationFailed');
  }

  return translated;
}

/**
 * Detect user's preferred currency from browser locale.
 */
function detectUserCurrency(): string {
  try {
    const locale = navigator.language || 'en-US';
    // Map common locales to their currencies
    const localeToCurrency: Record<string, string> = {
      'en-US': 'USD',
      'en-GB': 'GBP',
      'de-DE': 'EUR',
      'fr-FR': 'EUR',
      'es-ES': 'EUR',
      'it-IT': 'EUR',
      'ja-JP': 'JPY',
      'zh-CN': 'CNY',
      'zh-TW': 'TWD',
      'ko-KR': 'KRW',
      'ru-RU': 'RUB',
      'pt-BR': 'BRL',
      'hi-IN': 'INR',
      'ar-SA': 'SAR',
    };

    // Try exact match first
    if (localeToCurrency[locale]) {
      return localeToCurrency[locale];
    }

    // Try language-only match
    const lang = locale.split('-')[0];
    const langMatch = Object.entries(localeToCurrency).find(([key]) => key.startsWith(lang + '-'));
    if (langMatch) {
      return langMatch[1];
    }

    return 'USD'; // Default to USD
  } catch {
    return 'USD';
  }
}

function App() {
  const { t, i18n } = useTranslation();
  const { theme, resolvedTheme, setTheme } = useTheme();
  const { expression: input, setExpression: setInput, copyShareLink } = useUrlExpression('');

  // Get random examples from the examples.lino file (memoized to stay stable during session)
  const displayExamples = useMemo(() => getExamplesForDisplay(6), []);

  const [result, setResult] = useState<CalculationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [wasmReady, setWasmReady] = useState(false);
  const [version, setVersion] = useState('');
  const [linkCopied, setLinkCopied] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [ratesLoading, setRatesLoading] = useState(false);
  const [ratesInfo, setRatesInfo] = useState<{ date?: string; base?: string } | null>(null);
  const [computationTime, setComputationTime] = useState<number | null>(null);
  const [preferredCurrency, setPreferredCurrency] = useState<string>(() => {
    const prefs = loadPreferences();
    return prefs.currency || detectUserCurrency();
  });

  const workerRef = useRef<Worker | null>(null);
  const settingsRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<AutoResizeTextareaRef>(null);

  // Delayed loading indicator (shows after 300ms)
  const showLoading = useDelayedLoading(loading, 300);

  useEffect(() => {
    // Create web worker
    workerRef.current = new Worker(
      new URL('./worker.ts', import.meta.url),
      { type: 'module' }
    );

    workerRef.current.onmessage = (e) => {
      const { type, data } = e.data;

      if (type === 'ready') {
        setWasmReady(true);
        setVersion(data.version);
      } else if (type === 'result') {
        setResult(data);
        // Capture computation time from worker if provided
        if (data.computation_time_ms !== undefined) {
          setComputationTime(data.computation_time_ms);
        }
        setLoading(false);
      } else if (type === 'error') {
        setResult({
          result: '',
          lino_interpretation: '',
          steps: [],
          success: false,
          error: data.error,
        });
        setComputationTime(null);
        setLoading(false);
      } else if (type === 'ratesLoading') {
        setRatesLoading(data.loading);
      } else if (type === 'ratesLoaded') {
        if (data.success) {
          setRatesInfo({ date: data.date, base: data.base });
        }
      }
    };

    workerRef.current.onerror = (e) => {
      console.error('Worker error:', e);
      setResult({
        result: '',
        lino_interpretation: '',
        steps: [],
        success: false,
        error: t('errors.workerFailed'),
      });
      setLoading(false);
    };

    return () => {
      workerRef.current?.terminate();
    };
  }, [t]);

  const calculate = useCallback((expression?: string) => {
    const expr = expression ?? input;
    if (!expr.trim() || !wasmReady || !workerRef.current) {
      return;
    }

    setLoading(true);
    setComputationTime(null);
    workerRef.current.postMessage({ type: 'calculate', expression: expr });
  }, [wasmReady, input]);

  // Handle Enter key press to trigger calculation
  const handleKeyDown = useCallback((e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      calculate();
    }
  }, [calculate]);

  // Handle window resize to auto-resize textarea
  useEffect(() => {
    const handleResize = () => {
      textareaRef.current?.resize();
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const handleExampleClick = (example: string) => {
    setInput(example);
  };

  const handleCopyLink = async () => {
    const success = await copyShareLink();
    if (success) {
      setLinkCopied(true);
      setTimeout(() => setLinkCopied(false), 2000);
    }
  };

  const handleLanguageChange = (langCode: string) => {
    i18n.changeLanguage(langCode);
    const prefs = loadPreferences();
    savePreferences({ ...prefs, language: langCode });
  };

  const handleCurrencyChange = (currencyCode: string) => {
    setPreferredCurrency(currencyCode);
    const prefs = loadPreferences();
    savePreferences({ ...prefs, currency: currencyCode });
  };

  const handleReportIssue = () => {
    const pageState: PageState = {
      expression: input,
      result,
      wasmReady,
      version,
      theme: resolvedTheme,
      language: i18n.language,
      url: window.location.href,
      userAgent: navigator.userAgent,
      timestamp: new Date().toISOString(),
    };
    // Pass the translation function for localized issue reports
    const issueUrl = generateIssueUrl(pageState, t);
    window.open(issueUrl, '_blank', 'noopener,noreferrer');
  };

  // Close settings dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (settingsRef.current && !settingsRef.current.contains(event.target as Node)) {
        setSettingsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Determine if RTL language
  const isRtl = i18n.language === 'ar';

  return (
    <div className={`app-wrapper ${isRtl ? 'rtl' : ''}`} dir={isRtl ? 'rtl' : 'ltr'}>
      <div className="container">
        <header>
          <div className="header-top">
            <h1 className="brand-title">
              <LinkCalculatorLogo size={32} />
              <span>Link.Calculator</span>
            </h1>
            <div className="settings-wrapper" ref={settingsRef}>
              <button
                className="settings-button"
                onClick={() => setSettingsOpen(!settingsOpen)}
                aria-label={t('settings.theme')}
              >
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                  <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
                </svg>
              </button>
              {settingsOpen && (
                <div className="settings-dropdown">
                  <div className="settings-section">
                    <label>{t('settings.theme')}</label>
                    <div className="settings-buttons">
                      <button
                        className={theme === 'light' ? 'active' : ''}
                        onClick={() => setTheme('light')}
                      >
                        {t('settings.themeLight')}
                      </button>
                      <button
                        className={theme === 'dark' ? 'active' : ''}
                        onClick={() => setTheme('dark')}
                      >
                        {t('settings.themeDark')}
                      </button>
                      <button
                        className={theme === 'system' ? 'active' : ''}
                        onClick={() => setTheme('system')}
                      >
                        {t('settings.themeAuto')}
                      </button>
                    </div>
                  </div>
                  <div className="settings-section">
                    <label>{t('settings.language')}</label>
                    <select
                      value={i18n.language}
                      onChange={(e) => handleLanguageChange(e.target.value)}
                    >
                      <option value="">{t('settings.languageAutomatic')}</option>
                      {SUPPORTED_LANGUAGES.map((lang) => (
                        <option key={lang.code} value={lang.code}>
                          {lang.nativeName}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="settings-section">
                    <label>{t('settings.currency')}</label>
                    <select
                      value={preferredCurrency}
                      onChange={(e) => handleCurrencyChange(e.target.value)}
                    >
                      <optgroup label={t('settings.fiatCurrencies')}>
                        {FIAT_CURRENCIES.map((curr) => (
                          <option key={curr.code} value={curr.code}>
                            {curr.code} - {curr.name}
                          </option>
                        ))}
                      </optgroup>
                      <optgroup label={t('settings.cryptoCurrencies')}>
                        {CRYPTO_CURRENCIES.map((curr) => (
                          <option key={curr.code} value={curr.code}>
                            {curr.code} - {curr.name}
                          </option>
                        ))}
                      </optgroup>
                    </select>
                  </div>
                </div>
              )}
            </div>
          </div>
          <p>{t('app.subtitle')}</p>
        </header>

        <div className="calculator">
          <div className="input-section">
            <div className="input-wrapper">
              <AutoResizeTextarea
                ref={textareaRef}
                id="expression"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={t('input.placeholder')}
                disabled={!wasmReady}
                autoFocus
                minRows={2}
                maxRows={10}
              />
              <div className="input-buttons">
                {input && (
                  <button
                    className="share-button"
                    onClick={handleCopyLink}
                    title={t('share.copyLink')}
                  >
                    {linkCopied ? (
                      <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                      </svg>
                    ) : (
                      <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                      </svg>
                    )}
                  </button>
                )}
                <button
                  className="calculate-button"
                  onClick={() => calculate()}
                  disabled={!wasmReady || !input.trim()}
                  title={t('input.calculate')}
                >
                  <span>=</span>
                </button>
              </div>
            </div>
          </div>

          {/* Input interpretation section - before Result */}
          {result && result.success && result.lino_interpretation && (
            <div className="input-interpretation-section">
              <h2>{t('result.input')}</h2>
              <div className="lino-value">
                <ColorCodedLino lino={result.lino_interpretation} />
              </div>
            </div>
          )}

          <div className="result-section">
            <div className="result-header">
              <h2>{t('result.title')}</h2>
              <div className="result-meta">
                {showLoading && (
                  <div className="loading">
                    <div className="spinner" />
                    <span>{t('result.calculating')}</span>
                  </div>
                )}
                {!showLoading && computationTime !== null && (
                  <span className="computation-time">
                    {computationTime < 1 ? '<1' : computationTime.toFixed(0)} ms
                  </span>
                )}
              </div>
            </div>

            {!wasmReady ? (
              <div className="loading">
                <div className="spinner" />
                <span>{t('result.loading')}</span>
              </div>
            ) : result ? (
              <>
                {result.success ? (
                  <>
                    {/* Section 1: Result (mandatory) */}
                    {result.is_symbolic && result.latex_input && result.latex_result ? (
                      <div className="symbolic-result">
                        <Suspense fallback={<div className="result-value">{result.result}</div>}>
                          <div className="math-equation">
                            <MathRenderer latex={result.latex_input} display={true} />
                            <span className="equals">=</span>
                            <MathRenderer latex={result.latex_result} display={true} />
                          </div>
                        </Suspense>
                        <div className="result-text">{result.result}</div>
                      </div>
                    ) : (
                      <div className="result-value-section">
                        <div className="result-value">{result.result}</div>
                        {/* Show fraction representation if available */}
                        {result.fraction && (
                          <div className="fraction-hint">
                            = {result.fraction}
                          </div>
                        )}
                      </div>
                    )}

                    {/* Section 3: Repeating decimal notations (optional) */}
                    {result.repeating_decimal && (
                      <div className="notations-section">
                        <h3>{t('result.notations', 'Decimal Notations')}</h3>
                        <RepeatingDecimalNotations formats={result.repeating_decimal} />
                      </div>
                    )}

                    {/* Section 4: Plot (optional) */}
                    {result.plot_data && (
                      <div className="plot-section">
                        <h3>{t('result.plot', 'Function Plot')}</h3>
                        <Suspense fallback={<div className="loading"><div className="spinner" /></div>}>
                          <FunctionPlot data={result.plot_data} width={360} height={220} />
                        </Suspense>
                      </div>
                    )}

                    {/* Section 5: Steps / Reasoning (optional) */}
                    {result.steps.length > 0 && !result.is_symbolic && (
                      <div className="steps-section">
                        <h3>{t('result.steps')}</h3>
                        <ul className="steps-list">
                          {result.steps.map((step, i) => (
                            <li key={i}>{step}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </>
                ) : (
                  <>
                    <div className="result-value error">
                      {translateError(t, result.error_info, result.error)}
                    </div>
                    {result.issue_link && (
                      <div className="issue-link">
                        <a href={result.issue_link} target="_blank" rel="noopener noreferrer">
                          {t('errors.reportIssue')} →
                        </a>
                      </div>
                    )}
                  </>
                )}
              </>
            ) : (
              <div className="result-value" style={{ opacity: 0.5 }}>
                {t('result.placeholder')}
              </div>
            )}
          </div>

          <div className="examples-section">
            <h2>{t('examples.title')}</h2>
            <div className="examples-grid">
              {displayExamples.map((example) => (
                <button
                  key={example}
                  className="example-button"
                  onClick={() => handleExampleClick(example)}
                  disabled={!wasmReady}
                >
                  {example}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      <footer>
        <p>
          Link.Calculator {version && `v${version}`} · {t('footer.poweredBy')} ·{' '}
          <a
            href="https://github.com/link-assistant/calculator"
            target="_blank"
            rel="noopener noreferrer"
          >
            {t('footer.viewOnGitHub')}
          </a>
          {' · '}
          <button className="link-button" onClick={handleReportIssue}>
            {t('footer.reportIssue')}
          </button>
        </p>
        {ratesLoading && (
          <p className="rates-status loading">
            <span className="spinner-small" /> {t('footer.loadingRates')}
          </p>
        )}
        {!ratesLoading && ratesInfo && (
          <p className="rates-status">
            {t('footer.ratesInfo', { date: ratesInfo.date, source: 'fawazahmed0/currency-api' })}
          </p>
        )}
      </footer>
    </div>
  );
}

export default App;
