// usage: node census.mjs <ml js/src> <outdir with .rs>  -> census.json
import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
const [mlSrc, out] = process.argv.slice(2);
const { parseJavaScript } = await import(pathToFileURL(join(mlSrc, 'translation/javascript.js')));
const { checkProgram } = await import(pathToFileURL(join(mlSrc, 'translation/check.js')));
const walk = (d) => readdirSync(d).flatMap((f) => { const p = join(d, f); return statSync(p).isDirectory() ? walk(p) : f.endsWith('.rs') ? [p] : []; });
const rows = [];
for (const f of walk(out).sort()) {
  const lines = readFileSync(f, 'utf8').split('\n');
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^\/\/ meta-language:carried JavaScript (\S+) \((.*)\)$/); if (!m) continue;
    const src = []; let j = i + 1; while (j < lines.length && lines[j].startsWith('// |')) src.push(lines[j++].slice(5));
    const text = src.join('\n'); let message = null;
    if (['unsupported', 'syntax', 'type'].includes(m[2])) {
      try { checkProgram(parseJavaScript(text)); message = '(passes alone: needs sibling/imported names)'; } catch (e) { message = e.message; }
    }
    const name = (text.match(/(?:function|class|const|let)\s+([A-Za-z_$][\w$]*)/) ?? [])[1] ?? null;
    rows.push({ module: relative(out, f), term: m[1], reason: m[2], name, message });
  }
}
writeFileSync(join(out, 'census.json'), JSON.stringify(rows, null, 1));
const norm = (s) => s?.replace(/ at \d+\.\.\d+$/, '').replace(/`[^`]*`|'[^']*'|"[^"]*"/g, 'X') ?? '';
const count = {}; for (const r of rows) { const k = `${r.reason} :: ${norm(r.message)}`; count[k] = (count[k] ?? 0) + 1; }
for (const [k, n] of Object.entries(count).sort((a, b) => b[1] - a[1])) console.log(String(n).padStart(4), k);
