#!/usr/bin/env node

/**
 * The single push gate for multi-agent work (issue #254, AGENTS.md).
 *
 * Drafting agents never push. The one CI/CD agent that owns `git push` runs
 * this script first. It:
 *
 * 1. refuses (exit 1) while any workflow run on this branch's commit range
 *    (`<base>..HEAD` and the pushed branch head) is queued or in progress;
 * 2. otherwise downloads the failed-job logs of every failing run of the
 *    latest pushed commit to `ci-logs/`, so all failures are fixed together
 *    and pushed once, instead of one push per failure.
 *
 * Usage:
 *   node scripts/ci-push-gate.mjs [--branch <name>] [--base origin/main]
 *     [--repo <owner/repo>] [--logs-dir ci-logs] [--verbose]
 *
 * --verbose (or CI_PUSH_GATE_VERBOSE=1) lists every run that was considered.
 */
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ACTIVE = new Set(['queued', 'in_progress', 'waiting', 'requested', 'pending']);
const PASSING = new Set(['success', 'skipped', 'neutral']);

/**
 * Split the runs of a branch into the ones that block a push and the
 * failures of the latest pushed commit.
 * @param {Array<{databaseId: number, status: string, conclusion: string, headSha: string,
 *   workflowName: string, createdAt: string}>} runs
 * @param {Set<string>} commits the branch's commit range; empty means every run counts
 */
export function classifyRuns(runs, commits) {
  const relevant = runs
    .filter((run) => commits.size === 0 || commits.has(run.headSha))
    .sort((a, b) => b.createdAt.localeCompare(a.createdAt));
  const active = relevant.filter((run) => ACTIVE.has(run.status));
  const latestSha = relevant[0]?.headSha ?? null;
  const latestRuns = relevant.filter((run) => run.headSha === latestSha);
  const failures = latestRuns.filter((run) => run.status === 'completed' && !PASSING.has(run.conclusion));
  return { relevant, active, latestSha, latestRuns, failures };
}

/**
 * @param {{ databaseId: number, workflowName: string }} run
 */
export function logFileName(run) {
  const slug = run.workflowName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  return `${slug || 'workflow'}-${run.databaseId}.log`;
}

function parseArgs(argv) {
  const options = { base: 'origin/main', logsDir: 'ci-logs', verbose: process.env.CI_PUSH_GATE_VERBOSE === '1' };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === '--verbose') {
      options.verbose = true;
    } else if (['--branch', '--base', '--repo', '--logs-dir'].includes(flag) && argv[index + 1]) {
      const key = flag.slice(2).replace(/-(\w)/g, (_, letter) => letter.toUpperCase());
      options[key] = argv[index + 1];
      index += 1;
    } else {
      throw new Error(`unknown or incomplete argument: ${flag}`);
    }
  }
  return options;
}

function revList(run, range) {
  try {
    return run('git', ['rev-list', range], { encoding: 'utf8' }).split('\n').filter(Boolean);
  } catch {
    return [];
  }
}

/**
 * @param {string[]} argv
 * @param {{ run?: typeof execFileSync, write?: typeof writeFileSync, print?: (line: string) => void }} io
 * @returns {number} the exit code
 */
export function runGate(argv, { run = execFileSync, write = writeFileSync, print = console.log } = {}) {
  const options = parseArgs(argv);
  const branch = options.branch
    ?? run('git', ['branch', '--show-current'], { encoding: 'utf8' }).trim();
  const commits = new Set([
    ...revList(run, `${options.base}..HEAD`),
    ...revList(run, `${options.base}..origin/${branch}`),
  ]);
  const repoArgs = options.repo ? ['--repo', options.repo] : [];
  const runs = JSON.parse(run('gh', [
    'run', 'list', ...repoArgs, '--branch', branch, '--limit', '100',
    '--json', 'databaseId,status,conclusion,headSha,workflowName,createdAt,url',
  ], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
  const { relevant, active, latestSha, failures } = classifyRuns(runs, commits);
  if (options.verbose) {
    for (const entry of relevant) {
      print(`  ${entry.createdAt} ${entry.headSha.slice(0, 8)} ${entry.status}/${entry.conclusion || '-'} ${entry.workflowName} ${entry.url}`);
    }
  }
  if (active.length > 0) {
    print(`::error title=CI push gate::Do not push: ${active.length} run(s) on ${branch} are still queued or in progress. Wait for them, then collect all failures and push once.`);
    for (const entry of active) {
      print(`  ${entry.status} ${entry.workflowName} ${entry.headSha.slice(0, 8)} ${entry.url}`);
    }
    return 1;
  }
  if (latestSha === null) {
    print(`No workflow runs on ${branch} yet; push once when every draft is complete.`);
    return 0;
  }
  if (failures.length === 0) {
    print(`Every run of ${latestSha.slice(0, 8)} on ${branch} passed; push once when every draft is complete.`);
    return 0;
  }
  mkdirSync(options.logsDir, { recursive: true });
  print(`${failures.length} failing run(s) of ${latestSha.slice(0, 8)} on ${branch}; fix all of them, then push once:`);
  for (const entry of failures) {
    const path = join(options.logsDir, logFileName(entry));
    const log = run('gh', ['run', 'view', String(entry.databaseId), ...repoArgs, '--log-failed'], {
      encoding: 'utf8',
      maxBuffer: 256 * 1024 * 1024,
    });
    write(path, log);
    print(`  ${entry.conclusion} ${entry.workflowName} ${entry.url} -> ${path}`);
  }
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = runGate(process.argv.slice(2));
}
