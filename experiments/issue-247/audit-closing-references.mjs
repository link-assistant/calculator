// Replays scripts/check-closing-references.mjs over every PR description of the
// repository (issue #247). Input: JSON lines with number, state, merged_at,
// title and body, e.g. from
//   gh api repos/link-assistant/calculator/pulls?state=all --paginate --jq ".[] | {number,state,merged_at,title,body}"
import { checkClosingReferences } from '../../scripts/check-closing-references.mjs';
import { readFileSync } from 'node:fs';
const lines = readFileSync(process.argv[2], 'utf8').trim().split('\n').map((l) => JSON.parse(l));
let flagged = 0;
for (const pr of lines) {
  const r = checkClosingReferences(pr.body ?? '', 'link-assistant/calculator');
  if (r.errors.length) {
    flagged++;
    console.log(`#${pr.number} ${pr.state} merged=${!!pr.merged_at} closes=${r.closes.length} errors=${r.errors.length} ${pr.title.slice(0, 60)}`);
    for (const e of r.errors.slice(0, 4)) console.log('   ', e.rule, e.message.slice(0, 220));
  }
}
console.log(`flagged ${flagged} of ${lines.length}`);
