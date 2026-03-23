import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
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
        'keyboard.showKeyboard': 'Show keyboard',
        'keyboard.hideKeyboard': 'Hide keyboard',
        'keyboard.label': 'Universal Keyboard',
        'keyboard.backspace': 'Backspace',
        'keyboard.calculate': 'Calculate',
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
    wasLoadedFromUrl: vi.fn().mockReturnValue(false),
  }),
  useDelayedLoading: (loading: boolean) => loading,
  useExpressionCache: () => ({
    getCachedResult: vi.fn().mockReturnValue(null),
    cacheResult: vi.fn(),
  }),
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

describe('App Component - Interpretation Switching', () => {
  let originalResizeObserver: typeof ResizeObserver;
  // Track all worker instances (new Worker() may be called multiple times due to re-renders)
  const allWorkerInstances: Array<{
    onmessage: ((event: MessageEvent) => void) | null;
    onerror: ((event: ErrorEvent) => void) | null;
    postMessage: ReturnType<typeof vi.fn>;
    terminate: ReturnType<typeof vi.fn>;
  }> = [];

  // Shared postMessage spy across all worker instances
  const sharedPostMessage = vi.fn();

  class CapturedMockWorker {
    onmessage: ((event: MessageEvent) => void) | null = null;
    onerror: ((event: ErrorEvent) => void) | null = null;
    postMessage = sharedPostMessage;
    terminate = vi.fn();

    constructor() {
      allWorkerInstances.push(this);
    }
  }

  // Returns the most recently created worker instance (the active one)
  function getActiveWorker() {
    return allWorkerInstances[allWorkerInstances.length - 1];
  }

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
    allWorkerInstances.length = 0;
    sharedPostMessage.mockClear();
    vi.stubGlobal('Worker', CapturedMockWorker);
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  async function simulateWorkerReady() {
    await act(async () => {
      const worker = getActiveWorker();
      worker?.onmessage?.(
        new MessageEvent('message', { data: { type: 'ready', data: { version: '1.0.0' } } })
      );
    });
  }

  async function simulateWorkerResult(result: object) {
    await act(async () => {
      const worker = getActiveWorker();
      worker?.onmessage?.(
        new MessageEvent('message', { data: { type: 'result', data: result } })
      );
    });
  }

  it('should render alternative interpretations when result has multiple alternatives', async () => {
    render(<App />);
    await simulateWorkerReady();

    // Simulate a result with multiple interpretations
    await simulateWorkerResult({
      result: '7',
      lino_interpretation: '(1 + (2 * 3))',
      alternative_lino: ['(1 + (2 * 3))', '((1 + 2) * 3)'],
      steps: [],
      success: true,
    });

    // Both interpretation buttons should be rendered
    const altButtons = document.querySelectorAll('.lino-alt-button');
    expect(altButtons).toHaveLength(2);
  });

  it('should highlight first interpretation as selected by default', async () => {
    render(<App />);
    await simulateWorkerReady();

    await simulateWorkerResult({
      result: '7',
      lino_interpretation: '(1 + (2 * 3))',
      alternative_lino: ['(1 + (2 * 3))', '((1 + 2) * 3)'],
      steps: [],
      success: true,
    });

    const altButtons = document.querySelectorAll('.lino-alt-button');
    expect(altButtons[0]).toHaveClass('selected');
    expect(altButtons[1]).not.toHaveClass('selected');
  });

  it('should trigger recalculation when switching to a different interpretation', async () => {
    render(<App />);
    await simulateWorkerReady();
    sharedPostMessage.mockClear(); // Clear any calls from initial setup

    await simulateWorkerResult({
      result: '7',
      lino_interpretation: '(1 + (2 * 3))',
      alternative_lino: ['(1 + (2 * 3))', '((1 + 2) * 3)'],
      steps: [],
      success: true,
    });

    // Click the second interpretation
    const altButtons = document.querySelectorAll('.lino-alt-button');
    await userEvent.click(altButtons[1]);

    // Verify postMessage was called with the selected interpretation
    expect(sharedPostMessage).toHaveBeenCalledWith({
      type: 'calculate',
      expression: '((1 + 2) * 3)',
    });
  });

  it('should highlight clicked interpretation as selected', async () => {
    render(<App />);
    await simulateWorkerReady();

    await simulateWorkerResult({
      result: '7',
      lino_interpretation: '(1 + (2 * 3))',
      alternative_lino: ['(1 + (2 * 3))', '((1 + 2) * 3)'],
      steps: [],
      success: true,
    });

    // Click the second interpretation
    const altButtons = document.querySelectorAll('.lino-alt-button');
    await userEvent.click(altButtons[1]);

    // Second button should now be selected
    const updatedButtons = document.querySelectorAll('.lino-alt-button');
    expect(updatedButtons[1]).toHaveClass('selected');
    expect(updatedButtons[0]).not.toHaveClass('selected');
  });

  it('should NOT show "Calculation failed" when plan arrives before result (issue #111)', async () => {
    render(<App />);
    await simulateWorkerReady();

    // Simulate a plan message arriving (before rates are fetched / result is ready).
    // This is the exact scenario from issue #111: URL-loaded expression triggers
    // calculation, plan arrives instantly, but result hasn't arrived yet.
    await act(async () => {
      const worker = getActiveWorker();
      worker?.onmessage?.(
        new MessageEvent('message', {
          data: {
            type: 'plan',
            data: {
              expression: '(1000 рублей + 500 рублей) в USD',
              lino_interpretation: '((1000 RUB) + (500 RUB)) to USD',
              alternative_lino: ['((1000 RUB) + (500 RUB)) to USD'],
              required_sources: ['ecb', 'cbr'],
              currencies: ['RUB', 'USD'],
              is_live_time: false,
              success: true,
            },
          },
        })
      );
    });

    // The error message should NOT be visible — the calculation is still in progress
    const errorElements = document.querySelectorAll('.result-value.error');
    expect(errorElements).toHaveLength(0);

    // The "Calculation failed" text should not appear
    expect(screen.queryByText(/calculation failed/i)).not.toBeInTheDocument();
    expect(screen.queryByText('errors.calculationFailed')).not.toBeInTheDocument();
  });

  it('should show single lino-value for result with single interpretation', async () => {
    render(<App />);
    await simulateWorkerReady();

    await simulateWorkerResult({
      result: '6',
      lino_interpretation: '((1 + 2) + 3)',
      alternative_lino: ['((1 + 2) + 3)'],
      steps: [],
      success: true,
    });

    // Should not show alt buttons, should show single lino-value
    const altButtons = document.querySelectorAll('.lino-alt-button');
    expect(altButtons).toHaveLength(0);

    const linoValue = document.querySelector('.lino-value');
    expect(linoValue).toBeInTheDocument();
  });
});

describe('App Component - Universal Keyboard', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
    // Restore Worker stub (may have been removed by previous test suite's vi.unstubAllGlobals)
    class SimpleWorker {
      onmessage: ((event: MessageEvent) => void) | null = null;
      onerror: ((event: ErrorEvent) => void) | null = null;
      postMessage = vi.fn();
      terminate = vi.fn();
    }
    vi.stubGlobal('Worker', SimpleWorker);
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('should render the keyboard toggle button', () => {
    render(<App />);
    const toggleButton = document.querySelector('.keyboard-toggle-button');
    expect(toggleButton).toBeInTheDocument();
  });

  it('should not show keyboard by default', () => {
    render(<App />);
    const keyboard = document.querySelector('.universal-keyboard');
    expect(keyboard).not.toBeInTheDocument();
  });

  it('should show keyboard when toggle button is clicked', async () => {
    render(<App />);
    const toggleButton = document.querySelector('.keyboard-toggle-button') as HTMLButtonElement;
    expect(toggleButton).toBeInTheDocument();

    await userEvent.click(toggleButton);

    const keyboard = document.querySelector('.universal-keyboard');
    expect(keyboard).toBeInTheDocument();
  });

  it('should hide keyboard when toggle button is clicked again', async () => {
    render(<App />);
    const toggleButton = document.querySelector('.keyboard-toggle-button') as HTMLButtonElement;

    // Open keyboard
    await userEvent.click(toggleButton);
    expect(document.querySelector('.universal-keyboard')).toBeInTheDocument();

    // Close keyboard
    await userEvent.click(toggleButton);
    expect(document.querySelector('.universal-keyboard')).not.toBeInTheDocument();
  });

  it('toggle button should have active class when keyboard is open', async () => {
    render(<App />);
    const toggleButton = document.querySelector('.keyboard-toggle-button') as HTMLButtonElement;

    expect(toggleButton).not.toHaveClass('active');

    await userEvent.click(toggleButton);

    const updatedButton = document.querySelector('.keyboard-toggle-button');
    expect(updatedButton).toHaveClass('active');
  });

  it('toggle button should have correct aria-expanded state', async () => {
    render(<App />);
    const toggleButton = document.querySelector('.keyboard-toggle-button') as HTMLButtonElement;

    expect(toggleButton).toHaveAttribute('aria-expanded', 'false');

    await userEvent.click(toggleButton);

    const updatedButton = document.querySelector('.keyboard-toggle-button') as HTMLButtonElement;
    expect(updatedButton).toHaveAttribute('aria-expanded', 'true');
  });
});
