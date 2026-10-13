// Reproduces link-assistant/calculator#247 against hive-mind's own modules.
// Usage: node experiments/issue-247/hive-mind-link-repair-repro.mjs <hive-mind checkout at ed6a155>
// (the checkout needs `npm ci --ignore-scripts --omit=dev`). Output is in
// docs/case-studies/issue-247/data/hive-mind-link-repair-repro.txt.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const src = (file) => pathToFileURL(resolve(process.argv[2], 'src', file)).href;
const { repairRequiredIssueLinks } = await import(src('pr-issue-link-repair.lib.mjs'));
const { extractClosingIssueReferences } = await import(src('github-linking.lib.mjs'));

// #1 asks for a plan; #2 and #3 are the implementation issues the session
// created and attached as native sub-issues.
const issues = {
  1: "Analyse the problem and create one issue per part of the work. This issue's own result is the analysis and the issue creation.",
  2: 'Implement part A.',
  3: 'Implement part B.',
};
const subIssues = { 1: [2, 3], 2: [], 3: [] };
let prBody = 'Closes #1\n\nThis PR only plans the work: #2 and #3 stay open.';

const ok = (value) => ({ code: 0, stdout: JSON.stringify(value), stderr: '' });
const run = async (args) => {
  const [command, target] = args;
  if (command === 'api' && /issues\/\d+\/sub_issues$/.test(target)) {
    const number = Number(target.split('/').at(-2));
    return ok([subIssues[number].map((n) => ({ number: n, repository_url: 'https://api.github.com/repos/o/r' }))]);
  }
  if (command === 'api' && /issues\/\d+$/.test(target)) {
    const number = Number(target.split('/').at(-1));
    return ok({ number, body: issues[number] });
  }
  if (command === 'api' && /pulls\/10$/.test(target)) return ok({ number: 10, body: prBody });
  if (command === 'pr' && target === 'edit') {
    prBody = readFileSync(args[args.indexOf('--body-file') + 1], 'utf8');
    return { code: 0, stdout: '', stderr: '' };
  }
  throw new Error(`unexpected gh ${args.join(' ')}`);
};

const repair = () => repairRequiredIssueLinks({ owner: 'o', repo: 'r', issueNumber: 1, prNumber: 10, run, logger: async (m) => console.log('  log:', m) });

console.log('1. Session ends; the finally-block repair runs:');
await repair();
console.log(JSON.stringify(prBody));

console.log('2. The next session removes the lines again; the repair re-adds them:');
prBody = prBody.replace(/\n\n---\n\nFixes #2\nFixes #3$/, '');
await repair();
console.log(JSON.stringify(prBody));

console.log('3. hive-mind and GitHub disagree on negated keywords:');
const text = 'This PR does not close #2.';
console.log(`  extractClosingIssueReferences(${JSON.stringify(text)}) =`, JSON.stringify(extractClosingIssueReferences(text)));
console.log('  GitHub: closingIssuesReferences lists #2, and merging closes it (probe on link-assistant/calculator#261).');
