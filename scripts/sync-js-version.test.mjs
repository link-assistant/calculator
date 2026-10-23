import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';

import { JS_VERSION_FILES, cargoVersion, syncJsVersion, withVersion } from './sync-js-version.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));

describe('sync-js-version', () => {
  it('finds the committed versions in sync', () => {
    assert.deepEqual(syncJsVersion(root, { check: true }), []);
  });

  it('changes only the package entries of the lock file', () => {
    const lock = readFileSync(join(root, 'js/package-lock.json'), 'utf8');
    const bumped = JSON.parse(withVersion('js/package-lock.json', lock, '9.9.9'));
    assert.equal(bumped.version, '9.9.9');
    assert.equal(bumped.packages[''].version, '9.9.9');
    const before = JSON.parse(lock);
    for (const [name, entry] of Object.entries(bumped.packages)) {
      if (name !== '') {
        assert.equal(entry.version, before.packages[name].version, name);
      }
    }
  });

  it('bumps every file of a copy of the repository', () => {
    const copy = mkdtempSync(join(tmpdir(), 'sync-js-version-'));
    mkdirSync(join(copy, 'js/src'), { recursive: true });
    for (const path of ['Cargo.toml', ...JS_VERSION_FILES]) {
      writeFileSync(join(copy, path), readFileSync(join(root, path), 'utf8'));
    }
    const cargo = readFileSync(join(copy, 'Cargo.toml'), 'utf8').replace(/^version = "[^"]+"/m, 'version = "9.9.9"');
    writeFileSync(join(copy, 'Cargo.toml'), cargo);

    assert.deepEqual(syncJsVersion(copy, { check: true }), JS_VERSION_FILES);
    assert.deepEqual(syncJsVersion(copy), JS_VERSION_FILES);
    assert.deepEqual(syncJsVersion(copy, { check: true }), []);
    assert.equal(cargoVersion(cargo), '9.9.9');
    assert.equal(JSON.parse(readFileSync(join(copy, 'js/package.json'), 'utf8')).version, '9.9.9');
    assert.match(readFileSync(join(copy, 'js/src/index.js'), 'utf8'), /^export const VERSION = '9\.9\.9';$/m);
  });
});
