import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';

import { checkJavaScriptFirst, expandLocalActions, parseWorkflow, usesRust } from './check-javascript-first.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));
const dir = join(root, '.github', 'workflows');
const repoWorkflows = () =>
  new Map(
    readdirSync(dir)
      .filter((name) => name.endsWith('.yml'))
      .map((name) => [name, readFileSync(join(dir, name), 'utf8')]),
  );
const readAction = (path) => {
  const file = join(root, path, 'action.yml');
  return existsSync(file) ? readFileSync(file, 'utf8') : null;
};

/** The repository's workflows with one file edited. */
function withEdit(file, edit) {
  const workflows = repoWorkflows();
  const before = workflows.get(file);
  const after = edit(before);
  assert.notEqual(after, before, `the edit of ${file} changed nothing`);
  workflows.set(file, after);
  return checkJavaScriptFirst(workflows, readAction);
}

const matching = (problems, pattern) => problems.filter((problem) => pattern.test(problem));

describe('check-javascript-first', () => {
  it('accepts the committed workflows', () => {
    assert.deepEqual(checkJavaScriptFirst(repoWorkflows(), readAction), []);
  });

  it('rejects cargo in a JavaScript-stage job', () => {
    const problems = withEdit('release.yml', (s) =>
      s.replace('      - name: Run script tests\n', '      - run: cargo test\n\n      - name: Run script tests\n'),
    );
    assert.equal(matching(problems, /release\.yml job scripts uses Rust tooling/).length, 1, problems.join('\n'));
  });

  it('sees Rust tooling through a local composite action', () => {
    const problems = withEdit('workflows.yml', (s) =>
      s.replace(/(\n {6}- uses: actions\/checkout@\S+)/, '\n      - uses: ./.github/actions/setup-rust$1'),
    );
    assert.ok(matching(problems, /workflows\.yml job \S+ uses Rust tooling/).length > 0, problems.join('\n'));
  });

  it('rejects CodeQL for rust outside the Rust stage', () => {
    const problems = withEdit('security.yml', (s) =>
      s.replace('language: [actions, javascript-typescript, python]', 'language: [actions, javascript-typescript, python, rust]'),
    );
    assert.equal(matching(problems, /security\.yml job codeql uses Rust tooling/).length, 1, problems.join('\n'));
  });

  it('rejects a rust-stage that can start before a JavaScript job finished', () => {
    const problems = withEdit('release.yml', (s) => s.replace(/(  rust-stage:[\s\S]*?)      - js-lint\n/, '$1'));
    assert.deepEqual(matching(problems, /must need js-lint/).length, 1, problems.join('\n'));
  });

  it('rejects a rust-stage that ignores the JavaScript Stage result', () => {
    const problems = withEdit('release.yml', (s) => s.replace("      needs.javascript-stage.result == 'success' &&\n", ''));
    assert.equal(matching(problems, /must run only if needs\.javascript-stage/).length, 1, problems.join('\n'));
  });

  it('rejects a cancelling Rust stage', () => {
    const problems = withEdit('release.yml', (s) =>
      s.replace(/(group: rust-\$\{\{ github\.ref \}\}\n\s+cancel-in-progress:) false/, '$1 true'),
    );
    assert.equal(matching(problems, /rust-stage needs a per-branch concurrency group/).length, 1, problems.join('\n'));
    const inner = withEdit('rust.yml', (s) =>
      s.replace('      group: main-writer-main\n      cancel-in-progress: false', '      group: main-writer-main\n      cancel-in-progress: true'),
    );
    assert.ok(matching(inner, /rust\.yml job \S+ must not use cancel-in-progress/).length > 0, inner.join('\n'));
  });

  it('rejects a JavaScript job without literal cancel-in-progress: true', () => {
    const problems = withEdit('release.yml', (s) =>
      s.replace(
        'group: js-${{ github.workflow }}-${{ github.ref }}-js-lint\n      cancel-in-progress: true',
        "group: js-${{ github.workflow }}-${{ github.ref }}-js-lint\n      cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}",
      ),
    );
    assert.deepEqual(matching(problems, /job js-lint needs a job-level concurrency group/).length, 1, problems.join('\n'));
  });

  it('rejects workflow-level concurrency in the entry workflow', () => {
    const problems = withEdit('release.yml', (s) =>
      s.replace('\njobs:\n', '\nconcurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}\n  cancel-in-progress: true\n\njobs:\n'),
    );
    assert.equal(matching(problems, /release\.yml must not set workflow-level concurrency/).length, 1, problems.join('\n'));
  });

  it('rejects a Rust workflow with its own trigger', () => {
    const problems = withEdit('update-screenshots.yml', (s) => s.replace('on:\n  workflow_call:\n', 'on:\n  workflow_call:\n  push:\n'));
    assert.equal(matching(problems, /update-screenshots\.yml runs in the Rust stage/).length, 1, problems.join('\n'));
  });

  it('rejects calling rust.yml from another job', () => {
    const problems = withEdit('workflows.yml', (s) => s.replace(/\njobs:\n/, '\njobs:\n  sneak:\n    uses: ./.github/workflows/rust.yml\n'));
    assert.equal(matching(problems, /workflows\.yml job sneak calls rust\.yml/).length, 1, problems.join('\n'));
  });

  it('ignores Rust words in comments', () => {
    assert.equal(usesRust(parseWorkflow('jobs:\n  a:\n    steps:\n      # cargo build\n      - run: npm test # not cargo\n').jobs.get('a').text), false);
  });

  it('allows the --skip-cargo flag but not bounded-cargo', () => {
    assert.equal(usesRust('run: node scripts/check-dependency-freshness.mjs --skip-cargo'), false);
    assert.equal(usesRust('run: node scripts/bounded-cargo.mjs test'), true);
  });

  it('parses inline and list forms of needs and on', () => {
    const workflow = parseWorkflow(
      'on: [push, pull_request]\njobs:\n  a:\n    needs: [b, c]\n    runs-on: x\n  b:\n    needs:\n      - c\n  c:\n    needs: a\n',
    );
    assert.deepEqual(workflow.triggers, ['push', 'pull_request']);
    assert.deepEqual(workflow.jobs.get('a').needs, ['b', 'c']);
    assert.deepEqual(workflow.jobs.get('b').needs, ['c']);
    assert.deepEqual(workflow.jobs.get('c').needs, ['a']);
  });

  it('expands nested local actions once', () => {
    const actions = { './a': 'runs:\n  steps:\n    - uses: ./b\n', './b': 'runs:\n  steps:\n    - run: rustup show\n    - uses: ./a\n' };
    const expanded = expandLocalActions('    - uses: ./a', (path) => actions[path] ?? null);
    assert.ok(usesRust(expanded));
  });
});
