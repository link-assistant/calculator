## Problem

`detect-code-changes` decides `any-code-changed`, which gates these jobs: `changelog`, `fresh-merge`, `lint`, `script-tests`, `test`, `cargo-lock` and `docker-build`. Its code pattern does not include `.sh`, yet the pipeline itself runs 13 shell scripts from `scripts/` (`check-pipeline-status.sh`, `simulate-fresh-merge.sh`, `check-required-docs.sh`, `preflight-credentials.sh`, `read-job-cancel-in-progress.sh`, ...). A pull request that changes **only** a shell script therefore reports `any-code-changed=false`: it needs no changelog fragment, and all of them are skipped, so a broken script reaches `main` with a green PR.

Found while porting the template to link-assistant/calculator (https://github.com/link-assistant/calculator/issues/256, fix in https://github.com/link-assistant/calculator/pull/257, which also adds `.py` and `.json`).

## Reproduction

```bash
git clone https://github.com/link-foundation/rust-ai-driven-development-pipeline-template t && cd t
echo '# touch' >> scripts/simulate-fresh-merge.sh
git -c user.name=r -c user.email=r@x commit -qam 'touch a shell script'
# the pattern from scripts/detect-code-changes.rs:258
echo scripts/simulate-fresh-merge.sh | grep -cE '(\.(rs|toml|mjs|js|yml|yaml)$|(^|/)Cargo\.lock$|^\.github/workflows/)'
```

Observed (at 265219d):

```
0
```

## Workaround

Touch a file with a counted extension in the same PR (a changelog fragment alone does not help: `.md` is excluded from code changes).

## Suggested fix

In `code_change_pattern()` (`scripts/detect-code-changes.rs:258`) add `sh`:

```rust
Regex::new(r"(\.(rs|toml|mjs|js|sh|yml|yaml)$|(^|/)Cargo\.lock$|^\.github/(workflows|actions)/)").unwrap()
```

(`^\.github/actions/` also makes a composite action change count even if a future action ships a non-YAML file.)

Add a regression test that `scripts/simulate-fresh-merge.sh` counts as a code change while `docs/**`, `changelog.d/**` and `dev/log/**` do not.
