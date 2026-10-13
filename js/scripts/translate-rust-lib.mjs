/**
 * Pure helpers of translate-rust.mjs (issue #254 R2): module ordering, output
 * splitting, the ledger and allowlist formats, item naming, diagnostic
 * normalisation and the mapping of a diagnostic to its upstream issue.
 * Nothing here reads the file system or loads meta-language.
 */

import { createHash } from 'node:crypto';
import { dirname, posix } from 'node:path';

/** The largest number of lines a committed `.rs` file may have (scripts/check-file-size.mjs). */
export const MAX_LINES = 1000;

export const UPSTREAM_REPO = 'https://github.com/link-foundation/meta-language';
export const ISSUE_URL = (number) => `${UPSTREAM_REPO}/issues/${number}`;

/** The carried reason meta-language gives a free-standing comment it cannot write. */
export const FREE_COMMENT = 'comment the target cannot hold';

/** @param {string | Buffer} text */
export const sha256 = (text) => createHash('sha256').update(text).digest('hex');

// ---------------------------------------------------------------------------
// Patching the pinned meta-language module (loaded by the worker's load hook)
// ---------------------------------------------------------------------------

/**
 * Source edits to meta-language's `src/self-translation.js` at the pinned
 * commit. They change no output: they record the checker's diagnostic of each
 * carried item next to its reason (meta-language#214 drops it), and export
 * `translateModule`, which gives the translation and the signatures from one
 * run instead of two (meta-language#215).
 */
export const SELF_TRANSLATION_PATCHES = Object.freeze([
  {
    find: 'return carry(text, term, from, error.kind);',
    replace: 'return { ...carry(text, term, from, error.kind), diagnostic: error.reason ?? error.message };',
  },
  {
    find: 'recorded.push({ term, start, end, status: out.status, reason: out.reason ?? null });',
    replace: 'recorded.push({ term, start, end, status: out.status, reason: out.reason ?? null, diagnostic: out.diagnostic ?? null });',
  },
]);

/**
 * Applies {@link SELF_TRANSLATION_PATCHES}; each must match exactly once.
 * @param {string} source
 * @returns {string}
 */
export function patchSelfTranslation(source) {
  let out = source;
  for (const { find, replace } of SELF_TRANSLATION_PATCHES) {
    const count = out.split(find).length - 1;
    if (count !== 1) {
      throw new Error(`meta-language self-translation.js: expected one "${find}", found ${count}; update SELF_TRANSLATION_PATCHES for the new pin`);
    }
    out = out.replace(find, () => replace);
  }
  return `${out}\nexport { translateModule as __translateModule };\n`;
}

// ---------------------------------------------------------------------------
// Modules and their order
// ---------------------------------------------------------------------------

/**
 * Relative specifiers a module imports or re-exports, in source order.
 * @param {string} text
 * @returns {string[]}
 */
export function relativeSpecifiers(text) {
  return [...text.matchAll(/^\s*(?:import|export)[^'"]*?from\s+['"](\.[^'"]+)['"]/gm)].map((m) => m[1]);
}

/**
 * Resolves a relative specifier of module `from` (both relative to js/src).
 * @param {string} from
 * @param {string} specifier
 */
export function resolveSpecifier(from, specifier) {
  return posix.normalize(posix.join(posix.dirname(from), specifier));
}

/**
 * The translation plan: modules in import order (depth first, as the
 * experiment's run.mjs), each with the specifiers bound to modules translated
 * before it. An import that closes a cycle is not bound, so the result is the
 * same whatever order the workers finish in.
 * @param {Map<string, string>} sources module path (relative to js/src) -> text
 * @returns {{ order: string[], deps: Map<string, Array<{ specifier: string, module: string }>> }}
 */
export function planModules(sources) {
  const names = [...sources.keys()].sort();
  const order = [];
  const seen = new Set();
  const edges = new Map();
  for (const name of names) {
    edges.set(name, relativeSpecifiers(sources.get(name))
      .map((specifier) => ({ specifier, module: resolveSpecifier(name, specifier) }))
      .filter(({ module }) => sources.has(module)));
  }
  const visit = (name) => {
    if (seen.has(name)) return;
    seen.add(name);
    for (const { module } of edges.get(name)) visit(module);
    order.push(name);
  };
  names.forEach(visit);
  const position = new Map(order.map((name, index) => [name, index]));
  const deps = new Map();
  for (const name of order) {
    const bound = [];
    for (const edge of edges.get(name)) {
      if (position.get(edge.module) < position.get(name) && !bound.some((b) => b.specifier === edge.specifier)) bound.push(edge);
    }
    deps.set(name, bound);
  }
  return { order, deps };
}

/**
 * Scheduling priority: a module's own cost plus the costliest chain of
 * modules that wait for it, so the critical path starts first.
 * @param {string[]} order
 * @param {Map<string, Array<{ module: string }>>} deps
 * @param {(name: string) => number} cost
 * @returns {Map<string, number>}
 */
export function criticalPath(order, deps, cost) {
  const dependents = new Map(order.map((name) => [name, []]));
  for (const name of order) for (const { module } of deps.get(name)) dependents.get(module).push(name);
  const rank = new Map();
  for (const name of [...order].reverse()) {
    rank.set(name, cost(name) + Math.max(0, ...dependents.get(name).map((d) => rank.get(d))));
  }
  return rank;
}

/** The crate path of a module: `rust/libm/log.js` -> `['rust', 'libm']`. */
export function moduleDirectory(name) {
  const dir = dirname(name);
  return dir === '.' ? [] : dir.split('/');
}

// ---------------------------------------------------------------------------
// Output files
// ---------------------------------------------------------------------------

/** `rust/libm/log.js` -> `rust/libm/log` */
export const outputStem = (name) => name.replace(/\.m?js$/, '');

/** The file name of part `index` (0-based) of a module's output. */
export const partName = (name, index) => `${outputStem(name)}${index === 0 ? '' : `.part${index + 1}`}.rs`;

const BLOCK_START = /^\/\/ meta-language:(translated|carried) /;

/**
 * Splits a module's Rust into parts of at most `maxLines` lines each (as
 * check-file-size.mjs counts them), cutting before a translated or carried
 * marker when it can. Concatenating the parts gives the input back.
 * @param {string} code
 * @param {number} [maxLines]
 * @returns {string[]}
 */
export function splitOutput(code, maxLines = MAX_LINES) {
  const lines = code.split('\n');
  if (lines.length <= maxLines) return [code];
  // A part holds `size` lines plus its final line break: size + 1 <= maxLines.
  const size = maxLines - 1;
  const parts = [];
  let start = 0;
  while (lines.length - start > maxLines) {
    let cut = start + size;
    for (let at = start + size; at > start; at -= 1) {
      if (BLOCK_START.test(lines[at])) {
        cut = at;
        break;
      }
    }
    parts.push(`${lines.slice(start, cut).join('\n')}\n`);
    start = cut;
  }
  parts.push(lines.slice(start).join('\n'));
  return parts;
}

/**
 * The source sha256 and byte count in a self-translation header.
 * @param {string} code
 * @returns {{ sha256: string, bytes: number } | null}
 */
export function readHeader(code) {
  const m = code.match(/^\/\/ meta-language:self-translation:v1 source=JavaScript target=Rust sha256=([0-9a-f]{64}) bytes=(\d+)/);
  return m ? { sha256: m[1], bytes: Number(m[2]) } : null;
}

// ---------------------------------------------------------------------------
// Items, names and diagnostics
// ---------------------------------------------------------------------------

/**
 * A stable name for a top-level item, from its text: the declared name, or a
 * description of an import, re-export or statement.
 * @param {string} text
 * @param {string} term
 */
export function itemName(text, term) {
  const decl = text.match(/^(?:export\s+)?(?:default\s+)?(?:async\s+)?(?:function\s*\*?|class|const|let|var)\s+([A-Za-z_$][\w$]*)/);
  if (decl) return decl[1];
  const from = text.match(/\bfrom\s+['"]([^'"]+)['"]/);
  if (/^export\s*\*/.test(text) && from) return `export*:${from[1]}`;
  if (/^export\s*\{/.test(text) && from) return `export{}:${from[1]}`;
  if (/^export\s*\{/.test(text)) return 'export{}';
  if (/^import\b/.test(text) && from) return `import:${from[1]}`;
  if (/^import\s*['"]/.test(text)) return `import:${text.match(/['"]([^'"]+)['"]/)[1]}`;
  const call = text.match(/^([A-Za-z_$][\w$.]*)\s*[(=]/);
  if (call) return `${term}:${call[1]}`;
  return term;
}

/**
 * Names every item of a module and makes duplicates unique with `#2`, `#3`...
 * @param {Array<{ text: string, term: string }>} items
 * @returns {string[]}
 */
export function uniqueNames(items) {
  const counts = new Map();
  return items.map(({ text, term }) => {
    const base = itemName(text, term);
    const n = (counts.get(base) ?? 0) + 1;
    counts.set(base, n);
    return n === 1 ? base : `${base}#${n}`;
  });
}

/**
 * The construct of a diagnostic: the message without its span and with quoted
 * names replaced by X (the normalisation of the experiment's census.mjs).
 * @param {string | null} diagnostic
 * @param {string} reason
 */
export function construct(diagnostic, reason) {
  if (!diagnostic) return reason;
  return diagnostic
    .replace(/ at \d+\.\.\d+$/, '')
    // Keep the keyword or brace of `export "class"` / `export "{"`: it tells the construct.
    .replace(/^export "([^"\s]{1,8})"/, 'export $1')
    .replace(/`[^`]*`|'[^']*'|"[^"]*"/g, 'X')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * The name a cascade waits for, when the item is carried only because it
 * reads or calls an item that is itself carried, or null.
 * @param {string | null} diagnostic
 */
export function cascadeOf(diagnostic) {
  if (!diagnostic) return null;
  const m = diagnostic.match(/^(?:unknown name crate\.|function value )([A-Za-z_$][\w$]*)/)
    ?? diagnostic.match(/^unknown name ([A-Za-z_$][\w$]*)/);
  return m ? m[1] : null;
}

/**
 * Diagnostic -> upstream issue. Drafted issues that are not filed yet are
 * `drafted:<letter>` (experiments/issue-254/meta-language/README.md §5.2).
 */
const STRING_METHODS = 'charAt|charCodeAt|codePointAt|padStart|padEnd|replace|replaceAll|search|split|repeat|startsWith|endsWith|trim|trimStart|trimEnd|toLowerCase|toUpperCase|normalize|localeCompare|substring|match|matchAll';
const ARRAY_METHODS = 'map|filter|find|findIndex|some|every|push|pop|shift|unshift|join|reduce|forEach|includes|indexOf|sort|reverse|concat|fill|slice|splice|flatMap|flat|at|keys|values|entries';

/** @type {ReadonlyArray<[RegExp, string]>} */
export const UPSTREAM_RULES = Object.freeze([
  [/^export (\{|\*):/, 'drafted:G'],
  [/^namespace import:/, 'drafted:J'],
  [/^export class:|throw new \w+: throw an Error|^export \w+: only function and const|new expression|top-level class|\bclass\b|getter|setter|\bstatic\b|\bthis\b/, '210'],
  [/^assignment of [A-Z_a-z]+: only local variables/, 'drafted:H'],
  [/^top-level statement$/, 'drafted:H'],
  [/namespace property/, '206'],
  [/startsWith\(\) with 2/, '209'],
  [/destructured parameter|destructuring|without a value: an uninitialised let|case test|try statement|typeof|for with const/, '212'],
  [/expected string but found float|expected \w+ but found \w+ in an array/, '205'],
  [/used as a(n)? \w+ and as /, '213'],
  [/async|await|Promise/, '223'],
  [/unknown type crate\.never|import\(|Iterable</, 'drafted:C'],
  [/unknown type crate\.\w+/, 'drafted:B'],
  [new RegExp(`^${FREE_COMMENT}$`), 'drafted:A'],
  [/JSDoc type \{[^}]*null|\bnull\b|undefined|optional chaining|\?\?|nullish/, '204'],
  [/JSDoc type \{\[|JSDoc type \{\(|JSDoc type \{\{|Map<|Set<|Record<|@param needs|tuple/, '205'],
  [/JSDoc type \{[^}]*\|/, 'drafted:B'],
  [/JSDoc type \{(unknown|object|\*|any|Function|Object)\}/, '205'],
  [/JSDoc type \{[A-Z]\w*\}|JSDoc type \{\w+<|unknown type/, 'drafted:C'],
  [/object without a \$ tag|object literal|^@typedef \w+: a portable data type/, '206'],
  [/arrow function|function expression|function value/, '207'],
  [new RegExp(`method call \\.(${ARRAY_METHODS})\\(|Array\\.`), '208'],
  [new RegExp(`method call \\.(${STRING_METHODS})\\(|length of a string|String\\.`), '209'],
  // Methods of the engine's own classes.
  [/method call \.\w+\(/, '210'],
  [/regular expression|malformed number|unterminated|escape/, '211'],
  [/statement|label|spread|rest parameter/, '212'],
  [/bitwise|shift/, 'drafted:D'],
  [/exponentiation/, 'drafted:E'],
  [/toString with a radix|^(Number|BigInt|Math|Object|BigInt64Array|Float64Array|DataView|Symbol|Infinity|NaN)\b|standard library/, 'drafted:F'],
  [/^import \{|unknown name/, '220'],
  [/^no definition$/, '210'],
]);

/**
 * The upstream reference suggested for a construct, or null.
 * @param {string} text normalised construct
 */
export function suggestUpstream(text) {
  const rule = UPSTREAM_RULES.find(([re]) => re.test(text));
  if (!rule) return null;
  return rule[1].startsWith('drafted:') ? rule[1] : ISSUE_URL(rule[1]);
}

/** Whether an allowlist `upstream` value is acceptable. */
export const validUpstream = (value) => typeof value === 'string'
  && (/^https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+$/.test(value) || /^drafted:[A-Z]$/.test(value));

/**
 * The allowlist entries a translated module needs, from its items.
 * @param {string} module
 * @param {string} source module text
 * @param {Array<{ term: string, start: number, end: number, status: string, reason: string | null, diagnostic?: string | null }>} items
 */
export function requiredEntries(module, source, items) {
  const bytes = Buffer.from(source, 'utf8');
  const code = items.filter((item) => !/comment/.test(item.term));
  const names = uniqueNames(code.map((item) => ({ term: item.term, text: bytes.subarray(item.start, item.end).toString('utf8') })));
  const entries = [];
  code.forEach((item, index) => {
    if (item.status !== 'carried') return;
    const diagnostic = item.diagnostic ?? null;
    const needs = cascadeOf(diagnostic);
    if (needs) entries.push({ kind: 'cascade', module, name: names[index], needs });
    else entries.push({ kind: 'item', module, name: names[index], construct: construct(diagnostic, item.reason ?? 'carried') });
  });
  const comments = items.filter((item) => item.status === 'carried' && item.reason === FREE_COMMENT).length;
  if (comments > 0) entries.push({ kind: 'comments', module, count: comments });
  return entries;
}

/** Counts of a module's items by status and the allowlist entries they need. */
export function countItems(items) {
  const counts = { translated: 0, carried: 0, comment: 0 };
  for (const { status } of items) if (status in counts) counts[status] += 1;
  return counts;
}

export const entryKey = (entry) => `${entry.kind} ${entry.module}${entry.kind === 'comments' ? '' : ` ${entry.name}`}`;

// ---------------------------------------------------------------------------
// Links Notation records
// ---------------------------------------------------------------------------

const quote = (value) => JSON.stringify(String(value));
const word = (value) => (/^[^\s"'():]+$/.test(String(value)) ? String(value) : quote(value));

/**
 * Reads the records of the small Links Notation subset these files use: a
 * record line `kind arg...` and indented `key value` lines under it.
 * Values are bare words or JSON strings.
 * @param {string} text
 * @returns {Array<{ kind: string, args: string[], fields: Record<string, string[]> }>}
 */
export function parseRecords(text) {
  const records = [];
  for (const raw of text.split('\n')) {
    if (raw.trim() === '' || raw.trim().startsWith('#')) continue;
    const tokens = tokenizeLine(raw.trim());
    if (/^\s/.test(raw)) {
      if (!records.length) throw new Error(`field outside a record: ${raw}`);
      const [key, ...values] = tokens;
      records.at(-1).fields[key] = values;
    } else {
      const [kind, ...args] = tokens;
      records.push({ kind, args, fields: {} });
    }
  }
  return records;
}

function tokenizeLine(line) {
  const tokens = [];
  const re = /"(?:[^"\\]|\\.)*"|[^\s"]+/g;
  for (const [token] of line.matchAll(re)) tokens.push(token.startsWith('"') ? JSON.parse(token) : token);
  return tokens;
}

/**
 * @typedef {{ source: string, output: string, parts: number, translated: number,
 *   carried: number, comment: number, allowlisted: number }} ModuleRow
 * @typedef {{ commit: string, modules: Map<string, ModuleRow> }} Ledger
 */

const LEDGER_HEADER = [
  '# Rust generated from js/src by link-foundation/meta-language (issue #254 R2).',
  '# Written by `node js/scripts/translate-rust.mjs --write`. Edit only `commit`, the pin,',
  '# then run --write with meta-language checked out at it. `allowlisted` counts the',
  '# js/translation-allowlist.lino entries the module needs (items, cascades, comments).',
];

/**
 * @param {Ledger} ledger
 * @returns {string}
 */
export function renderLedger(ledger) {
  const rows = [...ledger.modules.entries()].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
  const total = (key) => rows.reduce((sum, [, row]) => sum + row[key], 0);
  const lines = [...LEDGER_HEADER, 'js_rust_translation',
    `  meta_language ${quote(UPSTREAM_REPO)}`,
    `  commit ${ledger.commit}`,
    `  modules ${rows.length}`,
    `  translated ${total('translated')}`,
    `  carried ${total('carried')}`,
    `  comment ${total('comment')}`];
  for (const [name, row] of rows) {
    lines.push(`module ${word(name)}`,
      `  source ${row.source}`,
      `  output ${row.output}`,
      `  parts ${row.parts}`,
      `  translated ${row.translated}`,
      `  carried ${row.carried}`,
      `  comment ${row.comment}`,
      `  allowlisted ${row.allowlisted}`);
  }
  return `${lines.join('\n')}\n`;
}

/**
 * @param {string} text
 * @returns {Ledger}
 */
export function parseLedger(text) {
  const records = parseRecords(text);
  const head = records.find((r) => r.kind === 'js_rust_translation');
  if (!head?.fields.commit?.[0]) throw new Error('ledger: no js_rust_translation commit');
  const modules = new Map();
  for (const r of records.filter((record) => record.kind === 'module')) {
    const num = (key) => Number(r.fields[key]?.[0] ?? NaN);
    modules.set(r.args[0], {
      source: r.fields.source?.[0], output: r.fields.output?.[0], parts: num('parts'),
      translated: num('translated'), carried: num('carried'), comment: num('comment'), allowlisted: num('allowlisted'),
    });
  }
  return { commit: head.fields.commit[0], modules };
}

const ALLOWLIST_HEADER = [
  '# Items of js/src that meta-language cannot translate to Rust yet (issue #254 R2).',
  '# `node js/scripts/translate-rust.mjs --write` adds and removes entries and keeps `upstream`',
  '# and `note`, which are edited by hand. --check fails on a carried item with no entry, on an',
  '# entry nothing carried matches (stale), on a construct that differs from the diagnostic,',
  '# and on an entry without an upstream issue URL or `drafted:<letter>` (an issue drafted in',
  '# experiments/issue-254/meta-language/README.md section 5.2, to be replaced by its URL once filed).',
  '#',
  '#   item <module> <name>       carried for `construct`, tracked by `upstream`',
  '#   cascade <module> <name>    carried only because it uses `needs`, itself carried or unbound',
  '#   comments <module>          `count` free-standing /** */ comments carried as source',
];

/**
 * One allowlist record: `item` (construct, upstream), `cascade` (needs) or
 * `comments` (count, upstream).
 * @typedef {{ kind: string, module: string, name?: string, construct?: string, needs?: string,
 *   count?: number, upstream?: string, note?: string }} Entry
 */

/**
 * @param {Entry[]} entries
 * @returns {string}
 */
export function renderAllowlist(entries) {
  const sorted = [...entries].sort((a, b) => (a.module < b.module ? -1 : a.module > b.module ? 1 : 0));
  const lines = [...ALLOWLIST_HEADER];
  for (const e of sorted) {
    if (e.kind === 'comments') {
      lines.push(`comments ${word(e.module)}`, `  count ${e.count}`, `  upstream ${quote(e.upstream ?? '')}`);
    } else if (e.kind === 'cascade') {
      lines.push(`cascade ${word(e.module)} ${word(e.name)}`, `  needs ${word(e.needs)}`);
    } else {
      lines.push(`item ${word(e.module)} ${word(e.name)}`, `  construct ${quote(e.construct)}`, `  upstream ${quote(e.upstream ?? '')}`);
    }
    if (e.note) lines.push(`  note ${quote(e.note)}`);
  }
  return `${lines.join('\n')}\n`;
}

/**
 * @param {string} text
 * @returns {Entry[]}
 */
export function parseAllowlist(text) {
  return parseRecords(text).map(({ kind, args, fields }) => {
    const note = fields.note?.[0];
    const extra = note ? { note } : {};
    if (kind === 'comments') return { kind, module: args[0], count: Number(fields.count?.[0]), upstream: fields.upstream?.[0], ...extra };
    if (kind === 'cascade') return { kind, module: args[0], name: args[1], needs: fields.needs?.[0], ...extra };
    if (kind === 'item') return { kind, module: args[0], name: args[1], construct: fields.construct?.[0], upstream: fields.upstream?.[0], ...extra };
    throw new Error(`allowlist: unknown record ${kind}`);
  });
}

/**
 * Compares the entries the translation needs with the allowlist.
 * @param {Entry[]} required
 * @param {Entry[]} allowlist
 * @returns {string[]} problems, empty when they agree
 */
export function compareAllowlist(required, allowlist) {
  const problems = [];
  const have = new Map();
  for (const e of allowlist) {
    const key = entryKey(e);
    if (have.has(key)) problems.push(`duplicate allowlist entry: ${key}`);
    have.set(key, e);
  }
  const needed = new Set();
  for (const r of required) {
    const key = entryKey(r);
    needed.add(key);
    const e = have.get(key);
    if (!e) {
      problems.push(`carried but not in the allowlist: ${key}${r.kind === 'item' ? ` (${r.construct})` : r.kind === 'cascade' ? ` (needs ${r.needs})` : ` (${r.count})`}`);
      continue;
    }
    if (r.kind === 'item' && e.construct !== r.construct) problems.push(`construct changed: ${key}: allowlist "${e.construct}", now "${r.construct}"`);
    if (r.kind === 'cascade' && e.needs !== r.needs) problems.push(`cascade changed: ${key}: allowlist needs ${e.needs}, now ${r.needs}`);
    if (r.kind === 'comments' && e.count !== r.count) problems.push(`comment count changed: ${key}: allowlist ${e.count}, now ${r.count}`);
  }
  for (const [key, e] of have) {
    if (!needed.has(key)) problems.push(`stale allowlist entry (now translates or is gone): ${key}`);
    else if (e.kind !== 'cascade' && !validUpstream(e.upstream)) problems.push(`no upstream issue: ${key}`);
  }
  return problems;
}

/**
 * The allowlist --write writes: the required entries, keeping the hand-edited
 * `upstream` and `note` of entries that still match and suggesting an
 * upstream for new ones.
 * @param {Entry[]} required
 * @param {Entry[]} previous
 * @returns {Entry[]}
 */
export function mergeAllowlist(required, previous) {
  const old = new Map(previous.map((e) => [entryKey(e), e]));
  return required.map((r) => {
    const e = old.get(entryKey(r));
    const kept = e && (r.kind !== 'item' || e.construct === r.construct);
    const note = e?.note ? { note: e.note } : {};
    if (r.kind === 'cascade') return { ...r, ...note };
    const upstream = kept && validUpstream(e.upstream) ? e.upstream : suggestUpstream(r.kind === 'comments' ? FREE_COMMENT : r.construct) ?? '';
    return { ...r, upstream, ...note };
  });
}

/**
 * Modules whose translated count fell below the ledger's.
 * @param {Map<string, ModuleRow>} before
 * @param {Map<string, { translated: number }>} after
 * @returns {string[]}
 */
export function regressions(before, after) {
  const out = [];
  for (const [name, row] of after) {
    const old = before.get(name);
    if (old && row.translated < old.translated) out.push(`${name}: translated ${old.translated} -> ${row.translated}`);
  }
  return out;
}
