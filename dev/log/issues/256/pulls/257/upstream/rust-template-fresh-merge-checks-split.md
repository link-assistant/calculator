## Problem

`scripts/simulate-fresh-merge.sh` documents `FRESH_MERGE_CHECKS` as an "optional space-separated override of the commands to run" and loops over it unquoted:

```bash
  # shellcheck disable=SC2086
  for check in ${FRESH_MERGE_CHECKS}; do
    echo "::group::${check}"
    eval "${check}" || status=1
```

Word splitting makes **every word** a separate command, so a multi-word command such as `cargo test` cannot be expressed: it runs `cargo` and then `test`. The result is a false-positive "semantic merge conflict" for checks that pass on the merged tree. The only current use, `FRESH_MERGE_CHECKS=true` in `tests/unit/ci-cd/issue_157.rs`, is a single word, so the tests do not catch it.

## Reproduction (template at 265219d)

```bash
d=$(mktemp -d); cd "$d"
git -c init.defaultBranch=main init -q --bare o.git
git -c init.defaultBranch=main init -q w; cd w; git remote add origin ../o.git
g(){ git -c user.name=t -c user.email=t@e "$@"; }
echo a>a; g add .; g commit -qm a; g push -q origin main
g checkout -qb f; echo b>b; g add .; g commit -qm b
g checkout -q main; echo c>c; g add .; g commit -qm c; g push -q origin main
g checkout -q f
FRESH_MERGE_CHECKS='test -f c' GITHUB_BASE_REF=main bash /path/to/template/scripts/simulate-fresh-merge.sh
```

```console
::group::test
::group::-f
::group::c
::error::Checks failed on the simulated merge with origin/main even though they pass on the pull request head. This is a semantic merge conflict.
```

`c` exists only on the merged tree, so `test -f c` passes there. Even so, the script exits 1.

## Fix

Use a newline-separated list, read line by line, with the defaults in the same form:

```bash
DEFAULT_CHECKS='cargo fmt --all -- --check
bash scripts/check-cargo-warnings.sh
cargo clippy --all-targets --all-features
cargo test --all-features'

status=0
while IFS= read -r check; do
  [ -n "${check}" ] || continue
  echo "::group::${check}"
  bash -c "${check}" || status=1
  echo "::endgroup::"
done <<< "${FRESH_MERGE_CHECKS:-$DEFAULT_CHECKS}"
```

A test with `FRESH_MERGE_CHECKS=$'test -f <file only on base>\ntest -f <file only on head>'` would prove both that the checks run on the merged tree and that multi-word commands work.

Downstream implementation with tests: link-assistant/calculator#257 (`scripts/simulate-fresh-merge.sh`, `scripts/simulate-fresh-merge.test.mjs`).

Two smaller points from the same port:
- `git rev-parse "origin/${BASE_REF}"` after `git fetch origin "${BASE_REF}"` relies on the remote's fetch refspec mapping that branch. `FETCH_HEAD` does not.
- The default bot email lacks the `41898282+` prefix that links commits to `github-actions[bot]`. The js template's script has it.

Found while working on link-assistant/calculator#256.
