#!/usr/bin/env node

/**
 * Keep the JavaScript engine's version equal to Cargo.toml's (issue #254).
 *
 * The web app shows the engine's version in its footer, and the engine is now
 * js/, so a release that bumps only Cargo.toml would show the old version.
 * The release scripts call syncJsVersion after the bump; Script Checks runs
 * this file with --check so the copies cannot drift in a pull request.
 *
 * Usage: node scripts/sync-js-version.mjs [--check]
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

/** The files that carry the version, relative to the repository root. */
export const JS_VERSION_FILES = ['js/package.json', 'js/package-lock.json', 'js/src/index.js'];

/** The version in Cargo.toml's [package] section. */
export function cargoVersion(cargoToml) {
  const match = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    throw new Error('Could not parse the version from Cargo.toml');
  }
  return match[1];
}

/**
 * The file with its version set; the package's own entries only, never a
 * dependency's.
 * @param {string} path one of JS_VERSION_FILES
 * @param {string} source
 * @param {string} version
 */
export function withVersion(path, source, version) {
  if (path.endsWith('package.json')) {
    return source.replace(/^( {2}"version": ")[^"]+(")/m, `$1${version}$2`);
  }
  if (path.endsWith('package-lock.json')) {
    // The top-level entry and the root package under "packages" -> "".
    return source
      .replace(/^( {2}"version": ")[^"]+(")/m, `$1${version}$2`)
      .replace(/^( {4}"": \{\n {6}"name": "[^"]+",\n {6}"version": ")[^"]+(")/m, `$1${version}$2`);
  }
  return source.replace(/^(export const VERSION = ')[^']+(';)/m, `$1${version}$2`);
}

/**
 * Set every JS version file to Cargo.toml's version.
 * @param {string} root repository root
 * @param {{ check?: boolean }} options with check, change nothing
 * @returns {string[]} the files that differed
 */
export function syncJsVersion(root, { check = false } = {}) {
  const version = cargoVersion(readFileSync(join(root, 'Cargo.toml'), 'utf8'));
  const changed = [];
  for (const path of JS_VERSION_FILES) {
    const file = join(root, path);
    const before = readFileSync(file, 'utf8');
    const after = withVersion(path, before, version);
    if (after !== before) {
      changed.push(path);
      if (!check) {
        writeFileSync(file, after);
      }
    }
  }
  return changed;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const check = process.argv.includes('--check');
  const changed = syncJsVersion(root, { check });
  if (check && changed.length > 0) {
    for (const path of changed) {
      console.error(`::error file=${path}::${path} does not carry Cargo.toml's version; run: node scripts/sync-js-version.mjs`);
    }
    process.exit(1);
  }
  console.log(check ? 'The JavaScript engine has the version of Cargo.toml.' : `Updated: ${changed.join(', ') || 'nothing'}`);
}
