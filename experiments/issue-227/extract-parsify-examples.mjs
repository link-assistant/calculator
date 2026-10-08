#!/usr/bin/env node
// Re-fetch the complete parsify reference; --cache DIR and --offline are supported.
import { run } from './refresh-documentation-corpus.mjs';
try { run(process.argv.slice(2), 'parsify'); }
catch (error) { console.error(error.message); process.exitCode = 1; }
