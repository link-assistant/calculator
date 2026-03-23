# Case Study: Issue #109 — CI/CD Build Failure (Type Mismatch)

## Summary

All CI/CD workflows on the `main` branch were failing due to a Rust compilation error (`E0308: mismatched types`) in `src/grammar/number_grammar.rs:94`.

## Affected CI Runs

| Run ID | Workflow | Failed Step | Conclusion |
|--------|----------|-------------|------------|
| [23405442443](https://github.com/link-assistant/calculator/actions/runs/23405442443) | Update Screenshots | Build WASM package | failure |
| [23405442430](https://github.com/link-assistant/calculator/actions/runs/23405442430) | CI/CD Pipeline | Run Clippy, Run tests | failure |

Both runs were triggered by commit `98ad7f0b` (merge of PR #97).

## Timeline of Events

1. **Commit `6c7639b9`** — Added duration unit support with `parse_unit()` returning `Result<Unit, CalculatorError>`. Duration parsing at line 94 correctly returned `Ok(Unit::Duration(dur))`.

2. **Commit `07275375`** (PR #97, issue #104) — Refactored `parse_unit()` into `parse_unit_with_alternatives()` to support unit ambiguity detection (e.g., "ton" as mass vs. cryptocurrency). The return type changed to `Result<(Unit, Vec<Unit>), CalculatorError>`. All return paths were updated to return tuples **except** the duration branch at line 94, which still returned `Ok(Unit::Duration(dur))` — a bare `Unit` instead of the expected `(Unit, Vec<Unit>)` tuple.

3. **Commit `1af65334`** — Style fixes (import reordering) were applied but the type mismatch was not caught.

4. **Commit `98ad7f0b`** — PR #97 was merged to `main`, causing all subsequent CI runs to fail.

## Root Cause

The refactoring in commit `07275375` changed the function signature of `parse_unit_with_alternatives` from returning `Result<Unit, ...>` to `Result<(Unit, Vec<Unit>), ...>` but missed updating the duration-parsing branch. This was a straightforward oversight during the refactoring.

The likely reason it was missed: the duration branch was added in a different commit (`6c7639b9`) and may not have been visible during the refactoring, or it was simply overlooked.

## Fix

Changed line 94 from:

```rust
return Ok(Unit::Duration(dur));
```

to:

```rust
return Ok((Unit::Duration(dur), alternatives));
```

This matches the pattern used by all other return paths in the function (e.g., `DataSize`, `Mass`, `Currency`).

## Lessons Learned

1. **Always run `cargo check` before merging**: This error would have been caught immediately by a compilation check. The CI pipeline does run these checks, but the PR was merged despite CI failures, or CI was not required to pass before merge.

2. **Signature-changing refactors need exhaustive review**: When changing a function's return type, every `return` and tail expression in the function body must be updated. IDE "find all references" or compiler errors should catch this, but only if the code is compiled before merging.

## Attached Logs

- `ci-logs-run1.log` — Full logs from "Update Screenshots" workflow (run 23405442443)
- `ci-logs-run2.log` — Full logs from "CI/CD Pipeline" workflow (run 23405442430)
