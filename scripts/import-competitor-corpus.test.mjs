import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync, execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { parseCorpus } from './competitor-coverage.mjs';

// Minimal examples of the actual upstream formats, including traps that must
// never become false expectations (parser output, variables, errors, comments).
const fixtures = {
  fend: { 'core/tests/integration_tests.rs': 'test_eval("2 + 2", "4");' },
  qalculate: { 'tests/operators.batch': '2 + 2\n\t4\n' },
  numbat: { 'numbat/tests/interpreter.rs': 'expect_output("2 + 2", "4");' },
  rink: { 'core/tests/query.rs': 'test("2 + 2", "4");' },
  mathjs: {
    'test/unit-tests/expression/parse.test.js': "assert.strictEqual(parseAndEval('2 + 2'), 4)",
    'test/unit-tests/function/arithmetic/sqrt.test.js': `
      const sqrt = math.sqrt
      assert.strictEqual(sqrt(4), 2)
      assert.strictEqual(sqrt(value), 3)
      assert.strictEqual(sqrt(9), answer)
      // assert.strictEqual(sqrt(16), 99)
      assert.strictEqual(math.evaluate('3 + 3'), 6)
    `,
    'test/unit-tests/type/unit/Unit.test.js': `
      const originalConfig = math.config()
      assert.strictEqual(new Unit(5, 'cm').toString(), '5 cm')
      assert.strictEqual(new Unit(5, 'cm').value, 0.05)
      Unit.createUnitSingle('cm', { definition: '20 m', override: true })
      assert.strictEqual(new Unit(5, 'cm').toString(), '99 cm')
      assert.strictEqual(math.evaluate('2 cm to m'), 40)
    `,
  },
  insect: { 'test/Main.purs': `
    expectOutput' "4" "2 + 2"
    expectOutput env4 "3" "f(1)"
    expectOutput' "Parse error at position 4" "3kg+"
    -- expectOutput' "99" "2 + 2"
  ` },
  kalker: {
    'kalk/src/integration_testing.rs': '#[test_case("groups")]\n#[test_case("variables")]\nfn test_file() {}',
    'tests/groups.kalker': '|-3| + floor(2.6) = 5 and\nceil(4.2) = 5',
    'tests/variables.kalker': 'x = 2\nx + 2 = 4',
  },
  hurmet: { 'test/test.cjs': `
    const parserTests = [["2+2", "\\\\mathrm{four}"]]
    const calcTests = [
      ["2 + 2 = @", "RPN", "4"],
      [\x60sqrt(4) = @\x60, \x60RPN\x60, "2"],
      ["x + 2 = @", "¿x RPN", "4"],
      ["1/0 = @", "RPN", "Error. Divide by zero."],
      // ["2 + 2 = @", "RPN", "99"],
    ];
  ` },
  speedcrunch: { 'src/tests/testevaluator.cpp': `
    void test_defaults() {
      CHECK_EVAL("2 + 2", "4");
      CHECK_EVAL(QStringLiteral("sqrt(4)"), QLatin1String("2"));
      // CHECK_EVAL("2 + 2", "99");
    }
    void test_degrees() {
      settings->angleUnit = Degree;
      CHECK_EVAL("sin(90)", "99");
    }
    void test_variables() {
      CHECK_EVAL("x=2", "2");
      CHECK_EVAL("x+2", "4");
    }
  ` },
};

function fixtureCheckouts(t) {
  const dir = mkdtempSync(join(tmpdir(), 'import-corpus-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const revisions = {};
  for (const [name, files] of Object.entries(fixtures)) {
    const root = join(dir, name);
    for (const [file, content] of Object.entries(files)) {
      mkdirSync(dirname(join(root, file)), { recursive: true });
      writeFileSync(join(root, file), content);
    }
    const git = (...args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim();
    git('init', '--quiet');
    git('add', '.');
    git('-c', 'user.name=Corpus Test', '-c', 'user.email=corpus@example.com', 'commit', '--quiet', '-m', 'fixture');
    revisions[name] = { sha: git('rev-parse', 'HEAD'), date: git('show', '-s', '--format=%cI', 'HEAD') };
  }
  return { dir, revisions };
}

function runImport(dir, args = []) {
  return spawnSync(process.execPath, [resolve('scripts/import-competitor-corpus.mjs'),
    '--checkouts', dir, '--out', join(dir, 'corpus.tsv'),
    '--inventory', join(dir, 'inventory.md'), '--lock', join(dir, 'upstreams.json'), ...args],
  { encoding: 'utf8', timeout: 20000 });
}

test('imports all nine suites with pinned provenance and literal expectations', { timeout: 30000 }, (t) => {
  const { dir, revisions } = fixtureCheckouts(t);
  const run = runImport(dir);
  assert.ifError(run.error);
  assert.equal(run.status, 0, run.stderr);
  const cases = parseCorpus(join(dir, 'corpus.tsv'));
  for (const source of ['fend', 'Qalculate!', 'Numbat', 'Rink', 'math.js', 'Insect', 'kalker', 'Hurmet', 'SpeedCrunch']) {
    assert(cases.some((row) => row.source === source), source);
  }
  const has = (source, expression, expected) => cases.some((row) => row.source === source && row.expression === expression && row.expected === expected);
  assert(has('math.js', 'sqrt(4)', '2'));
  assert(has('math.js', '3 + 3', '6'));
  assert(has('math.js', 'unit(5, "cm")', '5 cm'));
  assert(has('kalker', '|-3| + floor(2.6)', '5'));
  assert(has('kalker', 'ceil(4.2)', '5'));
  assert(has('Hurmet', '2 + 2', '4'));
  assert(has('Hurmet', 'sqrt(4)', '2'));
  assert(!cases.some((row) => row.expected === '99' || /error|mathrm/i.test(row.expected)));
  assert(!cases.some((row) => /x \+|f\(1\)|value/.test(row.expression)));
  assert(!cases.some((row) => row.expected === '99 cm' || row.expression === '2 cm to m'));
  const header = readFileSync(join(dir, 'corpus.tsv'), 'utf8');
  const inventory = readFileSync(join(dir, 'inventory.md'), 'utf8');
  const lock = JSON.parse(readFileSync(join(dir, 'upstreams.json'), 'utf8'));
  for (const [name, { sha, date }] of Object.entries(revisions)) {
    assert(header.includes(sha) && header.includes(date), name);
    assert(inventory.includes(sha) && inventory.includes(date), name);
    assert.equal(lock[name].revision, sha);
    assert.equal(lock[name].date, date);
  }
  assert.match(header, /GPL-2.0-or-later/);
  assert.match(header, /Apache-2.0/);
  assert.match(header, /MPL-2.0/);
  const before = [header, inventory, readFileSync(join(dir, 'upstreams.json'), 'utf8')];
  assert.equal(runImport(dir).status, 0);
  assert.deepEqual(['corpus.tsv', 'inventory.md', 'upstreams.json'].map((file) => readFileSync(join(dir, file), 'utf8')), before);
});

test('rejects dirty checkouts before overwriting generated artifacts', { timeout: 30000 }, (t) => {
  const { dir } = fixtureCheckouts(t);
  writeFileSync(join(dir, 'fend/core/tests/integration_tests.rs'), 'test_eval("2 + 2", "99");');
  writeFileSync(join(dir, 'corpus.tsv'), 'original');
  const run = runImport(dir);
  assert.equal(run.status, 2);
  assert.match(run.stderr, /dirty|uncommitted/i);
  assert.equal(readFileSync(join(dir, 'corpus.tsv'), 'utf8'), 'original');
});

test('rejects a missing or unrecognized suite without deleting the previous corpus', { timeout: 30000 }, (t) => {
  const { dir } = fixtureCheckouts(t);
  rmSync(join(dir, 'speedcrunch'), { recursive: true });
  writeFileSync(join(dir, 'corpus.tsv'), 'original');
  const run = runImport(dir);
  assert.equal(run.status, 2);
  assert.match(run.stderr, /speedcrunch/i);
  assert.equal(readFileSync(join(dir, 'corpus.tsv'), 'utf8'), 'original');
});

test('fails on an empty extractor and preserves all previous outputs', { timeout: 30000 }, (t) => {
  const { dir } = fixtureCheckouts(t);
  const root = join(dir, 'insect');
  writeFileSync(join(root, 'test/Main.purs'), 'expectNewOutput "4" "2 + 2"');
  execFileSync('git', ['-C', root, 'add', '.']);
  execFileSync('git', ['-C', root, '-c', 'user.name=Corpus Test', '-c', 'user.email=corpus@example.com', 'commit', '--quiet', '-m', 'changed assertion format']);
  for (const file of ['corpus.tsv', 'inventory.md', 'upstreams.json']) writeFileSync(join(dir, file), 'original');
  const run = runImport(dir);
  assert.equal(run.status, 2);
  assert.match(run.stderr, /insect: No literal assertions found/);
  for (const file of ['corpus.tsv', 'inventory.md', 'upstreams.json']) assert.equal(readFileSync(join(dir, file), 'utf8'), 'original');
});
