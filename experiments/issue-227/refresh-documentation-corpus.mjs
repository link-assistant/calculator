#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { parseCorpus } from '../../scripts/competitor-coverage.mjs';
import { sources } from './sources.mjs';
import { extract, fetchPage, links, unstable, escapeTsv, sha256, auditImage } from './documentation.mjs';

const imageExamples = JSON.parse(readFileSync(new URL('./image-examples.json', import.meta.url), 'utf8'));

export function collectSource(source, options) {
  const queue = [...(source.pages ?? [])];
  const indexes = [];
  if (source.index) {
    const index = fetchPage(source.index, options);
    indexes.push({ url: source.index, fetched: index.fetched, sha256: sha256(index.text) });
    queue.push(...links(index.text, source.index).filter((url) => new RegExp(source.include).test(url)));
    if (!queue.length) throw new Error(`No reference pages discovered for ${source.product}`);
  }
  const seen = new Set();
  const rows = [];
  const pages = [];
  while (queue.length) {
    const item = queue.shift();
    const page = typeof item === 'string' ? { url: item } : item;
    if (seen.has(page.url)) continue;
    seen.add(page.url);
    if (seen.size > 400) throw new Error(`Reference crawl exceeded 400 pages: ${source.product}`);
    const fetched = fetchPage(page.fetchUrl ?? page.url, { ...options, post: page.post });
    const examples = extract(source.kind, fetched.text);
    const unique = new Map();
    for (const example of examples) {
      const context = `${page.url} ${example.context ?? ''}`;
      const reason = unstable(example.expression, example.expected, context);
      const row = { source: source.product, lang: 'en', expression: example.expression,
        expected: reason ? '' : example.expected, note: `${page.url}${reason ? ` (${reason})` : ''}` };
      unique.set(row.expression, row);
    }
    rows.push(...unique.values());
    pages.push({ url: page.url, fetchUrl: page.fetchUrl, fetched: fetched.fetched,
      sha256: sha256(fetched.text), rows: unique.size });
    console.error(`${source.product}: ${page.url}: ${unique.size} examples`);
    if (source.crawl) queue.push(...links(fetched.text, page.url).filter((url) => new RegExp(source.crawl).test(url)));
    if (source.kind === 'parsify' && JSON.parse(fetched.text).cursor?.stack?.length) {
      throw new Error('Notion pagination changed; fetch remaining chunks before publishing');
    }
  }
  const images = [];
  for (const reference of imageExamples[source.id] ?? []) {
    images.push(auditImage(reference, options));
    for (const [expression, expected] of reference.examples) {
      rows.push({ source: source.product, lang: 'en', expression, expected,
        note: `${reference.page} (image transcription: ${reference.url}; preceding definitions required)` });
    }
  }
  if (!rows.length && !source.zeroExamples) throw new Error(`No examples extracted for ${source.product}`);
  return { rows, audit: { id: source.id, product: source.product, extractor: `experiments/issue-227/extract-${source.id}-examples.mjs`,
    indexes, pages, images, extractedRows: rows.length, zeroExamples: source.zeroExamples } };
}

export function run(argv, onlySource) {
  const options = { cache: 'ci-logs/documentation-cache', offline: false };
  let output = 'docs/case-studies/issue-227/corpus/closed-source-docs.tsv';
  let auditPath = 'docs/case-studies/issue-227/documentation-sources.json';
  let selected = onlySource;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--offline') options.offline = true;
    else if (arg === '--cache') options.cache = argv[++i];
    else if (arg === '--out') output = argv[++i];
    else if (arg === '--audit') auditPath = argv[++i];
    else if (arg === '--source') selected = argv[++i];
    else throw new Error(`Unknown option: ${arg}`);
  }
  const chosen = selected ? sources.filter((source) => source.id === selected) : sources;
  if (!chosen.length) throw new Error(`Unknown source: ${selected}`);
  const results = chosen.map((source) => collectSource(source, options));
  if (onlySource) {
    const result = results[0];
    process.stdout.write(`# ${chosen[0].product}; fetched ${result.audit.pages.map((page) => page.fetched).sort().at(-1)}\n`);
    for (const row of result.rows) process.stdout.write(serialize(row) + '\n');
    return;
  }
  // Preserve historical identifiers required by the existing regression gate.
  const legacy = parseCorpus(new URL('./legacy-examples.tsv', import.meta.url));
  const byExpression = new Map(legacy.map((row) => [key(row), row]));
  let audits = [];
  if (selected) {
    audits = existsSync(auditPath) ? JSON.parse(readFileSync(auditPath, 'utf8')).sources.filter((source) => source.id !== selected) : [];
    if (existsSync(output)) for (const row of parseCorpus(output)) if (row.source !== chosen[0].product) byExpression.set(key(row), row);
  }
  for (const result of results) {
    for (const row of result.rows) {
      const old = byExpression.get(key(row));
      // Keep a historical deterministic oracle if current docs omit the answer.
      const reason = unstable(row.expression, old?.expected ?? '', row.note);
      byExpression.set(key(row), { ...row, expected: reason ? '' : row.expected || old?.expected || '' });
    }
    audits.push(result.audit);
  }
  const rows = [...byExpression.values()].sort((a, b) => a.source.localeCompare(b.source) || a.expression.localeCompare(b.expression));
  for (const audit of audits) audit.rows = rows.filter((row) => row.source === audit.product).length;
  audits.sort((a, b) => a.id.localeCompare(b.id));
  const header = ['# Closed-source and documentation-only calculator examples (issues #227, #231).',
    '# Columns: source, lang, expression, expected, note. Empty expected = must evaluate.',
    '# Generated by experiments/issue-227/refresh-documentation-corpus.mjs; no documentation prose copied.',
    '# Historical seed examples are preserved and explicitly labeled; see documentation-corpus.md.',
    ...audits.map((audit) => `# ${audit.product}: fetched ${[...new Set(audit.pages.map((page) => page.fetched))].join(', ')}; ${audit.extractedRows} extracted page examples; ${audit.rows} unique corpus rows.`),
  ];
  writeFileSync(output, [...header, ...rows.map(serialize), ''].join('\n'));
  writeFileSync(auditPath, JSON.stringify({ schemaVersion: 1, sources: audits }, null, 2) + '\n');
  console.error(`Wrote ${rows.length} rows and ${audits.length} source audits.`);
}
const key = (row) => JSON.stringify([row.source, row.lang, row.expression]);
const serialize = (row) => [row.source, row.lang, row.expression, row.expected, row.note].map(escapeTsv).join('\t');
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { run(process.argv.slice(2)); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
