import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import {
  checkCratePublisher,
  packageNameFromManifest,
  publishProbeBody,
} from './preflight-crates-token.mjs';

const script = fileURLToPath(new URL('./preflight-crates-token.mjs', import.meta.url));

const respond = (status, body) => async () => new Response(JSON.stringify(body), { status });

test('the probe body is length-prefixed metadata with no archive', () => {
  const body = publishProbeBody('link-calculator');
  const length = body.readUInt32LE(0);
  assert.equal(body.length, 4 + length);
  assert.deepEqual(JSON.parse(body.subarray(4).toString()), {
    name: 'link-calculator',
    vers: '0.0.0-preflight',
  });
});

test('reads the package name, not a dependency or bin name', () => {
  const manifest = `[workspace]\nmembers = ["experiments"]\n\n[package]\nname = "link-calculator"\nversion = "1.0.0"\n\n[[bin]]\nname = "other"\n`;
  assert.equal(packageNameFromManifest(manifest), 'link-calculator');
  assert.equal(packageNameFromManifest('[workspace]\nmembers = []\n'), null);
});

test('only the exact missing-archive answer verifies the token', async () => {
  const verified = await checkCratePublisher({
    packageName: 'link-calculator',
    token: 't',
    fetchImpl: respond(400, { errors: [{ detail: 'invalid tarball length' }] }),
  });
  assert.equal(verified.verdict, 'verified');

  const otherBadRequest = await checkCratePublisher({
    packageName: 'link-calculator',
    token: 't',
    fetchImpl: respond(400, { errors: [{ detail: 'something else' }] }),
  });
  assert.equal(otherBadRequest.verdict, 'unknown');
});

test('a missing or rejected token is denied', async () => {
  assert.equal((await checkCratePublisher({ packageName: 'x', token: '' })).verdict, 'denied');
  for (const status of [401, 403]) {
    const result = await checkCratePublisher({
      packageName: 'x',
      token: 't',
      fetchImpl: respond(status, { errors: [{ detail: 'nope' }] }),
    });
    assert.equal(result.verdict, 'denied', `HTTP ${status}`);
  }
});

test('rate limits and network errors are unknown and never echo the response', async () => {
  const limited = await checkCratePublisher({
    packageName: 'x',
    token: 't',
    fetchImpl: respond(429, {}),
  });
  assert.equal(limited.verdict, 'unknown');

  const offline = await checkCratePublisher({
    packageName: 'x',
    token: 'secret-token',
    fetchImpl: async () => {
      throw new Error('connect failed for secret-token');
    },
  });
  assert.equal(offline.verdict, 'unknown');
  assert.doesNotMatch(offline.message, /secret-token/);
});

test('the CLI fails the release without a token before any request', { timeout: 15000 }, () => {
  const env = { ...process.env, CRATES_API: 'http://127.0.0.1:9' };
  delete env.CARGO_REGISTRY_TOKEN;
  delete env.CARGO_TOKEN;
  const result = spawnSync(process.execPath, [script], { encoding: 'utf8', env });
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /::error title=Release preflight::crates\.io: neither CARGO_REGISTRY_TOKEN/);
});

test('the CLI only warns when crates.io cannot be reached', { timeout: 15000 }, () => {
  const result = spawnSync(process.execPath, [script], {
    encoding: 'utf8',
    env: { ...process.env, CARGO_REGISTRY_TOKEN: 't', CRATES_API: 'http://127.0.0.1:9' },
  });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /::warning title=Release preflight::crates\.io: publish probe unreachable/);
});
