// usage: node assemble.mjs <translated dir> <crate dir>: one crate, one module per translated file
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, existsSync, rmSync } from 'node:fs';
import { join, relative, dirname, basename } from 'node:path';
const [from, crate] = process.argv.slice(2);
rmSync(crate, { recursive: true, force: true }); mkdirSync(join(crate, 'src'), { recursive: true });
writeFileSync(join(crate, 'Cargo.toml'), '[package]\nname = "ml_translated"\nversion = "0.0.0"\nedition = "2021"\npublish = false\n\n[lib]\npath = "src/lib.rs"\n\n[workspace]\n');
const snake = (s) => s.replace(/[-.]/g, '_');
const walk = (d) => readdirSync(d).flatMap((f) => { const p = join(d, f); return statSync(p).isDirectory() ? walk(p) : f.endsWith('.rs') ? [p] : []; });
const files = walk(from).map((f) => relative(from, f));
const children = new Map(); // module path (no ext) -> set of child names
const add = (parent, child) => { if (!children.has(parent)) children.set(parent, new Set()); children.get(parent).add(child); };
for (const f of files) {
  const parts = f.replace(/\.rs$/, '').split('/').map(snake);
  for (let i = 0; i < parts.length; i++) add(parts.slice(0, i).join('/'), parts[i]);
  const dest = join(crate, 'src', parts.join('/') + '.rs'); mkdirSync(dirname(dest), { recursive: true });
  writeFileSync(dest, readFileSync(join(from, f), 'utf8'));
}
for (const [parent, kids] of children) {
  const file = parent === '' ? join(crate, 'src/lib.rs') : join(crate, 'src', parent + '.rs');
  const decl = [...kids].sort().map((k) => `pub mod ${k};`).join('\n') + '\n';
  writeFileSync(file, (existsSync(file) ? readFileSync(file, 'utf8') + '\n' : '') + decl);
}
// --workarounds: (1) name modules as the translator's imports do (str -> str_, core -> core_),
// (2) re-root each module's own crate::ml* helper paths at the module (meta-language#219),
// (3) drop imported names the target module does not define (meta-language#220).
if (process.argv.includes('--workarounds')) {
  const RENAME = { str: 'str_', core: 'core_' };
  const all = walk(join(crate, 'src')).map((f) => relative(join(crate, 'src'), f));
  for (const f of all) { const b = basename(f, '.rs'); if (RENAME[b]) {
    const nf = join(dirname(f), RENAME[b] + '.rs'); writeFileSync(join(crate, 'src', nf), readFileSync(join(crate, 'src', f), 'utf8')); rmSync(join(crate, 'src', f));
    if (existsSync(join(crate, 'src', dirname(f), b))) { /* no children dirs for these */ }
    const parent = dirname(f) === '.' ? join(crate, 'src/lib.rs') : join(crate, 'src', dirname(f) + '.rs');
    writeFileSync(parent, readFileSync(parent, 'utf8').replace(`pub mod ${b};`, `pub mod ${RENAME[b]};`));
  } }
  const mods = walk(join(crate, 'src')).map((f) => relative(join(crate, 'src'), f)).filter((f) => f !== 'lib.rs');
  const defined = (path) => { const f = join(crate, 'src', path.join('/') + '.rs'); if (!existsSync(f)) return null; const t = readFileSync(f, 'utf8');
    return new Set([...t.matchAll(/^pub (?:fn|const|static|struct|enum|type|mod) (\w+)/gm)].map((m) => m[1])); };
  let pruned = 0, kept = 0;
  for (const f of mods) {
    const modPath = f.replace(/\.rs$/, '').split('/');
    let t = readFileSync(join(crate, 'src', f), 'utf8');
    t = t.replace(/crate::(ml\w*)/g, `crate::${modPath.join('::')}::$1`);
    t = t.replace(/^use crate::([\w:]+?)::(\{[^}]*\}|\w+(?: as \w+)?);$/gm, (line, path, names) => {
      const have = defined(path.split('::')); if (!have) { pruned++; return `// pruned: ${line}`; }
      const list = names.replace(/[{}]/g, '').split(',').map((s) => s.trim()).filter(Boolean);
      const ok = list.filter((n) => have.has(n.split(' as ')[0])); pruned += list.length - ok.length; kept += ok.length;
      return ok.length ? `use crate::${path}::{${ok.join(', ')}};` : `// pruned: ${line}`;
    });
    writeFileSync(join(crate, 'src', f), t);
  }
  console.log(`imports kept ${kept}, pruned ${pruned}`);
}
