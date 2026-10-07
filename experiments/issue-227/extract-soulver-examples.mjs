#!/usr/bin/env node
// Extract `expression | answer` example pairs from Soulver's public Markdown
// documentation (https://documentation.soulver.app/llms.txt lists the pages).
// Usage: node experiments/issue-227/extract-soulver-examples.mjs <dir-with-md-files> > rows.tsv
import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';

const dir = process.argv[2];
const DATE_DEPENDENT = /\b(today|now|yesterday|tomorrow|ago|from now|since|till|until|next |last |time in|date in|week of year|random|workdays in|days in Q|on march|one year ago)\b/i;
const rows = [];
for (const file of readdirSync(dir).sort()) {
  if (!file.endsWith('.md')) continue;
  const page = file.replace(/\.md$/, '').replace(/_/g, '/');
  const text = readFileSync(join(dir, file), 'utf8');
  for (const raw of text.split('\n')) {
    let line = raw.replace(/<\/?(strong|pre|code)>/g, '').replace(/\/\/.*$/, '').trim();
    if (!line.includes('|') || line.startsWith('|') || line.startsWith('#')) continue;
    const [expression, expected] = line.split('|').map((s) => s.trim());
    if (!expression || expression.includes(' = ') || /^(if|My tax|cost$)/.test(expression) || expression.length > 60) continue;
    if (/[A-Za-z]+ = /.test(expression) || /\b(earnings|BMI|income|expenses)\b/.test(expression)) continue;
    const dateDependent = DATE_DEPENDENT.test(expression) || DATE_DEPENDENT.test(expected || '');
    rows.push(['Soulver', 'en', expression, dateDependent ? '' : expected || '', `${page}${dateDependent ? ' (date/random dependent)' : ''}`]);
  }
}
process.stdout.write(rows.map((r) => r.join('\t')).join('\n') + '\n');
