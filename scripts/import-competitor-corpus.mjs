#!/usr/bin/env node

/**
 * Import published test suites of open-source calculators into the corpus
 * format consumed by `scripts/competitor-coverage.mjs` (issue #227).
 *
 * The importer reads a checkout of the upstream repository (never bundled
 * here; licenses differ per project) and extracts single-expression
 * assertions. Multi-statement programs, variable definitions and
 * assertions that depend on previous lines are skipped on purpose.
 *
 * Usage:
 *   node scripts/import-competitor-corpus.mjs --fend <fend-repo> --qalculate <libqalculate-repo> \
 *        --numbat <numbat-repo> --rink <rink-rs-repo> --mathjs <mathjs-repo> --out <file.tsv>
 *
 * Any subset of the source flags may be given. Each flag takes the repository root.
 */

import { readFileSync, readdirSync, writeFileSync } from 'fs';
import { join } from 'path';

const SOURCES = {
  fend: {
    description: 'fend core/tests/integration_tests.rs',
    url: 'https://github.com/printfn/fend/blob/main/core/tests/integration_tests.rs',
    license: 'MIT',
  },
  qalculate: {
    description: 'libqalculate tests/*.batch',
    url: 'https://github.com/Qalculate/libqalculate/tree/master/tests',
    license: 'GPL-2.0-or-later',
  },
  numbat: {
    description: 'numbat numbat/tests/interpreter.rs',
    url: 'https://github.com/sharkdp/numbat/blob/master/numbat/tests/interpreter.rs',
    license: 'MIT OR Apache-2.0',
  },
  rink: {
    description: 'rink-rs core/tests/query.rs',
    url: 'https://github.com/tiffany352/rink-rs/blob/master/core/tests/query.rs',
    license: 'MPL-2.0',
  },
  mathjs: {
    description: 'mathjs test/unit-tests/expression/parse.test.js',
    url: 'https://github.com/josdejong/mathjs/blob/develop/test/unit-tests/expression/parse.test.js',
    license: 'Apache-2.0',
  },
};

function parseArgs(argv) {
  const options = { out: 'docs/case-studies/issue-227/corpus/open-source-tests.tsv', repos: {} };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const value = argv[i + 1];
    if (arg === '--out') {
      options.out = value;
      i += 1;
    } else if (arg.startsWith('--') && SOURCES[arg.slice(2)]) {
      options.repos[arg.slice(2)] = value;
      i += 1;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (Object.keys(options.repos).length === 0) {
    throw new Error(`Give at least one of: ${Object.keys(SOURCES).map((s) => `--${s}`).join(', ')}`);
  }
  return options;
}

/** Decode a Rust/JS string literal body (escapes only, no raw strings). */
function decodeLiteral(body) {
  return body.replace(/\\(u\{([0-9a-f]+)\}|.)/gi, (match, c, hex) => {
    if (hex) {
      return String.fromCodePoint(Number.parseInt(hex, 16));
    }
    return { n: '\n', t: '\t', r: '\r', '\\': '\\', '"': '"', "'": "'", 0: '\0' }[c] ?? match;
  });
}

const RUST_STRING = '"((?:[^"\\\\]|\\\\.)*)"';

/** Extract `fn("input", "expected")` pairs with ordinary Rust string literals. */
function rustPairs(text, functionNames) {
  const pattern = new RegExp(`\\b(?:${functionNames.join('|')})\\(\\s*${RUST_STRING}\\s*,\\s*${RUST_STRING}\\s*,?\\s*\\)`, 'g');
  const pairs = [];
  for (const match of text.matchAll(pattern)) {
    pairs.push({ expression: decodeLiteral(match[1]), expected: decodeLiteral(match[2]) });
  }
  return pairs;
}

const CONTROL = /[\u0000-\u0008\u000b\u000c\u000e-\u001f]/;

/** Keep only one-line cases without control characters (string-escape tests are not calculator features). */
function singleLine(text) {
  return !text.includes('\n') && !CONTROL.test(text) && text.trim() !== '';
}

function importFend(root) {
  const text = readFileSync(join(root, 'core', 'tests', 'integration_tests.rs'), 'utf8');
  return rustPairs(text, ['test_eval', 'test_eval_simple'])
    .filter((p) => singleLine(p.expression) && singleLine(p.expected))
    .map((p) => ({ source: 'fend', lang: 'en', ...p, note: '' }));
}

function importQalculate(root) {
  const dir = join(root, 'tests');
  const cases = [];
  for (const file of readdirSync(dir).sort()) {
    if (!file.endsWith('.batch')) {
      continue;
    }
    const category = file.replace(/\.batch$/, '');
    const lines = readFileSync(join(dir, file), 'utf8').split('\n');
    let current = null;
    for (const rawLine of lines) {
      const line = rawLine.replace(/\r$/, '');
      if (line.trim() === '' || line.startsWith('#')) {
        current = null;
        continue;
      }
      if (line.startsWith('\t')) {
        if (current && current.expected === null) {
          current.expected = line.trim();
        }
        continue;
      }
      current = { source: 'Qalculate!', lang: 'en', expression: line, expected: null, note: category };
      cases.push(current);
    }
  }
  return cases.filter((c) => c.expected !== null).map((c) => ({ ...c }));
}

function importNumbat(root) {
  const text = readFileSync(join(root, 'numbat', 'tests', 'interpreter.rs'), 'utf8');
  return rustPairs(text, ['expect_output'])
    .filter((p) => singleLine(p.expression))
    .map((p) => ({ source: 'Numbat', lang: 'en', ...p, note: '' }));
}

function importRink(root) {
  const text = readFileSync(join(root, 'core', 'tests', 'query.rs'), 'utf8');
  return rustPairs(text, ['test'])
    .filter((p) => singleLine(p.expression))
    .map((p) => ({ source: 'Rink', lang: 'en', ...p, note: '' }));
}

function importMathjs(root) {
  const text = readFileSync(join(root, 'test', 'unit-tests', 'expression', 'parse.test.js'), 'utf8');
  const pattern = /assert\.strictEqual\(parseAndEval\('((?:[^'\\]|\\.)*)'\),\s*(-?\d+(?:\.\d+)?(?:e[-+]?\d+)?|true|false)\)/g;
  const cases = [];
  for (const match of text.matchAll(pattern)) {
    const expression = decodeLiteral(match[1]);
    if (singleLine(expression)) {
      cases.push({ source: 'math.js', lang: 'en', expression, expected: match[2], note: '' });
    }
  }
  return cases;
}

const IMPORTERS = { fend: importFend, qalculate: importQalculate, numbat: importNumbat, rink: importRink, mathjs: importMathjs };

function encode(text) {
  return text.replace(/\\/g, '\\\\').replace(/\t/g, '\\t').replace(/\n/g, '\\n');
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const lines = [
    '# Generated by scripts/import-competitor-corpus.mjs from upstream test suites.',
    '# Columns: source, lang, expression, expected, note. Upstream data is not relicensed;',
    '# each row cites the suite it was extracted from and is used only as an executable reference.',
  ];
  const counts = {};
  for (const [name, root] of Object.entries(options.repos)) {
    const meta = SOURCES[name];
    const cases = IMPORTERS[name](root);
    counts[name] = cases.length;
    lines.push('', `# ${name}: ${meta.description} (${meta.license}) ${meta.url}`);
    for (const c of cases) {
      lines.push([c.source, c.lang, encode(c.expression), encode(c.expected), encode(c.note)].join('\t'));
    }
  }
  writeFileSync(options.out, `${lines.join('\n')}\n`);
  for (const [name, count] of Object.entries(counts)) {
    console.log(`${name}: ${count} cases`);
  }
  console.log(`written: ${options.out}`);
}

try {
  main();
} catch (error) {
  console.error(`import-competitor-corpus: ${error.message}`);
  process.exit(2);
}
