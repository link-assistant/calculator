#!/usr/bin/env node

/**
 * Prove the crates.io publish token can write before the release commits a
 * version bump (issue #256, principle 16 "Prove You Can Publish Before You
 * Build" of the shared CI/CD best practices; ported from
 * link-foundation/rust-ai-driven-development-pipeline-template's
 * scripts/crates-publish-preflight.mjs).
 *
 * The release jobs push the version bump to main before `cargo publish` runs,
 * so an expired or revoked token used to leave main bumped and tagged while
 * crates.io never received the version (issue #223). A non-empty secret proves
 * nothing, so this sends a valid, length-prefixed publish metadata request with
 * NO archive: crates.io checks the token and its crate scope before reading the
 * archive length, and answering exactly "invalid tarball length" proves both
 * without publishing anything.
 *
 * Verdicts and exit codes:
 *   verified (0) - token and crate scope accepted
 *   denied   (1) - no token, or crates.io answered 401/403; fails the release
 *   unknown  (0) - timeout, 429 or any other answer; warns only, because it has
 *                  not said the credential is broken, and `cargo publish` still
 *                  fails loudly on a real rejection
 *
 * Usage: node scripts/preflight-crates-token.mjs
 *
 * Environment variables:
 *   - CARGO_REGISTRY_TOKEN / CARGO_TOKEN: the publish token (same order as publish-crate.mjs)
 *   - CRATES_API: registry base URL (default https://crates.io)
 *   - PREFLIGHT_TIMEOUT_SECONDS: request timeout (default 15)
 *   - PREFLIGHT_VERBOSE: set to 1 to log the HTTP status of the probe (default off)
 */

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { getCargoTomlPath, getRustRoot, parseRustRootConfig } from './rust-paths.mjs';

export function publishProbeBody(packageName) {
  const metadata = Buffer.from(JSON.stringify({ name: packageName, vers: '0.0.0-preflight' }));
  const body = Buffer.alloc(4 + metadata.length);
  body.writeUInt32LE(metadata.length);
  metadata.copy(body, 4);
  return body;
}

/**
 * Read the package name from the [package] table, not a dependency's name.
 * @param {string} cargoToml
 * @returns {string | null}
 */
export function packageNameFromManifest(cargoToml) {
  const table = cargoToml.match(/^\[package\]\s*$([\s\S]*?)(?=^\[|(?![\s\S]))/m);
  const name = table?.[1].match(/^name\s*=\s*"([^"]+)"/m);
  return name ? name[1] : null;
}

export async function checkCratePublisher({
  packageName,
  token,
  api = 'https://crates.io',
  fetchImpl = globalThis.fetch,
  timeoutMs = 15_000,
  log = () => {},
}) {
  if (!token) {
    return {
      verdict: 'denied',
      message: 'neither CARGO_REGISTRY_TOKEN nor CARGO_TOKEN is set, so cargo publish would fail',
    };
  }
  if (!packageName) {
    return { verdict: 'unknown', message: 'could not read the package name from Cargo.toml' };
  }
  try {
    const response = await fetchImpl(`${api}/api/v1/crates/new`, {
      method: 'PUT',
      headers: {
        authorization: token,
        'content-type': 'application/octet-stream',
        'user-agent': 'release-preflight (link-assistant/calculator)',
      },
      body: publishProbeBody(packageName),
      signal: AbortSignal.timeout(timeoutMs),
    });
    log(`crates.io publish probe answered HTTP ${response.status}`);
    if (response.status === 400) {
      const result = await response.json();
      if (result.errors?.length === 1 && result.errors[0].detail === 'invalid tarball length') {
        return {
          verdict: 'verified',
          message: `publish token and ${packageName} scope accepted; archive omitted, nothing was published`,
        };
      }
    }
    if (response.status === 401 || response.status === 403) {
      return {
        verdict: 'denied',
        message: `crates.io rejected the publish token or its ${packageName} scope (HTTP ${response.status})`,
      };
    }
    return {
      verdict: 'unknown',
      message: `publish probe returned HTTP ${response.status} without the exact missing-archive confirmation`,
    };
  } catch {
    // Remote bodies and errors may echo the token, so log only fixed messages.
    return { verdict: 'unknown', message: 'publish probe unreachable or returned an invalid response' };
  }
}

async function main() {
  const rustRoot = getRustRoot({ rustRoot: parseRustRootConfig() || undefined });
  const packageName = packageNameFromManifest(readFileSync(getCargoTomlPath({ rustRoot }), 'utf-8'));
  const verbose = process.env.PREFLIGHT_VERBOSE === '1';
  const result = await checkCratePublisher({
    packageName,
    token: process.env.CARGO_REGISTRY_TOKEN || process.env.CARGO_TOKEN,
    api: process.env.CRATES_API || 'https://crates.io',
    timeoutMs: Number(process.env.PREFLIGHT_TIMEOUT_SECONDS || 15) * 1000,
    log: verbose ? (line) => console.log(line) : () => {},
  });
  if (result.verdict === 'verified') {
    console.log(`crates.io: ${result.message}`);
  } else if (result.verdict === 'denied') {
    console.log(`::error title=Release preflight::crates.io: ${result.message}. Refusing to commit a version bump that cannot be published.`);
    process.exitCode = 1;
  } else {
    console.log(`::warning title=Release preflight::crates.io: ${result.message}. Continuing; cargo publish still fails on a real rejection.`);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  await main();
}
