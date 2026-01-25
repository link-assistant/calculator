import { useState, useEffect, useCallback, useRef } from 'react';

interface CalculationResult {
  result: string;
  lino_interpretation: string;
  steps: string[];
  success: boolean;
  error?: string;
  issue_link?: string;
}

const EXAMPLES = [
  '2 + 3',
  '(2 + 3) * 4',
  '84 USD - 34 EUR',
  '100 * 1.5',
  '3.14 + 2.86',
  '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)',
];

function App() {
  const [input, setInput] = useState('');
  const [result, setResult] = useState<CalculationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [wasmReady, setWasmReady] = useState(false);
  const [version, setVersion] = useState('');
  const workerRef = useRef<Worker | null>(null);

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
        error: 'Worker initialization failed',
      });
      setLoading(false);
    };

    return () => {
      workerRef.current?.terminate();
    };
  }, []);

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

  return (
    <>
      <div className="container">
        <header>
          <h1>Link Calculator</h1>
          <p>Grammar-based expression calculator with DateTime and Currency support</p>
        </header>

        <div className="calculator">
          <div className="input-section">
            <label htmlFor="expression">Expression</label>
            <div className="input-wrapper">
              <input
                id="expression"
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Enter an expression (e.g., 2 + 3, 84 USD - 34 EUR)"
                disabled={!wasmReady}
                autoFocus
              />
            </div>
          </div>

          <div className="result-section">
            <div className="result-header">
              <h2>Result</h2>
              {loading && (
                <div className="loading">
                  <div className="spinner" />
                  <span>Calculating...</span>
                </div>
              )}
            </div>

            {!wasmReady ? (
              <div className="loading">
                <div className="spinner" />
                <span>Loading calculator engine...</span>
              </div>
            ) : result ? (
              <>
                {result.success ? (
                  <>
                    <div className="result-value">{result.result}</div>

                    {result.lino_interpretation && (
                      <div className="lino-section">
                        <h3>Links Notation</h3>
                        <div className="lino-value">{result.lino_interpretation}</div>
                      </div>
                    )}

                    {result.steps.length > 0 && (
                      <div className="steps-section">
                        <h3>Steps</h3>
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
                          Report this as an issue →
                        </a>
                      </div>
                    )}
                  </>
                )}
              </>
            ) : (
              <div className="result-value" style={{ opacity: 0.5 }}>
                Enter an expression above
              </div>
            )}
          </div>

          <div className="examples-section">
            <h2>Try these examples:</h2>
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
          Link Calculator {version && `v${version}`} · Powered by Rust + WebAssembly ·{' '}
          <a
            href="https://github.com/link-assistant/calculator"
            target="_blank"
            rel="noopener noreferrer"
          >
            View on GitHub
          </a>
        </p>
      </footer>
    </>
  );
}

export default App;
