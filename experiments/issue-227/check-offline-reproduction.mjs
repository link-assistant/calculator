#!/usr/bin/env node
// Verify that the cached snapshot reproduces both generated artifacts exactly.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { sha256 } from './documentation.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const paths = ['corpus/closed-source-docs.tsv', 'documentation-sources.json']
  .map((path) => new URL(`../../docs/case-studies/issue-227/${path}`, import.meta.url));
const before = paths.map((path) => sha256(readFileSync(path)));
const result = spawnSync(process.execPath, ['experiments/issue-227/refresh-documentation-corpus.mjs',
  '--offline', '--cache', process.argv[2] ?? 'ci-logs/documentation-cache'], { cwd: root, stdio: 'inherit' });
assert.equal(result.status, 0, 'offline extraction failed');
paths.forEach((path, index) => assert.equal(sha256(readFileSync(path)), before[index], path.pathname));
console.log('Offline refresh exactly reproduces the corpus and source manifest.');
