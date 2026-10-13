#!/usr/bin/env node

/**
 * Check that the documents other files point to still exist and still carry
 * the sections they link to (issue #256; principle 12 of the shared CI/CD
 * best practices, ported from link-foundation/rust-ai-driven-development-
 * pipeline-template's scripts/check-required-docs.sh).
 *
 * A diff review reliably misses a deleted CONTRIBUTING.md or a gutted README,
 * and lychee (links.yml) does not check #fragments, so it gets greener, not
 * redder, when a linked section disappears.
 *
 * A heading counts only as a level-2 `## Section` line of its own: a table of
 * contents entry is not the section it links to.
 *
 * Usage: node scripts/check-required-docs.mjs [--root <dir>]
 * Exits with code 1 and one ::error annotation per missing document or section.
 */

import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// path -> sections; an empty list requires only that the file exists.
// Add the referrer next to each entry so it is clear why the entry is needed.
export const REQUIREMENTS = {
  'README.md': ['Features', 'Development Setup', 'Contributing', 'License'],
  // #competitor-coverage: docs/competitor-corpus.md, scripts/update-coverage-gap-issue.mjs
  // #closing-issues: scripts/check-closing-references.mjs
  // #rust-build-load: .cargo/config.toml, scripts/bounded-cargo.mjs
  'CONTRIBUTING.md': [
    'Development Setup',
    'Rust Build Load',
    'Pull Request Process',
    'Closing Issues',
    'Competitor Coverage',
    'Changelog Management',
    'Release Process',
  ],
  'CHANGELOG.md': [],
  // Named by check-changelog-fragment.mjs and check-version-modification.mjs errors.
  'changelog.d/README.md': [],
};

/**
 * @param {string} root
 * @param {Record<string, string[]>} requirements
 * @returns {string[]} one message per missing document or section
 */
export function findMissingDocs(root, requirements = REQUIREMENTS) {
  const missing = [];
  for (const [path, sections] of Object.entries(requirements)) {
    const file = join(root, path);
    if (!existsSync(file)) {
      missing.push(`${path} is missing`);
      continue;
    }
    const headings = new Set(
      readFileSync(file, 'utf-8')
        .split(/\r?\n/)
        .filter((line) => line.startsWith('## '))
        .map((line) => line.slice(3).trimEnd())
    );
    for (const section of sections) {
      if (!headings.has(section)) {
        missing.push(`${path} has no "## ${section}" section`);
      }
    }
  }
  return missing;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const rootIndex = process.argv.indexOf('--root');
  const root = rootIndex === -1 ? process.cwd() : process.argv[rootIndex + 1];
  const missing = findMissingDocs(root);
  for (const message of missing) {
    console.log(`::error title=Required documentation::${message}`);
  }
  if (missing.length > 0) {
    process.exit(1);
  }
  console.log(`All ${Object.keys(REQUIREMENTS).length} required documents carry their sections`);
}
