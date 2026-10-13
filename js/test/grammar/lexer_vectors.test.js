import { expect, test } from 'vitest';
import { Lexer } from '../../src/grammar/lexer.js';
import { Err } from '../../src/rust/core.js';
import { loadVectors } from '../vectors.js';

// The token stream of every input of parity/corpus.json plus lexer edge
// cases, printed by the Rust lexer itself
// (experiments/issue-254/rust-vectors/src/bin/lexer.rs).

/** @type {Array<{ input: string, tokens?: string[], error?: string }>} */
const VECTORS = loadVectors('lexer');

/** @param {string} input */
function tokenize(input) {
  try {
    return { input, tokens: Lexer.new(input).tokenize().map((t) => t.fmtDebug()) };
  } catch (thrown) {
    if (thrown instanceof Err) {
      return { input, error: /** @type {import("../../src/error.js").CalculatorError} */ (thrown.error).fmtDebug() };
    }
    throw thrown;
  }
}

test('lexer vectors match the Rust token streams', () => {
  expect(VECTORS.length > 2000).toBe(true);
  const mismatches = VECTORS
    .map((vector) => ({ expected: vector, actual: tokenize(vector.input) }))
    .filter(({ expected, actual }) => actual.error !== expected.error
      || JSON.stringify(actual.tokens) !== JSON.stringify(expected.tokens));
  expect(mismatches.slice(0, 5)).toEqual([]);
  expect(mismatches.length).toBe(0);
});
