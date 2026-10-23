import { expect, test } from 'vitest';
import { ExpressionParser } from '../../src/grammar/index.js';
import { Err } from '../../src/rust/core.js';
import { loadVectors } from '../vectors.js';

// The parsed Expression of every input of parity/corpus.json plus
// edge cases, printed by the Rust evaluator itself
// (experiments/issue-254/rust-vectors/src/bin/expression_parser.rs).

/**
 * @typedef {{
 *   input: string, lino?: string, display?: string, json?: unknown,
 *   live?: boolean, interpretations?: string[], error?: string,
 * }} Vector
 */

/** @type {Vector[]} */
const VECTORS = loadVectors('expression_parser');

const PARSER = ExpressionParser.new();

/** @param {string} input @returns {Vector} */
function parse(input) {
  try {
    const expr = PARSER.parse(input);
    /** @type {string[]} */
    let interpretations = [];
    try {
      interpretations = PARSER.parseInterpretations(input).map((e) => e.toLino());
    } catch (thrown) {
      if (!(thrown instanceof Err)) {
        throw thrown;
      }
    }
    return {
      input,
      lino: expr.toLino(),
      display: expr.toString(),
      json: JSON.parse(JSON.stringify(expr)),
      live: expr.containsLiveTime(),
      interpretations,
    };
  } catch (thrown) {
    if (thrown instanceof Err) {
      return { input, error: /** @type {import("../../src/error.js").CalculatorError} */ (thrown.error).fmtDebug() };
    }
    throw thrown;
  }
}

/**
 * Masks the `'current … time': <instant>` timestamps of a live lino.
 * @param {string | undefined} lino
 */
function maskInstant(lino) {
  return lino?.replace(/(': )\d{4}-\d{2}-\d{2}( \d{2}:\d{2}:\d{2})?/gu, '$1<now>');
}

/** @param {Vector} vector */
function comparable(vector) {
  if (vector.live === true) {
    // `now` embeds the current instant in "json"/"display".
    // The lino keeps the structure; only the labelled instant is masked.
    const { display: _display, json: _json, ...rest } = vector;
    return {
      ...rest,
      lino: maskInstant(rest.lino),
      interpretations: rest.interpretations?.map(maskInstant),
    };
  }
  return vector;
}

/**
 * JSON with sorted object keys, so key order does not matter.
 * @param {unknown} value
 * @returns {string}
 */
function canonical(value) {
  return JSON.stringify(value, (_key, v) => (v !== null && typeof v === 'object' && !Array.isArray(v)
    ? Object.fromEntries(Object.entries(v).sort(([a], [b]) => (a < b ? -1 : 1)))
    : v));
}

test('expression parser vectors match the Rust parser', () => {
  expect(VECTORS.length > 2000).toBe(true);
  const mismatches = VECTORS
    .map((vector) => ({ expected: comparable(vector), actual: comparable(parse(vector.input)) }))
    .filter(({ expected, actual }) => canonical(actual) !== canonical(expected));
  expect(mismatches.slice(0, 5)).toEqual([]);
  expect(mismatches.length).toBe(0);
});
