# Contributing to rust-ai-driven-development-pipeline-template

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/rust-ai-driven-development-pipeline-template.git
   cd rust-ai-driven-development-pipeline-template
   ```

2. **Install Rust**

   Install Rust using rustup (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install development tools**

   ```bash
   rustup component add rustfmt clippy
   ```

4. **Install pre-commit hooks** (optional but recommended)

   ```bash
   pip install pre-commit
   pre-commit install
   ```

5. **Build the project**

   ```bash
   cargo build
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Write code following the project's style guidelines
   - Add tests for any new functionality
   - Update documentation as needed

3. **Run quality checks**

   ```bash
   # Format code
   cargo fmt

   # Run Clippy lints
   cargo clippy --all-targets --all-features

   # Check file sizes
   node scripts/check-file-size.mjs

   # Run all checks together
   cargo fmt --check && cargo clippy --all-targets --all-features && node scripts/check-file-size.mjs
   ```

4. **Run tests**

   Local agent runs execute the JavaScript tests by default and build Rust
   only when a change needs it (see [Rust Build Load](#rust-build-load)):

   ```bash
   # JavaScript checks: run first, by default
   node --test scripts/*.test.mjs
   npm test --prefix web

   # Rust checks: only when needed, through the bounded wrapper
   node scripts/bounded-cargo.mjs test

   # Run all tests
   cargo test

   # Run tests with verbose output
   cargo test --verbose

   # Run doc tests
   cargo test --doc

   # Run a specific test
   cargo test test_name
   ```

5. **Add a changelog fragment**

   For any user-facing changes, create a changelog fragment:

   ```bash
   # Create a new file in changelog.d/
   # Format: YYYYMMDD_HHMMSS_description.md
   touch changelog.d/$(date +%Y%m%d_%H%M%S)_my_change.md
   ```

   Edit the file to document your changes:

   ```markdown
   ### Added
   - Description of new feature

   ### Fixed
   - Description of bug fix
   ```

   **Why fragments?** This prevents merge conflicts in CHANGELOG.md when multiple PRs are open simultaneously.

6. **Commit your changes**

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Pre-commit hooks will automatically run and check your code.

7. **Push and create a Pull Request**

   ```bash
   git push origin feature/my-feature
   ```

   Then create a Pull Request on GitHub.

## Code Style Guidelines

This project uses:

- **rustfmt** for code formatting
- **Clippy** for linting with pedantic and nursery lints enabled
- **cargo test** for testing

### Code Standards

- Follow Rust idioms and best practices
- Use documentation comments (`///`) for all public APIs
- Write tests for all new functionality
- Keep functions focused and reasonably sized
- Keep files under 1000 lines
- Use meaningful variable and function names

### Documentation Format

Use Rust documentation comments:

```rust
/// Brief description of the function.
///
/// Longer description if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when errors are returned
///
/// # Examples
///
/// ```
/// use my_package::example_function;
/// let result = example_function(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn example_function(arg1: i32, arg2: i32) -> i32 {
    arg1 + arg2
}
```

## Testing Guidelines

- Write tests for all new features
- Maintain or improve test coverage
- Use descriptive test names
- Organize tests in modules when appropriate
- Use `#[cfg(test)]` for test-only code

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod my_feature_tests {
        use super::*;

        #[test]
        fn test_basic_functionality() {
            assert_eq!(my_function(), expected_result);
        }

        #[test]
        fn test_edge_case() {
            assert_eq!(my_function(edge_case_input), expected_result);
        }
    }
}
```

## Rust Build Load

A full `cargo test` compiles the library, the CLI and every integration test
binary. Several agents doing that on one machine at the same time exhausted
its memory and disk (#254), so the Rust build load is bounded everywhere:

- **Local agent runs execute the JavaScript tests by default**:
  `node --test scripts/*.test.mjs` and `npm test --prefix web`. Build Rust
  only when the change touches Rust, the WASM bindings or a check that needs
  the binary, and then run Cargo through `scripts/bounded-cargo.mjs`.
- `.cargo/config.toml` limits Cargo to two parallel rustc processes, turns off
  incremental compilation (`CARGO_INCREMENTAL=0`) and reduces debug info:
  none for dependencies, line tables for this crate. CI sets
  `CARGO_BUILD_JOBS=4`, the core count of a standard runner.
- `node scripts/bounded-cargo.mjs <cargo arguments>` is the documented local
  workflow. It queues concurrent builds behind a lock file, sets
  `CARGO_INCREMENTAL=0` and the job limit, caps every process's address space
  with `prlimit` on Linux, and prunes the target directory before and after
  the build when it exceeds its disk budget: the incremental caches first,
  then the whole directory.
- CI caches through `.github/actions/setup-rust`, keyed by the job,
  `rust-toolchain.toml` and every `Cargo.lock`, with no restore keys: a new
  lock file starts from an empty `target` instead of piling new artifacts onto
  the old ones, so the cache cannot grow without bound.

The wrapper reads these variables, all optional:

| Variable | Default | Meaning |
| --- | --- | --- |
| `CALCULATOR_CARGO_TARGET_DIR` | `target/` | Shared target directory, for example one per machine for all worktrees |
| `CALCULATOR_CARGO_MAX_GIB` | `8` | Disk budget of the target directory |
| `CALCULATOR_CARGO_MEMORY_GIB` | `4` | Address-space limit per process (Linux) |
| `CALCULATOR_CARGO_JOBS` | `2` | Parallel rustc processes |
| `CALCULATOR_CARGO_LOCK_WAIT_SECONDS` | `3600` | How long to wait for another build |
| `BOUNDED_CARGO_VERBOSE` | off | `1` prints the sizes, lock owner and command |

```bash
# One shared, bounded build directory for every worktree on this machine
CALCULATOR_CARGO_TARGET_DIR=$HOME/.cache/calculator-target \
  node scripts/bounded-cargo.mjs clippy --all-targets --all-features
```

## CI Stages

`release.yml` (CI/CD Pipeline) runs in two stages (#254):

1. **JavaScript stage.** Script checks, changelog and version checks, secret
   scan, npm freshness, the `js/` lint and type check, and the `js/` tests on
   Node 20, Node 24 and Bun. Every job has its own concurrency group with
   `cancel-in-progress: true`, so a new push cancels the old JavaScript jobs.
   The `JavaScript Stage` job reports the result.
2. **Rust stage.** The `rust-stage` job calls `rust.yml` only after every
   JavaScript job passed. Its group is per branch with
   `cancel-in-progress: false`, so a started Rust stage always finishes.

`scripts/check-javascript-first.mjs` fails the Script Checks job when Cargo,
rustup, wasm-pack or CodeQL for Rust appears outside `rust.yml`, when a Rust
workflow gains its own trigger, or when the gate above is weakened.

The required checks of `main` are `JavaScript Stage` and
`Rust Stage / Rust Stage`, defined in `.github/main-ruleset.json`. Rulesets
are repository settings, so a pull request cannot apply them; until an admin
runs `gh api -X POST repos/link-assistant/calculator/rulesets --input .github/main-ruleset.json`,
Script Checks warns on pull requests and fails on `main`.

`node scripts/ci-runs-per-pr.mjs 5` counts CI runs and runs that reached the
Rust stage for the last five merged pull requests; the baseline is in
`docs/case-studies/issue-254/runs-per-pr.md`.

## Multi-Agent Work

When several agents work on one change, drafting sub-agents never push. One
CI/CD sub-agent owns `git push` and pushes once, after collecting all failures
of the latest run and never while a run is in progress on the same commit
range. [AGENTS.md](AGENTS.md) has the full rules; the gate is a script:

```bash
# Exit 1 while a run on this branch's commits is queued or in progress;
# otherwise save the failed-job logs of the latest pushed commit to ci-logs/.
node scripts/ci-push-gate.mjs --verbose
```

## Pull Request Process

1. Ensure all tests pass locally
2. Update documentation if needed
3. Add a changelog fragment (see step 5 in Development Workflow)
4. Ensure the PR description clearly describes the changes
5. Link any related issues in the PR description (see [Closing Issues](#closing-issues))
6. Wait for CI checks to pass
7. Address any review feedback

## Closing Issues

GitHub closes every issue that a closing keyword references when the PR is
merged into `main`, whatever the rest of the description says (#247: PR #228
said "closes only #227", yet its `Fixes` lines closed 11 unimplemented
sub-issues). The **Pull Request Description** workflow
(`scripts/check-closing-references.mjs`) runs on every description edit and
fails when:

- an issue has a closing keyword (`close`, `closes`, `closed`, `fix`, `fixes`,
  `fixed`, `resolve`, `resolves`, `resolved`) and a sentence says it stays
  open, is kept open, or is not closed (`keep-open`);
- the description says the PR "closes only #N" while keywords close others
  (`closes-only`);
- an issue is referenced both as `Part of #N` / `Refs #N` and with a keyword
  (`part-of`);
- several issues share one keyword: `Fixes #1, #2` and `Fixes #1 and #2`
  close only #1 (`missing-keyword`);
- a keyword is negated: "does not close #N" still closes #N (`negated-keyword`).

Write one keyword per issue the PR implements, each on its own line:

```markdown
Fixes #260
Fixes #247
```

Mention issues the PR only plans, documents or prepares without a keyword:
"Part of #227" or "#236 to #246 stay open". If an automated tool appends
closing references for a whole sub-issue tree, remove the ones for issues that
stay open before merging. Keywords inside code blocks, inline code and HTML
comments are ignored. Check a description locally with:

```bash
node scripts/check-closing-references.mjs --body-file description.md --verbose
# What GitHub will close, including issues linked in the Development sidebar:
gh pr view <number> --json closingIssuesReferences --jq '.closingIssuesReferences[].number'
```

## Competitor Coverage

CI builds the release CLI and checks all competitor corpus expressions against
`docs/case-studies/issue-227/results/coverage-baseline.json`. A status becoming
worse (`supported` → `different` → `unsupported` → `timeout`), or an expression
disappearing, fails the Competitor Coverage job. The regenerated
`coverage-report.md` is available in the `competitor-coverage-report` workflow
artifact, including the individual expressions with different results.

Run the same check locally, keeping generated output separate from the baseline:

```bash
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs \
  --baseline docs/case-studies/issue-227/results/coverage-baseline.json \
  --out ci-logs/competitor-coverage --quiet
```

When an engine change improves coverage, update the baseline **in the same PR**:
first run the check above and investigate any regressions, then regenerate the
committed baseline and report:

```bash
node scripts/competitor-coverage.mjs --quiet
git diff -- docs/case-studies/issue-227/results/coverage-baseline.json
git add docs/case-studies/issue-227/results/coverage-baseline.json \
  docs/case-studies/issue-227/results/coverage-report.md
```

Include the coverage improvement and any intentional corpus changes in the PR
description. Do not refresh the baseline to hide a regression. Corpus and
baseline changes also trigger the CI pipeline.

The Coverage Gap Report workflow runs every Monday at 06:00 UTC, whenever the
baseline changes on `main`, and on manual dispatch. It regenerates the full
report artifact and creates or updates one **Coverage gap report** issue with
the source and language tables, the measured commit, a link to the workflow
run and the open sub-issues of #227 that implement the missing features. The
same tables are in the run's job summary.

The report issue is a status page, not a work item. The workflow never reopens
it, so it can be closed (for example by a pull request that addresses every
open issue) and is still updated in place (#258). Work on the gaps through the
sub-issues of #227. The workflow fails only when gaps remain and none of those
sub-issues is open: then open one for the next missing feature. Preview the
issue body without changing anything:

```bash
node scripts/update-coverage-gap-issue.mjs --dry-run --verbose
```

## Changelog Management

This project uses a fragment-based changelog system similar to [Scriv](https://scriv.readthedocs.io/) (Python) and [Changesets](https://github.com/changesets/changesets) (JavaScript).

### Creating a Fragment

```bash
# Create a new fragment with timestamp
touch changelog.d/$(date +%Y%m%d_%H%M%S)_description.md
```

### Fragment Categories

Use these categories in your fragments:

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed in future
- **Removed**: Features that were removed
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### During Release

Fragments are automatically collected into CHANGELOG.md during the release process. The release workflow:

1. Collects all fragments
2. Updates CHANGELOG.md with the new version entry
3. Removes processed fragment files
4. Bumps the version in Cargo.toml
5. Creates a git tag and GitHub release

## Project Structure

```
.
├── .github/workflows/    # GitHub Actions CI/CD
├── changelog.d/          # Changelog fragments
│   ├── README.md         # Fragment instructions
│   └── *.md              # Individual changelog fragments
├── examples/             # Usage examples
├── scripts/              # Utility scripts
├── src/
│   ├── lib.rs            # Library entry point
│   └── main.rs           # Binary entry point
├── tests/                # Integration tests
├── .gitignore            # Git ignore patterns
├── .pre-commit-config.yaml  # Pre-commit hooks
├── Cargo.toml            # Project configuration
├── CHANGELOG.md          # Project changelog
├── CONTRIBUTING.md       # This file
├── LICENSE               # Unlicense (public domain)
└── README.md             # Project README
```

## Release Process

This project uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Releases are managed through GitHub releases. To trigger a release:

1. Manually trigger the release workflow with a version bump type
2. Or: Update the version in Cargo.toml and push to main

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions and general help
- Check existing issues and PRs before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing!

## Refreshing upstream test suites

Run `make checkout-corpus CORPUS_CHECKOUTS=/path/to/checkouts` to prepare the
pinned upstream revisions, then `make import-corpus CORPUS_CHECKOUTS=/path/to/checkouts`
to regenerate the open-source corpus, revision lock and competitor inventory.
Pass `CORPUS_LATEST=--latest` to the checkout target to refresh to current
upstream revisions. See [the corpus refresh guide](docs/competitor-corpus.md)
for extraction limits, provenance and the weekly refresh PR workflow.
