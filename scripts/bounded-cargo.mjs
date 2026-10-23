#!/usr/bin/env node

/**
 * Run Cargo with a bounded build load (issue #254).
 *
 * Local agent runs execute the JavaScript tests by default; use this wrapper
 * only when a Rust build is actually needed. It:
 *
 * - takes an exclusive lock next to the target directory, so concurrent
 *   agents queue for one build instead of compiling competing trees;
 * - prunes the target directory before and after the build when it exceeds
 *   its disk budget: first the incremental caches, then everything;
 * - caps parallel rustc processes (CARGO_BUILD_JOBS, default 2) and, on
 *   Linux, the address space of every process with `prlimit --as`;
 * - disables incremental compilation (CARGO_INCREMENTAL=0).
 *
 * Usage:
 *   node scripts/bounded-cargo.mjs <cargo arguments>
 *   node scripts/bounded-cargo.mjs test --test integration
 *
 * Environment (all optional):
 *   CALCULATOR_CARGO_TARGET_DIR  shared target directory (default: target/)
 *   CALCULATOR_CARGO_MAX_GIB     disk budget of the target directory (default 8)
 *   CALCULATOR_CARGO_MEMORY_GIB  address-space limit per process (default 4)
 *   CALCULATOR_CARGO_JOBS        parallel rustc processes (default 2)
 *   CALCULATOR_CARGO_LOCK_WAIT_SECONDS  how long to wait for the lock (default 3600)
 *   BOUNDED_CARGO_VERBOSE=1      print the sizes, the lock owner and the command
 */

import { spawnSync } from 'node:child_process';
import {
  closeSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeSync,
} from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const GIB = 1024 ** 3;
const root = fileURLToPath(new URL('..', import.meta.url));
const verbose = process.env.BOUNDED_CARGO_VERBOSE === '1';

function log(message) {
  if (verbose) {
    console.error(`[bounded-cargo] ${message}`);
  }
}

/**
 * Read a positive number from the environment.
 * @param {Record<string, string | undefined>} env
 * @param {string} name
 * @param {number} fallback
 */
export function positiveNumber(env, name, fallback) {
  const raw = env[name];
  if (raw === undefined || raw === '') {
    return fallback;
  }
  const value = Number(raw);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error(`${name} must be a positive number, got "${raw}"`);
  }
  return value;
}

/**
 * Resolve the wrapper settings from the environment.
 * @param {Record<string, string | undefined>} env
 */
export function readSettings(env) {
  return {
    targetDir: resolve(root, env.CALCULATOR_CARGO_TARGET_DIR || 'target'),
    maxBytes: positiveNumber(env, 'CALCULATOR_CARGO_MAX_GIB', 8) * GIB,
    memoryBytes: Math.floor(positiveNumber(env, 'CALCULATOR_CARGO_MEMORY_GIB', 4) * GIB),
    jobs: Math.floor(positiveNumber(env, 'CALCULATOR_CARGO_JOBS', 2)),
    lockWaitMs: positiveNumber(env, 'CALCULATOR_CARGO_LOCK_WAIT_SECONDS', 3600) * 1000,
  };
}

/**
 * Bytes used by a file or directory tree; symbolic links count as themselves.
 * @param {string} path
 * @returns {number}
 */
export function diskUsage(path) {
  let stat;
  try {
    stat = lstatSync(path);
  } catch (error) {
    if (error.code === 'ENOENT') {
      return 0;
    }
    throw error;
  }
  if (!stat.isDirectory()) {
    return stat.size;
  }
  let total = 0;
  for (const name of readdirSync(path)) {
    total += diskUsage(join(path, name));
  }
  return total;
}

/**
 * The incremental compilation caches under a target directory, such as
 * `target/debug/incremental` or `target/wasm32-unknown-unknown/release/incremental`.
 * @param {string} targetDir
 * @returns {string[]}
 */
export function incrementalDirs(targetDir) {
  const found = [];
  const visit = (dir, depth) => {
    let entries;
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch (error) {
      if (error.code === 'ENOENT') {
        return;
      }
      throw error;
    }
    for (const entry of entries) {
      if (!entry.isDirectory()) {
        continue;
      }
      const path = join(dir, entry.name);
      if (entry.name === 'incremental') {
        found.push(path);
      } else if (depth < 2) {
        visit(path, depth + 1);
      }
    }
  };
  visit(targetDir, 0);
  return found.sort();
}

/**
 * Bring a target directory under its disk budget: remove the incremental
 * caches first, then the whole directory if that is not enough.
 * @param {string} targetDir
 * @param {number} maxBytes
 * @returns {string[]} what was removed, for the log
 */
export function pruneTarget(targetDir, maxBytes) {
  const before = diskUsage(targetDir);
  log(`${targetDir} uses ${(before / GIB).toFixed(2)} GiB of ${(maxBytes / GIB).toFixed(2)} GiB`);
  if (before <= maxBytes) {
    return [];
  }
  const removed = [];
  for (const dir of incrementalDirs(targetDir)) {
    rmSync(dir, { recursive: true, force: true });
    removed.push(dir);
  }
  if (diskUsage(targetDir) > maxBytes) {
    rmSync(targetDir, { recursive: true, force: true });
    removed.push(targetDir);
  }
  return removed;
}

/**
 * Build the command line and environment. The job limit goes through
 * CARGO_BUILD_JOBS rather than `-j`, which subcommands such as `fmt` reject.
 * @param {string[]} cargoArgs
 * @param {{ targetDir: string, jobs: number, memoryBytes: number, platform: string }} options
 * @returns {{ command: string, args: string[], env: Record<string, string> }}
 */
export function buildCommand(cargoArgs, { targetDir, jobs, memoryBytes, platform }) {
  const env = {
    CARGO_TARGET_DIR: targetDir,
    CARGO_BUILD_JOBS: String(jobs),
    CARGO_INCREMENTAL: '0',
  };
  if (platform === 'linux') {
    return { command: 'prlimit', args: [`--as=${memoryBytes}`, '--', 'cargo', ...cargoArgs], env };
  }
  return { command: 'cargo', args: [...cargoArgs], env };
}

function processIsAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
}

/**
 * Take the lock file, waiting for a live holder and taking over a lock whose
 * process has exited.
 * @param {string} lockPath
 * @param {number} waitMs
 * @returns {number} file descriptor to close on release
 */
export function acquireLock(lockPath, waitMs) {
  const deadline = Date.now() + waitMs;
  const pause = new Int32Array(new SharedArrayBuffer(4));
  let announced = false;
  for (;;) {
    try {
      const descriptor = openSync(lockPath, 'wx');
      writeSync(descriptor, String(process.pid));
      return descriptor;
    } catch (error) {
      if (error.code !== 'EEXIST') {
        throw error;
      }
    }
    let owner = Number.NaN;
    try {
      owner = Number(readFileSync(lockPath, 'utf8').trim());
    } catch (error) {
      if (error.code !== 'ENOENT') {
        throw error;
      }
      continue;
    }
    if (Number.isInteger(owner) && owner > 0 && !processIsAlive(owner)) {
      log(`removing the stale lock of exited process ${owner}`);
      rmSync(lockPath, { force: true });
      continue;
    }
    if (Date.now() >= deadline) {
      throw new Error(`${lockPath} is still held by process ${owner}; another Cargo build is running`);
    }
    if (!announced) {
      console.error(`[bounded-cargo] waiting for the Cargo build of process ${owner} to finish`);
      announced = true;
    }
    Atomics.wait(pause, 0, 0, Math.max(1, Math.min(1000, deadline - Date.now())));
  }
}

function main() {
  const cargoArgs = process.argv.slice(2);
  if (cargoArgs.length === 0) {
    console.error('usage: node scripts/bounded-cargo.mjs <cargo arguments>');
    process.exit(2);
  }
  const settings = readSettings(process.env);
  mkdirSync(dirname(settings.targetDir), { recursive: true });
  const lockPath = `${settings.targetDir}.lock`;
  const descriptor = acquireLock(lockPath, settings.lockWaitMs);
  try {
    for (const path of pruneTarget(settings.targetDir, settings.maxBytes)) {
      console.error(`[bounded-cargo] pruned ${path} (over the disk budget)`);
    }
    const { command, args, env } = buildCommand(cargoArgs, { ...settings, platform: process.platform });
    log(`${Object.entries(env).map(([name, value]) => `${name}=${value}`).join(' ')} ${command} ${args.join(' ')}`);
    const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', env: { ...process.env, ...env } });
    if (result.error) {
      throw result.error;
    }
    process.exitCode = result.status ?? 1;
    for (const path of pruneTarget(settings.targetDir, settings.maxBytes)) {
      console.error(`[bounded-cargo] pruned ${path} after the build (over the disk budget)`);
    }
  } finally {
    closeSync(descriptor);
    rmSync(lockPath, { force: true });
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
