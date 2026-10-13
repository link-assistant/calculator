#!/usr/bin/env node

/**
 * Enforce the JavaScript-first CI split (issue #254).
 *
 * Rust tooling may only start from rust.yml, and rust.yml may only be called
 * by the rust-stage job of release.yml once every JavaScript job of that run
 * passed. The script fails when:
 *
 * - a job outside the Rust stage uses Rust tooling (cargo, rustc, rustup,
 *   rustfmt, clippy, wasm-pack, rust-toolchain, CodeQL for rust), directly or
 *   through a local composite action;
 * - rust.yml, or a workflow that uses Rust tooling, has a trigger other than
 *   workflow_call, or is called from anywhere but the Rust stage;
 * - rust-stage does not need every other job of release.yml, does not
 *   require the JavaScript Stage gate to succeed, or may cancel a running
 *   Rust stage;
 * - a JavaScript job lacks a literal `cancel-in-progress: true`, or a
 *   workflow-level concurrency group could cancel the Rust stage.
 *
 * The workflows are split into jobs by indentation, like
 * scripts/read-job-cancel-in-progress.sh, so no YAML package is needed.
 *
 * Usage: node scripts/check-javascript-first.mjs [--verbose]
 */

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ENTRY = 'release.yml';
export const RUST = 'rust.yml';
export const RUST_STAGE_JOB = 'rust-stage';
export const JS_GATE_JOB = 'javascript-stage';

const RUST_MARKERS = [
  // `--skip-cargo` is the flag that keeps a script away from Cargo.
  /(?<!--skip-)\b(?:cargo|rustc|rustup|rustfmt|clippy|wasm-pack|wasm-bindgen)\b/,
  /rust-toolchain/,
  /\blanguages?:.*\brust\b/,
];

const indentOf = (line) => line.length - line.trimStart().length;

/** Drop comment lines and trailing ` # comments`. */
export function stripComments(text) {
  return text
    .split('\n')
    .filter((line) => !line.trimStart().startsWith('#'))
    .map((line) => line.replace(/\s+#(?![^'"]*['"]\s*$).*$/, ''))
    .join('\n');
}

/** The indented block under the `key:` line at `start`. */
function blockAfter(lines, start) {
  const indent = indentOf(lines[start]);
  const block = [];
  for (const line of lines.slice(start + 1)) {
    if (line.trim() !== '' && indentOf(line) <= indent) {
      break;
    }
    block.push(line);
  }
  return block;
}

/** `[a, b]`, `a`, or a `- a` list under the key. */
function listValue(lines, start) {
  const inline = lines[start].split(':').slice(1).join(':').trim();
  if (inline.startsWith('[')) {
    return inline
      .slice(1, inline.lastIndexOf(']'))
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean);
  }
  if (inline !== '') {
    return [inline];
  }
  return blockAfter(lines, start)
    .map((line) => line.trim().match(/^-\s*(\S+)/))
    .filter(Boolean)
    .map((match) => match[1]);
}

function scalarAfter(line) {
  return line.split(':').slice(1).join(':').trim().replace(/^['"]|['"]$/g, '');
}

/**
 * Split a workflow into its triggers, workflow-level concurrency and jobs.
 * @param {string} source
 */
export function parseWorkflow(source) {
  const lines = stripComments(source).split('\n');
  const workflow = { triggers: [], concurrency: false, jobs: new Map() };
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (/^(?:on|"on"|'on'):/.test(line)) {
      const inline = scalarAfter(line);
      if (inline.startsWith('[')) {
        workflow.triggers = listValue(lines, index);
      } else if (inline !== '') {
        workflow.triggers = [inline];
      } else {
        workflow.triggers = blockAfter(lines, index)
          .filter((child) => indentOf(child) === 2 && /^\s+[\w-]+:/.test(child))
          .map((child) => child.trim().split(':')[0]);
      }
    } else if (/^concurrency:/.test(line)) {
      workflow.concurrency = true;
    } else if (/^jobs:\s*$/.test(line)) {
      for (const [offset, child] of blockAfter(lines, index).entries()) {
        const declared = child.match(/^ {2}([\w.-]+):\s*$/);
        if (declared) {
          workflow.jobs.set(declared[1], parseJob(lines, index + 1 + offset));
        }
      }
    }
  }
  return workflow;
}

function parseJob(lines, start) {
  const body = blockAfter(lines, start);
  const job = { text: body.join('\n'), needs: [], uses: null, if: '', cancelInProgress: 'none' };
  for (const [offset, line] of body.entries()) {
    const absolute = start + 1 + offset;
    if (indentOf(line) !== 4) {
      continue;
    }
    const key = line.trim().split(':')[0];
    if (key === 'needs') {
      job.needs = listValue(lines, absolute);
    } else if (key === 'uses') {
      job.uses = scalarAfter(line);
    } else if (key === 'if') {
      job.if = [scalarAfter(line), ...blockAfter(lines, absolute)].join(' ').replace(/\s+/g, ' ').trim();
    } else if (key === 'concurrency') {
      const cancel = blockAfter(lines, absolute)
        .map((child) => child.match(/^\s+cancel-in-progress:\s*(.*)$/))
        .find(Boolean);
      const raw = cancel ? cancel[1].trim().replace(/^['"]|['"]$/g, '') : 'false';
      job.cancelInProgress = raw.includes('${{') ? 'expression' : raw;
    }
  }
  return job;
}

/** The job text with every `uses: ./local-action` replaced by its action.yml. */
export function expandLocalActions(text, readAction, seen = new Set()) {
  return text.replace(/uses:\s*(\.\/[^\s'"]+)/g, (match, path) => {
    const action = readAction(path);
    if (action === null || seen.has(path)) {
      return match;
    }
    return `${match}\n${expandLocalActions(stripComments(action), readAction, new Set([...seen, path]))}`;
  });
}

export function usesRust(text) {
  return RUST_MARKERS.some((marker) => marker.test(text));
}

/**
 * @param {Map<string, string>} workflows file name -> source
 * @param {(path: string) => string | null} readAction local action path -> action.yml source
 * @returns {string[]} problems
 */
export function checkJavaScriptFirst(workflows, readAction) {
  const problems = [];
  const parsed = new Map([...workflows].map(([name, source]) => [name, parseWorkflow(source)]));
  const calledWorkflow = (job) => job.uses?.match(/^\.\/\.github\/workflows\/([^@]+)$/)?.[1] ?? null;

  const callers = new Map();
  for (const [file, workflow] of parsed) {
    for (const [name, job] of workflow.jobs) {
      const callee = calledWorkflow(job);
      if (callee) {
        callers.set(callee, [...(callers.get(callee) ?? []), { file, name }]);
      }
    }
  }

  if (!parsed.has(RUST) || !parsed.has(ENTRY)) {
    return [`.github/workflows/${ENTRY} and .github/workflows/${RUST} must both exist`];
  }

  // Workflows that may run Rust: rust.yml and what it (transitively) calls.
  const rustContext = new Set([RUST]);
  for (let grew = true; grew; ) {
    grew = false;
    for (const file of rustContext) {
      for (const job of parsed.get(file)?.jobs.values() ?? []) {
        const callee = calledWorkflow(job);
        if (callee && !rustContext.has(callee)) {
          rustContext.add(callee);
          grew = true;
        }
      }
    }
  }

  for (const file of rustContext) {
    const workflow = parsed.get(file);
    if (!workflow) {
      problems.push(`${file} is called from the Rust stage but does not exist`);
      continue;
    }
    if (workflow.triggers.join() !== 'workflow_call') {
      problems.push(
        `${file} runs in the Rust stage, so its only trigger must be workflow_call (found: ${workflow.triggers.join(', ') || 'none'})`,
      );
    }
    for (const caller of callers.get(file) ?? []) {
      const allowed =
        file === RUST
          ? caller.file === ENTRY && caller.name === RUST_STAGE_JOB
          : rustContext.has(caller.file) && caller.file !== ENTRY;
      if (!allowed) {
        problems.push(`${caller.file} job ${caller.name} calls ${file}; only the Rust stage may`);
      }
    }
    if (workflow.concurrency) {
      problems.push(`${file} must not set workflow-level concurrency; it could cancel a running Rust stage`);
    }
    for (const [name, job] of workflow.jobs) {
      if (job.cancelInProgress !== 'none' && job.cancelInProgress !== 'false') {
        problems.push(`${file} job ${name} must not use cancel-in-progress (found ${job.cancelInProgress})`);
      }
    }
  }
  if (!(callers.get(RUST) ?? []).some((caller) => caller.file === ENTRY && caller.name === RUST_STAGE_JOB)) {
    problems.push(`${ENTRY} job ${RUST_STAGE_JOB} must call ./.github/workflows/${RUST}`);
  }

  for (const [file, workflow] of parsed) {
    if (rustContext.has(file)) {
      continue;
    }
    for (const [name, job] of workflow.jobs) {
      if (usesRust(expandLocalActions(job.text, readAction))) {
        problems.push(`${file} job ${name} uses Rust tooling outside the Rust stage; move it into ${RUST}`);
      }
    }
  }

  const entry = parsed.get(ENTRY);
  if (entry.concurrency) {
    problems.push(`${ENTRY} must not set workflow-level concurrency; set it per job instead`);
  }
  const stage = entry.jobs.get(RUST_STAGE_JOB);
  if (stage) {
    for (const name of entry.jobs.keys()) {
      if (name !== RUST_STAGE_JOB && !stage.needs.includes(name)) {
        problems.push(`${ENTRY} job ${RUST_STAGE_JOB} must need ${name}: every JavaScript job gates the Rust stage`);
      }
    }
    if (!/needs\.javascript-stage\.result == 'success'/.test(stage.if)) {
      problems.push(`${ENTRY} job ${RUST_STAGE_JOB} must run only if needs.${JS_GATE_JOB}.result == 'success'`);
    }
    if (stage.cancelInProgress !== 'false') {
      problems.push(
        `${ENTRY} job ${RUST_STAGE_JOB} needs a per-branch concurrency group with cancel-in-progress: false (found ${stage.cancelInProgress})`,
      );
    }
  }
  if (!entry.jobs.has(JS_GATE_JOB)) {
    problems.push(`${ENTRY} must have the ${JS_GATE_JOB} gate job`);
  }
  for (const [name, job] of entry.jobs) {
    if (name !== RUST_STAGE_JOB && name !== JS_GATE_JOB && job.cancelInProgress !== 'true') {
      problems.push(
        `${ENTRY} job ${name} needs a job-level concurrency group with a literal cancel-in-progress: true (found ${job.cancelInProgress})`,
      );
    }
  }
  return problems;
}

function main() {
  const verbose = process.argv.includes('--verbose');
  const root = fileURLToPath(new URL('..', import.meta.url));
  const dir = join(root, '.github', 'workflows');
  const workflows = new Map(
    readdirSync(dir)
      .filter((name) => /\.ya?ml$/.test(name))
      .sort()
      .map((name) => [name, readFileSync(join(dir, name), 'utf8')]),
  );
  const readAction = (path) => {
    for (const name of ['action.yml', 'action.yaml']) {
      const file = join(root, path, name);
      if (existsSync(file)) {
        return readFileSync(file, 'utf8');
      }
    }
    return null;
  };
  if (verbose) {
    for (const [name, source] of workflows) {
      const workflow = parseWorkflow(source);
      console.log(`${name}: on [${workflow.triggers.join(', ')}], jobs [${[...workflow.jobs.keys()].join(', ')}]`);
    }
  }
  const problems = checkJavaScriptFirst(workflows, readAction);
  for (const problem of problems) {
    console.error(`::error title=JavaScript first (issue #254)::${problem}`);
  }
  if (problems.length > 0) {
    process.exit(1);
  }
  console.log(`JavaScript first: Rust runs only in ${RUST}, behind every JavaScript job of ${ENTRY}.`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
