# Case Study: Issue #78 — CI/CD Version Increment Not Working

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/78
**Status:** Root causes found and fixed (multiple iterations)
**Severity:** High — the automated release pipeline never committed version bumps to git, and the web app always showed a stale version

### Symptoms

1. `Cargo.toml` version stuck at `0.1.0` on `main` branch despite GitHub release `v0.2.0` existing
2. 37+ changelog fragments accumulated in `changelog.d/` without being cleared
3. WASM-compiled web app footer showed wrong version (because it reads from `CARGO_PKG_VERSION` at compile time)
4. Auto-release job ran on every push to `main` but logged "No changes to commit" after modifying Cargo.toml
5. After the first fix (PR#79), a `v0.1.1` release was created instead of the correct `v0.2.1` because Cargo.toml had been reverted to `0.1.0`
6. Web footer still showed `v0.1.0` even after the `v0.1.1` release succeeded, because the CI build/deploy run in parallel with the version bump

---

## Timeline

| Date | Event |
|------|-------|
| 2025-01-25 | Initial `v0.1.0` release published to crates.io |
| 2026-02-25T00:11 | Auto-release job runs for PR #66, logs "Updated Cargo.toml to 0.2.0" but "No changes to commit" — yet v0.2.0 release is created on GitHub and published to crates.io |
| 2026-02-25T18:27 | Auto-release job fails with npm 403 error (investigated in issue #76) |
| 2026-02-25T20:18 | PR #77 merged (pins npm packages), auto-release runs again — same "No changes to commit" pattern |
| 2026-02-26T09:09 | PR #79 merged (fixes `version-and-commit.mjs` bug), auto-release runs → creates `v0.1.1` release (wrong — should have been v0.2.x) |
| 2026-02-26T09:10 | CI run [22435338053](https://github.com/link-assistant/calculator/actions/runs/22435338053): `v0.1.1` committed to git, published to crates.io, GitHub Release created |
| 2026-02-26 | Web footer still shows `v0.1.0` — the web deploy ran in parallel with auto-release using the old Cargo.toml |

---

## Root Cause Analysis

### Root Cause 1 (Fixed in PR#79): `git diff --cached --quiet` exit code not properly handled

**File:** `scripts/version-and-commit.mjs`

**The Bug:** `command-stream`'s `$` function has `errexit: false` as the default setting. This means the `try/catch` block for detecting staged changes **never reached the catch**, because `git diff --cached --quiet` (which exits 1 when there ARE staged changes) never threw an exception.

**Fix:** Replace `try/catch` with explicit `diffResult.code === 0` check.

**Evidence:**
- CI run [22414258821](https://github.com/link-assistant/calculator/actions/runs/22414258821): "Updated ./Cargo.toml to version 0.2.0" → "No changes to commit"
- CI run [22375851874](https://github.com/link-assistant/calculator/actions/runs/22375851874): same pattern

---

### Root Cause 2: Version ordering became wrong after the fix

**Symptom:** After PR#79 fixed the scripting bug, the auto-release created `v0.1.1` instead of the expected `v0.2.1`.

**Why:** In PR#79, commit `47fc9fc1` reverted Cargo.toml from 0.2.0 back to 0.1.0 to pass the `version-check` CI (which blocks manual version changes in PRs). When the fixed pipeline ran, it computed:
```
0.1.0 + patch (one fragment with patch) = 0.1.1
```

But crates.io already had 0.2.0. So the release ordering became: v0.1.0 → v0.2.0 → v0.1.1 (semantically incorrect).

**Fix (PR#80):** Add a `minor` changelog fragment so the next auto-release computes:
```
0.1.1 + minor = 0.2.0 (already exists → pipeline treats as success, syncs git to 0.2.0)
```
After that sync, subsequent releases will compute from 0.2.0 (producing 0.2.1, 0.3.0, etc.).

---

### Root Cause 3: Web footer always shows stale version after release

**Symptom:** After every release, `link-assistant.github.io/calculator/` shows the old version in the footer until the NEXT code change triggers a new CI run.

**Why:** The CI/CD pipeline job dependency graph is:
```
lint + test → wasm-build → web-build → deploy-pages
lint + test → build     → auto-release
```

The `wasm-build`, `web-build`, and `deploy-pages` jobs run **in parallel** with `auto-release`. They check out the code and compile the WASM with the **current** Cargo.toml (before the version bump). The auto-release job then pushes a new commit bumping the version. But since that push uses `GITHUB_TOKEN`, GitHub Actions does **not** trigger a new CI run (by design, to prevent infinite loops).

**Evidence:**
- CI run [22435338053](https://github.com/link-assistant/calculator/actions/runs/22435338053): The `auto-release` job ran at 09:11, pushing commit `7e854bb0` (Cargo.toml = 0.1.1). But the `web-build` and `deploy-pages` jobs had already started earlier (09:10) with the old Cargo.toml (0.1.0). The deployed web app therefore shows `v0.1.0`.

**Fix (PR#80):** Add a new `deploy-after-release` job that:
1. Depends on `auto-release` completing successfully AND having committed a new version
2. Checks out `ref: main` (getting the version-bumped Cargo.toml)
3. Rebuilds the WASM with the new `CARGO_PKG_VERSION`
4. Rebuilds and redeploys the web app to GitHub Pages

---

## Secondary Issues Found

### Issue #76 (Resolved): npm 403 in CI
The auto-release job also failed with `npm error 403` when fetching `command-stream@latest` via `use-m`. Fixed in PR #77 by pinning package versions.

### No crates.io badge in README
The README had no badge linking to the crates.io page. Added `[![Crates.io](https://img.shields.io/crates/v/link-calculator.svg)](https://crates.io/crates/link-calculator)` badge in PR #80.

---

## Data Collected

- `ci-run-22414258821.log` — CI run showing "No changes to commit" bug (pre-fix)
- `ci-run-22375851874.log` — Earlier run with same bug, confirms it's systemic
- `ci-run-22410298646.log` — npm 403 failure run (issue #76)
- `ci-run-22435338053-latest.log` — Latest run showing v0.1.1 correctly committed but web deployed with old version
- `rust-template-release.yml` — Reference template workflow for comparison

---

### Root Cause 4 (Fixed in PR#81): 409 Conflict when uploading GitHub Pages artifact in `deploy-after-release`

**Symptom:** After applying the `deploy-after-release` fix (PR#80), the CI run [22439390199](https://github.com/link-assistant/calculator/actions/runs/22439390199) failed at the "Upload artifact" step of the `deploy-after-release` job with:

```
Failed to CreateArtifact: Received non-retryable error: Failed request: (409) Conflict: an artifact with this name already exists on the workflow run
```

**Why:** `actions/upload-pages-artifact` always uses the fixed artifact name `github-pages` (its hardcoded default). Since both `deploy-pages` and `deploy-after-release` call this action in the **same workflow run**, the second upload in `deploy-after-release` fails with a 409 Conflict because the name is already taken by the first upload in `deploy-pages`.

**Fix:** Use a unique artifact name in `deploy-after-release`:
- Pass `name: github-pages-after-release` to `actions/upload-pages-artifact@v3`
- Pass `artifact_name: github-pages-after-release` to `actions/deploy-pages@v4`

Both actions support custom artifact names via their `name`/`artifact_name` inputs, documented in:
- https://github.com/actions/upload-pages-artifact (supports `name` input)
- https://github.com/actions/deploy-pages (supports `artifact_name` input)

---

## Fix Summary

### PR#80 Fixes
1. **`.github/workflows/release.yml`**:
   - Add `outputs` to `auto-release` job (`version_committed`, `new_version`) so downstream jobs can inspect results
   - Add new `deploy-after-release` job that runs after `auto-release` when a new version was committed, rebuilding the WASM and web app with the correct version
2. **`README.md`**: Add crates.io badge, fix CI/CD badge URL to match actual workflow name
3. **`changelog.d/20260226_090000_fix_web_version_display.md`**: Changelog fragment for version bump (minor — adds new CI job feature)

### PR#81 Fixes
1. **`.github/workflows/release.yml`**: Fix `deploy-after-release` job to use unique Pages artifact name (`github-pages-after-release`) to avoid 409 Conflict with the artifact uploaded by the earlier `deploy-pages` job.
2. **`changelog.d/20260226_120000_fix_deploy_after_release_409_conflict.md`**: Changelog fragment (patch).

---

## How Root Cause 3 Was Introduced

The CI pipeline was designed to minimize build time by running jobs in parallel. Web build and auto-release both run after lint+test, so they start at the same time. The assumption was that `GITHUB_TOKEN` pushes would trigger new CI runs, but GitHub Actions intentionally prevents this to avoid infinite recursion loops. This is documented behavior but easy to miss when designing the pipeline.

The fix makes `deploy-after-release` an explicit sequential step after `auto-release`, trading a small amount of pipeline parallelism for correctness: the deployed web app always matches the latest published version.

## How Root Cause 4 Was Introduced

When adding the `deploy-after-release` job (PR#80), the developer copied the GitHub Pages deployment steps from `deploy-pages` without realizing that `actions/upload-pages-artifact` uses a hardcoded default artifact name `github-pages`. This is a subtle limitation of the GitHub Pages deployment actions: multiple jobs in the same workflow run cannot both upload a `github-pages` artifact.

---

## References

- Issue #78: https://github.com/link-assistant/calculator/issues/78
- Issue #76 (npm 403): https://github.com/link-assistant/calculator/issues/76
- PR #66 (initial version fix attempt): https://github.com/link-assistant/calculator/pull/66
- PR #77 (npm pinning fix): https://github.com/link-assistant/calculator/pull/77
- PR #79 (version-and-commit.mjs fix): https://github.com/link-assistant/calculator/pull/79
- PR #80 (deploy-after-release fix): https://github.com/link-assistant/calculator/pull/80
- PR #81 (409 Conflict fix): https://github.com/link-assistant/calculator/pull/81
- command-stream source: https://www.npmjs.com/package/command-stream
- GitHub Actions: Triggering a workflow — GITHUB_TOKEN restrictions: https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/triggering-a-workflow#triggering-a-workflow-from-a-workflow
- actions/upload-pages-artifact: https://github.com/actions/upload-pages-artifact
- actions/deploy-pages: https://github.com/actions/deploy-pages
- CI run 22414258821: https://github.com/link-assistant/calculator/actions/runs/22414258821
- CI run 22375851874: https://github.com/link-assistant/calculator/actions/runs/22375851874
- CI run 22435338053: https://github.com/link-assistant/calculator/actions/runs/22435338053
- CI run 22439390199 (409 Conflict failure): https://github.com/link-assistant/calculator/actions/runs/22439390199
