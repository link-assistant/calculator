# Case Study: Issue #78 — CI/CD Version Increment Not Working

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/78
**Status:** Root cause found and fixed
**Severity:** High — the automated release pipeline never committed version bumps to git

### Symptoms

1. `Cargo.toml` version stuck at `0.1.0` on `main` branch despite GitHub release `v0.2.0` existing
2. 37+ changelog fragments accumulated in `changelog.d/` without being cleared
3. WASM-compiled web app footer showed wrong version (because it reads from `CARGO_PKG_VERSION` at compile time, which remained `0.1.0`)
4. Auto-release job ran on every push to `main` but logged "No changes to commit" after modifying Cargo.toml

---

## Timeline

| Date | Event |
|------|-------|
| 2025-01-25 | Initial `v0.1.0` release published to crates.io |
| 2026-02-25T00:11 | Auto-release job runs for PR #66, logs "Updated Cargo.toml to 0.2.0" but "No changes to commit" — yet v0.2.0 release is created on GitHub and published to crates.io |
| 2026-02-25T18:27 | Auto-release job fails with npm 403 error (investigated in issue #76) |
| 2026-02-25T20:18 | PR #77 merged (pins npm packages), auto-release runs again — same "No changes to commit" pattern |
| 2026-02-25 (this fix) | Root cause identified and fixed |

---

## Root Cause Analysis

### Root Cause 1: `git diff --cached --quiet` exit code not properly handled in `version-and-commit.mjs`

**File:** `scripts/version-and-commit.mjs`
**Line:** The `try/catch` block for detecting staged changes

**The Code:**
```javascript
// Check if there are changes to commit
try {
  await $`git diff --cached --quiet`.run({ capture: true });
  // No changes to commit
  console.log('No changes to commit');
  setOutput('version_committed', 'false');
  setOutput('new_version', newVersion);
  return;
} catch {
  // There are changes to commit (git diff exits with 1 when there are differences)
}
```

**The Bug:** `command-stream`'s `$` function has `errexit: false` as the default setting (see `$.state.mjs` line 26: `errexit: false`). This means `$` tagged template literals **do NOT throw on non-zero exit codes by default**.

- `git diff --cached --quiet` exits with code **0** if there are NO staged changes
- `git diff --cached --quiet` exits with code **1** if there ARE staged changes
- With `errexit: false`, `await $\`git diff --cached --quiet\`` **never throws**, regardless of exit code
- Therefore, the `catch` block is **never reached**
- "No changes to commit" is **always** logged, even when Cargo.toml and CHANGELOG.md are staged

**Evidence:**
- CI run [22414258821](https://github.com/link-assistant/calculator/actions/runs/22414258821): "Updated ./Cargo.toml to version 0.2.0" → "No changes to commit"
- `get-version.mjs` running immediately after reports "Current version: 0.2.0" — confirming the file WAS written
- Same pattern seen in run [22375851874](https://github.com/link-assistant/calculator/actions/runs/22375851874)

**Fix:** Replace `try/catch` with explicit exit code check:
```javascript
const diffResult = await $`git diff --cached --quiet`.run({ capture: true });
if (diffResult.code === 0) {
  // No changes to commit
  console.log('No changes to commit');
  setOutput('version_committed', 'false');
  setOutput('new_version', newVersion);
  return;
}
// There are changes to commit
```

### Root Cause 2: Cargo.toml version out of sync

Because the version commit was never made to git, `Cargo.toml` on `main` still shows `0.1.0` even though:
- crates.io has `link-calculator@0.2.0` published
- GitHub has a `v0.2.0` release
- A `v0.2.0` tag exists on GitHub

**Fix:** Manually bump `Cargo.toml` to `0.2.0` and clean up `CHANGELOG.md` to reflect the accumulated changes.

### Root Cause 3: Changelog fragments never cleaned from git

The 37+ changelog fragments in `changelog.d/` were deleted from disk in CI but since the commit never happened, they remain in the git repository. This causes every subsequent push to `main` to:
1. Find 37+ fragments (they still exist in git)
2. Attempt another release (same "No changes to commit" loop)

**Fix:** Clean the `changelog.d/` directory as part of this fix commit.

---

## Secondary Issues Found

### Issue #76 (Resolved): npm 403 in CI
The auto-release job also failed with `npm error 403` when fetching `command-stream@latest` via `use-m`. Fixed in PR #77 by pinning package versions.

### Version Display in Web Footer
The web app displays version from `Calculator.version()` (WASM), which reads `CARGO_PKG_VERSION` at compile time. Since Cargo.toml was stuck at `0.1.0`, the deployed web app showed `v0.1.0` in the footer even after the v0.2.0 GitHub release.

**Fix:** Once `Cargo.toml` is bumped to `0.2.0` and the WASM is rebuilt and deployed, the footer will show `v0.2.0`.

---

## Data Collected

- `ci-run-22414258821.log` — latest CI run showing "No changes to commit" bug
- `ci-run-22375851874.log` — earlier run with same bug, confirms it's systemic
- `ci-run-22410298646.log` — npm 403 failure run (issue #76)
- `rust-template-release.yml` — reference template workflow for comparison

---

## Fix Summary

1. **`scripts/version-and-commit.mjs`**: Replace `try/catch` exit code detection with explicit `diffResult.code` check
2. **`Cargo.toml`**: Bump version to `0.2.0` to match published release
3. **`CHANGELOG.md`**: Add consolidated changelog entry for all changes since `0.1.0`
4. **`changelog.d/`**: Clean up all accumulated changelog fragments (they are consumed into CHANGELOG.md)

---

## How the Bug Was Introduced

The `try/catch` pattern was probably inspired by shell scripting where `set -e` causes scripts to exit on error. The equivalent in `command-stream` would be enabling `errexit` mode. The bug was introduced when the code was converted from shell scripts to Node.js scripts without accounting for this behavioral difference.

---

## Upstream Issue

This bug could affect any project using `command-stream` with the same pattern. The library's default `errexit: false` behavior is not obvious and differs from the "throw on error" expectation of `async/await` code. Consider reporting to the `command-stream` maintainers.

---

## References

- Issue #78: https://github.com/link-assistant/calculator/issues/78
- Issue #76 (npm 403): https://github.com/link-assistant/calculator/issues/76
- PR #66 (version fix attempt): https://github.com/link-assistant/calculator/pull/66
- PR #77 (npm pinning fix): https://github.com/link-assistant/calculator/pull/77
- command-stream source: https://www.npmjs.com/package/command-stream
- CI run 22414258821: https://github.com/link-assistant/calculator/actions/runs/22414258821
- CI run 22375851874: https://github.com/link-assistant/calculator/actions/runs/22375851874
