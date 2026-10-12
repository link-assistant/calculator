---
title: "Keep the imported open-source calculator test suites current and add the missing engines"
---
Part of #227.

## Problem

`scripts/import-competitor-corpus.mjs` extracts single-expression assertions from five upstream suites (fend 1098 rows, libqalculate 656, Numbat 289, Rink 210, math.js 297). The import is a one-off snapshot: the upstream revisions are not pinned, and several open-source calculators that publish tests are not imported at all.

| Engine | Test source | License | Status |
| --- | --- | --- | --- |
| fend | `core/tests/integration_tests.rs` | MIT | imported |
| libqalculate | `tests/*.batch` (17 files) | GPL-2.0-or-later | imported |
| Numbat | `numbat/tests/interpreter.rs` | MIT OR Apache-2.0 | imported |
| Rink | `core/tests/query.rs` | MPL-2.0 | imported |
| math.js | `test/unit-tests/expression/parse.test.js` | Apache-2.0 | imported (parse tests only) |
| Insect | `src/Insect/*.purs` tests | MIT | not imported |
| kalker | `kalk/src/integration_testing.rs` | MIT | not imported |
| Hurmet | `test/*.js` | MIT | not imported |
| GNU Units | manual examples only | GPL-3.0 | not imported |
| SpeedCrunch | `src/tests/*.cpp` | GPL-2.0 | not imported |
| Numara | `src/js` (no expression tests, uses math.js) | MIT | covered through math.js |

## Acceptance criteria

- The importer records the upstream commit SHA and date in the generated TSV header, and the case-study `competitor-inventory.md` lists them.
- Importers exist for Insect, kalker, Hurmet and SpeedCrunch, and math.js unit/function tests are imported in addition to parse tests.
- A documented command (`npm run import-corpus` or a Makefile target) refreshes all suites from a directory of checkouts; a scheduled workflow opens a PR when the imported corpus changes.
- Upstream data is never relicensed: each block keeps the license line and URL already emitted by the importer.

## Solution options

1. Extend the existing regex-based extractors per engine (current approach, fast, tolerant of upstream refactors).
2. Run each upstream engine in CI to generate expected values for arbitrary expressions. Gives ground truth for new expressions, but needs five toolchains (Rust, PureScript, Node, C++, Haskell) and is better done as an opt-in local script.

## Blockers

None.
