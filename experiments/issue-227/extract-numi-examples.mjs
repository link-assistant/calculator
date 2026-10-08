#!/usr/bin/env node
// Re-fetch the complete numi reference; --cache DIR and --offline are supported.
import { run } from './refresh-documentation-corpus.mjs';
try { run(process.argv.slice(2), 'numi'); }
catch (error) { console.error(error.message); process.exitCode = 1; }
