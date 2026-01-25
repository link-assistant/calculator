import { useState, useEffect, useCallback, useRef, lazy, Suspense } from 'react';
import { useTranslation } from 'react-i18next';
import { useTheme, useUrlExpression, useDelayedLoading } from './hooks';
import { SUPPORTED_LANGUAGES, loadPreferences, savePreferences } from './i18n';
import { generateIssueUrl, type PageState } from './utils/reportIssue';
import { AutoResizeTextarea } from './components';
import type { CalculationResult } from './types';

// Lazy load the math and plot components for better initial bundle size
const MathRenderer = lazy(() => import('./components/MathRenderer'));
const FunctionPlot = lazy(() => import('./components/FunctionPlot'));

const EXAMPLES = [
  '2 + 3',
  '(2 + 3) * 4',
  '84 USD - 34 EUR',
  '100 * 1.5',
  'integrate sin(x)/x dx',
  'integrate(x^2, x, 0, 3)',
];

function App() {
  const { t, i18n } = useTranslation();
  const { theme, resolvedTheme, setTheme } = useTheme();
  const { expression: input, setExpression: setInput, copyShareLink } = useUrlExpression('');

  const [result, setResult] = useState<CalculationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [wasmReady, setWasmReady] = useState(false);
  const [version, setVersion] = useState('');
  const [linkCopied, setLinkCopied] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const workerRef = useRef<Worker | null>(null);
  const settingsRef = useRef<HTMLDivElement>(null);

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
        setLoading(false);
      } else if (type === 'error') {
        setResult({
          result: '',
          lino_interpretation: '',
          steps: [],
          success: false,
          error: data.error,
        });
        setLoading(false);
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

  const calculate = useCallback((expression: string) => {
    if (!expression.trim() || !wasmReady || !workerRef.current) {
      return;
    }

    setLoading(true);
    workerRef.current.postMessage({ type: 'calculate', expression });
  }, [wasmReady]);

  useEffect(() => {
    const debounce = setTimeout(() => {
      if (input.trim()) {
        calculate(input);
      } else {
        setResult(null);
      }
    }, 300);

    return () => clearTimeout(debounce);
  }, [input, calculate]);

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
    const issueUrl = generateIssueUrl(pageState);
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
            <h1>{t('app.title')}</h1>
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
                        {t('settings.themeSystem')}
                      </button>
                    </div>
                  </div>
                  <div className="settings-section">
                    <label>{t('settings.language')}</label>
                    <select
                      value={i18n.language}
                      onChange={(e) => handleLanguageChange(e.target.value)}
                    >
                      {SUPPORTED_LANGUAGES.map((lang) => (
                        <option key={lang.code} value={lang.code}>
                          {lang.nativeName}
                        </option>
                      ))}
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
            <label htmlFor="expression">{t('input.label')}</label>
            <div className="input-wrapper">
              <AutoResizeTextarea
                id="expression"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder={t('input.placeholder')}
                disabled={!wasmReady}
                autoFocus
                minRows={1}
                maxRows={10}
              />
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
            </div>
          </div>

          <div className="result-section">
            <div className="result-header">
              <h2>{t('result.title')}</h2>
              {showLoading && (
                <div className="loading">
                  <div className="spinner" />
                  <span>{t('result.calculating')}</span>
                </div>
              )}
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
                    {/* Symbolic result with LaTeX rendering */}
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
                      <div className="result-value">{result.result}</div>
                    )}

                    {/* Plot rendering for symbolic results */}
                    {result.plot_data && (
                      <div className="plot-section">
                        <h3>{t('result.plot', 'Function Plot')}</h3>
                        <Suspense fallback={<div className="loading"><div className="spinner" /></div>}>
                          <FunctionPlot data={result.plot_data} width={360} height={220} />
                        </Suspense>
                      </div>
                    )}

                    {result.lino_interpretation && !result.is_symbolic && (
                      <div className="lino-section">
                        <h3>{t('result.linksNotation')}</h3>
                        <div className="lino-value">{result.lino_interpretation}</div>
                      </div>
                    )}

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
                    <div className="result-value error">{result.error}</div>
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
              {EXAMPLES.map((example) => (
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
          Link Calculator {version && `v${version}`} · {t('footer.poweredBy')} ·{' '}
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
      </footer>
    </div>
  );
}

export default App;
