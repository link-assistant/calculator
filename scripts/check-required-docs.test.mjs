import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { REQUIREMENTS, findMissingDocs } from './check-required-docs.mjs';

const script = fileURLToPath(new URL('./check-required-docs.mjs', import.meta.url));
const repoRoot = fileURLToPath(new URL('..', import.meta.url));

// Build the fixture from the same table the check reads, so the two cannot drift.
function fixture(t, edit = (docs) => docs) {
  const root = mkdtempSync(join(tmpdir(), 'required-docs-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const docs = Object.fromEntries(
    Object.entries(REQUIREMENTS).map(([path, sections]) => [
      path,
      ['# Title', ...sections.map((section) => `## ${section}\n\ntext`)].join('\n\n'),
    ])
  );
  for (const [path, content] of Object.entries(edit(docs))) {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), content);
  }
  return root;
}

test('the repository satisfies its own requirements', () => {
  assert.deepEqual(findMissingDocs(repoRoot), []);
});

test('a complete fixture passes', (t) => {
  assert.deepEqual(findMissingDocs(fixture(t)), []);
});

test('reports a deleted document', (t) => {
  const root = fixture(t, ({ 'CONTRIBUTING.md': _, ...rest }) => rest);
  assert.deepEqual(findMissingDocs(root), ['CONTRIBUTING.md is missing']);
});

test('reports every missing section, and a mention is not a heading', (t) => {
  const root = fixture(t, (docs) => ({
    ...docs,
    'CONTRIBUTING.md': '# Contributing\n\nSee [Competitor Coverage](#competitor-coverage).\n\n### Release Process\n',
  }));
  const missing = findMissingDocs(root);
  assert.equal(missing.length, REQUIREMENTS['CONTRIBUTING.md'].length);
  assert.ok(missing.includes('CONTRIBUTING.md has no "## Competitor Coverage" section'));
});

test('the CLI annotates each problem and exits 1', { timeout: 15000 }, (t) => {
  const root = fixture(t, (docs) => ({ ...docs, 'README.md': '# Calculator\n' }));
  const result = spawnSync(process.execPath, [script, '--root', root], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.match(result.stdout, /::error title=Required documentation::README\.md has no "## License" section/);
});
