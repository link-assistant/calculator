import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  acquireLock,
  buildCommand,
  diskUsage,
  incrementalDirs,
  positiveNumber,
  pruneTarget,
  readSettings,
} from './bounded-cargo.mjs';

const script = fileURLToPath(new URL('./bounded-cargo.mjs', import.meta.url));

function fixtureTarget() {
  const dir = mkdtempSync(join(tmpdir(), 'bounded-cargo-'));
  const target = join(dir, 'target');
  mkdirSync(join(target, 'debug', 'incremental', 'calc-1'), { recursive: true });
  mkdirSync(join(target, 'debug', 'deps'), { recursive: true });
  mkdirSync(join(target, 'wasm32-unknown-unknown', 'release', 'incremental'), { recursive: true });
  writeFileSync(join(target, 'debug', 'incremental', 'calc-1', 'query-cache.bin'), Buffer.alloc(600));
  writeFileSync(join(target, 'wasm32-unknown-unknown', 'release', 'incremental', 'dep-graph.bin'), Buffer.alloc(300));
  writeFileSync(join(target, 'debug', 'deps', 'libcalc.rlib'), Buffer.alloc(100));
  return { dir, target };
}

test('reads the defaults and rejects invalid limits', () => {
  const settings = readSettings({});
  assert.match(settings.targetDir, /target$/);
  assert.equal(settings.maxBytes, 8 * 1024 ** 3);
  assert.equal(settings.memoryBytes, 4 * 1024 ** 3);
  assert.equal(settings.jobs, 2);
  assert.equal(positiveNumber({ LIMIT: '1.5' }, 'LIMIT', 3), 1.5);
  assert.equal(positiveNumber({ LIMIT: '' }, 'LIMIT', 3), 3);
  assert.throws(() => positiveNumber({ LIMIT: '0' }, 'LIMIT', 3), /LIMIT must be a positive number/);
  assert.throws(() => positiveNumber({ LIMIT: 'many' }, 'LIMIT', 3), /LIMIT must be a positive number/);
});

test('uses a shared target directory when one is configured', () => {
  const shared = join(tmpdir(), 'calculator-shared-target');
  assert.equal(readSettings({ CALCULATOR_CARGO_TARGET_DIR: shared }).targetDir, shared);
});

test('passes the limits through the environment and prlimit on Linux', () => {
  const env = { CARGO_TARGET_DIR: '/shared/target', CARGO_BUILD_JOBS: '3', CARGO_INCREMENTAL: '0' };
  assert.deepEqual(
    buildCommand(['test', '--', '--nocapture'], { targetDir: '/shared/target', jobs: 3, memoryBytes: 1024, platform: 'darwin' }),
    { command: 'cargo', args: ['test', '--', '--nocapture'], env },
  );
  assert.deepEqual(
    buildCommand(['fmt', '--check'], { targetDir: '/shared/target', jobs: 3, memoryBytes: 1024, platform: 'linux' }),
    { command: 'prlimit', args: ['--as=1024', '--', 'cargo', 'fmt', '--check'], env },
  );
});

test('finds the incremental caches of every profile and target triple', () => {
  const { dir, target } = fixtureTarget();
  try {
    assert.deepEqual(incrementalDirs(target), [
      join(target, 'debug', 'incremental'),
      join(target, 'wasm32-unknown-unknown', 'release', 'incremental'),
    ]);
    assert.equal(diskUsage(target), 1000);
    assert.deepEqual(incrementalDirs(join(dir, 'missing')), []);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('keeps a target directory that fits its budget', () => {
  const { dir, target } = fixtureTarget();
  try {
    assert.deepEqual(pruneTarget(target, 1000), []);
    assert.equal(diskUsage(target), 1000);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('prunes the incremental caches first and keeps the dependencies', () => {
  const { dir, target } = fixtureTarget();
  try {
    assert.equal(pruneTarget(target, 500).length, 2);
    assert.ok(existsSync(join(target, 'debug', 'deps', 'libcalc.rlib')));
    assert.equal(diskUsage(target), 100);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('removes the whole target directory when the caches are not enough', () => {
  const { dir, target } = fixtureTarget();
  try {
    assert.deepEqual(pruneTarget(target, 50).at(-1), target);
    assert.equal(existsSync(target), false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('takes over the lock of an exited process and times out on a live one', () => {
  const dir = mkdtempSync(join(tmpdir(), 'bounded-cargo-lock-'));
  const lock = join(dir, 'target.lock');
  try {
    const exited = spawnSync(process.execPath, ['-e', 'process.stdout.write(String(process.pid))'], {
      encoding: 'utf8',
    });
    writeFileSync(lock, exited.stdout);
    const descriptor = acquireLock(lock, 1000);
    assert.equal(readFileSync(lock, 'utf8'), String(process.pid));
    assert.throws(() => acquireLock(lock, 1), /still held by process/);
    rmSync(lock);
    assert.ok(descriptor > 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('prints the usage without arguments', () => {
  const result = spawnSync(process.execPath, [script], { encoding: 'utf8', timeout: 15000 });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /usage: node scripts\/bounded-cargo\.mjs <cargo arguments>/);
});

test('runs cargo against the configured target directory and releases the lock', () => {
  const dir = mkdtempSync(join(tmpdir(), 'bounded-cargo-run-'));
  const target = join(dir, 'shared-target');
  try {
    const result = spawnSync(process.execPath, [script, '--version'], {
      encoding: 'utf8',
      timeout: 15000,
      // CI sets CALCULATOR_CARGO_JOBS for the Rust stage; pin it here.
      env: { ...process.env, CALCULATOR_CARGO_TARGET_DIR: target, CALCULATOR_CARGO_JOBS: '2', BOUNDED_CARGO_VERBOSE: '1' },
    });
    if (result.error?.code === 'ENOENT') {
      return;
    }
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^cargo /);
    assert.match(result.stderr, /CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0/);
    assert.equal(existsSync(`${target}.lock`), false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
