import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { extract, unstable, pair, plain, escapeTsv, fetchPage, sha256, auditImage } from '../experiments/issue-227/documentation.mjs';
import { collectSource } from '../experiments/issue-227/refresh-documentation-corpus.mjs';
import { parseCorpus } from './competitor-coverage.mjs';

const expressions = (kind, text) => extract(kind, text).map((row) => row.expression);

test('Soulver imports code, variables, long expressions, HTML tables and inline answer pairs', () => {
  const input = '```\na = 2 | 2\nif a > 1 then 10 else 20 | 10\n```\n' +
    '<table><tr><th>Statistic</th><th>Example</th><th>Answer</th></tr>' +
    '<tr><td>mean</td><td>average of 2 and 4</td><td>3</td></tr></table>\n' +
    '`20k after 6 months at 10% per month` → **35,431**\n' +
    'sind(90) = 1.0\n';
  const rows = extract('soulver', input);
  for (const input of ['a = 2', 'if a > 1 then 10 else 20', 'average of 2 and 4',
    '20k after 6 months at 10% per month', 'sind(90)']) assert.ok(rows.some((row) => row.expression === input), input);
  assert.equal(rows.find((row) => row.expression === 'average of 2 and 4').expected, '3');
  assert.equal(plain('2 < 4 and 5 > 3'), '2 < 4 and 5 > 3');
  assert.deepEqual(pair('5 | 3 | 7'), { expression: '5 | 3', expected: '7' });
});

test('Numi includes every example column and indentation example, without copying operation descriptions', () => {
  const input = '    8 times 9\n    6 (3) = 18\n' +
    '<table><tr><th>Description</th><th>Function</th><th>Example</th></tr>' +
    '<tr><td>Natural logarithm</td><td>ln</td><td>ln 3</td></tr></table>';
  const rows = extract('numi', input);
  assert.equal(rows.find((row) => row.expression === '6 (3)').expected, '18');
  assert.ok(rows.some((row) => row.expression === 'ln 3'));
  assert.ok(!rows.some((row) => row.expression === 'Natural logarithm'));
});

test('Numi preserves variable assignments and excludes JavaScript extension APIs', () => {
  const rows = extract('numi', '    v = $20\n    1 inch in px\u00a0=\u00a096 px\n`js numi.setVariable("yyy", 122);`');
  assert.ok(rows.some((row) => row.expression === 'v = $20' && !row.expected));
  assert.ok(rows.some((row) => row.expression === '1 inch in px' && row.expected === '96 px'));
  assert.ok(!rows.some((row) => row.expression.startsWith('js ')));
});

test('SpeedCrunch keeps definitions, final answers and no answer-side explanatory comments', () => {
  const rows = extract('speedcrunch', '<pre>(2+3)*4 = 5*4 = 20\na = 5.123 ? calibration value\n[cm_s] = [cm/s] ? centimeters per second</pre>');
  assert.equal(rows[0].expected, '20');
  assert.equal(rows[1].expression, 'a = 5.123 ? calibration value');
  assert.equal(rows[2].expected, '[cm/s]');
});

test('Calca and SpeedCrunch decode markup while preserving syntax and answers', () => {
  assert.deepEqual(extract('calca', '<pre><code>3 * (1 + 10%) =&gt; 3.3\nx = 2</code></pre>')
    .map(({ expression, expected }) => [expression, expected]), [['3 * (1 + 10%)', '3.3'], ['x = 2', '']]);
  assert.deepEqual(extract('speedcrunch', '<pre>0b1.01\n= 1.25\nhex(12341)\n= 0x3035</pre>')
    .map(({ expression, expected }) => [expression, expected]), [['0b1.01', '1.25'], ['hex(12341)', '0x3035']]);
});

test('SpeedCrunch angle display settings do not become fixed numeric expectations', () => {
  const cache = mkdtempSync(join(tmpdir(), 'speedcrunch-angle-mode-'));
  const url = 'https://example.com/syntax.html';
  try {
    const text = "<pre>pi\n= 180°00'00\n1 + 3\n= 4</pre>";
    writeFileSync(join(cache, sha256(url + 'null') + '.json'), JSON.stringify({ url, text, fetched: '2026-10-08' }));
    const { rows } = collectSource({ id: 'mode-fixture', product: 'SpeedCrunch', kind: 'speedcrunch', pages: [url] }, { cache, offline: true });
    assert.equal(rows.find(row => row.expression === 'pi').expected, '');
    assert.match(rows.find(row => row.expression === 'pi').note, /display mode dependent/);
    assert.equal(rows.find(row => row.expression === '1 + 3').expected, '4');
  } finally { rmSync(cache, { recursive: true, force: true }); }
});

test('Frink imports annotated calculator input, excluding launcher commands', () => {
  assert.deepEqual(expressions('frink', '<CODE CLASS="input">javaws x</CODE><CODE CLASS="input">10 feet -&gt; cm</CODE>'), ['10 feet -> cm']);
});

test('Frink hash-delimited date inputs remain complete expressions', () => {
  const date = '# JD [212263942933679/86400000,\n     70754647644893/28800000] #';
  const rows = expressions('frink', '<CODE CLASS="input">##</CODE><CODE CLASS="input">' + date + '</CODE>');
  assert.ok(rows.includes('##'));
  assert.ok(rows.includes(date));
});

test('inline numeric examples are included while documentation metadata and outputs are excluded', () => {
  const unicode = String.raw`"\u2764"`;
  assert.deepEqual(expressions('soulver', '`discount = 20%` and `5pm - 9pm`\n`x-soulver://create?expression=1+1`'), ['discount = 20%', '5pm - 9pm']);
  assert.deepEqual(expressions('calca', '<code>-1</code>'), ['-1']);
  assert.deepEqual(expressions('frink', '<code>22/7</code><code>floor[3.14159, 0.001]</code>' +
    '<code>Ctrl-1</code><code>"UTF-8"</code><code CLASS="output">3.141</code><code>' + unicode + '</code>'),
  ['22/7', 'floor[3.14159, 0.001]', unicode]);
});

test('Hurmet uses calculation input attributes, excludes text editing and JavaScript batch integration', () => {
  const input = '<code>**Bold**</code><h2 id="calculation-tutorial">Tutorial</h2>' +
    '<code>2 + 2 = ?</code><span data-entry="sin(π//6) = 0.5"></span>' +
    '<h2 id="batch-mode">Batch</h2><code>const fs = require("fs")</code>';
  assert.deepEqual(expressions('hurmet', input), ['2 + 2 = ?', 'sin(π//6) = 0.5']);
  assert.throws(() => extract('hurmet', '<html>Not the manual</html>'), /heading missing/);
});

test('Wolfram extracts actual input queries, not captions or duplicated display text', () => {
  const data = { props: { pageProps: { examplesData: { sections: [{ caption: 'Add numbers',
    inputs: [{ text: 'Pretty display', props: { to: { pathname: '/input', query: { i: '125 + 375' } } } }] }] } } } };
  assert.deepEqual(expressions('wolfram', `<script id="__NEXT_DATA__">${JSON.stringify(data)}</script>`), ['125 + 375']);
  assert.throws(() => extract('wolfram', '<html>Error page</html>'), /data missing/);
});

test('Notion extracts code blocks from both record-map schemas without copying prose', () => {
  for (const nested of [false, true]) {
    const block = { type: 'code', properties: { title: [['12+5 | 17\n// comment\nx = 2 | 2']] } };
    const data = { recordMap: { block: { x: { value: nested ? { value: block } : block },
      y: { value: { type: 'text', properties: { title: [['Documentation prose']] } } } } } };
    assert.deepEqual(expressions('parsify', JSON.stringify(data)), ['12+5', 'x = 2']);
  }
});

test('Notion extracts custom-unit definitions regardless of header order or unit spelling', () => {
  const row = (properties) => ({ value: { type: 'table_row', parent_id: 'units', properties } });
  const blocks = { example: row({ name: [['custom']], definition: [['100 cm']] }),
    header: row({ name: [['Name']], definition: [['Definition']] }) };
  assert.deepEqual(expressions('parsify', JSON.stringify({ recordMap: { block: blocks } })), ['100 cm']);
});

test('date, live-rate and locale-dependent expectations are blanked; explicit fixed rates stay deterministic', () => {
  for (const input of ['current timestamp', 'Tokyo time', '6pm Sydney in Chicago',
    '12/02/1988 + 32 years', '$300 + VAT', '$200 + €200', '12 USD to PLN', 'sunday evening',
    'Euro 35 in CHF', 'in about 2 days']) assert.ok(unstable(input), input);
  assert.ok(unstable('2,100', '', 'region-settings'));
  assert.equal(unstable('50 EUR in USD at 1.05 USD/EUR'), '');
  assert.equal(unstable('$1000 for 3 years at 7%'), '');
});

test('TSV preserves multiline inputs, literal backslashes, tabs and bitwise OR', () => {
  const dir = mkdtempSync(join(tmpdir(), 'documentation-tsv-'));
  try {
    const input = 'x = 2\nx \\ 3\t| 4';
    const path = join(dir, 'rows.tsv');
    writeFileSync(path, `Example\ten\t${escapeTsv(input)}\t\thttps://example.com\n`);
    assert.equal(parseCorpus(path)[0].expression, input);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('offline extraction discovers every indexed page, records provenance and rejects missing pages', () => {
  const cache = mkdtempSync(join(tmpdir(), 'documentation-cache-'));
  const save = (url, text) => writeFileSync(join(cache, sha256(url + 'null') + '.json'), JSON.stringify({ url, text, fetched: '2026-10-08' }));
  try {
    save('https://example.com/index', '[A](https://example.com/a.md)\n[B](https://example.com/b.md)');
    save('https://example.com/a.md', '1 + 2 | 3\n`1 + 2`');
    save('https://example.com/b.md', 'now | noon');
    const result = collectSource({ id: 'fixture', product: 'Fixture', kind: 'soulver',
      index: 'https://example.com/index', include: '\\.md$' }, { cache, offline: true });
    assert.equal(result.rows.length, 2);
    assert.equal(result.rows[0].expected, '3', 'inline repetition must preserve the documented answer');
    assert.equal(result.rows[1].expected, '');
    assert.equal(result.audit.pages.length, 2);
    assert.equal(result.audit.pages[0].fetched, '2026-10-08');
    assert.throws(() => fetchPage('https://example.com/missing', { cache, offline: true }), /Missing cached/);
    assert.throws(() => collectSource({ id: 'empty', product: 'Empty', kind: 'soulver', pages: ['https://example.com/index'] }, { cache, offline: true }), /No examples/);
    const image = { url: 'https://example.com/image', sha256: 'fake', examples: [] };
    writeFileSync(join(cache, sha256(image.url) + '.image'), '<html>Download failed</html>');
    assert.throws(() => auditImage(image, { cache, offline: true }), /Not a PNG/);
  } finally { rmSync(cache, { recursive: true, force: true }); }
});
