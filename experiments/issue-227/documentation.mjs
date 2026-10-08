// Public documentation extraction only. Never execute upstream code.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

export function decode(text) {
  const entities = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ',
    times: '×', divide: '÷', minus: '−', pi: 'π', rarr: '→', deg: '°' };
  return text.replace(/&(#x[\da-f]+|#\d+|\w+);/gi, (all, value) => {
    if (value.startsWith('#')) {
      const number = value[1].toLowerCase() === 'x' ? parseInt(value.slice(2), 16) : Number(value.slice(1));
      return number <= 0x10ffff ? String.fromCodePoint(number) : all;
    }
    return entities[value] ?? all;
  });
}

export function plain(html) {
  return decode(html.replace(/<(script|style)\b[^>]*>[\s\S]*?<\/\1>/gi, '')
    .replace(/<br\s*\/?\s*>/gi, '\n').replace(/<\/(?:p|div|li|tr|pre|h[1-6])>/gi, '\n')
    .replace(/<\/?[a-z][a-z\d:-]*(?:\s[^<>]*?)?\s*\/?>/gi, '')).trim();
}

export function elements(html, tag) {
  return [...html.matchAll(new RegExp(`<${tag}\\b([^>]*)>([\\s\\S]*?)<\\/${tag}>`, 'gi'))]
    .map((match) => ({ attrs: match[1], text: plain(match[2]), html: match[2] }));
}

export function links(text, base) {
  const values = [...text.matchAll(/href=["']([^"']+)["']/gi)].map((m) => decode(m[1]));
  values.push(...[...text.matchAll(/\]\((https?:\/\/[^\s)]+)\)/g)].map((m) => m[1]));
  return [...new Set(values.map((value) => {
    try { const url = new URL(value, base); url.hash = ''; return url.href; } catch { return null; }
  }).filter(Boolean))].sort();
}

export function unstable(expression, expected = '', context = '') {
  // Provenance prose (for example "current page") is not expression semantics.
  const text = `${expression} ${expected}`;
  if (/\b(today|now|yesterday|tomorrow|ago|since|till|until|next|last|current|random|rand|randint|savedate)\b|\bin about \d+ (days?|weeks?|months?|years?)/i.test(text)) return 'date/random dependent';
  if (/\b(time|date)\s+(in|diff)|\b\w+\s+time\b|\b(PST|PDT|GMT|UTC|AEST|CEST|EDT|Sydney|Chicago|Tokyo|Paris|Berlin|LAX|Japan|Seattle|Moscow)\b/i.test(expression)
      || /\b(?:datetime|epoch|fromunix)\s*\(/i.test(expression)
      || /time.zones|timestamps|dates.and.times|workdays|clock.time|calendar|date and time/i.test(context)
      || /\b(January|February|March|April|May|June|July|August|September|October|November|December|sunday|monday|tuesday|wednesday|thursday|friday|saturday)\b|\d+[/.]\d+[/.]\d+/i.test(expression)) return 'date/locale dependent';
  if (/inflation|income.tax|sales.tax|\b(VAT|GST|CPI)\b/i.test(text)) return 'locale/data dependent';
  const codes = expression.match(/\b(?:USD|EUR|GBP|AUD|CAD|RUB|CHF|PLN|DKK|BTC|ETH|JPY|CNY|euros?|dollars?|pounds?|yen)\b|[$€£¥₽]/gi) || [];
  if ((new Set(codes.map((code) => code.toUpperCase())).size > 1 || /currenc/i.test(context))
      && !/\b(?:at|@)\s*\d/i.test(expression)) return 'live rate dependent';
  if (/region.settings|cultures|locale|decimal.separator|grouping/i.test(context)) return 'locale dependent';
  return '';
}

// Split the documentation's answer separator, never an expression's bitwise OR.
export function pair(line, separator = '|') {
  const trimmed = line.trim();
  if (!trimmed || trimmed.startsWith('//') || trimmed.startsWith('#') || /^[-+:|=\s]+$/.test(trimmed)) return null;
  let index = -1;
  if (separator === '|') {
    const match = [...trimmed.matchAll(/\s+\|\s+/g)].at(-1);
    if (match) index = match.index + match[0].indexOf('|');
  } else index = trimmed.indexOf(separator);
  if (index < 0) return { expression: trimmed, expected: '' };
  const expression = trimmed.slice(0, index).trim();
  const expected = trimmed.slice(index + separator.length).trim();
  return expression ? { expression, expected } : null;
}

function codeLines(text, separator, context = '') {
  return text.split('\n').map((line) => pair(line, separator)).filter(Boolean).map((row) => ({ ...row, context }));
}

export function extract(kind, text) {
  const rows = [];
  if (kind === 'soulver') {
    // GitBook uses HTML pre blocks and plain expression | answer lines.
    let heading = '';
    for (let line of text.split('\n')) {
      if (/^#+ /.test(line)) heading = line;
      line = plain(line).replace(/\*\*/g, '').trim();
      if (/\s\|\s/.test(line) && !line.startsWith('|')) {
        const row = pair(line);
        if (row) rows.push({ ...row, context: heading });
      }
    }
    for (const block of elements(text, 'pre')) rows.push(...codeLines(block.text, '|'));
    for (const block of text.matchAll(/^```[^\n]*\n([\s\S]*?)^```/gm)) rows.push(...codeLines(block[1], '|'));
    for (const table of elements(text, 'table')) {
      const headers = elements(table.html, 'th').map((cell) => cell.text);
      const i = headers.findIndex((cell) => /^(example|expression|input|syntax)$/i.test(cell));
      const answer = headers.findIndex((cell) => /^(answer|result|output)$/i.test(cell));
      if (i < 0) continue;
      for (const tr of elements(table.html, 'tr')) {
        const cells = elements(tr.html, 'td');
        if (cells[i]) rows.push({ expression: cells[i].text, expected: cells[answer]?.text ?? '', context: '' });
      }
    }
    for (const match of text.matchAll(/`([^`\n]+)`\s*→\s*\*\*([^*\n]+)\*\*/g)) {
      rows.push({ expression: match[1], expected: match[2], context: '' });
    }
    for (const match of text.matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)) {
      if (/\d/.test(match[1]) && !/^\w[\w+-]*:\/\//.test(match[1])) {
        rows.push({ expression: match[1], expected: '', context: '' });
      }
    }
    for (const line of text.split('\n')) {
      if (/^[a-z]+\([^\n]+\) = [\d-]/i.test(line)) {
        const row = pair(line, ' = '); if (row) rows.push(row);
      }
    }
    // Include proper Markdown tables (the original extractor discarded them).
    let headers = [];
    for (const line of text.split('\n')) {
      if (!line.startsWith('|')) continue;
      const cells = line.replace(/^\||\|$/g, '').split(/(?<!\\)\|/).map((cell) => plain(cell).replace(/`|\*\*/g, '').trim());
      if (cells.some((cell) => /^(expression|input|example|syntax)$/i.test(cell))) { headers = cells; continue; }
      const i = headers.findIndex((cell) => /^(expression|input|example|syntax)$/i.test(cell));
      if (i >= 0 && cells[i] && !/^[-:]+$/.test(cells[i])) rows.push({ expression: cells[i], expected: '', context: '' });
    }
  } else if (kind === 'numi') {
    for (const block of text.matchAll(/(?:^ {4}[^\n]*\n?)+/gm)) {
      for (const line of block[0].replace(/^ {4}/gm, '').split('\n')) {
        const normalized = line.replace(/\s+=\s+/g, ' = ');
        const row = /^[a-z]\w* = /i.test(normalized) ? pair(line, '|') : pair(normalized, ' = ');
        if (row) rows.push(row);
      }
    }
    for (const table of elements(text, 'table')) {
      const header = elements(table.html, 'th').map((cell) => cell.text);
      const i = header.findIndex((cell) => /^Example$/i.test(cell));
      if (i < 0) continue;
      for (const tr of elements(table.html, 'tr')) {
        const cells = elements(tr.html, 'td');
        if (cells[i]) rows.push({ expression: cells[i].text, expected: '', context: '' });
      }
    }
    for (const line of text.split('\n')) if (/^\$30/.test(line)) rows.push({ expression: line, expected: '', context: 'currency' });
    for (const match of text.matchAll(/`([^`\n]+)`/g)) {
      if (/\d/.test(match[1]) && !/^(ISO |js )/.test(match[1])) rows.push({ expression: match[1], expected: '', context: '' });
    }
  } else if (kind === 'calca') {
    for (const block of elements(text, 'pre')) rows.push(...codeLines(block.text, '=>', /#@|culture/.test(block.text) ? 'cultures' : ''));
    for (const block of elements(text.replace(/<pre\b[^>]*>[\s\S]*?<\/pre>/gi, ''), 'code')) {
      if (block.text.includes('=>') || /\d/.test(block.text)) rows.push(...codeLines(block.text, '=>'));
    }
  } else if (kind === 'frink') {
    // CLASS=input identifies runnable examples; other CODE tags are names/prose.
    for (const code of elements(text, 'code')) {
      if (/class=["']input["']/i.test(code.attrs) && !/^javaws\b|import frink\.parser\.Frink/.test(code.text)) {
        rows.push(...codeLines(code.text, ' ==> '));
      }
      // The manual also uses unclassified inline code for worked literals,
      // unit arithmetic and function examples; output/comment classes are separate.
      const stringExample = /^"(?:[^"\\]|\\.)*"$/.test(code.text) && !/^"(?:UTF|ISO)-?\d/i.test(code.text);
      const escapeExample = /^\\u(?:[\da-f]{4}|\{[\da-f]+\})$/i.test(code.text);
      if (!code.attrs.trim() && /\d/.test(code.text) && !code.text.includes('\n')
          && (stringExample || escapeExample || /^(?:[+-]?(?:\d|\.\d)|[([]|[a-z_]\w*(?:\[|\(|\s|.*[*/^])|#\s*(?:AD|BC|JD))/i.test(code.text))
          && !/\(exactly\b|^[1-9A-HJ-NP-Za-km-z]{20,}$/.test(code.text)) {
        rows.push({ expression: code.text, expected: '', context: '' });
      }
    }
    for (const block of elements(text, 'p')) {
      if (/class=["']code["']/i.test(block.attrs) && !/class=["']input["']/i.test(block.html)) {
        rows.push(...codeLines(block.text, ' ==> '));
      }
    }
  } else if (kind === 'hurmet') {
    // Restrict to calculation tutorial/reference, excluding Markdown/TeX editing.
    const start = text.search(/id=["']calculation-tutorial["']/i);
    if (start < 0) throw new Error('Hurmet calculation tutorial heading missing');
    const end = text.search(/<h2\b[^>]*id=['"]batch-mode['"]/i);
    const body = text.slice(start, end < 0 ? undefined : end);
    for (const code of elements(body, 'code')) {
      if (code.text && !/^https?:/.test(code.text)) rows.push({ expression: code.text, expected: '', context: '' });
    }
    for (const match of body.matchAll(/data-entry=(?:'([^']*)'|"([^"]*)")/gi)) {
      const expression = decode(match[1] ?? match[2]);
      if (expression.trim()) rows.push({ expression, expected: '', context: '' });
    }
  } else if (kind === 'speedcrunch') {
    const speedPair = (line) => {
      if (/^\s*\?/.test(line)) return null;
      if (/^\s*[a-z]\w*(?:\([a-z;,\s]+\))?\s*=/i.test(line)) return pair(line, '|');
      const row = pair(line, ' = ');
      if (row?.expected) row.expected = row.expected.split(' = ').at(-1).replace(/\s+\?.*$/, '').trim();
      return row;
    };
    for (const block of elements(text, 'pre')) {
      let previous = null;
      for (const line of block.text.split('\n')) {
        if (/^\s*=/.test(line) && previous) previous.expected = line.replace(/^\s*=\s*/, '');
        else if (line.trim()) { previous = speedPair(line); if (previous) rows.push(previous); }
      }
    }
    for (const code of elements(text.replace(/<pre\b[^>]*>[\s\S]*?<\/pre>/gi, ''), 'code')) {
      if (/[\d]|\b(?:pi|ans|j)\b/.test(code.text) && !/^[-.\w]+\(\)$/.test(code.text)) {
        const row = speedPair(code.text); if (row) rows.push(row);
      }
    }
  } else if (kind === 'wolfram') {
    const match = text.match(/<script[^>]+id="__NEXT_DATA__"[^>]*>([\s\S]*?)<\/script>/i);
    if (!match) throw new Error('Wolfram example data missing');
    const data = JSON.parse(match[1]).props.pageProps.examplesData;
    function walk(value) {
      if (!value || typeof value !== 'object') return;
      if (value.pathname === '/input' && typeof value.query?.i === 'string') rows.push({ expression: value.query.i, expected: '', context: '' });
      for (const child of Object.values(value)) walk(child);
    }
    walk(data);
  } else if (kind === 'parsify') {
    const data = JSON.parse(text);
    const blocks = Object.values(data.recordMap.block).map((item) => item.value?.value ?? item.value);
    const definitionKeys = new Map();
    for (const block of blocks.filter((block) => block.type === 'table_row')) {
      for (const [key, field] of Object.entries(block.properties ?? {})) {
        if (field.map((item) => item[0]).join('') === 'Definition') definitionKeys.set(block.parent_id, key);
      }
    }
    for (const block of blocks) {
      if (block.type === 'code') {
        const code = (block.properties?.title ?? []).map((item) => item[0]).join('');
        rows.push(...codeLines(code, '|'));
      }
      if (block.type === 'table_row') {
        // Custom-unit definitions are calculator inputs; names and aliases are metadata.
        for (const [key, field] of Object.entries(block.properties ?? {})) {
          const value = field.map((item) => item[0]).join('');
          if (key === definitionKeys.get(block.parent_id) && value.trim() && value !== 'Definition') {
            rows.push({ expression: value, expected: '', context: 'custom unit definition' });
          }
        }
      }
    }
  } else if (kind === 'raycast' || kind === 'apple') {
    for (const code of elements(text, 'code')) rows.push({ expression: code.text, expected: '', context: '' });
  } else if (kind === 'google') {
    for (const link of links(text, 'https://support.google.com')) {
      const url = new URL(link);
      if (/google\.com$/.test(url.hostname) && url.searchParams.has('q')) {
        const expression = url.searchParams.get('q');
        if (!/^(calculator|unit converter)$/i.test(expression)) rows.push({ expression, expected: '', context: '' });
      }
    }
    const section = text.match(/<h[23][^>]*>\s*Examples\s*<\/h[23]>([\s\S]*?)(?=<h[23]|$)/i);
    if (section) for (const li of elements(section[1], 'li')) rows.push({ expression: li.text, expected: '', context: '' });
  } else if (kind !== 'audit') throw new Error(`Unknown extractor: ${kind}`);
  return rows.filter((row) => row.expression.trim()).map((row) => ({ ...row, expression: row.expression.trim(), expected: row.expected?.trim() ?? '' }));
}

export const escapeTsv = (value) => value.replace(/\\/g, '\\\\').replace(/\t/g, '\\t').replace(/\r?\n/g, '\\n');
export const sha256 = (value) => createHash('sha256').update(value).digest('hex');

export function fetchPage(url, { cache, offline = false, post } = {}) {
  mkdirSync(cache, { recursive: true });
  const path = join(cache, `${sha256(url + JSON.stringify(post ?? null))}.json`);
  if (existsSync(path)) return JSON.parse(readFileSync(path, 'utf8'));
  if (offline) throw new Error(`Missing cached page: ${url}`);
  const args = ['--fail', '--silent', '--show-error', '--location', url];
  if (post) args.push('-H', 'Content-Type: application/json', '--data', JSON.stringify(post));
  const result = spawnSync('curl', args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  if (result.status !== 0) throw new Error(`Fetch failed for ${url}: ${result.stderr}`);
  const page = { url, fetched: new Date().toISOString().slice(0, 10), text: result.stdout };
  writeFileSync(path, JSON.stringify(page));
  return page;
}

export function auditImage(reference, { cache, offline = false }) {
  const path = join(cache, `${sha256(reference.url)}.image`);
  if (!existsSync(path)) {
    if (offline) throw new Error(`Missing cached image: ${reference.url}`);
    const result = spawnSync('curl', ['--fail', '--silent', '--show-error', '--location', reference.url, '--output', path]);
    if (result.status !== 0) throw new Error(`Image fetch failed: ${reference.url}`);
  }
  const bytes = readFileSync(path);
  const valid = bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
    || bytes.subarray(0, 3).equals(Buffer.from([255, 216, 255]));
  if (!valid) throw new Error(`Not a PNG/JPEG image: ${reference.url}`);
  if (sha256(bytes) !== reference.sha256) throw new Error(`Documentation image changed; review transcription: ${reference.url}`);
  return { url: reference.url, sha256: reference.sha256, rows: reference.examples.length };
}
