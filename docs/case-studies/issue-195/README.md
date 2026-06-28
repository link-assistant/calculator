# Case Study: Issue #195 - CI/CD False Positives, Warnings, and Push Races

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/195
**Pull request:** https://github.com/link-assistant/calculator/pull/196
**Status:** Root causes found and fixed in PR #196
**Severity:** Medium - the release pipeline succeeded, but the standalone screenshot workflow failed after a release because two CI jobs wrote to `main` concurrently.

## Requirements From The Issue

- Download logs and data for the cited CI runs into `docs/case-studies/issue-195/`.
- Reconstruct the timeline and identify root causes for each CI/CD failure, warning, and false positive.
- Compare repository CI/CD workflows and scripts against the referenced JS, Rust, Python, and C# pipeline templates.
- Apply best practices from the templates and official docs where relevant.
- Report the same issue upstream in a template repository if it is present there.
- Add debug/verbose behavior or regression coverage where data is insufficient.

## Raw Data Collected

- `ci-logs/run-28194099705.log` - failed standalone `Update Screenshots` run.
- `ci-logs/run-28194099832.log` - successful `CI/CD Pipeline` run from the same merge.
- `run-data/run-28194099705.json` - metadata and job state for the failed run.
- `run-data/run-28194099832.json` - metadata and job state for the successful run.
- `run-data/recent-branch-runs.json` - latest runs for `issue-195-f6006bdc4a9c` at investigation time; there were no runs yet.
- `issue.json`, `issue-comments.json`, `pr-196.json`, `pr-review-comments.json`, `pr-conversation-comments.json`, and `pr-reviews.json` - GitHub issue/PR evidence.
- `calculator-file-tree.txt` and `calculator-automation-files.txt` - local file tree and automation file manifest.
- `template-file-trees/*.txt` and `template-ci-files/*` - file-tree manifests plus copied workflow/action files from the referenced templates.
- `template-ci-files/rust-release-vs-calculator-release.diff` - focused workflow diff against the Rust template.
- `run-data/wasm-pack-v0.15.0-release.json` - upstream wasm-pack release metadata used to pin CI installation.
- `template-js-issue-98.json` - upstream template issue filed for the matching JS template push-race pattern.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-06-25T19:08:36 | Merge commit `b3d3eb82409fadbf3c11ee6dd0c4aa4b997c0ee5` triggers both `Update Screenshots` and `CI/CD Pipeline` on `main`. |
| 2026-06-25T19:10:45 | `CI/CD Pipeline / Auto Release` commits `5d21753c` (`chore: release v0.20.3`). |
| 2026-06-25T19:10:46 | `Auto Release` pushes `main` from `b3d3eb82` to `5d21753c`, then pushes tag `v0.20.3`. |
| 2026-06-25T19:10:59 | Standalone `Update Screenshots` commits `e641d19` from the older `b3d3eb82` base. |
| 2026-06-25T19:11:00 | Standalone `Update Screenshots` fails: `! [rejected] main -> main (fetch first)`. |
| 2026-06-25T19:14:08 | Release workflow invokes `Update Screenshots After Release`, after the release and deploy jobs complete. |
| 2026-06-25T19:16:15 | Reusable screenshot job pushes successfully because it checked out latest `main` after the release commit. |

## Root Causes

### Root Cause 1: Generated Screenshot Commit Raced The Release Commit

The failed workflow checked out `main`, generated screenshot files, committed them, and ran plain `git push`. During the same window, the release workflow committed and pushed a version bump to `main`. Git correctly rejected the screenshot push because it was no longer a fast-forward.

Evidence:

- `ci-logs/run-28194099705.log:796-804` contains the rejected push and `Process completed with exit code 1`.
- `ci-logs/run-28194099832.log:4770-4772` shows the release push to `main` completing first.

Fix:

- Added `scripts/git-push-with-rebase.mjs`.
- Replaced direct generated pushes in `update-screenshots.yml` and `update-currency-rates.yml` with `node scripts/git-push-with-rebase.mjs --branch main`.
- Added shared job-level `concurrency: main-writer-main` for jobs that write generated commits to `main`.
- Added `scripts/git-push-with-rebase.test.mjs`, which reproduces a non-fast-forward rejection with a bare remote and two clones.

### Root Cause 2: Release Commit Script Lacked The Template's Rebase/Retry Hardening

The Rust template's `version-and-commit.rs` fetches/rebases before changing files and retries a failed push after `git pull --rebase`. The calculator's `scripts/version-and-commit.mjs` did neither. The cited run happened to succeed because release pushed first, but the same race could have failed the release job if screenshot generation pushed first.

Fix:

- `version-and-commit.mjs` now syncs with the remote branch before computing a release version.
- It pushes the release commit through `pushCurrentBranchWithRebase`.
- It recreates the release tag after the branch push/rebase path, so the tag points at the final commit that landed on `main`.

### Root Cause 3: Cargo Publish Used Deprecated `--token`

The successful CI run emitted:

```text
warning: `cargo publish --token` is deprecated in favor of using `cargo login` and environment variables
```

Fix:

- `scripts/publish-crate.mjs` now sets `CARGO_REGISTRY_TOKEN` in the process environment and runs `cargo publish --allow-dirty`.
- `release.yml` maps both `CARGO_REGISTRY_TOKEN` and the legacy `CARGO_TOKEN` secret for compatibility.

### Root Cause 4: CI Installed An Outdated wasm-pack

The logs show the RustWasm installer placed `wasm-pack 0.13.1` on the runner while wasm-pack reported `0.15.0` was available.

Fix:

- Added `.github/actions/install-wasm-pack/action.yml`.
- Replaced the moving installer script with a pinned `wasm-pack v0.15.0` release binary in all CI jobs that build WASM.

### Root Cause 5: Vite Chunk Warning Was Expected For This App

The web build warned because `index-*.js` was 517.62 kB, only slightly above Vite's default 500 kB warning threshold. The app already code-splits MathRenderer and FunctionPlot, and the warning did not indicate a failed optimization.

Fix:

- Set `build.chunkSizeWarningLimit` to `700` in `web/vite.config.ts`, making the expected threshold explicit while preserving future regression signal.

### Root Cause 6: Git Checkout Default-Branch Hint Was Repeated CI Noise

`actions/checkout@v6` invoked `git init`, which printed Git's `Using 'master' as the name for the initial branch` hint. This warning is harmless but noisy.

Fix:

- Added `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_0=init.defaultBranch`, and `GIT_CONFIG_VALUE_0=main` to workflow environments.

### Not Fixed Here: Third-Party GitHub Pages Action Deprecation Warnings

The successful release run logged Node deprecation warnings from `actions/download-artifact` / GitHub Pages deployment action internals. Those warnings are not emitted by calculator code or scripts. The local workflow already uses current action majors (`actions/download-artifact@v7`, `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`, `actions/configure-pages@v6`), so there is no local code change available beyond keeping action versions current.

## Template Comparison

### Rust Template

The Rust template contains several CI/CD practices now applied here:

- `CARGO_NET_RETRY: '10'` and `CARGO_HTTP_MULTIPLEXING: 'false'` to reduce transient Cargo network failures.
- Rebase-before-modify and retry-after-rebase in the release commit script.
- Script-level regression coverage for release workflow policy and push behavior.

Calculator-specific differences remain intentional:

- Calculator has a WASM/web build and GitHub Pages deploy path.
- Calculator has a standalone screenshot workflow and currency-rate workflow, both of which generate commits.
- Calculator uses Node `.mjs` scripts rather than `rust-script` for release automation.

### JS Template

The JS template's `example-app.yml` has a matching generated-artifact push pattern:

```text
git commit -m "chore(preview): regenerate example-app preview images [skip ci]"
git push origin HEAD:main
```

That pattern can fail the same way if another workflow advances `main` first. Upstream issue filed:

- https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/98

### Python and C# Templates

The Python and C# templates reinforce these practices:

- Use workflow-level concurrency groups that do not cancel `main` runs.
- Use pull-request creation actions for manual release requests instead of direct ad hoc branch mutation.
- Include script-level tests for release-policy behavior in the C# template.

No identical standalone screenshot/update workflow was found in those two templates during this pass.

## Implemented Files

- `.github/actions/install-wasm-pack/action.yml`
- `.github/workflows/release.yml`
- `.github/workflows/update-currency-rates.yml`
- `.github/workflows/update-screenshots.yml`
- `scripts/git-push-with-rebase.mjs`
- `scripts/git-push-with-rebase.test.mjs`
- `scripts/publish-crate.mjs`
- `scripts/version-and-commit.mjs`
- `web/vite.config.ts`
- `changelog.d/20260628_220000_issue_195_ci_cd_hardening.md`

## Verification Completed

The following local checks passed, with logs saved under `verification/`:

- `node --test scripts/git-push-with-rebase.test.mjs`
- `node --check scripts/git-push-with-rebase.mjs`
- `node --check scripts/git-push-with-rebase.test.mjs`
- `node --check scripts/version-and-commit.mjs`
- `node --check scripts/publish-crate.mjs`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features`
- `cargo test --doc`
- `node scripts/check-file-size.mjs`
- `npm ci` in `web/`
- `npm test` in `web/`
- `npm run build` in `web/`

Local `npm run build` required generating the WASM web package first with
`wasm-pack build --target web --out-dir web/public/pkg`, matching the workflow
order. After that, the build completed without the previous Vite chunk warning.

## References

- GitHub Actions concurrency docs: https://docs.github.com/en/actions/using-jobs/using-concurrency
- Cargo publishing docs: https://doc.rust-lang.org/cargo/reference/publishing.html
- Vite build options: https://vite.dev/config/build-options.html#build-chunksizewarninglimit
- wasm-pack v0.15.0 release: https://github.com/wasm-bindgen/wasm-pack/releases/tag/v0.15.0
- Failed screenshot run: https://github.com/link-assistant/calculator/actions/runs/28194099705
- Successful release run: https://github.com/link-assistant/calculator/actions/runs/28194099832
- Upstream JS template issue: https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/98
