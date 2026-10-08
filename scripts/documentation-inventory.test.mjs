import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, copyFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

test('documentation inventory refresh preserves the upstream inventory and surrounding text', () => {
  const root = mkdtempSync(join(tmpdir(), 'documentation-inventory-'));
  try {
    const docs = join(root, 'docs/case-studies/issue-227');
    const experiments = join(root, 'experiments/issue-227');
    mkdirSync(docs, { recursive: true });
    mkdirSync(experiments, { recursive: true });
    const script = join(experiments, 'write-corpus-summary.mjs');
    copyFileSync(new URL('../experiments/issue-227/write-corpus-summary.mjs', import.meta.url), script);
    const original = '# Inventory\n\n<!-- upstream-tests:start -->\nPinned test suites\n<!-- upstream-tests:end -->\n\nMaintainer notes.\n';
    const path = join(docs, 'competitor-inventory.md');
    writeFileSync(path, original);
    writeFileSync(join(docs, 'documentation-sources.json'), JSON.stringify({ sources: [{
      id: 'fixture', product: 'Fixture|Calculator', pages: [{ url: 'https://example.com', fetched: '2026-10-08' }],
      extractedRows: 2, rows: 2, extractor: 'experiments/issue-227/extract-fixture-examples.mjs',
    }] }));
    const run = () => assert.equal(spawnSync(process.execPath, [script], { encoding: 'utf8' }).status, 0);
    run();
    const generated = readFileSync(path, 'utf8');
    assert.ok(generated.startsWith(original.trimEnd()), 'upstream table or maintainer notes were removed');
    assert.match(generated, /<!-- documentation-corpus:start -->/);
    assert.ok(generated.includes('[Fixture\\|Calculator]'), 'product names must escape Markdown table separators');
    run();
    assert.equal(readFileSync(path, 'utf8'), generated, 'repeat refresh must not duplicate sections');
  } finally { rmSync(root, { recursive: true, force: true }); }
});
