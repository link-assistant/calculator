# Reproducing the documentation corpus

Issue #231 expands the original 343-row corpus into attributed examples from
the public calculator references in [competitor-inventory.md](competitor-inventory.md).
The file name `closed-source-docs.tsv` is historical: Hurmet and SpeedCrunch
also have open-source implementations. Fetch dates are UTC dates in the TSV
header and in [documentation-sources.json](documentation-sources.json).

## Fetch and extraction

From the repository root, with Node.js and curl installed:

```sh
node experiments/issue-227/refresh-documentation-corpus.mjs --cache ci-logs/documentation-cache
node experiments/issue-227/write-corpus-summary.mjs
node --test scripts/documentation-*.test.mjs
```

Use a new cache directory to fetch a fresh snapshot. Existing cached pages
retain their original fetch dates. To reproduce extraction from that snapshot:

```sh
node experiments/issue-227/refresh-documentation-corpus.mjs --offline --cache ci-logs/documentation-cache
```

Every product has an `extract-PRODUCT-examples.mjs` entry point in
`experiments/issue-227/`; the product identifiers appear in the inventory.
For example, this writes only Numi's extracted TSV rows to stdout:

```sh
node experiments/issue-227/extract-numi-examples.mjs --offline --cache ci-logs/documentation-cache
```

The refresh command also accepts `--source numi`, `--out PATH` and `--audit PATH`.
A source refresh preserves other products in the existing output. The shared
fetcher caches page bytes and records SHA-256 hashes. Indexed references are
discovered from their own links; missing cached pages, changed Notion pagination,
missing structured input data and unexpected empty extractions fail visibly.
Network requests and downloaded code are never evaluated.

## Reference boundaries and interpretation

- Soulver: every syntax-reference and documentation Markdown page in its
  published `llms.txt`, including HTML/Markdown tables, fenced examples,
  expression/answer lines and inline worked pairs.
- Numi: the complete wiki Home document, including tables, indentation examples,
  variables, functions, units and numeric inline snippets. JavaScript extension
  API snippets and installation commands are outside calculator syntax.
- Calca: the complete reference and the homepage's worked calculator examples.
- Parsify: Getting Started and Custom Units from its public Notion block data.
  Code examples and numeric custom-unit definitions are extracted; aliases,
  plugins, configuration prose and application instructions are excluded.
- Hurmet: calculation tutorial and calculation reference in the manual.
  Markdown/TeX document editing and JavaScript batch integration are excluded.
  Both literal code examples and rendered calculation `data-entry` inputs are
  included; grammar fragments and symbolic inputs may require surrounding state.
- SpeedCrunch: first steps, syntax, advanced use and all linked function,
  constant, integer, unit and IEEE-754 references. Definitions remain inputs;
  answer chains use the final answer. Answer-side comments are excluded.
  Syntax fragments, partial input and documented error examples are retained as
  documentation coverage; the current runner cannot distinguish expected errors.
- Frink: annotated calculator input and code paragraphs throughout its manual.
  Multiline programs are represented by short code lines; comments, launcher
  commands and Java integration scaffolding are excluded. A line can depend on
  definitions or control flow shown elsewhere in its original block.
- Wolfram|Alpha: all recursively linked example pages under Mathematics,
  Units & Measures and Dates & Times. Structured input queries are extracted;
  decorative captions and rendered output are excluded. Other encyclopedic
  domains are outside calculator syntax. Repeated expressions share one corpus
  identity; every fetched page and its count remains in the source manifest.
- Google and Raycast: the official calculator reference/help pages, including
  linked Google example queries and Raycast's literal code examples.
- Apple Math Notes: the linked iPad/iPhone math, graph, conversion and basic
  calculator help pages. Worked travel-budget and caffeine-graph inputs are
  transcribed from their documentation images. Button-only conversion displays
  are not invented as typed expression syntax.

The four official help references below have no typed calculator examples:

- Windows Calculator describes modes and unit/currency converter buttons.
- Samsung describes calculator buttons, history and conversion; its calculator
  screenshot has an empty display.
- Xiaomi shows app-launch instructions, a keypad at zero and mortgage controls.
- DuckDuckGo's Instant Answers help mentions calculations without calculator
  syntax examples.

These products have per-source scripts, fetch dates, hashes and explicit
zero-example explanations in the manifest. Their absence of typed examples is
an evidenced source limitation, not fabricated coverage. Samsung and Xiaomi's
images were downloaded, validated as images and inspected. Image URLs, hashes
and Apple's short transcriptions live in
`experiments/issue-227/image-examples.json`; a changed image fails extraction and
requires reviewing the transcription. Images and full documentation prose are
kept only in the local cache, outside the committed corpus.

## Expected answers and historical rows

Each row has the full reference-page URL in its note. Date, timezone, random,
live exchange-rate, tax/inflation and locale-sensitive answers are blank:
the coverage runner requires evaluation without comparing a stale answer.
Explicit fixed-rate arithmetic remains deterministic. Missing published answers
are also blank; the extraction does not calculate replacement answers.

The original 343 source/language/expression identities remain in
`experiments/issue-227/legacy-examples.tsv` and are merged into each refresh.
Its 77 hand-curated examples are labeled as historical seeds, not asserted to
appear verbatim on the current reference page. A fresh documented match replaces
that note. A historical deterministic answer is retained when the current page
omits an answer. The initial migration can be reproduced with
`prepare-legacy.mjs ORIGINAL.tsv`; it is not part of routine refreshes.

Variables, prior-result operators and image worksheets can require preceding
definitions. The existing coverage runner evaluates individual rows, so those
rows measure unsupported syntax or missing context rather than reproduce a
whole notebook session. This change expands documentation evidence; it does not
add calculator language features or claim support for every competitor example.

Only attributed short expression/answer pairs and code snippets are committed,
not copied explanatory prose or full pages. `inspect-docs.py` is a local research
helper for inspecting fetched HTML and does not generate corpus rows.

## Verification

```sh
cargo build --locked --release --bin link-calculator
node scripts/competitor-coverage.mjs --binary experiments/issue-227/bounded-calculator.sh --baseline docs/case-studies/issue-227/results/coverage-baseline.json
```

The Linux/bash wrapper caps each CLI process at 512 MiB of virtual memory and
8 MiB of stack. The runner retains its existing five-second per-expression
timeout and output limit. This bounds extreme examples such as SpeedCrunch's
`1e+536870911`. CI uses the same wrapper. Offline regression tests cover missing
products, provenance, historical identities, unstable oracles, every parser
format, bitwise OR, assignments, answer chains, page discovery and failed image
downloads; live network access is not required by the tests.
