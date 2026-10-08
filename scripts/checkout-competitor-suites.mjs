#!/usr/bin/env node
/** Prepare clean upstream checkouts from the corpus lock; --latest refreshes pins. */
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function checkoutSuites({ checkouts, lock = 'scripts/competitor-upstreams.json', latest = false }) {
  const sources = JSON.parse(readFileSync(lock, 'utf8'));
  mkdirSync(checkouts, { recursive: true });
  for (const [name, { repo, revision }] of Object.entries(sources)) {
    if (!/^[a-z][a-z0-9]*$/.test(name) || !/^[a-f0-9]{40}$/.test(revision)) throw new Error(`Invalid upstream lock entry: ${name}`);
    const root = join(checkouts, name);
    const git = (...args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim();
    const fresh = !existsSync(root);
    if (fresh) execFileSync('git', ['clone', '--depth', '1', '--no-checkout', repo, root], { stdio: 'inherit' });
    else {
      if (resolve(git('rev-parse', '--show-toplevel')) !== resolve(root)) throw new Error(`${name}: expected a repository root`);
      if (git('status', '--porcelain', '--untracked-files=all')) throw new Error(`${name}: checkout is dirty; refusing to replace local work`);
      if (git('remote', 'get-url', 'origin').replace(/\.git$/, '') !== repo.replace(/\.git$/, '')) throw new Error(`${name}: origin does not match the upstream lock`);
      if (!latest && git('rev-parse', 'HEAD') === revision) continue;
    }
    // HEAD follows the upstream default branch even if that branch is renamed.
    git('fetch', '--depth', '1', 'origin', latest ? 'HEAD' : revision);
    git('checkout', '--detach', 'FETCH_HEAD');
    console.log(`${name}: ${git('rev-parse', 'HEAD')}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const options = { latest: false };
    for (let i = 2; i < process.argv.length; i++) {
      const arg = process.argv[i];
      if (arg === '--latest') options.latest = true;
      else if (arg === '--checkouts' || arg === '--lock') {
        const value = process.argv[++i];
        if (!value || value.startsWith('--')) throw new Error(`Missing value for ${arg}`);
        options[arg.slice(2)] = value;
      } else throw new Error(`Unknown argument: ${arg}`);
    }
    if (!options.checkouts) throw new Error('Give --checkouts <directory> [--latest] [--lock <json>]');
    checkoutSuites(options);
  } catch (error) {
    console.error(`checkout-competitor-suites: ${error.message}`);
    process.exitCode = 2;
  }
}
