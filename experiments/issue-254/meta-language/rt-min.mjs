import { pathToFileURL } from 'node:url'; import { join } from 'node:path';
const ml = await import(pathToFileURL(join(process.argv[2], 'self-translation.js')));
for (const src of ['const A = 1.5; /* note */\nconst B = 2.5;\n', 'const A = 1.5; // note\nconst B = 2.5;\n']) {
  const rs = ml.selfTranslate(src, 'JavaScript', 'Rust').code; const back = ml.selfTranslate(rs, 'Rust', 'JavaScript').code;
  console.log(JSON.stringify(src), '->', back === src ? 'exact' : JSON.stringify(back));
}
