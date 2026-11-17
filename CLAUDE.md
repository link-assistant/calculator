# Repository Workflow

Follow [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

- Drafting sub-agents never push. Exactly one CI/CD sub-agent owns `git push`:
  it waits for every draft, runs `node scripts/ci-push-gate.mjs` to collect
  all failures of the latest run, fixes them together and pushes once. It
  never pushes while a run on the same commit range is queued or in progress.
- Run the JavaScript checks locally by default. Build Rust only when needed,
  through `node scripts/bounded-cargo.mjs`, sharing one bounded target
  directory instead of starting competing builds.
