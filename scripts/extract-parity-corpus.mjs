#!/usr/bin/env node

/**
 * Collects the parity corpus: every one-line string literal in the Rust test
 * code (`tests/*.rs`, `src/**\/*_tests.rs` and the `#[cfg(test)]` part of
 * every other source file), plus the examples in `data/examples.lino`.
 *
 * Both engines evaluate each corpus input in each scenario of
 * `parity/corpus.json`; the Rust test `tests/parity_goldens.rs` records the
 * results in `parity/goldens/`, and the JavaScript tests compare against them.
 * Assertion, panic and format messages are skipped; any other literal that is
 * not an expression still makes a useful case, since both engines must reject
 * it with the same error.
 *
 * Usage:
 *   node scripts/extract-parity-corpus.mjs           # fail if inputs are missing
 *   node scripts/extract-parity-corpus.mjs --write   # add the missing inputs
 *
 * PARITY_VERBOSE=1 lists every input that was found and where.
 */
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const MAX_LENGTH = 200;
/** The harness that records the goldens holds no calculator inputs. */
const HARNESS = 'tests/parity_goldens.rs';

/**
 * Decode the escapes of a Rust (non-raw) string literal body.
 * @param {string} body
 */
export function unescapeRust(body) {
  return body.replace(/\\(u\{([0-9a-fA-F]+)\}|x([0-9a-fA-F]{2})|\n\s*|.)/g, (_, escape, unicode, hex) => {
    if (unicode) {
      return String.fromCodePoint(Number.parseInt(unicode, 16));
    }
    if (hex) {
      return String.fromCharCode(Number.parseInt(hex, 16));
    }
    if (escape.startsWith('\n')) {
      return '';
    }
    return { n: '\n', r: '\r', t: '\t', 0: '\0', '\\': '\\', '"': '"', "'": "'" }[escape] ?? escape;
  });
}

const MESSAGE_ARGUMENT = new Map([
  ['assert!', 1],
  ['debug_assert!', 1],
  ['assert_eq!', 2],
  ['assert_ne!', 2],
  ['debug_assert_eq!', 2],
  ['debug_assert_ne!', 2],
  ['write!', 1],
  ['writeln!', 1],
  ['panic!', 0],
  ['print!', 0],
  ['println!', 0],
  ['eprint!', 0],
  ['eprintln!', 0],
  ['format!', 0],
  ['unreachable!', 0],
  ['todo!', 0],
  ['unimplemented!', 0],
  ['expect', 0],
]);

/**
 * The macro or function name right before an opening bracket at `index`.
 * @param {string} source
 * @param {number} index
 */
function callName(source, index) {
  const match = /([A-Za-z_][\w]*!?)\s*$/.exec(source.slice(Math.max(0, index - 64), index));
  return match ? match[1] : '';
}

/**
 * Every string literal of a Rust source text, decoded, in source order, with
 * the call it is an argument of. Comments and character literals are
 * skipped.
 * @param {string} source
 * @returns {Array<{ value: string, call: string, argument: number }>}
 */
export function rustStringLiterals(source) {
  const literals = [];
  const frames = [{ call: '', argument: 0 }];
  const push = (value) => {
    const { call, argument } = frames.at(-1);
    literals.push({ value, call, argument });
  };
  let index = 0;
  while (index < source.length) {
    const rest = source.slice(index, index + 16);
    const char = source[index];
    if (rest.startsWith('//')) {
      const end = source.indexOf('\n', index);
      index = end === -1 ? source.length : end + 1;
      continue;
    }
    if (rest.startsWith('/*')) {
      const end = source.indexOf('*/', index + 2);
      index = end === -1 ? source.length : end + 2;
      continue;
    }
    const raw = /^b?r(#*)"/.exec(rest);
    if (raw && !/[\w]/.test(source[index - 1] ?? '')) {
      const terminator = `"${raw[1]}`;
      const start = index + raw[0].length;
      const end = source.indexOf(terminator, start);
      push(source.slice(start, end));
      index = end + terminator.length;
      continue;
    }
    if (char === '"') {
      let end = index + 1;
      while (end < source.length && source[end] !== '"') {
        end += source[end] === '\\' ? 2 : 1;
      }
      push(unescapeRust(source.slice(index + 1, end)));
      index = end + 1;
      continue;
    }
    const character = /^'(\\u\{[0-9a-fA-F]+\}|\\.|[^\\'])'/u.exec(source.slice(index, index + 12));
    if (character) {
      index += character[0].length;
      continue;
    }
    if (char === '(' || char === '[' || char === '{') {
      frames.push({ call: char === '(' ? callName(source, index) : '', argument: 0 });
    } else if ((char === ')' || char === ']' || char === '}') && frames.length > 1) {
      frames.pop();
    } else if (char === ',') {
      frames.at(-1).argument += 1;
    }
    index += 1;
  }
  return literals;
}

/**
 * Whether a literal is an assertion, panic or format message rather than
 * data: `assert!(cond, "message")`, `.expect("message")`, `format!("...")`.
 * @param {{ call: string, argument: number }} literal
 */
export function isMessage({ call, argument }) {
  const first = MESSAGE_ARGUMENT.get(call);
  return first !== undefined && argument >= first;
}

/**
 * The part of a source file that is test code.
 * @param {string} path repository-relative path
 * @param {string} source
 */
export function testSection(path, source) {
  if (path.startsWith('tests/') || path.endsWith('_tests.rs') || path.endsWith('/tests.rs')) {
    return source;
  }
  const marker = source.indexOf('#[cfg(test)]');
  return marker === -1 ? '' : source.slice(marker);
}

/**
 * Whether a literal is worth evaluating as a calculator input: one line, not
 * too long and not a format template (`{}`, `{v}`, `{:?}`).
 * @param {string} literal
 */
export function isCandidate(literal) {
  return literal.trim() !== ''
    && !literal.includes('\n')
    && !/\{\w*(:[^}]*)?\}/.test(literal)
    && [...literal].length <= MAX_LENGTH;
}

function rustFiles(directory) {
  return readdirSync(join(root, directory), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map((entry) => relative(root, join(entry.parentPath, entry.name)))
    .sort();
}

/**
 * The inputs listed in `data/examples.lino` (`(expression "...")` links).
 * @param {string} source
 */
export function exampleInputs(source) {
  return [...source.matchAll(/\(expression "((?:[^"\\]|\\.)*)"\)/g)].map((match) => match[1].replace(/\\(.)/g, '$1'));
}

/**
 * @returns {Map<string, string>} input -> the first file it was found in
 */
export function collectInputs() {
  const inputs = new Map();
  for (const path of [...rustFiles('tests'), ...rustFiles('src')].filter((file) => file !== HARNESS)) {
    const section = testSection(path, readFileSync(join(root, path), 'utf8'));
    for (const literal of rustStringLiterals(section)) {
      if (!isMessage(literal) && isCandidate(literal.value) && !inputs.has(literal.value)) {
        inputs.set(literal.value, path);
      }
    }
  }
  for (const input of exampleInputs(readFileSync(join(root, 'data', 'examples.lino'), 'utf8'))) {
    if (isCandidate(input) && !inputs.has(input)) {
      inputs.set(input, 'data/examples.lino');
    }
  }
  return inputs;
}

function main(argv) {
  const write = argv.includes('--write');
  const corpusPath = join(root, 'parity', 'corpus.json');
  const corpus = JSON.parse(readFileSync(corpusPath, 'utf8'));
  const known = new Set(corpus.inputs);
  const found = collectInputs();
  if (process.env.PARITY_VERBOSE === '1') {
    for (const [input, path] of found) {
      console.log(`${path}: ${JSON.stringify(input)}`);
    }
  }
  const missing = [...found.keys()].filter((input) => !known.has(input));
  if (missing.length === 0) {
    console.log(`parity/corpus.json covers all ${found.size} test inputs (${known.size} in total).`);
    return 0;
  }
  if (write) {
    corpus.inputs = [...corpus.inputs, ...missing];
    writeFileSync(corpusPath, `${JSON.stringify(corpus, null, 2)}\n`);
    console.log(`Added ${missing.length} input(s) to parity/corpus.json; regenerate the goldens with:`);
    console.log('  UPDATE_PARITY_GOLDENS=1 node scripts/bounded-cargo.mjs test --test parity_goldens');
    return 0;
  }
  console.log(`::error title=Parity corpus::${missing.length} test input(s) are missing from parity/corpus.json. Run node scripts/extract-parity-corpus.mjs --write and regenerate the goldens.`);
  for (const input of missing.slice(0, 50)) {
    console.log(`  ${found.get(input)}: ${JSON.stringify(input)}`);
  }
  return 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
