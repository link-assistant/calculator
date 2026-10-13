// Compares the issues scripts/check-closing-references.mjs says a description
// closes with GitHub's own closingIssuesReferences (issue #247). Input: the JSON
// array from
//   gh pr list --repo link-assistant/calculator --state all --limit 1000 \
//     --json number,state,baseRefName,body,closingIssuesReferences
// Differences are expected only for PRs that do not target the default branch
// (GitHub links nothing for them) and for issues linked by hand in the
// Development sidebar (they are not in the description).
import { checkClosingReferences } from '../../scripts/check-closing-references.mjs';
import { readFileSync } from 'node:fs';

const repository = 'link-assistant/calculator';
const prs = JSON.parse(readFileSync(process.argv[2], 'utf8'));
let differ = 0;
for (const pr of prs) {
  const parsed = checkClosingReferences(pr.body ?? '', repository).closes.slice().sort();
  const github = (pr.closingIssuesReferences ?? [])
    .map((issue) => `${issue.repository.owner.login}/${issue.repository.name}#${issue.number}`)
    .sort();
  if (parsed.join() !== github.join()) {
    differ++;
    const only = (a, b) => a.filter((x) => !b.includes(x)).join(', ') || '-';
    console.log(`#${pr.number} ${pr.state} base=${pr.baseRefName} parsed-only: ${only(parsed, github)} github-only: ${only(github, parsed)}`);
  }
}
console.log(`${prs.length - differ} of ${prs.length} PRs match closingIssuesReferences`);
