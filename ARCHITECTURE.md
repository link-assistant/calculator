# Architecture

Link Calculator is a Rust WebAssembly calculator with a React/TypeScript frontend. It supports arithmetic, DateTime operations, currency conversions (fiat + crypto), unit conversions, and links notation expression representation.

## High-level overview

```
┌──────────────────────────────────────────────────────────┐
│                    Browser (Main Thread)                  │
│                                                          │
│  React App (App.tsx)                                     │
│  ├── Expression input (AutoResizeTextarea)                │
│  ├── URL ↔ Expression sync (useUrlExpression)            │
│  ├── Result cache (useExpressionCache, localStorage)     │
│  └── Sends/receives messages to/from Web Worker          │
│                                                          │
│  ┌────────────────── Web Worker ──────────────────────┐  │
│  │  worker.ts                                         │  │
│  │  ├── WASM Calculator (Rust → WebAssembly)          │  │
│  │  ├── Rate Coordination (worker-rate-coordination)  │  │
│  │  └── Rate Fetchers (ECB, CBR, CoinGecko)          │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Plan → Execute pipeline

When a calculation is requested, the worker follows a three-step pipeline:

```
Expression
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 1: PLAN (instant, no network)                          │
│                                                             │
│ calculator.plan(expression)                                 │
│   → Parse expression into AST                               │
│   → Walk AST to collect all currency codes                  │
│   → Map currencies to rate sources:                         │
│       USD, EUR, GBP, ... → ECB (Frankfurter API)           │
│       RUB              → CBR (Central Bank of Russia)       │
│       TON, BTC, ETH, ... → Crypto (CoinGecko)             │
│   → Generate LINO interpretation + alternatives             │
│   → Detect live time references (now, UTC time)            │
│   → Return CalculationPlan                                  │
│                                                             │
│ Send plan to UI immediately (interpretation shown           │
│ while rates load)                                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 2: FETCH (network, only what's needed)                 │
│                                                             │
│ rateCoordination.ensureRatesForSources(plan.required_sources)│
│                                                             │
│ For each source in plan.required_sources:                    │
│   idle?    → trigger fetch, wait for completion             │
│   loading? → wait (already in-flight)                       │
│   loaded?  → skip (instant)                                 │
│                                                             │
│ Pure math (required_sources=[]) → skip entirely, zero delay │
│                                                             │
│ Sources fetch in parallel when multiple are needed.          │
│ Once loaded, subsequent calculations reuse cached rates.    │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 3: EXECUTE (computation, rates guaranteed available)   │
│                                                             │
│ calculator.execute(expression)                              │
│   → Parse, evaluate, generate steps                         │
│   → Currency conversions use loaded rates                   │
│   → Return CalculationResult with:                          │
│       result, steps, LINO, alternatives, LaTeX,            │
│       repeating decimal info, plot data, etc.              │
│                                                             │
│ Send result to UI → display value, steps, etc.             │
└─────────────────────────────────────────────────────────────┘
```

## Rate sources

| Source | API | Currencies | Update frequency |
|--------|-----|-----------|------------------|
| ECB | [Frankfurter](https://frankfurter.dev/) | 30+ fiat (USD, EUR, GBP, JPY, …) | Daily ~16:00 CET |
| CBR | [cbr.ru](https://cbr.ru) + local .lino fallback | RUB-based rates | Daily |
| Crypto | [CoinGecko](https://www.coingecko.com/en/api) | TON, BTC, ETH, BNB, SOL, XRP, ADA, DOGE, DOT, LTC, LINK, UNI | Near real-time |

The CBR source has a fallback: when CORS blocks direct API access (common in browsers), the worker loads rates from local `.lino` files served via GitHub Pages. These are updated weekly by CI.

## Rust core (`src/`)

The calculator engine is written in Rust and compiled to WebAssembly.

### Modules

- **`lib.rs`** — `Calculator` struct with `plan()`, `execute()`, `calculate()`. `CalculationPlan` and `CalculationResult` types.
- **`wasm.rs`** — WebAssembly bindings: `fetch_exchange_rates()`, `fetch_cbr_rates()`, `fetch_crypto_rates()`, LINO parsers.
- **`grammar/`** — Expression parsing pipeline:
  - `lexer.rs` — Tokenizer (numbers, operators, keywords, currencies, dates)
  - `token_parser.rs` — Recursive descent parser → `Expression` AST
  - `expression_parser.rs` — `ExpressionParser` combining all grammars, with `parse_and_evaluate()`
  - `datetime_grammar.rs` — DateTime literal parsing
  - `number_grammar.rs` — Number literal parsing (including localized formats)
  - `math_functions.rs` — Built-in functions (sin, cos, sqrt, log, …)
  - `integral.rs` — Indefinite integral evaluation
- **`types/`** — Core data types:
  - `expression.rs` — `Expression` enum (AST nodes), `to_lino()`, `to_latex()`, `collect_currencies()`, `alternative_lino()`
  - `value.rs` — `Value` type with `ValueKind` (Number, Rational, DateTime, Duration, Boolean), arithmetic operations with currency conversion
  - `decimal.rs` — Arbitrary-precision decimal (wraps `rust_decimal`)
  - `rational.rs` — Exact fractions (wraps `num_rational`)
  - `datetime.rs` — DateTime with timezone support
  - `currency.rs` — `CurrencyDatabase` for storing/retrieving exchange rates
  - `unit.rs` — `Unit` enum (Currency, Duration, DataSize, Mass, Custom)
- **`lino/`** — Links notation structures
- **`currency_api.rs`** — HTTP client for ECB/Frankfurter and CBR APIs
- **`crypto_api.rs`** — HTTP client for CoinGecko API
- **`error.rs`** — Error types with i18n support

### Expression pipeline

```
Input string
    → Lexer → Vec<Token>
    → TokenParser → Expression AST
    → Expression::to_lino() → Links notation string
    → evaluate_with_steps() → (Value, Vec<Step>)
    → CalculationResult (JSON)
```

### Key types

```rust
enum Expression {
    Number { value: Decimal, unit: Unit },
    DateTime(DateTime),
    Now,
    Binary { left, op, right },
    FunctionCall { name, args },
    UnitConversion { value, target_unit },
    // ... and more
}

enum Unit {
    None,
    Currency(String),   // "USD", "TON", "RUB"
    Duration(DurationUnit),
    DataSize(DataSizeUnit),
    Mass(MassUnit),
}
```

## Web frontend (`web/src/`)

### Components

- **`App.tsx`** — Main application. Manages worker lifecycle, state, and rendering.
- **`AutoResizeTextarea`** — Input textarea that grows with content.
- **`ColorCodedLino`** — Renders LINO expressions with color-coded parentheses.
- **`UniversalKeyboard`** — Optional on-screen math keyboard.
- **`MathRenderer`** — LaTeX rendering via KaTeX (lazy-loaded).
- **`FunctionPlot`** — Function visualization (lazy-loaded).
- **`RepeatingDecimalNotations`** — Alternative decimal notation display.

### Hooks

- **`useUrlExpression`** — Syncs expression with URL query parameter (`?q=base64url`). Supports legacy format migration.
- **`useExpressionCache`** — Caches results in localStorage for instant display on page reload.
- **`useDelayedLoading`** — Shows loading spinner only after a delay (default 300ms) to avoid flicker.
- **`useTheme`** — Theme management (light/dark/system).

### Worker (`worker.ts`)

Runs in a Web Worker to avoid blocking the UI thread. Hosts the WASM calculator instance and manages rate fetching.

**Messages sent to worker:**
- `{ type: 'calculate', expression }` — Run plan→fetch→execute pipeline
- `{ type: 'refreshRates' }` — Re-fetch all rate sources
- `{ type: 'fetchRates', baseCurrency }` — Fetch rates for specific currency
- `{ type: 'getRatesStatus' }` — Query current rates state

**Messages sent from worker:**
- `{ type: 'ready', data: { version } }` — WASM initialized
- `{ type: 'plan', data: CalculationPlan }` — Plan ready (instant, before rates)
- `{ type: 'result', data: CalculationResult }` — Calculation complete
- `{ type: 'error', data: { error } }` — Calculation or initialization error
- `{ type: 'ratesLoading', data: { loading } }` — Rate fetch started/stopped
- `{ type: 'ratesLoaded', data: { success, date, base } }` — ECB rates loaded
- `{ type: 'cbrRatesLoaded', data: { success, date } }` — CBR rates loaded
- `{ type: 'cryptoRatesLoaded', data: { success, base, date } }` — Crypto rates loaded

### Rate coordination (`worker-rate-coordination.ts`)

Manages on-demand rate loading with three states per source: `idle → loading → loaded`.

Primary API: `ensureRatesForSources(sources: Set<RateSource>)` — accepts the `required_sources` from the plan and ensures they're loaded before returning.

Legacy API: `ensureRatesForExpression(expression)` — uses a TypeScript string heuristic. Kept for backwards compatibility with tests.

## Data flow

### Page load with URL expression

```
1. Browser loads ?q=BASE64URL
2. useUrlExpression decodes expression
3. useExpressionCache checks localStorage → show cached result instantly
4. App sends { type: 'calculate', expression } to worker
5. Worker: plan() → plan sent to UI → fetch needed rates → execute()
6. UI updates with fresh result, caches it
```

### User types expression and presses Enter

```
1. App sends { type: 'calculate', expression } to worker
2. Loading indicator shows after 300ms (useDelayedLoading)
3. Worker: plan() → plan sent to UI → fetch rates if needed → execute()
4. Result displayed with steps, LINO interpretation, alternatives
```

### Alternative interpretations

Ambiguous expressions (e.g., `2 + 3 * 4`) produce multiple LINO interpretations. The plan includes these in `alternative_lino`, and the UI allows switching between them with `selectedLinoIndex`.

## Build and test

```bash
# Rust
cargo test --all-features        # All Rust tests
cargo clippy --all-targets       # Linting
cargo fmt --check                # Formatting

# WASM
wasm-pack build --target web --out-dir web/public/pkg

# Web
cd web
npm install
npm run dev                      # Development server
npm run build                    # Production build (tsc + vite)
npm run test                     # Vitest unit tests
npm run test:e2e                 # Playwright E2E tests
```

## Directory layout

```
├── src/                     # Rust source code
│   ├── grammar/             # Lexer, parser, evaluation
│   ├── types/               # Expression, Value, Currency, Unit
│   ├── lino/                # Links notation
│   ├── lib.rs               # Calculator struct, plan/execute API
│   ├── wasm.rs              # WASM bindings
│   └── ...
├── tests/                   # Rust integration tests
├── web/
│   ├── src/
│   │   ├── components/      # React components
│   │   ├── hooks/           # Custom React hooks
│   │   ├── i18n/            # Internationalization
│   │   ├── utils/           # Utilities
│   │   ├── examples/        # Example expressions
│   │   ├── worker.ts        # Web Worker (plan→execute pipeline)
│   │   ├── worker-rate-coordination.ts
│   │   ├── App.tsx          # Main React app
│   │   └── types.ts         # TypeScript type definitions
│   ├── e2e/                 # Playwright E2E tests
│   └── public/              # Static assets + WASM pkg
├── data/currency/           # Historical rate .lino files
├── changelog.d/             # Changelog fragments
├── scripts/                 # Build/release automation
└── .github/workflows/       # CI/CD pipelines
```
