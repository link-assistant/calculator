// Lists the source lines of js/src/rust/libm/*.js that the libm vectors never
// reach, using V8 block coverage. Usage (from the repository root):
//
//   node experiments/issue-254/libm-coverage.mjs
//
// It re-runs libm-check.mjs with NODE_V8_COVERAGE set and reads the result.

import { execFileSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = mkdtempSync(join(tmpdir(), 'libm-cov-'));
const check = fileURLToPath(new URL('libm-check.mjs', import.meta.url));
execFileSync(process.execPath, [check], { env: { ...process.env, NODE_V8_COVERAGE: dir }, stdio: 'ignore' });

for (const file of readdirSync(dir)) {
  const report = JSON.parse(readFileSync(join(dir, file), 'utf8'));
  for (const script of report.result) {
    if (!/\/js\/src\/rust\/libm(\/|\.js)/.test(script.url)) {
      continue;
    }
    const path = fileURLToPath(script.url);
    const source = readFileSync(path, 'utf8');
    /** @type {Set<number>} */
    const lines = new Set();
    const lineOf = (/** @type {number} */ offset) => source.slice(0, offset).split('\n').length;
    for (const fn of script.functions) {
      for (const range of fn.ranges) {
        if (range.count === 0) {
          const text = source.slice(range.startOffset, range.endOffset).trim();
          if (text === '' || text === '}' || /^}\s*(else)?\s*{?\s*$/.test(text)) {
            continue;
          }
          for (let l = lineOf(range.startOffset); l <= lineOf(range.endOffset); l += 1) {
            lines.add(l);
          }
        }
      }
    }
    const src = source.split('\n');
    const shown = [...lines].sort((a, b) => a - b).filter((l) => src[l - 1].trim() !== '' && src[l - 1].trim() !== '}');
    console.log(`${path.replace(/.*\/js\//, 'js/')}: ${shown.length} unreached lines`);
    for (const l of shown) {
      console.log(`  ${String(l).padStart(4)}: ${src[l - 1].trim()}`);
    }
  }
}
rmSync(dir, { recursive: true, force: true });
