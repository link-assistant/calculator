#!/usr/bin/env node
// One-time migration of the 2026-10-07 seed corpus; preserve expression IDs.
import { readFileSync, writeFileSync } from 'node:fs';
import { unstable, escapeTsv } from './documentation.mjs';
const input = process.argv[2];
if (!input) throw new Error('Usage: prepare-legacy.mjs ORIGINAL.tsv');
const urls = {
  Numi: 'https://github.com/nikolaeu/numi/wiki', Calca: 'https://calca.io/reference',
  Raycast: 'https://manual.raycast.com/calculator',
  'Wolfram|Alpha': 'https://www.wolframalpha.com/examples/mathematics/elementary-math/arithmetic',
  Google: 'https://support.google.com/websearch/answer/3284611?hl=en',
  'Apple Math Notes': 'https://support.apple.com/guide/ipad/solve-math-with-math-notes-ipadeb38d0f8/ipados',
  Frink: 'https://frinklang.org/frinkdocs.html',
};
const rows = readFileSync(input, 'utf8').split('\n').filter((line) => line && !line.startsWith('#')).map((line) => {
  const [source, lang, expression, expected, note] = line.split('\t');
  const path = note.split(' (')[0];
  const url = source === 'Soulver' ? `https://documentation.soulver.app/${path}` : urls[source];
  const reason = unstable(expression, expected, source === 'Soulver' ? path : note);
  const provenance = source === 'Soulver' ? '2026-10-07 documentation extraction' : '2026-10-07 hand-curated seed; not verified as verbatim on current page';
  return [source, lang, expression, reason ? '' : expected, `${url} (${provenance}; ${note}${reason ? `; ${reason}` : ''})`].map(escapeTsv).join('\t');
});
writeFileSync(new URL('./legacy-examples.tsv', import.meta.url), '# Historical seed rows, retained for coverage-baseline identity.\n' + rows.join('\n') + '\n');
