# Refreshing the open-source calculator corpus

The importer reads nine clean Git checkouts without building or executing any
upstream engine. Node.js 24+, Git and Make are sufficient. Checkouts use the
names `fend`, `qalculate`, `numbat`, `rink`, `mathjs`, `insect`, `kalker`, `hurmet`
and `speedcrunch`. Keep them outside this repository to avoid bundling upstream
code under this project's Unlicense.

## Reproduce the committed snapshot

```sh
make checkout-corpus CORPUS_CHECKOUTS=/tmp/calculator-upstreams
make import-corpus CORPUS_CHECKOUTS=/tmp/calculator-upstreams
```

`checkout-corpus` clones/fetches the exact 40-character commit IDs recorded in
[scripts/competitor-upstreams.json](../scripts/competitor-upstreams.json).
Already pinned clean checkouts can be reused offline. If you have your own
checkouts in the named subdirectories, only `import-corpus` is necessary.
Dirty checkouts are rejected so a SHA never attributes uncommitted test data.
The checkout helper also rejects unexpected Git origins and preserves local edits.

The importer updates three artifacts together after all suites are validated:

- `docs/case-studies/issue-227/corpus/open-source-tests.tsv`: assertion rows,
  original per-engine license and source URL, SHA, Git commit date and pinned URL.
- `docs/case-studies/issue-227/competitor-inventory.md`: revision/date table,
  extraction source, license and imported row count.
- `scripts/competitor-upstreams.json`: revisions to use for the next reproduction.

Output has no refresh timestamp and uses a fixed engine/file order. Repeating
an import from the same revisions produces identical files. Dates are upstream
Git commit dates (`%cI`), including the recorded timezone. The inventory's
text outside the generated markers is preserved.

## Import newer revisions

```sh
make checkout-corpus CORPUS_CHECKOUTS=/tmp/calculator-upstreams CORPUS_LATEST=--latest
make import-corpus CORPUS_CHECKOUTS=/tmp/calculator-upstreams
node --test scripts/*.test.mjs
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs \
  --baseline docs/case-studies/issue-227/results/coverage-baseline.json \
  --out ci-logs/competitor-coverage --quiet
```

Review changed expectations and removed expressions before updating the coverage
baseline. Run the coverage check against the existing baseline first, as described
in [CONTRIBUTING.md](../CONTRIBUTING.md#competitor-coverage). The refresh script
never resets it. New rows can expose unsupported capabilities; importing them
is not a claim that our calculator supports them.

Without Make, run `node scripts/checkout-competitor-suites.mjs --checkouts <dir>`
(with `--latest` for an update), then
`node scripts/import-competitor-corpus.mjs --checkouts <dir>`.
The importer accepts `--out`, `--inventory` and `--lock` for separate outputs,
and `--verbose` for per-suite tracing. A missing checkout, an empty extractor or
an unrecognized source format fails before generated files are written.

## Extraction boundaries

These are static assertion extractors, not complete ports of upstream runners.
They copy literal expectations, never calculate answers from our engine or
execute upstream JavaScript, Rust, PureScript or C++.

| Engine | Extraction |
| --- | --- |
| fend | Original `test_eval` / `test_eval_simple` string pairs. |
| libqalculate | Sorted `tests/*.batch`; expression plus first indented result. |
| Numbat | Original `expect_output` string pairs. |
| Rink | Original `test` string pairs. |
| math.js | Original numeric/boolean `parseAndEval` assertions plus literal equality/approximate assertions from expression, function and Unit tests. API calls with literal arguments are translated into expression syntax, e.g. `math.sqrt(4)` → `sqrt(4)`, `new Unit(5, 'cm').toString()` → `unit(5, "cm")`. Shared variables, object properties, custom instances and nonliteral expectations are skipped. Additional extraction stops at the first shared-instance configuration, import or unit mutation in each file, conservatively excluding later assertions even if a hook restores the state. |
| Insect | `expectOutput'` assertions using `initialEnvironment` in `test/Main.purs`; environment-dependent tests and error messages are skipped. |
| kalker | Files enumerated by `kalk/src/integration_testing.rs`; single-line equality assertions with numeric/boolean right sides, excluding references to variables/functions defined in the file. Compound programs and nonliteral right sides are skipped. |
| Hurmet | Literal `calcTests` triples in `test/test.cjs`; calculation result, not TeX/RPN parser output. Shared `vars` references marked by `¿`, expected errors and extra output-format directives are skipped. Plain `= @`/`= @@` output markers are removed and the original input is retained in the note. |
| SpeedCrunch | Literal `CHECK_EVAL` / `CHECK_EVAL_PRECISE` pairs in `src/tests/*.cpp`; Qt literal wrappers are supported. Sections that change settings or define user functions are skipped, as are assignments and references to variables defined in the section. The official source moved to `heldercorreia/speedcrunch`. |

The five original imports remain intact for coverage-baseline continuity,
including their original string-escape handling. Additional extractors exclude
comments, decoded multiline/control strings and JavaScript template interpolation.
Corpus rows are references under the upstream licenses, **not** public-domain
code or data. Each block retains its original license and URL; pinned URLs and
row notes allow reviewers to locate the exact assertions. GNU Units has manual
examples rather than a suite in this import; Numara uses math.js; Mathics remains
outside issue #230's requested engines.

## Scheduled updates

[Refresh Competitor Corpus](../.github/workflows/refresh-competitor-corpus.yml)
runs each Sunday at 06:00 UTC and supports manual dispatch. It fetches upstream
default branches, imports all suites, runs the importer tests, and opens or
updates one PR on `automation/competitor-corpus` when the generated files change.
It does not push to the default branch or alter the coverage baseline.

Enable **Allow GitHub Actions to create and approve pull requests** in repository
Actions settings. The default `GITHUB_TOKEN` needs contents and pull-request write
permissions. To have the refresh PR trigger normal pull-request CI automatically,
configure `CORPUS_REFRESH_TOKEN` with repository contents and pull-request write
access. GitHub suppresses new workflow runs for PRs created by `GITHUB_TOKEN`;
without the dedicated token, run the normal PR checks through the repository's
usual manual review process. Upstream format changes fail the refresh job so a
partial import cannot silently shrink the corpus.
