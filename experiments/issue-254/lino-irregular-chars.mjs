// Lists the control and irregular whitespace characters in the bundled .lino files.
import { readFileSync } from 'node:fs';

const IRREGULAR = /[\0-\x08\x0B-\x1F\x7F-\x9F\u{a0}\u{1680}\u{180e}\u{2000}-\u{200b}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]/gu;
for (const file of process.argv.slice(2)) {
  const text = readFileSync(file, 'utf8');
  const found = text.match(IRREGULAR) ?? [];
  console.log(file, found.length, [...new Set(found)].map((c) => c.codePointAt(0).toString(16)));
}
