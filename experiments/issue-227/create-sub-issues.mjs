#!/usr/bin/env node
// Create the #227 sub-issues from docs/case-studies/issue-227/sub-issues/*.md,
// replace the #S<n> placeholders with real numbers, link them as sub-issues of
// the parent and register "Blocked by" lines as GitHub issue dependencies.
// Idempotent: progress is stored in sub-issues/issues.json.
import { execFileSync } from 'node:child_process';
import { readFileSync, readdirSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const REPO = 'link-assistant/calculator';
const PARENT = 227;
const DIR = 'docs/case-studies/issue-227/sub-issues';
const STATE = join(DIR, 'issues.json');

function gh(args, input) {
  return execFileSync('gh', args, { encoding: 'utf8', input }).trim();
}
function api(method, path, fields) {
  const args = ['api', '-X', method, path];
  for (const [k, v] of Object.entries(fields ?? {})) args.push('-F', `${k}=${v}`);
  return JSON.parse(gh(args));
}

const files = readdirSync(DIR).filter((f) => /^\d\d-.*\.md$/.test(f)).sort();
const state = existsSync(STATE) ? JSON.parse(readFileSync(STATE, 'utf8')) : {};

function parse(file) {
  const text = readFileSync(join(DIR, file), 'utf8');
  const m = text.match(/^---\ntitle: "(.*)"\n---\n([\s\S]*)$/);
  if (!m) throw new Error(`bad frontmatter in ${file}`);
  return { title: m[1], body: m[2].trim() };
}

// Phase 1: create
for (const file of files) {
  const key = `S${Number.parseInt(file.slice(0, 2), 10)}`;
  if (state[key]) continue;
  const { title, body } = parse(file);
  const url = gh(['issue', 'create', '--repo', REPO, '--title', title, '--label', 'enhancement', '--body-file', '-'], body);
  const number = Number.parseInt(url.split('/').pop(), 10);
  const id = api('GET', `repos/${REPO}/issues/${number}`).id;
  state[key] = { file, number, url, id, title };
  writeFileSync(STATE, `${JSON.stringify(state, null, 2)}\n`);
  console.log(`created ${key} -> #${number}`);
}

// Phase 2: substitute placeholders in files and issue bodies
const sub = (text) => text.replace(/#S(\d+)\b/g, (m, n) => (state[`S${n}`] ? `#${state[`S${n}`].number}` : m));
for (const file of files) {
  const path = join(DIR, file);
  const original = readFileSync(path, 'utf8');
  const updated = sub(original);
  if (updated !== original) writeFileSync(path, updated);
  const key = `S${Number.parseInt(file.slice(0, 2), 10)}`;
  if (!state[key].bodySynced) {
    gh(['issue', 'edit', String(state[key].number), '--repo', REPO, '--body-file', '-'], parse(file).body);
    state[key].bodySynced = true;
    writeFileSync(STATE, `${JSON.stringify(state, null, 2)}\n`);
    console.log(`synced body #${state[key].number}`);
  }
}

// Phase 3: sub-issue links and blocked-by dependencies
for (const file of files) {
  const key = `S${Number.parseInt(file.slice(0, 2), 10)}`;
  const entry = state[key];
  if (!entry.linked) {
    try {
      api('POST', `repos/${REPO}/issues/${PARENT}/sub_issues`, { sub_issue_id: entry.id });
    } catch (e) {
      if (!String(e.stdout ?? e.message).includes('already')) throw e;
    }
    entry.linked = true;
    writeFileSync(STATE, `${JSON.stringify(state, null, 2)}\n`);
    console.log(`linked #${entry.number} as sub-issue of #${PARENT}`);
  }
  const body = parse(file).body;
  const blockedLine = body.match(/Blocked by:([^\n]*)/);
  const blockers = blockedLine ? [...blockedLine[1].matchAll(/#(\d+)/g)].map((m) => Number(m[1])) : [];
  entry.blockedBy = entry.blockedBy ?? [];
  for (const n of blockers) {
    if (entry.blockedBy.includes(n)) continue;
    const blocker = Object.values(state).find((s) => s.number === n);
    if (!blocker) throw new Error(`unknown blocker #${n} in ${file}`);
    try {
      api('POST', `repos/${REPO}/issues/${entry.number}/dependencies/blocked_by`, { issue_id: blocker.id });
    } catch (e) {
      if (!String(e.stdout ?? e.message).includes('already')) throw e;
    }
    entry.blockedBy.push(n);
    writeFileSync(STATE, `${JSON.stringify(state, null, 2)}\n`);
    console.log(`#${entry.number} blocked by #${n}`);
  }
}
console.log('done');
