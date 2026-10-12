import test from 'node:test';
import assert from 'node:assert/strict';

import {
  classify,
  findCargoBlocker,
  findNpmBlocker,
  formatFreshnessReport,
  freshnessEnforcement,
  parseCargoOutdated,
  parseNpmOutdated,
  pendingLockfileUpdates,
} from './check-dependency-freshness.mjs';

const CARGO_OUTDATED = JSON.stringify({
  crate_name: 'probe',
  dependencies: [
    {
      name: 'num-bigint',
      project: '0.4.8',
      compat: '---',
      latest: '0.5.1',
      kind: 'Normal',
      platform: null,
    },
    {
      name: 'thiserror',
      project: '1.0.69',
      compat: '---',
      latest: '2.0.21',
      kind: 'Normal',
      platform: null,
    },
  ],
});

const MANIFEST = `[package]
name = "probe"

[dependencies]
# Blocked until https://github.com/owner/repo/issues/12 is fixed.
num-bigint = "0.4"
thiserror = "1.0"
serde = { version = "1.0" } # see https://github.com/owner/repo/issues/34

[dependencies.regex]
version = "1" # https://github.com/owner/repo/issues/56
`;

test('parses cargo outdated JSON', () => {
  assert.deepEqual(parseCargoOutdated(CARGO_OUTDATED), [
    { name: 'num-bigint', current: '0.4.8', latest: '0.5.1' },
    { name: 'thiserror', current: '1.0.69', latest: '2.0.21' },
  ]);
  assert.deepEqual(parseCargoOutdated('{"crate_name":"x","dependencies":[]}'), []);
});

test('finds blocker comments on or above the dependency line', () => {
  assert.equal(findCargoBlocker(MANIFEST, 'num-bigint'), 'https://github.com/owner/repo/issues/12');
  assert.equal(findCargoBlocker(MANIFEST, 'serde'), 'https://github.com/owner/repo/issues/34');
  assert.equal(findCargoBlocker(MANIFEST, 'thiserror'), null);
  assert.equal(findCargoBlocker(MANIFEST, 'missing'), null);
});

test("another dependency's comment on the line above does not count", () => {
  const manifest = 'a = "1" # https://github.com/owner/repo/issues/1\nb = "1"\n';
  assert.equal(findCargoBlocker(manifest, 'a'), 'https://github.com/owner/repo/issues/1');
  assert.equal(findCargoBlocker(manifest, 'b'), null);
});

test('a URL that is not in a comment does not count', () => {
  const manifest = 'foo = { git = "https://github.com/owner/repo/issues/1" }\n';
  assert.equal(findCargoBlocker(manifest, 'foo'), null);
});

test('classifies outdated dependencies', () => {
  const { failures, blocked } = classify(parseCargoOutdated(CARGO_OUTDATED), (name) =>
    findCargoBlocker(MANIFEST, name)
  );
  assert.deepEqual(
    failures.map((dependency) => dependency.name),
    ['thiserror']
  );
  assert.deepEqual(
    blocked.map((dependency) => dependency.blocker),
    ['https://github.com/owner/repo/issues/12']
  );
});

test('parses npm outdated JSON and blockers', () => {
  const outdated = parseNpmOutdated(
    JSON.stringify({
      react: { current: '18.3.1', wanted: '18.3.1', latest: '19.2.0' },
      vite: { current: '7.0.0', wanted: '7.0.0', latest: '7.0.0' },
    })
  );
  assert.deepEqual(outdated, [{ name: 'react', current: '18.3.1', latest: '19.2.0' }]);
  assert.deepEqual(parseNpmOutdated(''), []);

  const packageJson = {
    dependencyBlockers: { react: 'needs https://github.com/owner/repo/issues/7', vite: 'soon' },
  };
  assert.equal(findNpmBlocker(packageJson, 'react'), 'https://github.com/owner/repo/issues/7');
  assert.equal(findNpmBlocker(packageJson, 'vite'), null);
  assert.equal(findNpmBlocker({}, 'react'), null);
});

test('detects pending lockfile updates', () => {
  const output = `    Updating crates.io index
     Locking 2 packages to latest compatible versions
    Updating serde v1.0.200 -> v1.0.228
      Adding foo v0.1.0
warning: aborting update due to dry run`;
  assert.deepEqual(pendingLockfileUpdates(output), [
    'Updating serde v1.0.200 -> v1.0.228',
    'Adding foo v0.1.0',
  ]);
  assert.deepEqual(
    pendingLockfileUpdates('    Updating crates.io index\nwarning: aborting update due to dry run'),
    []
  );
});

test('freshnessEnforcement blocks pull requests and warns on other events', () => {
  assert.equal(freshnessEnforcement('pull_request'), 'fail');
  assert.equal(freshnessEnforcement('merge_group'), 'fail');
  assert.equal(freshnessEnforcement(undefined), 'fail');
  assert.equal(freshnessEnforcement('push'), 'warn');
  assert.equal(freshnessEnforcement('schedule'), 'warn');
  assert.equal(freshnessEnforcement('workflow_dispatch'), 'warn');
});

test('formatFreshnessReport passes when nothing is stale', () => {
  const { exitCode, lines } = formatFreshnessReport([], 'fail');
  assert.equal(exitCode, 0);
  assert.deepEqual(lines, ['All direct dependencies are at their latest releases.']);
});

test('formatFreshnessReport fails with error annotations in fail mode', () => {
  const { exitCode, lines } = formatFreshnessReport(['web/package.json: vite 8.3.3 is behind 8.3.4'], 'fail');
  assert.equal(exitCode, 1);
  assert.equal(lines[0], '::error title=Stale dependency::web/package.json: vite 8.3.3 is behind 8.3.4');
  assert.match(lines[1], /^1 dependency problem\(s\)\./);
});

test('formatFreshnessReport only warns in warn mode (push to main, issue #256)', () => {
  const { exitCode, lines } = formatFreshnessReport(['Cargo.toml: serde_json 1.0.151 is behind 1.0.152'], 'warn');
  assert.equal(exitCode, 0);
  assert.equal(lines[0], '::warning title=Stale dependency::Cargo.toml: serde_json 1.0.151 is behind 1.0.152');
  assert.match(lines[1], /only pull requests are blocked/);
});
