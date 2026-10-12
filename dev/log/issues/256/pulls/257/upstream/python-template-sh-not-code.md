## Problem

`detect-code-changes` decides `any-code-changed`, which gates these jobs: `changelog`, `lint`, `test`, `build` and `docker-build`. Its code pattern does not include `.sh`, yet the pipeline itself runs 6 shell scripts from `scripts/` (`simulate-fresh-merge.sh`, `run-with-budget-warning.sh`, `check-required-docs.sh`, ...). A pull request that changes **only** a shell script therefore reports `any-code-changed=false`: it needs no changelog fragment, and all of them are skipped, so a broken script reaches `main` with a green PR.

Found while porting the template to link-assistant/calculator (https://github.com/link-assistant/calculator/issues/256, fix in https://github.com/link-assistant/calculator/pull/257, which also adds `.py` and `.json`).

## Reproduction

```bash
git clone https://github.com/link-foundation/python-ai-driven-development-pipeline-template t && cd t
echo '# touch' >> scripts/simulate-fresh-merge.sh
git -c user.name=r -c user.email=r@x commit -qam 'touch a shell script'
GITHUB_EVENT_NAME=push python3 scripts/detect_code_changes.py | grep changed=
```

Observed (at 533e0b7):

```
any-code-changed=false
docs-changed=false
```

## Workaround

Touch a file with a counted extension in the same PR (a changelog fragment alone does not help: `.md` is excluded from code changes).

## Suggested fix

In `scripts/detect_code_changes.py:47` add `.sh`:

```python
CODE_EXTENSIONS = (".py", ".sh", ".toml", ".yml", ".yaml")
```

Add a regression test that `scripts/simulate-fresh-merge.sh` counts as a code change while `docs/**`, `changelog.d/**` and `dev/log/**` do not.
