import { readFileSync, writeFileSync } from 'node:fs'; import { pathToFileURL } from 'node:url'; import { join } from 'node:path';
const ml = await import(pathToFileURL(join(process.argv[2], 'self-translation.js')));
const src = readFileSync(process.argv[3], 'utf8');
const back = ml.selfTranslate(ml.selfTranslate(src, 'JavaScript', 'Rust').code, 'Rust', 'JavaScript').code;
writeFileSync('/tmp/ml-research/rt-back.js', back);
