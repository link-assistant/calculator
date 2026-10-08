import { readFileSync, readdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { balancedEnd, calls, displayLiteral, singleLine, splitArguments, stringLiteral, stripComments } from './corpus-literals.mjs';

function filesUnder(root, extension) {
  const files = [];
  for (const entry of readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, 'en'))) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path, extension));
    else if (entry.isFile() && entry.name.endsWith(extension)) files.push(path);
  }
  return files;
}

function row(source, expression, expected, note) {
  return { source, lang: 'en', expression, expected, note };
}

/** Translate a literal math.js API call to its expression-language equivalent. */
function mathExpression(text, aliases) {
  text = text.trim();
  const str = stringLiteral(text);
  if (str !== null) return JSON.stringify(str);
  if (displayLiteral(text) !== null && !text.startsWith('[')) return text;
  if (text.startsWith('[') && balancedEnd(text, 0) === text.length) {
    const elements = splitArguments(text.slice(1, -1)).map((arg) => mathExpression(arg, aliases));
    return elements.every((element) => element !== null) ? `[${elements.join(', ')}]` : null;
  }
  // Display assertions have the same meaning as evaluating the Unit object.
  text = text.replace(/\.toString\(\)$/, '');
  const match = text.match(/^(?:new\s+)?((?:math\.)?[\w]+)\s*\(/);
  if (!match) return null;
  const start = match[0].lastIndexOf('(');
  if (balancedEnd(text, start) !== text.length) return null;
  const name = match[1].startsWith('math.') ? match[1].slice(5) : aliases.get(match[1]);
  if (!name) return null;
  const args = splitArguments(text.slice(start + 1, -1));
  if (name === 'evaluate' || name === 'parseAndEval') return args.length === 1 ? stringLiteral(args[0]) : null;
  const values = args.map((arg) => mathExpression(arg, aliases));
  if (!values.length || values.some((value) => value === null)) return null;
  return `${name === 'Unit' ? 'unit' : name}(${values.join(', ')})`;
}

export function importMathjsAdditional(root) {
  const cases = [];
  // The legacy parse extractor remains first to preserve the existing baseline.
  const dirs = ['test/unit-tests/expression', 'test/unit-tests/function', 'test/unit-tests/type/unit'];
  for (const dir of dirs) {
    for (const file of filesUnder(join(root, dir), '.test.js')) {
      const text = stripComments(readFileSync(file, 'utf8'));
      const aliases = new Map([['parseAndEval', 'parseAndEval'], ['Unit', 'Unit']]);
      for (const match of text.matchAll(/\b(?:const|let|var)\s+(\w+)\s*=\s*math\.(\w+)\b/g)) aliases.set(match[1], match[2]);
      // Once the shared instance/configuration is mutated, later assertions
      // may depend on custom units, overridden builtins or mode changes. Do
      // not try to execute hooks or infer whether upstream resets that state.
      let independentUntil = text.length;
      for (const { args, index } of calls(text, ['math\\.(?:config|import|createUnit)', 'Unit\\.(?:createUnit|createUnitSingle|deleteUnit)'])) {
        if (args.length === 0 && text.slice(index).startsWith('math.config')) continue;
        independentUntil = index;
        break;
      }
      for (const { args, line, index } of calls(text, ['assert\\.(?:strictEqual|equal|deepStrictEqual|deepEqual)', 'approxEqual', 'approxDeepEqual'])) {
        if (index >= independentUntil) break;
        if (args.length < 2 || args.length > 3) continue;
        const expression = mathExpression(args[0], aliases);
        const expected = displayLiteral(args[1]);
        if (expression !== null && expected !== null && singleLine(expression) && singleLine(expected)) {
          cases.push(row('math.js', expression, expected, `${relative(root, file)}:${line}; literal API assertion`));
        }
      }
    }
  }
  return cases;
}

export function importInsect(root) {
  const file = 'test/Main.purs';
  const text = stripComments(readFileSync(join(root, file), 'utf8'), 'purs');
  const literal = '"((?:[^"\\\\]|\\\\.)*)"';
  const pattern = new RegExp(`\\bexpectOutput'\\s+${literal}\\s+${literal}`, 'g');
  const cases = [];
  for (const match of text.matchAll(pattern)) {
    const expected = stringLiteral(`"${match[1]}"`);
    const expression = stringLiteral(`"${match[2]}"`);
    if (singleLine(expression) && singleLine(expected) && !/error|Unknown identifier|Wrong number/i.test(expected)) {
      cases.push(row('Insect', expression, expected, `${file}:${text.slice(0, match.index).split('\n').length}; initialEnvironment`));
    }
  }
  return cases;
}

/** The Rust integration harness enumerates files, each of which must yield true. */
export function importKalker(root) {
  const harness = readFileSync(join(root, 'kalk/src/integration_testing.rs'), 'utf8');
  const cases = [];
  for (const match of harness.matchAll(/#\[test_case\("([\w/-]+)"\)\]/g)) {
    const file = `tests/${match[1]}.kalker`;
    const text = readFileSync(join(root, file), 'utf8');
    const defined = new Set([...text.matchAll(/^\s*([\p{L}_][\p{L}\p{N}_]*)\s*(?:\(\s*[\p{L}_][\p{L}\p{N}_]*(?:\s*,\s*[\p{L}_][\p{L}\p{N}_]*)*\s*\))?\s*=(?!=)/gmu)].map((m) => m[1]));
    text.split(/\r?\n/).forEach((line, index) => {
      // Upstream combines independent checks with `and`; the corpus runs each.
      for (const check of line.split(/\s+and\s*$/).filter(Boolean)) {
        const pair = check.trim().match(/^(.*?)\s+=(?!=)\s+([-+]?\d+(?:\.\d+)?(?:e[-+]?\d+)?|true|false)$/i);
        if (!pair || !singleLine(pair[1]) || pair[1].trim().startsWith('#')) continue;
        const identifiers = pair[1].match(/[\p{L}_][\p{L}\p{N}_]*/gu) || [];
        if (identifiers.some((name) => defined.has(name))) continue;
        cases.push(row('kalker', pair[1], pair[2], `${file}:${index + 1}; equality assertion`));
      }
    });
  }
  return cases;
}

export function importHurmet(root) {
  const file = 'test/test.cjs';
  const text = stripComments(readFileSync(join(root, file), 'utf8'));
  const match = /\bconst\s+calcTests\s*=\s*\[/.exec(text);
  if (!match) throw new Error('Hurmet calculation table calcTests not found');
  const start = match.index + match[0].lastIndexOf('[');
  const table = text.slice(start + 1, balancedEnd(text, start) - 1);
  const cases = [];
  for (const entry of splitArguments(table)) {
    if (!entry.startsWith('[') || balancedEnd(entry, 0) !== entry.length) continue;
    const values = splitArguments(entry.slice(1, -1)).map(stringLiteral);
    if (values.length !== 3 || values.some((value) => value === null)) continue;
    const [input, rpn, expected] = values;
    // RPN marks references to the shared vars object with ¿. Render-only,
    // formatting directives, error assertions and user functions are excluded.
    const expression = input.match(/^(.*?)\s*=\s*@{1,2}\s*$/s)?.[1];
    if (!expression || rpn.includes('¿') || /Error\./i.test(expected)) continue;
    if (singleLine(expression) && singleLine(expected)) {
      cases.push(row('Hurmet', expression.trim(), expected, `${file}; calcTests; input: ${input}`));
    }
  }
  return cases;
}

export function importSpeedcrunch(root) {
  const cases = [];
  for (const file of filesUnder(join(root, 'src/tests'), '.cpp')) {
    const text = stripComments(readFileSync(file, 'utf8'), 'cpp');
    // Treat each test function as a unit so mode changes and user definitions
    // cannot silently contaminate expectations imported in the default mode.
    const boundaries = [...text.matchAll(/\bvoid\s+test_\w+\s*\([^)]*\)\s*\{/g)].map((m) => m.index);
    boundaries.unshift(0);
    boundaries.push(text.length);
    for (let i = 0; i < boundaries.length - 1; i++) {
      const section = text.slice(boundaries[i], boundaries[i + 1]);
      if (/settings\s*->|CHECK_USERFUNC_SET|setVariable\s*\(/.test(section)) continue;
      const pairs = [...calls(section, ['CHECK_EVAL', 'CHECK_EVAL_PRECISE'])];
      const definitions = new Set();
      for (const { args } of pairs) {
        const expression = stringLiteral(args[0]);
        const name = expression?.match(/^\s*([\p{L}_][\p{L}\p{N}_]*)\s*=(?!=)/u)?.[1];
        if (name) definitions.add(name);
      }
      for (const { args, line } of pairs) {
        if (args.length !== 2) continue;
        const expression = stringLiteral(args[0]);
        const expected = stringLiteral(args[1]);
        if (expression === null || expected === null || !singleLine(expression) || !singleLine(expected)) continue;
        if (/(?:^|[^<>=!])=(?!=)|\bans\b|\brandom\s*\(/i.test(expression)) continue;
        if ((expression.match(/[\p{L}_][\p{L}\p{N}_]*/gu) || []).some((name) => definitions.has(name))) continue;
        cases.push(row('SpeedCrunch', expression, expected, `${relative(root, file)}:${text.slice(0, boundaries[i]).split('\n').length + line - 1}`));
      }
    }
  }
  return cases;
}
