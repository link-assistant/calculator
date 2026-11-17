# Agent Instructions

These rules apply to every automated agent and sub-agent working in this
repository. [CONTRIBUTING.md](CONTRIBUTING.md) has the full development
workflow; [CLAUDE.md](CLAUDE.md) points here.

## Push Gate

Every push starts a full CI run, and every red run on a pull request branch
costs a fix, a push and another run (#254: on one branch, 6 of 8 pipeline runs
failed or were cancelled). Pushes are therefore batched:

- **Drafting sub-agents never push.** They investigate, edit, run local checks
  and commit coherent steps, then hand their result to the CI/CD agent.
- **Exactly one CI/CD sub-agent owns `git push`.** It waits until every draft
  is complete, then runs `node scripts/ci-push-gate.mjs`, which:
  - exits 1 while any run on the branch's commit range is queued or in
    progress. **Never push while a run is in progress on the same commit
    range**: wait for it to finish;
  - downloads the failed-job logs of **all** failing runs of the latest
    pushed commit to `ci-logs/`.
- The CI/CD agent assigns one bulk correction covering every collected
  failure, verifies it locally and **pushes once**. One push per failure is
  the waste this rule exists to stop.
- Never cancel a run on `main` or a started Rust stage. Do not replace a
  repeated timeout with a longer timeout without tracing its cause.
- Preserve commit history and raw CI logs (`ci-logs/` is ignored by git; quote
  the relevant lines in the pull request instead).

## Local Checks

Run the JavaScript checks locally by default; they are fast and need no
Rust build:

```bash
node --test scripts/*.test.mjs
npm test --prefix web
```

Build Rust only when a change touches Rust, the WASM bindings or a check that
needs the binary, and then only through the bounded wrapper (see
CONTRIBUTING.md, "Rust Build Load"). Drafting agents share one bounded target
directory instead of starting their own compile trees:

```bash
CALCULATOR_CARGO_TARGET_DIR=$HOME/.cache/calculator-target \
  node scripts/bounded-cargo.mjs test
```

## Pull Requests

- Work on the prepared branch and preserve unrelated changes. Default-branch
  changes go through pull requests.
- Write one closing keyword per issue the pull request implements, each on its
  own line (CONTRIBUTING.md, "Closing Issues"). Mention planned or follow-up
  issues without a keyword.
- Add a changelog fragment to `changelog.d/` for user-facing changes.
- Keep finite reproduction scripts in `experiments/` and real use cases in
  `examples/`.
- Before marking a pull request ready, the CI/CD agent checks the final diff, a
  clean worktree, that `main` is merged, every required check and every issue
  requirement.
