#!/usr/bin/env node

/**
 * Competitor capability coverage checker (issue #227).
 *
 * Runs every expression of one or more corpus files through the CLI binary
 * and classifies each case as `supported`, `different`, `unsupported` or
 * `timeout`. The report answers "what do we have, what are we missing" with
 * exact expressions instead of feature claims.
 *
 * Corpus format (tab-separated, UTF-8, `#` starts a comment line):
 *
 *   source <TAB> lang <TAB> expression <TAB> expected <TAB> note
 *
 *   - `source`     product or document the expression comes from
 *   - `lang`       BCP-47 language of the expression (`en`, `ru`, `zh`, ...)
 *   - `expression` verbatim input; `\t` and `\n` escapes are decoded
 *   - `expected`   expected display result; empty means "must evaluate"
 *   - `note`       free text (citation, semantic caveat, link)
 *
 * Usage:
 *   node scripts/competitor-coverage.mjs [options]
 *     --binary <path>      calculator binary (default: target/release/link-calculator,
 *                          falling back to target/debug/link-calculator)
 *     --corpus <path>      corpus file or directory; repeatable
 *                          (default: docs/case-studies/issue-227/corpus)
 *     --out <dir>          report directory (default: docs/case-studies/issue-227/results)
 *     --timeout <ms>       per-expression timeout (default: 5000)
 *     --source <name>      only run cases whose source contains <name>
 *     --baseline <json>    previous coverage-baseline.json; exit 1 when a case
 *                          disappears or its status gets worse (in order:
 *                          supported, different, unsupported, timeout)
 *     --quiet              do not print the per-source table
 */

import { spawnSync } from 'child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { join, resolve } from 'path';
import { fileURLToPath } from 'url';

const STATUSES = ['supported', 'different', 'unsupported', 'timeout'];

function parseArgs(argv) {
  const options = {
    binary: null,
    corpus: [],
    out: 'docs/case-studies/issue-227/results',
    timeout: 5000,
    source: null,
    baseline: null,
    quiet: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => {
      i += 1;
      if (i >= argv.length) {
        throw new Error(`Missing value for ${arg}`);
      }
      return argv[i];
    };
    switch (arg) {
      case '--binary':
        options.binary = next();
        break;
      case '--corpus':
        options.corpus.push(next());
        break;
      case '--out':
        options.out = next();
        break;
      case '--timeout':
        options.timeout = Number.parseInt(next(), 10);
        break;
      case '--source':
        options.source = next();
        break;
      case '--baseline':
        options.baseline = next();
        break;
      case '--quiet':
        options.quiet = true;
        break;
      case '--help':
      case '-h':
        console.log(readFileSync(new URL(import.meta.url)).toString().split('*/')[0]);
        process.exit(0);
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (options.corpus.length === 0) {
    options.corpus.push('docs/case-studies/issue-227/corpus');
  }
  return options;
}

function findBinary(explicit) {
  const candidates = explicit
    ? [explicit]
    : ['target/release/link-calculator', 'target/debug/link-calculator'];
  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return resolve(candidate);
    }
  }
  throw new Error(
    `Calculator binary not found (tried: ${candidates.join(', ')}). ` +
      'Build it with `cargo build --release --bin link-calculator`.'
  );
}

function decodeEscapes(text) {
  return text.replace(/\\(t|n|\\)/g, (_, c) => (c === 't' ? '\t' : c === 'n' ? '\n' : '\\'));
}

/**
 * Collect corpus files from file or directory paths.
 * @param {string[]} paths
 * @returns {string[]}
 */
function collectCorpusFiles(paths) {
  const files = [];
  for (const path of paths) {
    if (!existsSync(path)) {
      throw new Error(`Corpus path does not exist: ${path}`);
    }
    if (statSync(path).isDirectory()) {
      for (const entry of readdirSync(path).sort()) {
        if (entry.endsWith('.tsv')) {
          files.push(join(path, entry));
        }
      }
    } else {
      files.push(path);
    }
  }
  return files;
}

/**
 * Parse one corpus file into case objects.
 * @param {string} file
 */
function parseCorpus(file) {
  const cases = [];
  const lines = readFileSync(file, 'utf8').split('\n');
  lines.forEach((rawLine, index) => {
    const line = rawLine.replace(/\r$/, '');
    if (line.trim() === '' || line.startsWith('#')) {
      return;
    }
    const columns = line.split('\t');
    if (columns.length < 3) {
      throw new Error(`${file}:${index + 1}: expected at least 3 tab-separated columns`);
    }
    const [source, lang, expression, expected = '', note = ''] = columns;
    cases.push({
      file,
      line: index + 1,
      source: source.trim(),
      lang: lang.trim(),
      expression: decodeEscapes(expression),
      expected: decodeEscapes(expected).trim(),
      note: note.trim(),
    });
  });
  return cases;
}

const NUMBER = /^[-+]?\d+(?:\.\d+)?(?:e[-+]?\d+)?$/i;

/** Normalize a display string for loose textual comparison. */
function normalizeText(text) {
  return text
    .trim()
    .replace(/^approx\.\s*/i, '')
    .replace(/[   ]/g, ' ')
    .replace(/\s+/g, ' ')
    .replace(/,(?=\d{3}\b)/g, '')
    .replace(/(\.\d*?[1-9])0+\b/g, '$1')
    .replace(/\.0+\b/g, '')
    .toLowerCase();
}

/** Split a display value into leading number and trailing text (unit). */
function splitNumber(text) {
  const match = text.match(/^([-+]?\d+(?:\.\d+)?(?:e[-+]?\d+)?)(.*)$/i);
  if (!match) {
    return null;
  }
  return { number: Number.parseFloat(match[1]), literal: match[1], rest: match[2].trim() };
}

/**
 * Compare actual and expected display strings.
 * Numbers match when equal after rounding to the expected precision or
 * within a 1e-9 relative tolerance; trailing unit text must match loosely.
 */
function resultsMatch(actual, expected) {
  if (expected === '') {
    return true;
  }
  const a = normalizeText(actual);
  const e = normalizeText(expected);
  if (a === e) {
    return true;
  }
  const an = splitNumber(a);
  const en = splitNumber(e);
  if (!an || !en || an.rest !== en.rest) {
    return false;
  }
  if (!NUMBER.test(an.literal) || !NUMBER.test(en.literal)) {
    return false;
  }
  const decimals = (en.literal.split('.')[1] || '').length;
  const rounded = Number(an.number.toFixed(decimals));
  if (rounded === en.number) {
    return true;
  }
  const scale = Math.max(1, Math.abs(en.number));
  return Math.abs(an.number - en.number) / scale < 1e-9;
}

/**
 * Evaluate one expression with the CLI binary.
 * @returns {{status: string, actual: string, error: string}}
 */
function evaluate(binary, expression, timeout) {
  const run = spawnSync(binary, [expression], {
    encoding: 'utf8',
    timeout,
    maxBuffer: 1024 * 1024,
    env: { ...process.env, NO_COLOR: '1' },
  });
  if (run.error && run.error.code === 'ETIMEDOUT') {
    return { status: 'timeout', actual: '', error: `timeout after ${timeout} ms` };
  }
  const stdout = run.stdout || '';
  const resultLine = stdout.split('\n').find((line) => line.startsWith('Result: '));
  if (run.status === 0 && resultLine) {
    return { status: 'ok', actual: resultLine.slice('Result: '.length), error: '' };
  }
  const errorLine = stdout.split('\n').find((line) => line.startsWith('Error: '));
  const error = errorLine ? errorLine.slice('Error: '.length) : (run.stderr || '').trim();
  return { status: 'unsupported', actual: '', error: error || `exit code ${run.status}` };
}

function classify(binary, testCase, timeout) {
  const outcome = evaluate(binary, testCase.expression, timeout);
  let status = outcome.status;
  if (status === 'ok') {
    status = resultsMatch(outcome.actual, testCase.expected) ? 'supported' : 'different';
  }
  return { ...testCase, status, actual: outcome.actual, error: outcome.error };
}

function summarize(results) {
  const bySource = new Map();
  for (const result of results) {
    if (!bySource.has(result.source)) {
      bySource.set(result.source, { total: 0, supported: 0, different: 0, unsupported: 0, timeout: 0 });
    }
    const bucket = bySource.get(result.source);
    bucket.total += 1;
    bucket[result.status] += 1;
  }
  const total = { total: 0, supported: 0, different: 0, unsupported: 0, timeout: 0 };
  for (const bucket of bySource.values()) {
    for (const key of Object.keys(total)) {
      total[key] += bucket[key];
    }
  }
  return { bySource, total };
}

function percent(part, whole) {
  return whole === 0 ? '0.0%' : `${((100 * part) / whole).toFixed(1)}%`;
}

function escapeCell(text) {
  return String(text).replace(/\|/g, '\\|').replace(/\n/g, '\\n');
}

function renderTable(bySource, total) {
  const lines = [
    '| Source | Cases | Supported | Different | Unsupported | Timeout | Coverage |',
    '| --- | ---: | ---: | ---: | ---: | ---: | ---: |',
  ];
  const sources = [...bySource.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  for (const [source, b] of sources) {
    lines.push(
      `| ${escapeCell(source)} | ${b.total} | ${b.supported} | ${b.different} | ${b.unsupported} | ${b.timeout} | ${percent(b.supported, b.total)} |`
    );
  }
  lines.push(
    `| **All** | ${total.total} | ${total.supported} | ${total.different} | ${total.unsupported} | ${total.timeout} | ${percent(total.supported, total.total)} |`
  );
  return lines.join('\n');
}

function renderReport(results, summary, options, binary) {
  const sections = [];
  sections.push('# Competitor coverage report');
  sections.push('');
  sections.push(
    'Generated by `scripts/competitor-coverage.mjs`. Each row is one verbatim expression from a corpus file; ' +
      '`supported` means the calculator evaluated it and the result matched the expected display value, ' +
      '`different` means it evaluated to another value (formatting or semantic conflict), ' +
      '`unsupported` means it was rejected.'
  );
  sections.push('');
  sections.push(`- Binary: \`${binary}\``);
  sections.push(`- Corpus: ${options.corpus.map((c) => `\`${c}\``).join(', ')}`);
  sections.push(`- Timeout per expression: ${options.timeout} ms`);
  sections.push('');
  sections.push('## Coverage by source');
  sections.push('');
  sections.push(renderTable(summary.bySource, summary.total));
  sections.push('');

  const byLang = new Map();
  for (const result of results) {
    if (!byLang.has(result.lang)) {
      byLang.set(result.lang, { total: 0, supported: 0, different: 0, unsupported: 0, timeout: 0 });
    }
    const bucket = byLang.get(result.lang);
    bucket.total += 1;
    bucket[result.status] += 1;
  }
  if (byLang.size > 1) {
    sections.push('## Coverage by language');
    sections.push('');
    sections.push(renderTable(byLang, summary.total));
    sections.push('');
  }

  for (const status of ['different', 'unsupported', 'timeout']) {
    const rows = results.filter((r) => r.status === status);
    if (rows.length === 0) {
      continue;
    }
    sections.push(`## ${status[0].toUpperCase()}${status.slice(1)} (${rows.length})`);
    sections.push('');
    sections.push('| Source | Lang | Expression | Expected | Actual / error | Note |');
    sections.push('| --- | --- | --- | --- | --- | --- |');
    for (const r of rows) {
      const actual = status === 'different' ? r.actual : r.error;
      sections.push(
        `| ${escapeCell(r.source)} | ${r.lang} | \`${escapeCell(r.expression)}\` | ${escapeCell(r.expected)} | ${escapeCell(actual)} | ${escapeCell(r.note)} |`
      );
    }
    sections.push('');
  }
  return `${sections.join('\n')}\n`;
}

function compareWithBaseline(results, baseline) {
  const key = (row) => JSON.stringify([row.source, row.lang, row.expression]);
  const previous = new Map();
  for (const row of baseline.results) {
    const rank = STATUSES.indexOf(row.status);
    if (rank === -1) {
      throw new Error(`Unknown baseline status: ${row.status}`);
    }
    const existing = previous.get(key(row));
    if (!existing || rank < STATUSES.indexOf(existing.status)) {
      previous.set(key(row), row);
    }
  }
  const regressions = results.filter((row) => {
    const old = previous.get(key(row));
    return old && STATUSES.indexOf(row.status) > STATUSES.indexOf(old.status);
  });
  const current = new Set(results.map(key));
  for (const [id, row] of previous) {
    if (!current.has(id)) {
      regressions.push({ ...row, status: 'missing', error: 'expression missing from corpus' });
    }
  }
  return regressions;
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  // Read before writing reports: --baseline may be in the output directory.
  const baseline = options.baseline ? JSON.parse(readFileSync(options.baseline, 'utf8')) : null;
  const binary = findBinary(options.binary);
  const files = collectCorpusFiles(options.corpus);
  let cases = files.flatMap(parseCorpus);
  if (options.source) {
    const matchesSource = (row) => row.source.toLowerCase().includes(options.source.toLowerCase());
    cases = cases.filter(matchesSource);
    if (baseline) {
      baseline.results = baseline.results.filter(matchesSource);
    }
  }
  if (cases.length === 0) {
    throw new Error('No corpus cases found.');
  }

  const started = Date.now();
  const results = cases.map((c) => classify(binary, c, options.timeout));
  const summary = summarize(results);
  const regressions = baseline ? compareWithBaseline(results, baseline) : [];

  mkdirSync(options.out, { recursive: true });
  writeFileSync(join(options.out, 'coverage-report.md'), renderReport(results, summary, options, binary));
  writeFileSync(
    join(options.out, 'coverage-results.json'),
    `${JSON.stringify(
      {
        binary,
        corpus: options.corpus,
        timeout: options.timeout,
        total: summary.total,
        bySource: Object.fromEntries(summary.bySource),
        results: results.map(({ file, line, ...rest }) => ({ ...rest, file, line })),
      },
      null,
      2
    )}\n`
  );

  // One case per line keeps the committed baseline small and diff-friendly.
  const baselineRows = results.map(({ source, lang, expression, status }) =>
    JSON.stringify({ source, lang, expression, status })
  );
  writeFileSync(
    join(options.out, 'coverage-baseline.json'),
    `{\n"generated": ${JSON.stringify(new Date().toISOString().slice(0, 10))},\n"total": ${JSON.stringify(summary.total)},\n"results": [\n${baselineRows.join(',\n')}\n]\n}\n`
  );

  if (!options.quiet) {
    console.log(renderTable(summary.bySource, summary.total));
    console.log(`\n${results.length} cases in ${((Date.now() - started) / 1000).toFixed(1)}s; report: ${join(options.out, 'coverage-report.md')}`);
  }

  if (options.baseline) {
    if (regressions.length > 0) {
      console.error(`\n${regressions.length} regression(s) against ${options.baseline}:`);
      for (const r of regressions) {
        console.error(`  [${r.source}] ${r.expression} -> ${r.status}: ${r.actual || r.error}`);
      }
      process.exit(1);
    }
    console.log(`No regressions against ${options.baseline}.`);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`competitor-coverage: ${error.message}`);
    process.exit(2);
  }
}

export { resultsMatch, normalizeText, parseCorpus, decodeEscapes, STATUSES, compareWithBaseline };
