# Issue #256 / PR #257: false positives, false negatives, warnings and errors in CI/CD

Issue: https://github.com/link-assistant/calculator/issues/256
Pull request: https://github.com/link-assistant/calculator/pull/257

This folder holds the evidence collected for the issue and the analysis behind
every change in PR #257.

## Evidence index

| Path | What it is |
| --- | --- |
| `ci-logs/run-38138958055.{log,json}` | The failing `main` run named in the issue (commit `c2d2e61`, merge of PR #255) |
| `ci-logs/run-38136842934.{log,json}` | Corpus refresh run that opened PR #255 with `GITHUB_TOKEN` |
| `ci-logs/run-38136861895.{log,json}` | PR #255's `pull_request` run: approval-pending, zero jobs, concluded `failure` |
| `ci-logs/run-37762910436.*`, `run-37762910912.*`, `run-37993754202.*` | Earlier green `main` and scheduled runs, used for the warning inventory |
| `ci-logs/pr257-run-38166546592-failed.log` | Dependency review failure on this PR ("not supported on this repository") |
| `ci-logs/pr257-run-38166546758-failed.log` | Changelog fragment check failure on this PR (fragment not yet added) |
| `ci-logs/pr257-run-38167541*.log` | The first all-green runs of this PR (Workflows, Security, Links, CI/CD) |
| `annotations.txt` | Every check-run annotation of the failing `main` run and its siblings |
| `annotations-after.txt` | The annotations left on this PR after the fixes |
| `jobs.tsv` | Job IDs of the runs above |
| `recent-runs.json` | The recent runs on `main` at the start of the work |
| `pr-255/` | PR #255 metadata, its zero-job run, and the GitHub docs passage on `GITHUB_TOKEN` events |
| `e2e-main-deps.log`, `e2e-new-deps.log` | Playwright suite on `main` dependencies (8 failures) and after the update |
| `upstream/*.md`, `upstream/filed.txt` | Bodies of the issues filed in other repositories, and their URLs |

GitHub run logs expire. These copies keep the analysis reproducible.

## Timeline (2026-10-11, UTC)

| Time | Event |
| --- | --- |
| 11:47:36 | The corpus refresh run [38136842934](https://github.com/link-assistant/calculator/actions/runs/38136842934) opens PR #255 with `GITHUB_TOKEN`, because `CORPUS_REFRESH_TOKEN` is not set |
| 11:47 | PR #255's `pull_request` run [38136861895](https://github.com/link-assistant/calculator/actions/runs/38136861895) waits for maintainer approval and never starts a job |
| 12:21:33 | PR #255 is merged without any CI having run |
| 12:21:35 | Run 38136861895 is concluded `failure` with zero jobs |
| 12:21:36 | `main` run [38138958055](https://github.com/link-assistant/calculator/actions/runs/38138958055) starts on `c2d2e61` |
| 12:21:57 | Its **Dependency Freshness** job fails: `serde_json 1.0.151 is behind 1.0.152`, plus five `web/package.json` packages. Every other job succeeds, including the release and the Pages deploy |
| 18:20:07 | Issue #256 is opened |
| 18:20:49 | PR #257 is opened |
| PR head `e81aad75` | The new Security run 38166546592 fails in Dependency Review ("not supported on this repository"). CI/CD run 38166546758 fails the changelog fragment check |
| PR head `65cb28df` | Dependency review now probes the dependency graph first, and the fragment is added |
| PR head `2e65a7be` | First fully green set of runs (`ci-logs/pr257-run-38167541*.log`) |
| later | Links, secrets scan, fresh merge, publish preflight, required docs, ruff and the change-detection fixes follow (see the commits table) |

## Requirements from the issue

1. Fix the failing `main` run 38138958055.
2. Find and fix all false positives, false negatives, warnings and errors in CI/CD.
3. Compare every workflow and CI/CD script with the link-foundation
   [rust](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template),
   [js](https://github.com/link-foundation/js-ai-driven-development-pipeline-template) and
   [python](https://github.com/link-foundation/python-ai-driven-development-pipeline-template)
   templates and reuse their best practices.
4. Report the problems that also apply to the templates.
5. Follow [hive-mind docs/CI-CD-BEST-PRACTICES.md](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md).
6. Do it all in PR #257.
7. From the session: collect the evidence here, find the root causes, add verbose
   modes (off by default), file upstream issues with a reproduction, a
   workaround and a fix, and apply each fix everywhere it applies.

## Problems, root causes and fixes

### False positives (red without a real defect)

| # | Symptom | Root cause | Fix | Commit |
| --- | --- | --- | --- | --- |
| P1 | `main` run 38138958055 failed in Dependency Freshness | Freshness depends on external state: upstream released `serde_json 1.0.152` and others after PR #255 was checked. A push can only report that, not prevent it (principle 17) | Freshness fails pull requests and only emits `::warning::` on push, schedule and dispatch. The stale dependencies are updated | `1fab5d91`, `586f3f87` |
| P2 | PR #255's run 38136861895 was "failed" with zero jobs | A PR opened with `GITHUB_TOKEN` gets an approval-required `pull_request` run ([GitHub docs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#triggering-a-workflow-from-a-workflow), copy in `pr-255/`). Merging it unapproved concludes the run as failed | New `checks` release mode. The corpus refresh and the manual release PR dispatch it on the PR branch, because `workflow_dispatch` runs start without approval | `880d8ed4`, `896e93a4` |
| P3 | The pipeline-status gate failed on a superseded PR run | On `pull_request`, `github.sha` is the synthetic merge commit and `github.ref_name` is `<n>/merge`, which `git ls-remote origin refs/heads/<n>/merge` cannot resolve | The gate receives the PR head SHA and head branch. `experiments/` holds a reproduction | `a6c7a4a2`, `5098a56f` |
| P4 | Security run 38166546592 failed: "Dependency review is not supported on this repository" | The repository's Dependency graph setting is off | Probe the SBOM endpoint first. A 404 warns with the settings link; any other unexpected answer fails | `65cb28df` |

### False negatives (green while something was broken)

| # | Symptom | Root cause | Fix | Commit |
| --- | --- | --- | --- | --- |
| N1 | 8 of 50 Playwright tests failed on `main` unnoticed | No job ran `web/e2e`. Tests had rotted: browser-commander `fillTextArea` appends and presses Enter, `:has-text` gets rewritten unscoped, and `textContent()` reads raced | Fix the tests (`setInput` helper, `data-testid`, retrying assertions) and run Playwright in Web Build | `2c7978f7` |
| N2 | `npm run lint` called ESLint, which was never installed | Dead script that no job ran. `typescript-eslint` caps TypeScript at `<6.1`, but the project uses TypeScript 7 | Replace it with oxlint, fix its one finding, and run it in CI | `197fd228` |
| N3 | A failed job could hide behind skipped dependants. A job killed by `timeout-minutes` shows as "cancelled", not failed | There was no terminal gate | A `pipeline-status` job in every workflow, from the rust template, with `PIPELINE_STATUS_VERBOSE=1` tracing | `e2dfb108`, `e81aad75`, `d837644a` |
| N4 | A PR touching only `.github/actions/install-wasm-pack` skipped lint and wasm-build, the one job that runs the action | Missing from the trigger paths and from the workflow-change detection | Count `.github/actions/**` as a workflow change and a trigger | `880d8ed4` |
| N5 | Changes to only `.py`, `.sh` or `.json` files counted as "no code change", which skipped lint, build and web build and needed no changelog fragment | `CODE_PATTERN` lacked these extensions. Lint was not gated on `any-code-changed` | Extend the pattern, gate lint on any code change, and exclude `dev/log/` evidence | `aca4f22a`, `9f4b6908` |
| N6 | CI Python scripts were never linted (unused imports, a placeholder-less f-string) | No Python linter | `ruff.toml` with a pinned rule set, and a `ruff check` plus `ruff format --check` step | `98f5d6fd`, `aca4f22a` |
| N7 | Deleting `CONTRIBUTING.md#competitor-coverage` (linked from docs and the coverage-gap issue) passed every check | lychee does not check `#fragments` | `scripts/check-required-docs.mjs` (principle 12) | `671e2b84` |
| N8 | A revoked crates.io token left `main` bumped on an unpublished version (issue #223) | The bump is pushed before `cargo publish`, and a non-empty secret proves nothing | `scripts/preflight-crates-token.mjs` probes `PUT /api/v1/crates/new` without an archive (principle 16). `PREFLIGHT_VERBOSE=1` logs the probe status | `348c53c0` |
| N9 | No check for committed credentials, and no re-test against the current base | Missing jobs (principles 7 and 11) | Secrets Scan (secretlint) and Fresh Merge Simulation (`FRESH_MERGE_VERBOSE=1`) | `5bf6106d` |
| N10 | Workflows had template injection, persisted credentials, `ubuntu-latest` and unpinned third-party actions, with nothing to stop regressions | No workflow linting (principle 14) | Harden every workflow and add `workflows.yml` (actionlint with shellcheck, zizmor) | `822df334`, `e2dfb108`, `e81aad75` |
| N11 | No dependency audit (principle 15) | Missing workflow | `security.yml`: cargo audit, npm audit, CodeQL and dependency review | `e81aad75` |
| N12 | No link checking | Missing workflow | `links.yml` (lychee, re-check, Web Archive suggestions) from the rust template | `d837644a`, `9a1898c7` |

### Warnings

Counts come from `ci-logs/run-38138958055.log` and the earlier runs. The
inventory command is in the session log; `N` replaces version numbers.

| Warning | Where | Root cause | Status |
| --- | --- | --- | --- |
| `The ubuntu-latest label will migrate to Ubuntu 26 beginning October 19, 2026` (every job, `annotations.txt`) | all workflows | Moving runner label | Fixed: pinned `ubuntu-24.04` (`822df334`, `e2dfb108`) |
| `npm warn install-scripts 2 packages have install scripts not yet covered by allowScripts` (better-sqlite3, node-pty) | `npm ci` in web/ | Transitive dependencies of browser-commander that the project does not need | Fixed: denied in `allowScripts` (`586f3f87`) |
| `npm warn deprecated glob@11.1.0` | `npm ci` in web/ | Old transitive glob | Fixed by the dependency update (`586f3f87`). Not present in PR runs |
| `5 high severity vulnerabilities` | `npm ci` in web/ | Outdated transitive dependencies | Fixed: `npm audit` reports 0 (`586f3f87`). `security.yml` now gates it |
| `warning: --depth is ignored in local clones; use file:// instead.` | competitor checkout test | Local path as the clone URL | Fixed: `file://` URL (`52eed728`) |
| `hint: Using 'master' as the name for the initial branch` | checkouts in the corpus, coverage, security and workflows workflows | `actions/checkout` runs `git init` before any step | Fixed: `GIT_CONFIG_*` env in every workflow (`2e65a7be`) |
| `warning: ignoring example ... as experiments/....rs is not included in the published package` (×6) | `cargo package` | Experiments registered as root `[[example]]` targets | Fixed: unpublished `link-calculator-experiments` member (`7c44ec38`) |
| lychee `No files found for this input source` | Broken Link Checker | `./**/*.html` matched nothing after the Vite HTML exclusion | Fixed: directory input (`9a1898c7`) |
| `[DEP0005] Buffer() is deprecated` | Deploy to GitHub Pages | `actions/download-artifact` itself; v8 still emits it | Upstream: [actions/download-artifact#484](https://github.com/actions/download-artifact/issues/484) |
| `[DEP0040] The punycode module is deprecated` (×2) | Deploy to GitHub Pages, Deploy Web After Release | `actions/deploy-pages` dependency | Upstream: [actions/deploy-pages#434](https://github.com/actions/deploy-pages/issues/434), [#413](https://github.com/actions/deploy-pages/issues/413) |
| `Enable the Dependency graph ...` | Dependency Review | Repository setting is off | Intentional warning until a maintainer enables it (see Remaining limits) |

On the PR head, the CI/CD, Workflows and Links runs log no warnings.
Security logs only the Dependency-graph warning (`annotations-after.txt`).

## Template comparison

The templates were compared at rust `265219d`, js `0a62b80` and python
`533e0b7`.

| Template practice | Status here |
| --- | --- |
| `detect-changes` with per-type outputs | Already present. Extended to cover `.github/actions`, `.py`, `.sh` and `.json` |
| `pipeline-status` terminal gate | Ported to release, security, workflows and links |
| `workflows.yml` (actionlint, zizmor) | Ported. The actionlint image is pinned by digest so shellcheck is included |
| `security.yml` (audit, CodeQL, dependency review) | Ported for Rust plus the web/ npm tree |
| `links.yml` (lychee) | Ported with the re-check and Web Archive scripts |
| `secrets-scan`, `fresh-merge` | Ported. Checks are newline-separated (see rust #203) |
| `release-preflight` (publish token probe) | Ported as a step before the version bump in both release jobs |
| `validate-docs` (required docs) | Ported as a lint step and a fresh-merge default check |
| Script tests (`node --test scripts/*.test.mjs`) | Already present |
| `cargo-lock` guard | Not ported. `Cargo.lock` is committed, `--locked` builds and `cargo audit --file Cargo.lock` already fail without it |
| `docker-*` jobs, `desktop-release.yml`, `example-app.yml`, `docs.yml` | Not applicable: no container image, desktop app or docs site here. Pages deploy is project-specific |
| `coverage` job | Already present |

These are this repository's own workflows, not in the templates:
`update-currency-rates.yml` and `update-screenshots.yml` push data commits to
`main` with `[skip ci]`, and `coverage-gap-report.yml` and
`refresh-competitor-corpus.yml` maintain the competitor corpus. All of them got
the same hardening.

## Upstream reports

Each report has a reproduction, a workaround and a suggested fix.

| Report | Problem |
| --- | --- |
| [rust template #200](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/200) | pipeline-status gate cannot prove a PR run superseded (merge ref) |
| [rust template #201](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/201) | `changelog-pr` PR opened with `GITHUB_TOKEN` gets an approval-pending run |
| [rust template #202](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/202) | stale action pins (`dtolnay/rust-toolchain` before its input fix, first-party majors) |
| [rust template #203](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/203) | fresh-merge splits `cargo test` into two commands |
| [rust template #204](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/204) | `.sh` changes are not code changes |
| [js template #234](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/234) | stale action pins |
| [js template #235](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/235) | `.sh` changes are not code changes |
| [python template #106](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/106) | lychee `*.html` glob warning, stale pins |
| [python template #107](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/107) | `.sh` changes are not code changes |
| [browser-commander #170](https://github.com/link-foundation/browser-commander/issues/170) | `:has-text` selector loses its scope |

## Existing tools used or evaluated

- [actionlint](https://github.com/rhysd/actionlint) with shellcheck: workflow syntax, expressions and shell. Used.
- [zizmor](https://github.com/zizmorcore/zizmor): GitHub Actions security audits (template injection, credentials, pins). Used.
- [lychee](https://github.com/lycheeverse/lychee): links. Used. It does not check fragments, hence `check-required-docs.mjs`.
- [secretlint](https://github.com/secretlint/secretlint): committed credentials. Used, as in the templates.
- [ruff](https://github.com/astral-sh/ruff): Python lint and format, pinned to `0.17.0`. Version 0.17 widened its default rules, so the rule set is pinned in `ruff.toml`.
- [cargo-audit](https://github.com/rustsec/rustsec), `npm audit`, [CodeQL](https://codeql.github.com/), [dependency-review-action](https://github.com/actions/dependency-review-action). Used.
- [oxlint](https://oxc.rs/docs/guide/usage/linter): replaces ESLint, which cannot lint TypeScript 7.
- crates.io publish probe: a metadata-only `PUT /api/v1/crates/new`. Tested live: no token gives 403, a bogus token gives 401, and a valid token gives 400 `invalid tarball length` without publishing.

## Verbose modes (off by default)

| Variable | Script |
| --- | --- |
| `PIPELINE_STATUS_VERBOSE=1` | `scripts/check-pipeline-status.sh` |
| `FRESH_MERGE_VERBOSE=1` | `scripts/simulate-fresh-merge.sh` |
| `PREFLIGHT_VERBOSE=1` | `scripts/preflight-crates-token.mjs` |

## Remaining limits

- **Dependency graph is off.** Dependency review warns and skips until a
  maintainer enables it at
  `https://github.com/link-assistant/calculator/settings/security_analysis` (maintainers only).
- **`CORPUS_REFRESH_TOKEN` is not set.** Refresh PRs fall back to the
  dispatched `checks` run. Its check runs attach to the PR head, but the PR's
  own `pull_request` run still waits for approval.
- **`DEP0005` and `DEP0040`** come from GitHub's own actions and are reported
  upstream.
- **Unknown preflight answers only warn.** A timeout or 429 from crates.io only
  warns, unlike the rust template, which fails. A crates.io outage should not
  block a release, and `cargo publish` still fails loudly on a real rejection.
