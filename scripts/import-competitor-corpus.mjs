#!/usr/bin/env node

/**
 * Import published test suites of open-source calculators into the corpus
 * format consumed by `scripts/competitor-coverage.mjs` (issue #227).
 *
 * Reads clean upstream Git checkouts, records revisions and licenses, and
 * extracts literal assertions. See docs/competitor-corpus.md for limits.
 *
 * Usage: make import-corpus CORPUS_CHECKOUTS=/path/to/checkouts
 * Or: node scripts/import-competitor-corpus.mjs --checkouts /path/to/checkouts
 */

import { readFileSync, readdirSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { importMathjsAdditional, importInsect, importKalker, importHurmet, importSpeedcrunch } from './competitor-extractors.mjs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

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
    description: 'mathjs test/unit-tests/{expression,function,type/unit}/**/*.test.js',
    url: 'https://github.com/josdejong/mathjs/blob/develop/test/unit-tests/expression/parse.test.js',
    license: 'Apache-2.0',
  },
  insect: {
    description: 'Insect test/Main.purs (initialEnvironment assertions)',
    url: 'https://github.com/sharkdp/insect/blob/master/test/Main.purs',
    license: 'MIT',
  },
  kalker: {
    description: 'kalker kalk/src/integration_testing.rs and tests/*.kalker',
    url: 'https://github.com/PaddiM8/kalker/tree/master/tests',
    license: 'MIT',
  },
  hurmet: {
    description: 'Hurmet test/test.cjs (calcTests)',
    url: 'https://github.com/ronkok/Hurmet/blob/master/test/test.cjs',
    license: 'MIT',
  },
  speedcrunch: {
    description: 'SpeedCrunch src/tests/*.cpp (literal CHECK_EVAL assertions)',
    url: 'https://github.com/heldercorreia/speedcrunch/tree/master/src/tests',
    license: 'GPL-2.0-or-later',
  },
};

export function parseArgs(argv) {
  const options = {
    checkouts: null,
    out: 'docs/case-studies/issue-227/corpus/open-source-tests.tsv',
    inventory: 'docs/case-studies/issue-227/competitor-inventory.md',
    lock: 'scripts/competitor-upstreams.json',
    verbose: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--verbose') options.verbose = true;
    else if (arg === '--help' || arg === '-h') return null;
    else if (['--checkouts', '--out', '--inventory', '--lock'].includes(arg)) {
      const value = argv[++i];
      if (!value || value.startsWith('--')) throw new Error(`Missing value for ${arg}`);
      options[arg.slice(2)] = value;
    } else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!options.checkouts) throw new Error('Give --checkouts <directory containing all nine repositories>');
  return options;
}

const RUST_STRING = '"((?:[^"\\\\]|\\\\.)*)"';

// Preserve the five original suite imports byte-for-byte so extending the
// corpus does not remove expressions tracked by the existing coverage gate.
function decodeLiteral(body) {
  return body.replace(/\\(u\{([0-9a-f]+)\}|.)/gi, (match, c, hex) => {
    if (hex) return String.fromCodePoint(Number.parseInt(hex, 16));
    return { n: '\n', t: '\t', r: '\r', '\\': '\\', '"': '"', "'": "'", 0: '\0' }[c] ?? match;
  });
}

function singleLine(text) {
  return !text.includes('\n') && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/.test(text) && text.trim() !== '';
}

/** Extract `fn("input", "expected")` pairs with ordinary Rust string literals. */
function rustPairs(text, functionNames) {
  const pattern = new RegExp(`\\b(?:${functionNames.join('|')})\\(\\s*${RUST_STRING}\\s*,\\s*${RUST_STRING}\\s*,?\\s*\\)`, 'g');
  const pairs = [];
  for (const match of text.matchAll(pattern)) {
    pairs.push({ expression: decodeLiteral(match[1]), expected: decodeLiteral(match[2]) });
  }
  return pairs;
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

const IMPORTERS = { fend: importFend, qalculate: importQalculate, numbat: importNumbat, rink: importRink, mathjs: (root) => {
  const legacy = importMathjs(root);
  const keys = new Set(legacy.map((c) => JSON.stringify([c.expression, c.expected])));
  return [...legacy, ...importMathjsAdditional(root).filter((c) => {
    const key = JSON.stringify([c.expression, c.expected]);
    if (keys.has(key)) return false;
    keys.add(key);
    return true;
  })];
}, insect: importInsect, kalker: importKalker, hurmet: importHurmet, speedcrunch: importSpeedcrunch };

function encode(text) {
  return text.replace(/\\/g, '\\\\').replace(/\t/g, '\\t').replace(/\n/g, '\\n');
}

export function revisionOf(root) {
  const git = (...args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  if (resolve(git('rev-parse', '--show-toplevel')) !== resolve(root)) throw new Error('Checkout must be a repository root');
  if (git('status', '--porcelain', '--untracked-files=all')) throw new Error('Checkout is dirty (uncommitted or untracked files)');
  return { revision: git('rev-parse', 'HEAD'), date: git('show', '-s', '--format=%cI', 'HEAD') };
}

export function importCorpus(options) {
  const lines = [
    '# Generated by scripts/import-competitor-corpus.mjs from upstream test suites.',
    '# Columns: source, lang, expression, expected, note. Upstream data is not relicensed;',
    '# each block cites the suite and its original license. Used as an executable reference.',
  ];
  const lock = {};
  const table = [
    '| Engine | Upstream commit | Commit date | Original license | Imported artifact | Rows |',
    '| --- | --- | --- | --- | --- | ---: |',
  ];
  for (const [name, meta] of Object.entries(SOURCES)) {
    const root = join(options.checkouts, name);
    try {
      const { revision, date } = revisionOf(root);
      const cases = IMPORTERS[name](root);
      if (!cases.length) throw new Error('No literal assertions found; inspect upstream format changes');
      const repo = meta.url.split('/').slice(0, 5).join('/');
      const pinnedUrl = meta.url.replace(/\/(blob|tree)\/[^/]+\//, `/$1/${revision}/`);
      lock[name] = { repo, revision, date };
      lines.push('', `# ${name}: ${meta.description} (${meta.license}) ${meta.url}`,
        `# Upstream commit: ${revision}; commit date: ${date}; pinned source: ${pinnedUrl}`);
      for (const c of cases) lines.push([c.source, c.lang, encode(c.expression), encode(c.expected), encode(c.note)].join('\t'));
      table.push(`| [${name}](${repo}) | [${revision}](${repo}/commit/${revision}) | ${date} | ${meta.license} | [${meta.description}](${pinnedUrl}) | ${cases.length} |`);
      console.log(`${name}: ${cases.length} cases (${revision})`);
      if (options.verbose) console.log(`  ${root}: ${meta.description}; ${date}; ${meta.license}`);
    } catch (error) {
      throw new Error(`${name}: ${error.message}`);
    }
  }
  // Validate every suite before writing any output: missing repos or changed
  // assertion formats must never silently replace the corpus with a subset.
  const start = '<!-- upstream-tests:start -->';
  const end = '<!-- upstream-tests:end -->';
  const generated = `${start}\n${table.join('\n')}\n${end}`;
  let inventory = existsSync(options.inventory) ? readFileSync(options.inventory, 'utf8') : `# Competitor inventory for issue #227

${start}
${end}

Generated from clean upstream checkouts by the corpus importer. Dates are Git
commit dates, not refresh times. The lock file, TSV headers and commit links
identify the exact revisions. Test data retains each upstream license.

Coverage is measured separately in [results/coverage-report.md](results/coverage-report.md).
See [the refresh guide](../../competitor-corpus.md) for commands and extraction limits.

GNU Units (manual examples), Numara (uses math.js) and Mathics are outside this
suite import. Closed-source documentation and multilingual quotes remain in
separate corpus files and are not changed by the importer.
`;
  const begin = inventory.indexOf(start);
  const finish = inventory.indexOf(end);
  if (begin < 0 || finish < begin) throw new Error('Inventory is missing upstream-tests markers');
  inventory = inventory.slice(0, begin) + generated + inventory.slice(finish + end.length);
  for (const [path, content] of [[options.out, `${lines.join('\n')}\n`], [options.inventory, inventory], [options.lock, `${JSON.stringify(lock, null, 2)}\n`]]) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, content);
  }
  console.log(`written: ${options.out}, ${options.inventory}, ${options.lock}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const options = parseArgs(process.argv.slice(2));
    if (options) importCorpus(options);
    else console.log('Usage: node scripts/import-competitor-corpus.mjs --checkouts <dir> [--out <tsv>] [--inventory <md>] [--lock <json>] [--verbose]');
  } catch (error) {
    console.error(`import-competitor-corpus: ${error.message}`);
    process.exitCode = 2;
  }
}
