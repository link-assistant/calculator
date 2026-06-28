#!/usr/bin/env node

/**
 * Push the current HEAD to a branch, rebasing and retrying if the remote moved.
 *
 * This is intended for CI jobs that generate commits on main. A plain `git push`
 * fails when another workflow commits first; retrying after `git pull --rebase`
 * keeps those generated commits forward-moving without force-pushing.
 */

import { spawnSync } from 'child_process';
import { pathToFileURL } from 'url';

function runGit(args, { allowFailure = false } = {}) {
  console.log(`$ git ${args.join(' ')}`);
  const result = spawnSync('git', args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }

  if (result.status !== 0 && !allowFailure) {
    throw new Error(`git ${args.join(' ')} failed with exit code ${result.status}`);
  }

  return result;
}

export function resolveBranch(branch) {
  if (branch) {
    return branch;
  }

  if (process.env.GITHUB_REF_NAME) {
    return process.env.GITHUB_REF_NAME;
  }

  const current = runGit(['branch', '--show-current'], { allowFailure: true })
    .stdout.trim();
  return current || 'main';
}

export function syncWithRemote({ remote = 'origin', branch } = {}) {
  const targetBranch = resolveBranch(branch);
  const fetch = runGit(['fetch', remote, targetBranch], { allowFailure: true });
  if (fetch.status !== 0) {
    console.warn(`Could not fetch ${remote}/${targetBranch}; continuing without pre-rebase`);
    return targetBranch;
  }

  const local = runGit(['rev-parse', 'HEAD']).stdout.trim();
  const remoteHead = runGit(['rev-parse', `${remote}/${targetBranch}`], {
    allowFailure: true,
  }).stdout.trim();

  if (remoteHead && local !== remoteHead) {
    console.log(`Rebasing local HEAD onto ${remote}/${targetBranch}`);
    try {
      runGit(['rebase', `${remote}/${targetBranch}`]);
    } catch (error) {
      runGit(['rebase', '--abort'], { allowFailure: true });
      throw error;
    }
  }

  return targetBranch;
}

export function pushCurrentBranchWithRebase({
  remote = 'origin',
  branch,
  attempts = 3,
} = {}) {
  const targetBranch = resolveBranch(branch);

  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    const push = runGit(['push', remote, `HEAD:${targetBranch}`], {
      allowFailure: true,
    });
    if (push.status === 0) {
      return;
    }

    if (attempt === attempts) {
      throw new Error(
        `git push failed after ${attempts} attempts for ${remote}/${targetBranch}`
      );
    }

    console.warn(
      `Push failed on attempt ${attempt}/${attempts}; rebasing onto ${remote}/${targetBranch}`
    );
    try {
      runGit(['pull', '--rebase', remote, targetBranch]);
    } catch (error) {
      runGit(['rebase', '--abort'], { allowFailure: true });
      throw error;
    }
  }
}

function parseCliArgs(argv) {
  const options = {
    remote: 'origin',
    branch: undefined,
    attempts: 3,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--remote') {
      options.remote = argv[index + 1];
      index += 1;
    } else if (arg === '--branch') {
      options.branch = argv[index + 1];
      index += 1;
    } else if (arg === '--attempts') {
      options.attempts = Number.parseInt(argv[index + 1], 10);
      index += 1;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!Number.isInteger(options.attempts) || options.attempts < 1) {
    throw new Error('--attempts must be a positive integer');
  }

  return options;
}

export function main(argv = process.argv.slice(2)) {
  const options = parseCliArgs(argv);
  syncWithRemote(options);
  pushCurrentBranchWithRebase(options);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
