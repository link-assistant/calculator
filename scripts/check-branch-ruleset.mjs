#!/usr/bin/env node

/**
 * Check that the default branch requires both CI stages (issue #254).
 *
 * The required checks live in repository settings, which a pull request
 * cannot change, so .github/main-ruleset.json holds the intended ruleset and
 * this script compares it with the live ones. A missing check is a warning on
 * pull requests and an error on main, with the command that applies it.
 *
 * Usage: node scripts/check-branch-ruleset.mjs [--strict]
 * Environment: GITHUB_REPOSITORY (owner/repo), GITHUB_TOKEN (optional),
 *   RULESET_STRICT=true (same as --strict).
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const RULESET_FILE = '.github/main-ruleset.json';

/** The status check contexts a ruleset requires. */
export function requiredContexts(ruleset) {
  return (ruleset.rules ?? [])
    .filter((rule) => rule.type === 'required_status_checks')
    .flatMap((rule) => rule.parameters?.required_status_checks ?? [])
    .map((check) => check.context);
}

/**
 * @param {string[]} wanted contexts the committed ruleset requires
 * @param {object[]} live rulesets active on the default branch (with rules)
 * @returns {string[]} contexts no live ruleset requires
 */
export function missingContexts(wanted, live) {
  const have = new Set(live.filter((ruleset) => ruleset.enforcement === 'active').flatMap(requiredContexts));
  return wanted.filter((context) => !have.has(context));
}

async function api(path) {
  const headers = { Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }
  const response = await fetch(`https://api.github.com/${path}`, { headers });
  if (!response.ok) {
    throw new Error(`GET ${path}: ${response.status} ${await response.text()}`);
  }
  return response.json();
}

async function main() {
  const strict = process.argv.includes('--strict') || process.env.RULESET_STRICT === 'true';
  const repository = process.env.GITHUB_REPOSITORY || 'link-assistant/calculator';
  const root = fileURLToPath(new URL('..', import.meta.url));
  const wanted = requiredContexts(JSON.parse(readFileSync(`${root}/${RULESET_FILE}`, 'utf8')));
  const summaries = await api(`repos/${repository}/rulesets?includes_parents=true`);
  const live = await Promise.all(summaries.map((summary) => api(`repos/${repository}/rulesets/${summary.id}`)));
  const missing = missingContexts(wanted, live);
  if (missing.length === 0) {
    console.log(`The default branch requires ${wanted.join(', ')}.`);
    return;
  }
  const level = strict ? 'error' : 'warning';
  console.log(
    `::${level} title=Required checks (issue #254)::The default branch does not require ${missing.join(', ')}. ` +
      `A repository admin must run: gh api -X POST repos/${repository}/rulesets --input ${RULESET_FILE}`,
  );
  if (strict) {
    process.exit(1);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await main();
}
