import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isCiConfigFile, isCodeChange } from './detect-code-changes.mjs';

test('workflows and local composite actions count as CI config (issue #256)', () => {
  assert.equal(isCiConfigFile('.github/workflows/release.yml'), true);
  assert.equal(isCiConfigFile('.github/actions/install-wasm-pack/action.yml'), true);
});

test('other files are not CI config', () => {
  assert.equal(isCiConfigFile('.github/zizmor.yml'), false);
  assert.equal(isCiConfigFile('docs/.github/workflows/example.yml'), false);
  assert.equal(isCiConfigFile('src/lib.rs'), false);
});

test('scripts and package manifests the pipeline runs count as code (issue #256)', () => {
  for (const file of [
    'scripts/download_historical_rates.py',
    'scripts/simulate-fresh-merge.sh',
    'web/package.json',
    'web/package-lock.json',
    'src/lib.rs',
    'web/src/App.tsx',
    '.github/actions/install-wasm-pack/action.yml',
  ]) {
    assert.equal(isCodeChange(file), true, file);
  }
});

test('docs, fragments, experiments and data are not code', () => {
  for (const file of [
    'README.md',
    'changelog.d/20261011_191500_ci.md',
    'docs/case-studies/issue-227/results/coverage-baseline.json',
    'experiments/issue-256/harden-workflow.py',
    'examples/basic.rs',
    'dev/log/issues/256/pulls/257/recent-runs.json',
    'data/currency/usd-eur.lino',
  ]) {
    assert.equal(isCodeChange(file), false, file);
  }
});
