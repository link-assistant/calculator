// usage: node run.mjs <meta-language js/src dir> <calculator js/src> <outdir> [--bind]
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from 'node:fs';
import { join, relative, dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const [mlSrc, src, out] = process.argv.slice(2);
const bind = process.argv.includes('--bind');
const ml = await import(pathToFileURL(join(mlSrc, 'self-translation.js')));
const walk = (d) => readdirSync(d).flatMap((f) => { const p = join(d, f); return statSync(p).isDirectory() ? walk(p) : /\.m?js$/.test(f) ? [p] : []; });
const files = walk(src).sort();
const importsOf = (text, file) => [...text.matchAll(/^\s*(?:import|export)[^'"]*?from\s+['"](\.[^'"]+)['"]/gm)].map((m) => [m[1], resolve(dirname(file), m[1])]);
// topological order
const order = []; const seen = new Set();
const visit = (f) => { if (seen.has(f)) return; seen.add(f); for (const [, d] of importsOf(readFileSync(f, 'utf8'), f)) if (files.includes(d)) visit(d); order.push(f); };
files.forEach(visit);
const sigs = new Map(); const report = [];
for (const f of order) {
  const rel = relative(src, f); const text = readFileSync(f, 'utf8');
  const opts = {};
  if (bind) {
    opts.moduleDirectory = dirname(rel) === '.' ? [] : dirname(rel).split('/');
    opts.imports = Object.fromEntries(importsOf(text, f).filter(([, d]) => sigs.has(d)).map(([s, d]) => [s, sigs.get(d)]));
  }
  const t0 = Date.now(); let r;
  try {
    r = ml.selfTranslate(text, 'JavaScript', 'Rust', opts);
    if (bind && ml.selfTranslationSignatures) sigs.set(f, ml.selfTranslationSignatures(text, 'JavaScript', opts));
  } catch (e) { report.push({ module: rel, error: String(e.message ?? e) }); console.error(rel, 'ERROR', e.message); continue; }
  const o = join(out, rel.replace(/\.m?js$/, '.rs')); mkdirSync(dirname(o), { recursive: true }); writeFileSync(o, r.code);
  report.push({ module: rel, ms: Date.now() - t0, items: r.items.map(({ term, status, reason }) => ({ term, status, reason })) });
  const c = {}; for (const i of r.items) c[i.status] = (c[i.status] ?? 0) + 1;
  console.error(rel, Date.now() - t0, 'ms', JSON.stringify(c));
}
writeFileSync(join(out, 'report.json'), JSON.stringify(report, null, 1));
