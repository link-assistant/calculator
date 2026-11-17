import { readFileSync } from 'node:fs'; import { pathToFileURL } from 'node:url'; import { join } from 'node:path';
const ml = await import(pathToFileURL(join(process.argv[2], 'self-translation.js')));
const src = readFileSync(process.argv[3], 'utf8');
const a = ml.selfTranslate(src, 'JavaScript', 'Rust').code, b = ml.selfTranslate(src, 'JavaScript', 'Rust').code;
const back = ml.selfTranslate(a, 'Rust', 'JavaScript').code;
console.log('deterministic', a === b, 'round-trip byte-exact', back === src);
