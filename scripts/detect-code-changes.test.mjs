import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isCiConfigFile } from './detect-code-changes.mjs';

test('workflows and local composite actions count as CI config (issue #256)', () => {
  assert.equal(isCiConfigFile('.github/workflows/release.yml'), true);
  assert.equal(isCiConfigFile('.github/actions/install-wasm-pack/action.yml'), true);
});

test('other files are not CI config', () => {
  assert.equal(isCiConfigFile('.github/zizmor.yml'), false);
  assert.equal(isCiConfigFile('docs/.github/workflows/example.yml'), false);
  assert.equal(isCiConfigFile('src/lib.rs'), false);
});
