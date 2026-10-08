#!/usr/bin/env node
// Offline, deterministic assembly from retained seeds, licensed snapshots and
// localized scientific statements. No calculator results are used as oracles.
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { locales } from './natural-language-locales.mjs';

const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8');
const snapshot = (name) => JSON.parse(read(`./sources/${name}.json`));
const wiki = (page) => `https://en.wikipedia.org/wiki/${page}`;
const nist = 'https://physics.nist.gov/cuu/Units';
const cases = {
  separators: [
    ['1234.5', '1234.5 m'], ['1234567.89', '1234567.89 m'],
    ['299792458', '299792458 m'], ['12345.678', '12345.678 m'],
    ['1000000.25', '1000000.25 m'], ['9876543.21', '9876543.21 m'],
  ].map(([a, expected]) => ({ a, expected })),
  'si-prefix': [
    { a: '5', unit: 'µm', expected: '5 µm' },
    { a: '12', unit: 'nm', expected: '12 nm' },
    { a: '3', unit: 'km', expected: '3 km' },
    { a: '2.5', unit: 'mm', expected: '2.5 mm' },
  ],
  'scientific-notation': [
    { a: '1.5 × 10³', expected: '1500 Hz' },
    { a: '2.5e-3', expected: '0.0025 Hz' },
    { a: '3 × 10⁸', expected: '300000000 Hz' },
    { a: '6.02 × 10^23', expected: '6.02e23 Hz' },
  ],
  uncertainty: [
    { a: '12.3', b: '0.4', expected: '' }, { a: '100', b: '2', expected: '' },
    { a: '1.25', b: '0.05', expected: '' }, { a: '20', b: '0.1', expected: '' },
  ],
  'percentage-change': [
    { a: '80', b: '25', expected: '100' }, { a: '200', b: '10', expected: '220' },
    { a: '50', b: '20', expected: '60' }, { a: '120', b: '5', expected: '126' },
  ],
  'fold-change': [
    { a: '40', b: '2.5', expected: '100' }, { a: '8', b: '3', expected: '24' },
    { a: '100', b: '0.5', expected: '50' }, { a: '12', b: '4', expected: '48' },
  ],
  'order-of-magnitude': [
    { a: '5', b: '3', expected: '5000' }, { a: '7', b: '2', expected: '700' },
    { a: '2.5', b: '4', expected: '25000' }, { a: '6', b: '5', expected: '600000' },
  ],
  ratio: [
    { a: '22', b: '7', expected: '3.142857' }, { a: '3', b: '4', expected: '0.75' },
    { a: '5', b: '2', expected: '2.5' }, { a: '1', b: '8', expected: '0.125' },
  ],
  'named-constant': [
    { expected: '3.141592653589793' }, { expected: '2.718281828459045' },
    { expected: '299792458 m/s' }, { expected: '6.02214076e23 1/mol' },
  ],
  power: [
    { a: '3', b: '2', expected: '9' }, { a: '4', b: '3', expected: '64' },
    { a: '2', b: '10', expected: '1024' }, { a: '5', b: '4', expected: '625' },
  ],
  root: [
    { a: '81', b: '2', expected: '9' }, { a: '27', b: '3', expected: '3' },
    { a: '16', b: '4', expected: '2' }, { a: '32', b: '5', expected: '2' },
  ],
  'unit-conversion': [
    { a: '5', unit: 'µm', target: 'nm', expected: '5000 nm' },
    { a: '3', unit: 'km', target: 'm', expected: '3000 m' },
    { a: '2500', unit: 'mm', target: 'm', expected: '2.5 m' },
    { a: '25', unit: '°C', target: 'K', expected: '298.15 K' },
  ],
};
const citations = {
  separators: [`${nist}/checklist.html`, 'public-domain'],
  'si-prefix': [`${nist}/prefixes.html`, 'public-domain'],
  'scientific-notation': [wiki('Scientific_notation'), 'CC-BY-SA-4.0'],
  uncertainty: ['https://physics.nist.gov/cuu/Uncertainty/index.html', 'public-domain'],
  'percentage-change': [wiki('Percentage_change'), 'CC-BY-SA-4.0'],
  'fold-change': [wiki('Fold_change'), 'CC-BY-SA-4.0'],
  'order-of-magnitude': [wiki('Order_of_magnitude'), 'CC-BY-SA-4.0'],
  ratio: [wiki('Ratio'), 'CC-BY-SA-4.0'],
  'named-constant': ['https://physics.nist.gov/cuu/Constants/index.html', 'public-domain'],
  power: [wiki('Exponentiation'), 'CC-BY-SA-4.0'],
  root: [wiki('Nth_root'), 'CC-BY-SA-4.0'],
  'unit-conversion': [`${nist}/units.html`, 'public-domain'],
  other: [wiki('Mathematical_notation'), 'CC-BY-SA-4.0'],
};

function number(value, lang, grouped = false) {
  const { decimal, group } = locales[lang];
  let [integer, fraction] = value.split('.');
  if (grouped) {
    // Indian grouping is also retained in the Hindi and Bengali samples.
    integer = lang === 'hi' || lang === 'bn'
      ? integer.replace(/(\d+)(\d{3})$/, (_, first, last) => `${first.replace(/\B(?=(\d{2})+(?!\d))/g, group)}${group}${last}`)
      : integer.replace(/\B(?=(\d{3})+(?!\d))/g, group);
  }
  return integer + (fraction === undefined ? '' : `${decimal}${fraction}`);
}

function note(construct, kind, url, license, rest = '', evaluateOnly = false) {
  const licenseUrl = license === 'public-domain'
    ? 'https://www.nist.gov/nist-research-library/library-faqs'
    : `https://creativecommons.org/licenses/${license === 'CC-BY-4.0' ? 'by' : 'by-sa'}/4.0/`;
  return (`construct=${construct}; kind=${kind}; license=${license}; ${url}; license-url=${licenseUrl}; ` +
    `${evaluateOnly ? 'oracle=evaluate-only; ' : ''}${rest}`).trimEnd();
}

function seedRows() {
  let theorem = 'Pythagorean_theorem';
  const rows = [];
  for (const line of read('./seed-natural-language.tsv').split('\n')) {
    if (line.startsWith('# Kepler')) theorem = 'Kepler%27s_laws_of_planetary_motion';
    if (line.startsWith('# Euler')) theorem = 'Basel_problem';
    if (line.startsWith('# Mass')) theorem = 'Mass%E2%80%93energy_equivalence';
    if (line.startsWith('# Ohm')) theorem = 'Ohm%27s_law';
    if (!line || line.startsWith('#')) continue;
    const [source, lang, expression, expected, original = ''] = line.split('\t');
    let citation = citations.other;
    let kind = 'derived';
    if (source.startsWith('Wikipedia quote')) {
      citation = [snapshot(`wikipedia-${lang}`).url, 'CC-BY-SA-4.0'];
      if (source === 'Wikipedia quote') kind = 'legacy-quote';
    } else if (source === 'Theorem in words') citation = [wiki(theorem), 'CC-BY-SA-4.0'];
    const construct = expression.includes('±') ? 'uncertainty' : 'other';
    if (construct === 'uncertainty') citation = citations.uncertainty;
    rows.push([source, lang, expression, expected,
      note(construct, kind, ...citation, `seed=2026-10-07; ${original}`, expected === '')]);
  }
  return rows;
}

export function buildCorpus() {
  const rows = seedRows();
  for (const [lang, locale] of Object.entries(locales)) {
    for (const [construct, examples] of Object.entries(cases)) {
      examples.forEach((example, index) => {
        const parameters = { ...example };
        if (construct === 'order-of-magnitude') {
          const forms = { ru: ['порядка', 'порядков'], uk: ['порядки', 'порядків'], pl: ['rzędy', 'rzędów'] }[lang];
          if (forms) parameters.orders = forms[Number(example.b) < 5 ? 0 : 1];
        }
        for (const key of ['a', 'b']) {
          if (parameters[key]) parameters[key] = number(parameters[key], lang, construct === 'separators');
          if (construct === 'separators' && index % 2 === 1) {
            const digits = { ar: '٠١٢٣٤٥٦٧٨٩', fa: '۰۱۲۳۴۵۶۷۸۹', hi: '०१२३४५६७८९', bn: '০১২৩৪৫৬৭৮৯', th: '๐๑๒๓๔๕๖๗๘๙' }[lang];
            if (digits && parameters[key]) parameters[key] = parameters[key].replace(/\d/g, (digit) => digits[Number(digit)]);
          }
        }
        if (construct === 'named-constant') parameters.name = locale.constants[index];
        const expression = locale.phrases[construct].replace(/\{(\w+)\}/g, (_, key) => parameters[key]);
        const citation = construct === 'named-constant' && index < 2
          ? [wiki(index === 0 ? 'Pi' : 'E_(mathematical_constant)'), 'CC-BY-SA-4.0']
          : citations[construct];
        rows.push(['Scientific statement (derived)', lang, expression, example.expected,
          note(construct, 'derived', ...citation,
            `adaptation=localized template with synthetic numbers; case=${construct}-${index + 1}; ` +
            `parameters=${JSON.stringify(example)};`, example.expected === '')]);
      });
    }
    const source = snapshot(`wikipedia-${lang}`);
    for (const expression of source.candidates) {
      rows.push(['Wikipedia sentence', lang, expression, '',
        note('named-constant', 'verbatim', source.revisionUrl, source.license,
          `authors=${source.authors}; title=${source.title}; fetched=${source.fetched};`, true)]);
    }
  }
  const selections = [
    ['1602.03837v1', 'All uncertainties define 90% credible intervals.', 'uncertainty'],
    ['1706.01812v2', 'The source luminosity distance is $880^{+450}_{-390}~\\mathrm{Mpc}$ corresponding to a redshift of $z = 0.18^{+0.08}_{-0.07}$.', 'uncertainty'],
    ['1710.05832v1', 'between 0.86 and 2.26 $M_\\odot$', 'other'],
  ];
  for (const [id, expression, construct] of selections) {
    const source = snapshot(`arxiv-${id}`);
    if (!source.extract.includes(expression)) throw new Error(`${id}: selected quote not found`);
    rows.push(['Open-access paper quote', 'en', expression, '',
      note(construct, 'verbatim', source.url, source.license,
        `authors=${source.authors}; title=${source.title}; section=abstract; fetched=${source.fetched};`, true)]);
  }
  const escape = (value) => value.replace(/\\/g, '\\\\').replace(/\t/g, '\\t').replace(/\r?\n/g, '\\n');
  return '# Multilingual scientific statements: 23 languages, 12 constructs (issues #227/#232).\n' +
    '# Columns: source, lang, expression, expected, note. See ../natural-language-corpus.md.\n' +
    '# Generated offline by experiments/issue-227/build-natural-language-corpus.mjs; preserve licensed attribution.\n' +
    `${rows.map((row) => row.map(escape).join('\t')).join('\n')}\n`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const corpus = new URL('../../docs/case-studies/issue-227/corpus/natural-language-quotes.tsv', import.meta.url);
  writeFileSync(corpus, buildCorpus());
  console.log(`Wrote ${fileURLToPath(corpus)}`);
}
