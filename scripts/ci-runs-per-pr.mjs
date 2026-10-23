#!/usr/bin/env node

/**
 * Count CI runs and Rust-stage runs per pull request (issue #254).
 *
 * The JavaScript-first split should cut the Rust work per pull request: a
 * push that fails a JavaScript job never starts Cargo. This prints, for the
 * last N merged pull requests, how many CI/CD Pipeline runs there were and
 * how many of them reached the Rust stage, as a Markdown table.
 *
 * Usage: node scripts/ci-runs-per-pr.mjs [count=5] [owner/repo]
 * Needs the gh CLI, authenticated.
 */

import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const gh = (args) => JSON.parse(execFileSync('gh', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }));

/**
 * @param {{ jobs: { name: string, conclusion: string | null }[] }[]} runs jobs of each run
 * @returns {{ runs: number, rust: number }}
 */
export function summarize(runs) {
  // Before the split (no JavaScript Stage job) every run started Cargo.
  const reachedRust = (run) =>
    !run.jobs.some((job) => job.name === 'JavaScript Stage') ||
    run.jobs.some((job) => job.name.startsWith('Rust Stage /') && job.conclusion !== 'skipped');
  return { runs: runs.length, rust: runs.filter(reachedRust).length };
}

function main() {
  const count = Number(process.argv[2] ?? 5);
  const repo = process.argv[3] ?? 'link-assistant/calculator';
  const prs = gh(['pr', 'list', '--repo', repo, '--state', 'merged', '--limit', String(count), '--json', 'number,headRefName,title']);
  console.log('| PR | CI runs | Runs that reached Rust |\n| --- | ---: | ---: |');
  for (const pr of prs) {
    const runs = gh(['api', '--paginate', '--slurp',
      `repos/${repo}/actions/workflows/release.yml/runs?event=pull_request&branch=${encodeURIComponent(pr.headRefName)}&per_page=100`])
      .flatMap((page) => page.workflow_runs);
    const withJobs = runs.map((run) => ({
      jobs: gh(['api', '--paginate', '--slurp', `repos/${repo}/actions/runs/${run.id}/jobs?per_page=100`]).flatMap((page) => page.jobs),
    }));
    const { runs: total, rust } = summarize(withJobs);
    console.log(`| #${pr.number} | ${total} | ${rust} |`);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
