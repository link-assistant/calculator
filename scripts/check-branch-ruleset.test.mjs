import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

import { missingContexts, requiredContexts } from './check-branch-ruleset.mjs';

const committed = JSON.parse(readFileSync(new URL('../.github/main-ruleset.json', import.meta.url), 'utf8'));

describe('check-branch-ruleset', () => {
  it('requires both CI stages', () => {
    assert.deepEqual(requiredContexts(committed), ['JavaScript Stage', 'Rust Stage / Rust Stage']);
  });

  it('reports every context when no ruleset exists', () => {
    assert.deepEqual(missingContexts(requiredContexts(committed), []), requiredContexts(committed));
  });

  it('accepts the committed ruleset once applied', () => {
    assert.deepEqual(missingContexts(requiredContexts(committed), [committed]), []);
  });

  it('ignores rulesets that are not active', () => {
    const disabled = { ...committed, enforcement: 'evaluate' };
    assert.equal(missingContexts(requiredContexts(committed), [disabled]).length, 2);
  });
});
