#!/usr/bin/env node
// Re-fetch the complete soulver reference; --cache DIR and --offline are supported.
import { run } from './refresh-documentation-corpus.mjs';
try { run(process.argv.slice(2), 'soulver'); }
catch (error) { console.error(error.message); process.exitCode = 1; }
