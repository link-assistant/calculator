/**
 * Worker of translate-rust.mjs: loads the pinned meta-language once, with the
 * load hook of translate-rust-lib.mjs, and translates the modules it is sent.
 */

import { registerHooks } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parentPort, workerData } from 'node:worker_threads';
import { patchSelfTranslation } from './translate-rust-lib.mjs';

const target = pathToFileURL(join(workerData.metaLanguage, 'src', 'self-translation.js')).href;

registerHooks({
  load(url, context, nextLoad) {
    const result = nextLoad(url, context);
    if (url !== target) return result;
    return { ...result, source: patchSelfTranslation(String(result.source)), shortCircuit: true };
  },
});

const ml = await import(target);

parentPort.on('message', ({ module, source, moduleDirectory, imports }) => {
  const started = performance.now();
  try {
    const { translation, signatures } = ml.__translateModule(source, 'JavaScript', 'Rust', { moduleDirectory, imports });
    parentPort.postMessage({
      module,
      code: translation.code,
      items: translation.items.map(({ term, start, end, status, reason, diagnostic }) => ({ term, start, end, status, reason, diagnostic })),
      signatures,
      ms: performance.now() - started,
    });
  } catch (error) {
    parentPort.postMessage({ module, error: String(error?.stack ?? error), ms: performance.now() - started });
  }
});
