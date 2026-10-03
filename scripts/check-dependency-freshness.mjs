#!/usr/bin/env node

/**
 * Fail when a direct dependency is behind its latest release (issue #223).
 *
 * Checks:
 * - Rust: `cargo outdated --root-deps-only --format json` (direct dependencies
 *   in Cargo.toml) and `cargo update --dry-run` (no semver-compatible lockfile
 *   update pending).
 * - JavaScript: `npm outdated --json` in every directory listed in NPM_DIRS.
 *
 * A dependency that cannot be updated yet is allowed when its blocker is
 * documented with an issue URL:
 * - Cargo.toml: a comment on the dependency line or a comment line just above it,
 *   e.g. `foo = "1" # blocked by https://github.com/owner/repo/issues/1`.
 * - package.json (JSON has no comments): a `dependencyBlockers` object mapping
 *   the package name to the issue URL.
 *
 * Usage: node scripts/check-dependency-freshness.mjs [--skip-cargo] [--skip-npm]
 */

import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const NPM_DIRS = ['web'];
const ISSUE_URL = /https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+/;

/**
 * Parse `cargo outdated --format json` output into outdated dependencies.
 * @param {string} json
 * @returns {{name: string, current: string, latest: string}[]}
 */
export function parseCargoOutdated(json) {
  // cargo-outdated prints one JSON document per workspace member.
  const documents = json
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.startsWith('{'))
    .map((line) => JSON.parse(line));
  return documents.flatMap((document) =>
    (document.dependencies ?? [])
      .filter((dependency) => dependency.latest && dependency.latest !== '---')
      .filter((dependency) => dependency.latest !== dependency.project)
      .map((dependency) => ({
        name: dependency.name,
        current: dependency.project,
        latest: dependency.latest,
      }))
  );
}

/**
 * Parse `npm outdated --json` output into outdated dependencies.
 * @param {string} json
 * @returns {{name: string, current: string, latest: string}[]}
 */
export function parseNpmOutdated(json) {
  if (!json.trim()) {
    return [];
  }
  return Object.entries(JSON.parse(json))
    .filter(([, info]) => info.latest && info.current !== info.latest)
    .map(([name, info]) => ({
      name,
      current: info.current ?? info.wanted ?? 'missing',
      latest: info.latest,
    }));
}

/**
 * Find the blocker issue URL documented for a Cargo.toml dependency.
 * @param {string} manifest - Cargo.toml contents
 * @param {string} name - dependency name
 * @returns {string | null}
 */
export function findCargoBlocker(manifest, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const declaration = new RegExp(
    `^\\s*(?:"?${escaped}"?\\s*=|\\[(?:[\\w.-]*\\.)?dependencies\\.${escaped}\\])`
  );
  const lines = manifest.split('\n');
  for (let index = 0; index < lines.length; index++) {
    if (!declaration.test(lines[index])) {
      continue;
    }
    const previous = lines[index - 1] ?? '';
    const candidates = [lines[index], previous.trim().startsWith('#') ? previous : ''];
    for (const line of candidates) {
      const comment = line.includes('#') ? line.slice(line.indexOf('#')) : '';
      const match = comment.match(ISSUE_URL);
      if (match) {
        return match[0];
      }
    }
  }
  return null;
}

/**
 * Find the blocker issue URL documented for a package.json dependency.
 * @param {object} packageJson - parsed package.json
 * @param {string} name - package name
 * @returns {string | null}
 */
export function findNpmBlocker(packageJson, name) {
  const blocker = packageJson.dependencyBlockers?.[name];
  const match = typeof blocker === 'string' ? blocker.match(ISSUE_URL) : null;
  return match ? match[0] : null;
}

/**
 * Split outdated dependencies into failures and documented blockers.
 * @param {{name: string, current: string, latest: string}[]} outdated
 * @param {(name: string) => string | null} findBlocker
 */
export function classify(outdated, findBlocker) {
  const failures = [];
  const blocked = [];
  for (const dependency of outdated) {
    const blocker = findBlocker(dependency.name);
    if (blocker) {
      blocked.push({ ...dependency, blocker });
    } else {
      failures.push(dependency);
    }
  }
  return { failures, blocked };
}

/**
 * Lines reported by `cargo update --dry-run` that would change the lockfile.
 * @param {string} output - stderr of `cargo update --dry-run`
 * @returns {string[]}
 */
export function pendingLockfileUpdates(output) {
  return output
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => /^(Updating|Adding|Removing|Downgrading)\s/.test(line))
    .filter((line) => !line.startsWith('Updating crates.io index'))
    .filter((line) => !/^Updating\s+git repository/.test(line));
}

function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8' });
  if (result.error) {
    throw new Error(`failed to run ${command}: ${result.error.message}`);
  }
  return result;
}

function report(label, { failures, blocked }) {
  for (const dependency of blocked) {
    console.log(
      `  ${label}: ${dependency.name} ${dependency.current} -> ${dependency.latest} (blocked: ${dependency.blocker})`
    );
  }
  for (const dependency of failures) {
    console.error(
      `  ${label}: ${dependency.name} ${dependency.current} is behind ${dependency.latest}`
    );
  }
  return failures.length;
}

function checkCargo() {
  let problems = 0;
  const outdated = run('cargo', ['outdated', '--root-deps-only', '--format', 'json'], '.');
  if (outdated.status !== 0) {
    console.error(outdated.stderr);
    throw new Error('cargo outdated failed (install it with `cargo install cargo-outdated`)');
  }
  const manifest = readFileSync('Cargo.toml', 'utf8');
  problems += report(
    'Cargo.toml',
    classify(parseCargoOutdated(outdated.stdout), (name) => findCargoBlocker(manifest, name))
  );

  const update = run('cargo', ['update', '--dry-run'], '.');
  for (const line of pendingLockfileUpdates(update.stderr)) {
    console.error(`  Cargo.lock: ${line} (run \`cargo update\`)`);
    problems++;
  }
  return problems;
}

function checkNpm() {
  let problems = 0;
  for (const directory of NPM_DIRS) {
    const manifestPath = join(directory, 'package.json');
    if (!existsSync(manifestPath)) {
      continue;
    }
    const packageJson = JSON.parse(readFileSync(manifestPath, 'utf8'));
    // `npm outdated` exits 1 when anything is outdated; the JSON is what counts.
    const outdated = run('npm', ['outdated', '--json'], directory);
    problems += report(
      manifestPath,
      classify(parseNpmOutdated(outdated.stdout), (name) => findNpmBlocker(packageJson, name))
    );
  }
  return problems;
}

function main() {
  const args = process.argv.slice(2);
  let problems = 0;
  if (!args.includes('--skip-cargo')) {
    console.log('Checking Rust dependencies...');
    problems += checkCargo();
  }
  if (!args.includes('--skip-npm')) {
    console.log('Checking JavaScript dependencies...');
    problems += checkNpm();
  }
  if (problems > 0) {
    console.error(
      `\n${problems} dependency problem(s). Update them, or document the blocker with an issue URL ` +
        '(a Cargo.toml comment, or `dependencyBlockers` in package.json).'
    );
    process.exit(1);
  }
  console.log('All direct dependencies are at their latest releases.');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
