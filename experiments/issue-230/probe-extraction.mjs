// Finite probe of checked-out suites. No upstream code is evaluated.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { stripComments } from '../../scripts/corpus-literals.mjs';
const root = process.argv[2];
function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) walk(path);
    else if (path.endsWith('.test.js')) {
      try { stripComments(readFileSync(path, 'utf8')); }
      catch (error) { console.error(path, error.stack); }
    }
  }
}
walk(join(root, 'test/unit-tests'));
