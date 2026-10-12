#!/usr/bin/env node

/**
 * Detect code changes for CI/CD pipeline
 *
 * This script detects what types of files have changed between two commits
 * and outputs the results for use in GitHub Actions workflow conditions.
 *
 * Key behavior:
 * - For PRs: compares PR head against base branch
 * - For pushes: compares HEAD against HEAD^
 * - Excludes certain folders and file types from "code changes" detection
 *
 * Excluded from code changes (don't require changelog fragments):
 * - Markdown files (*.md) in any folder
 * - changelog.d/ folder (changelog fragments)
 * - dev/log/ folder (collected CI evidence, e.g. run JSON)
 * - docs/ folder (documentation)
 * - experiments/ folder (experimental scripts)
 * - examples/ folder (example scripts)
 *
 * Usage:
 *   node scripts/detect-code-changes.mjs
 *
 * Environment variables (set by GitHub Actions):
 *   - GITHUB_EVENT_NAME: 'pull_request' or 'push'
 *   - GITHUB_BASE_SHA: Base commit SHA for PR
 *   - GITHUB_HEAD_SHA: Head commit SHA for PR
 *
 * Outputs (written to GITHUB_OUTPUT):
 *   - rs-changed: 'true' if any .rs files changed
 *   - toml-changed: 'true' if any .toml files changed
 *   - mjs-changed: 'true' if any .mjs files changed
 *   - docs-changed: 'true' if any .md files changed
 *   - workflow-changed: 'true' if any .github/workflows/ or .github/actions/
 *     files changed (a composite action runs inside the workflows)
 *   - any-code-changed: 'true' if any code files changed (excludes docs, changelog.d, experiments, examples)
 *     Detects: .rs, .toml, .mjs, .ts, .tsx, .js, .css, .html, .yml, .yaml, .py, .sh, .json files and workflow changes
 */

import { execSync } from 'child_process';
import { appendFileSync } from 'fs';
import { fileURLToPath } from 'url';

/**
 * Whether a file configures CI: a workflow or a local composite action.
 * Issue #256: a PR that only changed .github/actions/ skipped lint and with it
 * wasm-build, the one job that runs the changed install-wasm-pack action.
 * @param {string} file - Repository-relative path
 * @returns {boolean}
 */
export function isCiConfigFile(file) {
  return file.startsWith('.github/workflows/') || file.startsWith('.github/actions/');
}

/**
 * Execute a shell command and return trimmed output
 * @param {string} command - The command to execute
 * @returns {string} - The trimmed command output
 */
function exec(command) {
  try {
    return execSync(command, { encoding: 'utf-8' }).trim();
  } catch (error) {
    console.error(`Error executing command: ${command}`);
    console.error(error.message);
    return '';
  }
}

/**
 * Write output to GitHub Actions output file
 * @param {string} name - Output name
 * @param {string} value - Output value
 */
function setOutput(name, value) {
  const outputFile = process.env.GITHUB_OUTPUT;
  if (outputFile) {
    appendFileSync(outputFile, `${name}=${value}\n`);
  }
  console.log(`${name}=${value}`);
}

/**
 * Get the list of changed files between two commits
 * @returns {string[]} Array of changed file paths
 */
function getChangedFiles() {
  const eventName = process.env.GITHUB_EVENT_NAME || 'local';

  if (eventName === 'pull_request') {
    const baseSha = process.env.GITHUB_BASE_SHA;
    const headSha = process.env.GITHUB_HEAD_SHA;

    if (baseSha && headSha) {
      console.log(`Comparing PR: ${baseSha}...${headSha}`);
      try {
        // Ensure we have the base commit
        try {
          execSync(`git cat-file -e ${baseSha}`, { stdio: 'ignore' });
        } catch {
          console.log('Base commit not available locally, attempting fetch...');
          execSync(`git fetch origin ${baseSha}`, { stdio: 'inherit' });
        }
        const output = exec(`git diff --name-only ${baseSha} ${headSha}`);
        return output ? output.split('\n').filter(Boolean) : [];
      } catch (error) {
        console.error(`Git diff failed: ${error.message}`);
      }
    }
  }

  // For push events or fallback
  console.log('Comparing HEAD^ to HEAD');
  try {
    const output = exec('git diff --name-only HEAD^ HEAD');
    return output ? output.split('\n').filter(Boolean) : [];
  } catch {
    // If HEAD^ doesn't exist (first commit), list all files in HEAD
    console.log('HEAD^ not available, listing all files in HEAD');
    const output = exec('git ls-tree --name-only -r HEAD');
    return output ? output.split('\n').filter(Boolean) : [];
  }
}

/**
 * Check if a file should be excluded from code changes detection
 * @param {string} filePath - The file path to check
 * @returns {boolean} True if the file should be excluded
 */
function isExcludedFromCodeChanges(filePath) {
  // Exclude markdown files in any folder
  if (filePath.endsWith('.md')) {
    return true;
  }

  // Exclude specific folders from code changes
  const excludedFolders = ['changelog.d/', 'dev/log/', 'docs/', 'experiments/', 'examples/'];

  for (const folder of excludedFolders) {
    if (filePath.startsWith(folder)) {
      return true;
    }
  }

  return false;
}

// .py, .sh and .json joined in issue #256: CI runs scripts/*.py and *.sh, and
// web/package.json pins the web dependencies, yet a PR touching only those
// needed no changelog fragment and skipped the fresh-merge simulation.
const CODE_PATTERN = /\.(rs|toml|mjs|ts|tsx|js|css|html|yml|yaml|py|sh|json)$/;

/**
 * Whether a changed file is code: it needs a changelog fragment and the checks.
 * @param {string} file - Repository-relative path
 * @returns {boolean}
 */
export function isCodeChange(file) {
  return !isExcludedFromCodeChanges(file) && (CODE_PATTERN.test(file) || isCiConfigFile(file));
}

/**
 * Main function to detect changes
 */
function detectChanges() {
  console.log('Detecting file changes for CI/CD...\n');

  const changedFiles = getChangedFiles();

  console.log('Changed files:');
  if (changedFiles.length === 0) {
    console.log('  (none)');
  } else {
    changedFiles.forEach((file) => console.log(`  ${file}`));
  }
  console.log('');

  // Detect .rs file changes (Rust source)
  const rsChanged = changedFiles.some((file) => file.endsWith('.rs'));
  setOutput('rs-changed', rsChanged ? 'true' : 'false');

  // Detect .toml file changes (Cargo.toml, Cargo.lock, etc.)
  const tomlChanged = changedFiles.some((file) => file.endsWith('.toml'));
  setOutput('toml-changed', tomlChanged ? 'true' : 'false');

  // Detect .mjs file changes (scripts)
  const mjsChanged = changedFiles.some((file) => file.endsWith('.mjs'));
  setOutput('mjs-changed', mjsChanged ? 'true' : 'false');

  // Detect documentation changes (any .md file)
  const docsChanged = changedFiles.some((file) => file.endsWith('.md'));
  setOutput('docs-changed', docsChanged ? 'true' : 'false');

  // Detect workflow changes
  const workflowChanged = changedFiles.some(isCiConfigFile);
  setOutput('workflow-changed', workflowChanged ? 'true' : 'false');

  // Detect code changes (excluding docs, changelog.d, experiments, examples folders, and markdown files)
  const codeChangedFiles = changedFiles.filter(
    (file) => !isExcludedFromCodeChanges(file)
  );

  console.log('\nFiles considered as code changes:');
  if (codeChangedFiles.length === 0) {
    console.log('  (none)');
  } else {
    codeChangedFiles.forEach((file) => console.log(`  ${file}`));
  }
  console.log('');

  const codeChanged = changedFiles.some(isCodeChange);
  setOutput('any-code-changed', codeChanged ? 'true' : 'false');

  console.log('\nChange detection completed.');
}

// Run the detection when executed directly (the test imports the classifiers)
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  detectChanges();
}
