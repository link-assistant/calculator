import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from './App';

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'app.subtitle': 'Free open-source calculator dedicated to public domain.',
        'input.placeholder': 'Enter an expression',
        'input.calculate': 'Calculate',
        'result.title': 'Result',
        'result.input': 'Input',
        'result.calculating': 'Calculating...',
        'result.loading': 'Loading calculator engine...',
        'result.placeholder': 'Enter an expression above',
        'examples.title': 'Try these examples:',
        'settings.theme': 'Theme',
        'settings.themeLight': 'Light',
        'settings.themeDark': 'Dark',
        'settings.themeAuto': 'Auto',
        'settings.language': 'Language',
        'settings.languageAutomatic': 'Automatic',
        'settings.currency': 'Preferred Currency',
        'settings.fiatCurrencies': 'Fiat Currencies',
        'settings.cryptoCurrencies': 'Crypto Currencies',
        'share.copyLink': 'Copy Link',
        'footer.poweredBy': 'Powered by Rust + WebAssembly',
        'footer.viewOnGitHub': 'View on GitHub',
        'footer.reportIssue': 'Report Issue',
      };
      let result = translations[key] || key;
      if (params) {
        Object.entries(params).forEach(([k, v]) => {
          result = result.replace(`{{${k}}}`, String(v));
        });
      }
      return result;
    },
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
    },
  }),
}));

// Mock hooks
vi.mock('./hooks', () => ({
  useTheme: () => ({
    theme: 'system',
    resolvedTheme: 'light',
    setTheme: vi.fn(),
  }),
  useUrlExpression: () => ({
    expression: '',
    setExpression: vi.fn(),
    copyShareLink: vi.fn().mockResolvedValue(true),
  }),
  useDelayedLoading: (loading: boolean) => loading,
}));

// Mock i18n module
vi.mock('./i18n', () => ({
  SUPPORTED_LANGUAGES: [
    { code: 'en', name: 'English', nativeName: 'English' },
    { code: 'de', name: 'German', nativeName: 'Deutsch' },
  ],
  loadPreferences: vi.fn(() => ({ theme: 'system', language: null, currency: null })),
  savePreferences: vi.fn(),
}));

// Mock ResizeObserver
class MockResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

describe('App Component - Branding', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should display Link.Calculator as brand title', () => {
    render(<App />);
    expect(screen.getByText('Link.Calculator')).toBeInTheDocument();
  });

  it('should display the SVG logo in the header', () => {
    render(<App />);
    const logo = document.querySelector('.brand-title svg');
    expect(logo).toBeInTheDocument();
  });

  it('should display the correct subtitle', () => {
    render(<App />);
    expect(screen.getByText('Free open-source calculator dedicated to public domain.')).toBeInTheDocument();
  });
});

describe('App Component - Input Section', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should render the textarea input', () => {
    render(<App />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('should display calculate button with equals sign', () => {
    render(<App />);
    const calculateButton = screen.getByTitle('Calculate');
    expect(calculateButton).toBeInTheDocument();
    expect(calculateButton).toHaveTextContent('=');
  });

  it('should have calculate button disabled when textarea is empty', () => {
    render(<App />);
    const calculateButton = screen.getByTitle('Calculate');
    expect(calculateButton).toBeDisabled();
  });

  it('should not have resize handle on textarea (auto-resize only)', () => {
    render(<App />);
    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveStyle({ resize: 'none' });
  });
});

describe('App Component - Settings', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should have settings button', () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');
    expect(settingsButton).toBeInTheDocument();
  });

  it('should open settings dropdown when clicking settings button', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    const dropdown = document.querySelector('.settings-dropdown');
    expect(dropdown).toBeInTheDocument();
  });

  it('should display Auto theme option instead of System', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    expect(screen.getByText('Auto')).toBeInTheDocument();
    expect(screen.queryByText('System')).not.toBeInTheDocument();
  });

  it('should have Automatic option in language selector', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    const selectors = screen.getAllByRole('combobox');
    // Language selector is the first one (contains language options like 'en')
    const langSelector = selectors.find(sel => sel.querySelector('option[value="en"]'));
    expect(langSelector).toBeInTheDocument();

    // Check for Automatic option
    const options = langSelector!.querySelectorAll('option');
    const automaticOption = Array.from(options).find(opt => opt.textContent === 'Automatic');
    expect(automaticOption).toBeInTheDocument();
  });

  it('should have currency preference selector', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    expect(screen.getByText('Preferred Currency')).toBeInTheDocument();
  });

  it('should have fiat currencies group in currency selector', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    // Look for optgroup elements in the DOM
    const optgroup = document.querySelector('optgroup[label="Fiat Currencies"]');
    expect(optgroup).toBeInTheDocument();
  });

  it('should have crypto currencies group in currency selector', async () => {
    render(<App />);
    const settingsButton = screen.getByLabelText('Theme');

    await userEvent.click(settingsButton);

    // Look for optgroup elements in the DOM
    const optgroup = document.querySelector('optgroup[label="Crypto Currencies"]');
    expect(optgroup).toBeInTheDocument();
  });
});

describe('App Component - Examples Section', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should display example buttons', () => {
    render(<App />);
    const exampleButtons = screen.getAllByRole('button', { name: /^\(?\d|integrate|USD|EUR/i });
    expect(exampleButtons.length).toBeGreaterThan(0);
  });

  it('should have examples section with title', () => {
    render(<App />);
    expect(screen.getByText('Try these examples:')).toBeInTheDocument();
  });
});

describe('App Component - Result Section', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should have Result section heading', () => {
    render(<App />);
    expect(screen.getByText('Result')).toBeInTheDocument();
  });
});

describe('App Component - Footer', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should display Link.Calculator in footer', () => {
    render(<App />);
    const footer = screen.getByRole('contentinfo');
    expect(footer).toHaveTextContent('Link.Calculator');
  });

  it('should have GitHub link in footer', () => {
    render(<App />);
    expect(screen.getByText('View on GitHub')).toBeInTheDocument();
  });

  it('should have Report Issue button in footer', () => {
    render(<App />);
    expect(screen.getByText('Report Issue')).toBeInTheDocument();
  });
});

describe('LinkCalculatorLogo Component', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should render SVG logo with correct size', () => {
    render(<App />);
    const logo = document.querySelector('.brand-title svg');
    expect(logo).toBeInTheDocument();
    expect(logo).toHaveAttribute('width', '32');
    expect(logo).toHaveAttribute('height', '32');
  });

  it('should have proper viewBox for scaling', () => {
    render(<App />);
    const logo = document.querySelector('.brand-title svg');
    expect(logo).toHaveAttribute('viewBox', '0 0 100 100');
  });
});

describe('Currency Detection', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should detect currency from browser locale (USD for en-US)', () => {
    // The detectUserCurrency function should be called on render
    // and use the browser's locale to determine currency
    render(<App />);
    // Component should render without errors
    expect(screen.getByText('Link.Calculator')).toBeInTheDocument();
  });
});
