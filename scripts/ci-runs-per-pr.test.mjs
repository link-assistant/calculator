import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { summarize } from './ci-runs-per-pr.mjs';

describe('ci-runs-per-pr', () => {
  it('counts only runs whose Rust stage ran', () => {
    const runs = [
      { jobs: [{ name: 'JavaScript Stage', conclusion: 'failure' }, { name: 'Rust Stage', conclusion: 'skipped' }] },
      { jobs: [{ name: 'JavaScript Stage', conclusion: 'success' }, { name: 'Rust Stage / Test', conclusion: 'success' }] },
      { jobs: [{ name: 'JavaScript Stage', conclusion: 'success' }, { name: 'Rust Stage / Lint', conclusion: 'skipped' }] },
    ];
    assert.deepEqual(summarize(runs), { runs: 3, rust: 1 });
  });

  it('counts every run from before the split', () => {
    assert.deepEqual(summarize([{ jobs: [{ name: 'Lint and Format Check', conclusion: 'failure' }] }]), { runs: 1, rust: 1 });
  });
});
