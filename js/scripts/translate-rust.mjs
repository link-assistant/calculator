#!/usr/bin/env node
/**
 * Generates Rust from the JavaScript engine (js/src) with
 * link-foundation/meta-language self-translation (issue #254 R2).
 *
 *   node scripts/translate-rust.mjs --write  [--meta-language DIR] [--jobs N] [--allow-regression]
 *       translate every module of js/src into js/rust-generated/, rewrite the
 *       ledger (js/rust-generated/translation.lino) and merge the allowlist
 *       (js/translation-allowlist.lino); refuse when a module's translated
 *       count falls below the ledger's, unless --allow-regression.
 *   node scripts/translate-rust.mjs --check  [--meta-language DIR] [--jobs N]
 *       translate again in memory and byte-compare with the committed files and
 *       the ledger; fail on a carried item missing from the allowlist, a stale
 *       or changed allowlist entry, or an entry without an upstream issue.
 *   node scripts/translate-rust.mjs --verify
 *       no meta-language, about a second: the ledger's source hashes match js/src,
 *       the committed output matches the ledger and its headers, and the
 *       allowlist has the entries the ledger counts. Runs on every CI run.
 *   node scripts/translate-rust.mjs --why MODULE [--meta-language DIR]
 *       print each carried item of MODULE with its full diagnostic.
 *   node scripts/translate-rust.mjs --pin
 *       print the pinned meta-language commit (for the CI checkout).
 *
 * meta-language is a checkout of link-foundation/meta-language at the ledger's
 * commit; DIR is its js/ directory with `npm ci --omit=dev --ignore-scripts`
 * done. Default: $META_LANGUAGE_DIR, else <repo>/.meta-language/js.
 */

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { availableParallelism } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Worker } from 'node:worker_threads';
import {
  compareAllowlist,
  countItems,
  criticalPath,
  mergeAllowlist,
  moduleDirectory,
  parseAllowlist,
  parseLedger,
  partName,
  planModules,
  readHeader,
  regressions,
  renderAllowlist,
  renderLedger,
  requiredEntries,
  sha256,
  splitOutput,
  uniqueNames,
} from './translate-rust-lib.mjs';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
export const JS_DIR = join(SCRIPT_DIR, '..');
export const REPO_ROOT = join(JS_DIR, '..');
export const SRC_DIR = join(JS_DIR, 'src');
export const OUT_DIR = join(JS_DIR, 'rust-generated');
export const LEDGER = join(OUT_DIR, 'translation.lino');
export const ALLOWLIST = join(JS_DIR, 'translation-allowlist.lino');

const posixRelative = (from, to) => relative(from, to).split('\\').join('/');

/** js/src modules: path relative to js/src -> text. */
export function readSources(srcDir = SRC_DIR) {
  const files = readdirSync(srcDir, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && /\.m?js$/.test(entry.name))
    .map((entry) => posixRelative(srcDir, join(entry.parentPath, entry.name)))
    .sort();
  return new Map(files.map((file) => [file, readFileSync(join(srcDir, file), 'utf8')]));
}

/** The committed output files: path relative to the output dir -> text. */
function readOutputs(outDir = OUT_DIR) {
  if (!existsSync(outDir)) return new Map();
  return new Map(readdirSync(outDir, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map((entry) => posixRelative(outDir, join(entry.parentPath, entry.name)))
    .sort()
    .map((file) => [file, readFileSync(join(outDir, file), 'utf8')]));
}

function metaLanguageDir(args) {
  const at = args.indexOf('--meta-language');
  const dir = resolve(at >= 0 ? args[at + 1] : process.env.META_LANGUAGE_DIR ?? join(REPO_ROOT, '.meta-language', 'js'));
  if (!existsSync(join(dir, 'src', 'self-translation.js'))) {
    throw new Error(`no meta-language at ${dir}: check out link-foundation/meta-language at the pin (--pin) and run npm ci --omit=dev --ignore-scripts in its js/`);
  }
  return dir;
}

/** Checks the meta-language checkout is at the pin when it is a git checkout. */
function checkPin(dir, commit) {
  let head;
  try {
    head = execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch {
    console.warn(`warning: ${dir} is not a git checkout; cannot confirm it is at the pin ${commit}`);
    return;
  }
  if (head !== commit) throw new Error(`meta-language at ${dir} is ${head}, the ledger pins ${commit}`);
}

/**
 * Translates `modules` (and the modules they import, for their signatures)
 * on a pool of worker threads, critical path first.
 * @param {Map<string, string>} sources
 * @param {{ metaLanguage: string, jobs: number, only?: string[] | null, log?: (line: string) => void }} options
 * @returns {Promise<Map<string, { code: string, items: any[], signatures: any[], ms: number }>>}
 */
export async function translateAll(sources, { metaLanguage, jobs, only = null, log = (_line) => {} }) {
  const { order, deps } = planModules(sources);
  let wanted = order;
  if (only) {
    const need = new Set();
    const add = (name) => {
      if (need.has(name)) return;
      need.add(name);
      for (const { module } of deps.get(name)) add(module);
    };
    only.forEach(add);
    wanted = order.filter((name) => need.has(name));
  }
  const rank = criticalPath(wanted, deps, (name) => sources.get(name).length);
  const results = new Map();
  const pending = new Set(wanted);
  const running = new Set();
  const size = Math.max(1, Math.min(jobs, wanted.length));
  const workers = Array.from({ length: size }, () => new Worker(join(SCRIPT_DIR, 'translate-rust-worker.mjs'), { workerData: { metaLanguage } }));
  const idle = [...workers];
  try {
    await /** @type {Promise<void>} */ (new Promise((resolveAll, reject) => {
      const ready = () => [...pending].filter((name) => deps.get(name).every(({ module }) => results.has(module)))
        .sort((a, b) => rank.get(b) - rank.get(a) || (a < b ? -1 : 1));
      const dispatch = () => {
        if (pending.size === 0 && running.size === 0) return resolveAll();
        for (const name of ready()) {
          const worker = idle.pop();
          if (!worker) break;
          pending.delete(name);
          running.add(name);
          const imports = Object.fromEntries(deps.get(name).map(({ specifier, module }) => [specifier, results.get(module).signatures]));
          worker.postMessage({ module: name, source: sources.get(name), moduleDirectory: moduleDirectory(name), imports });
        }
        if (running.size === 0 && pending.size > 0) reject(new Error(`cannot schedule: ${[...pending].join(', ')}`));
      };
      for (const worker of workers) {
        worker.on('error', reject);
        worker.on('message', (message) => {
          running.delete(message.module);
          idle.push(worker);
          if (message.error) return reject(new Error(`${message.module}: ${message.error}`));
          results.set(message.module, message);
          const c = countItems(message.items);
          log(`${message.module.padEnd(40)} ${(message.ms / 1000).toFixed(1).padStart(6)} s  translated ${c.translated} carried ${c.carried} comment ${c.comment}`);
          dispatch();
        });
      }
      dispatch();
    }));
  } finally {
    await Promise.all(workers.map((worker) => worker.terminate()));
  }
  return results;
}

/** Output files, ledger rows and allowlist entries of a translation. */
export function assemble(sources, results, commit) {
  const files = new Map();
  const modules = new Map();
  const required = [];
  for (const [name, result] of [...results].sort(([a], [b]) => (a < b ? -1 : 1))) {
    const parts = splitOutput(result.code);
    parts.forEach((part, index) => files.set(partName(name, index), part));
    const entries = requiredEntries(name, sources.get(name), result.items);
    required.push(...entries);
    modules.set(name, {
      source: sha256(sources.get(name)), output: sha256(result.code), parts: parts.length,
      ...countItems(result.items), allowlisted: entries.length,
    });
  }
  return { files, ledger: { commit, modules }, required };
}

function readLedger() {
  if (!existsSync(LEDGER)) throw new Error(`no ledger at ${posixRelative(REPO_ROOT, LEDGER)}`);
  return parseLedger(readFileSync(LEDGER, 'utf8'));
}

const readAllowlist = () => (existsSync(ALLOWLIST) ? parseAllowlist(readFileSync(ALLOWLIST, 'utf8')) : []);

function jobsOf(args) {
  const at = args.indexOf('--jobs');
  return at >= 0 ? Number(args[at + 1]) : Math.max(1, availableParallelism());
}

/** --verify: problems between js/src, the ledger, the committed output and the allowlist. */
export function verify({ sources = readSources(), ledger = readLedger(), outputs = readOutputs(), allowlist = readAllowlist() } = {}) {
  const problems = [];
  const expected = new Set();
  for (const [name, text] of sources) {
    const row = ledger.modules.get(name);
    if (!row) {
      problems.push(`${name}: not in the ledger (run --write)`);
      continue;
    }
    if (row.source !== sha256(text)) problems.push(`${name}: changed since the Rust was generated (run --write)`);
    const parts = [];
    for (let index = 0; index < row.parts; index += 1) {
      const file = partName(name, index);
      expected.add(file);
      if (!outputs.has(file)) problems.push(`${name}: missing ${file}`);
      else parts.push(outputs.get(file));
    }
    const code = parts.join('');
    if (parts.length === row.parts && sha256(code) !== row.output) problems.push(`${name}: the committed Rust differs from the ledger`);
    const header = readHeader(code);
    if (!header || header.sha256 !== row.source || header.bytes !== Buffer.byteLength(text)) problems.push(`${name}: the Rust header does not describe the source`);
    const listed = allowlist.filter((e) => e.module === name).length;
    if (listed !== row.allowlisted) problems.push(`${name}: the allowlist has ${listed} entries, the ledger counts ${row.allowlisted}`);
  }
  for (const name of ledger.modules.keys()) if (!sources.has(name)) problems.push(`${name}: in the ledger but not in js/src`);
  for (const file of outputs.keys()) if (!expected.has(file)) problems.push(`${file}: no module of the ledger writes it`);
  for (const e of allowlist) if (!sources.has(e.module)) problems.push(`allowlist names ${e.module}, not in js/src`);
  return problems;
}

/** --check: problems between a fresh translation and the committed state. */
export function compareFresh(fresh, { ledgerText, outputs, allowlist }) {
  const problems = [];
  const expected = renderLedger(fresh.ledger);
  if (ledgerText !== expected) problems.push('the ledger differs from a fresh translation');
  for (const [file, text] of fresh.files) {
    if (!outputs.has(file)) problems.push(`${file}: missing`);
    else if (outputs.get(file) !== text) problems.push(`${file}: differs from a fresh translation`);
  }
  for (const file of outputs.keys()) if (!fresh.files.has(file)) problems.push(`${file}: not produced by a fresh translation`);
  problems.push(...compareAllowlist(fresh.required, allowlist));
  return problems;
}

function writeOutputs(files) {
  rmSync(OUT_DIR, { recursive: true, force: true });
  for (const [file, text] of files) {
    const path = join(OUT_DIR, file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, text);
  }
}

function fail(title, problems) {
  console.error(`${title}:`);
  for (const p of problems) console.error(`  - ${p}`);
  process.exit(1);
}

async function main(args) {
  const started = performance.now();
  const elapsed = () => `${((performance.now() - started) / 1000).toFixed(1)} s`;
  const log = (line) => console.log(line);
  if (args.includes('--pin')) {
    console.log(readLedger().commit);
    return;
  }
  if (args.includes('--verify')) {
    const problems = verify();
    if (problems.length) fail('The Rust generated from js/src is out of date', problems);
    console.log(`translate-rust --verify: ${readLedger().modules.size} modules match the ledger (${elapsed()})`);
    return;
  }
  const whyAt = args.indexOf('--why');
  if (whyAt >= 0) {
    const name = args[whyAt + 1];
    const sources = readSources();
    if (!sources.has(name)) throw new Error(`${name} is not a module of js/src`);
    const results = await translateAll(sources, { metaLanguage: metaLanguageDir(args), jobs: jobsOf(args), only: [name] });
    const { items } = results.get(name);
    const bytes = Buffer.from(sources.get(name), 'utf8');
    const code = items.filter((item) => !/comment/.test(item.term));
    const names = uniqueNames(code.map((item) => ({ term: item.term, text: bytes.subarray(item.start, item.end).toString('utf8') })));
    code.forEach((item, index) => {
      if (item.status !== 'carried') return;
      const line = bytes.subarray(0, item.start).toString('utf8').split('\n').length;
      console.log(`${name}:${line} ${names[index]}: ${item.reason}${item.diagnostic ? `: ${item.diagnostic}` : ''}`);
    });
    return;
  }
  const write = args.includes('--write');
  if (!write && !args.includes('--check')) {
    console.error('usage: translate-rust.mjs --write | --check | --verify | --why MODULE | --pin');
    process.exit(2);
  }
  const ledgerText = existsSync(LEDGER) ? readFileSync(LEDGER, 'utf8') : null;
  const previous = ledgerText ? parseLedger(ledgerText) : null;
  const commitAt = args.indexOf('--commit');
  const commit = commitAt >= 0 ? args[commitAt + 1] : previous?.commit;
  if (!commit) throw new Error('no pin: pass --commit SHA the first time');
  const metaLanguage = metaLanguageDir(args);
  checkPin(metaLanguage, commit);
  const sources = readSources();
  const jobs = jobsOf(args);
  console.log(`translate-rust: ${sources.size} modules, meta-language ${commit.slice(0, 12)}, ${jobs} workers`);
  const results = await translateAll(sources, { metaLanguage, jobs, log });
  const fresh = assemble(sources, results, commit);
  if (write) {
    const fallen = previous ? regressions(previous.modules, fresh.ledger.modules) : [];
    if (fallen.length && !args.includes('--allow-regression')) fail('Refusing to write: the translated count fell', fallen);
    writeOutputs(fresh.files);
    writeFileSync(LEDGER, renderLedger(fresh.ledger));
    const merged = mergeAllowlist(fresh.required, readAllowlist());
    writeFileSync(ALLOWLIST, renderAllowlist(merged));
    const missing = merged.filter((e) => e.kind !== 'cascade' && !e.upstream);
    const t = renderTotals(fresh.ledger);
    console.log(`translate-rust --write: ${t}; ${fresh.files.size} files; ${merged.length} allowlist entries (${elapsed()})`);
    if (missing.length) fail('Allowlist entries need an upstream issue (edit js/translation-allowlist.lino)', missing.map((e) => `${e.kind} ${e.module} ${e.name ?? ''}`));
    return;
  }
  const problems = compareFresh(fresh, { ledgerText: ledgerText ?? '', outputs: readOutputs(), allowlist: readAllowlist() });
  if (problems.length) fail('The committed Rust is not what meta-language generates from js/src (run --write)', problems);
  console.log(`translate-rust --check: ${renderTotals(fresh.ledger)}; committed output and allowlist match (${elapsed()})`);
}

function renderTotals(ledger) {
  const rows = [...ledger.modules.values()];
  const sum = (key) => rows.reduce((total, row) => total + row[key], 0);
  return `${rows.length} modules, translated ${sum('translated')}, carried ${sum('carried')}, comment ${sum('comment')}`;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(error?.stack ?? error);
    process.exit(1);
  });
}
