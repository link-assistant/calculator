#!/usr/bin/env node
/**
 * Checks the JavaScript engine against the Rust parity goldens (issue #254).
 *
 * Mirrors tests/parity_goldens.rs: evaluates every input of
 * parity/corpus.json in each scenario, at the fixed instant `clock`, with the
 * exchange rates of data/currency/*.lino dated on or before `ratesCutoff`,
 * and compares the plan and result JSON with parity/goldens/<scenario>.json.
 *
 * Usage: node scripts/check-parity.mjs [--verbose] [--limit N]
 *   --verbose / PARITY_VERBOSE=1  print every mismatch (and each input as it runs)
 *   PARITY_FILTER=<substring>      only inputs containing the substring
 */

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Calculator } from '../src/index.js';
import { setFixedNow } from '../src/clock.js';
import { compareStrings, Panic } from '../src/rust/core.js';
import { DateTime, Utc } from '../src/rust/chrono.js';
import { CurrencyDatabase } from '../src/types/index.js';
import { lines, splitWhitespace, stripPrefix, trim, trimMatches } from '../src/rust/str.js';
import { parseF64 } from '../src/rust/num.js';

const root = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const verbose = args.includes('--verbose') || process.env.PARITY_VERBOSE === '1';
const limitIndex = args.indexOf('--limit');
const limit = limitIndex >= 0 ? Number(args[limitIndex + 1]) : 20;
const filter = process.env.PARITY_FILTER ?? '';

/**
 * @typedef {{ from: string, to: string, source: string, rows: [string, number][], content: string }} RateFile
 */

/**
 * The latest positive rate, chosen the way build.rs chooses it.
 * @param {RateFile} file
 * @returns {[string, number] | null}
 */
function latest(file) {
  /** @type {[string, number] | null} */
  let best = null;
  for (const [date, rate] of file.rows) {
    const isNewer = best === null || compareStrings(date, best[0]) > 0;
    if (Number.isFinite(rate) && rate > 0 && isNewer) {
      best = [date, rate];
    }
  }
  return best;
}

/** @param {string} s @returns {number | null} */
function parseRate(s) {
  try {
    return parseF64(s);
  } catch {
    return null;
  }
}

/**
 * @param {string} cutoff
 * @returns {RateFile[]}
 */
function readRateFiles(cutoff) {
  const dir = join(root, 'data/currency');
  const names = readdirSync(dir).filter((n) => n.endsWith('.lino')).sort(compareStrings);
  return names.map((name) => {
    const text = readFileSync(join(dir, name), 'utf8');
    /** @type {RateFile} */
    const file = { from: '', to: '', source: 'unknown', rows: [], content: '' };
    let inRates = false;
    for (const line of lines(text)) {
      const trimmed = trim(line);
      let rest;
      if (trimmed === 'data:' || (trimmed === 'rates:' && file.from !== '')) {
        inRates = true;
      } else if (inRates) {
        const parts = splitWhitespace(trimmed);
        if (parts.length >= 2) {
          const [date, value] = parts;
          if (compareStrings(date, cutoff) > 0) {
            continue;
          }
          const rate = parseRate(value);
          if (rate !== null) {
            file.rows.push([date, rate]);
          }
        }
      } else if ((rest = stripPrefix(trimmed, 'from ')) !== null) {
        file.from = trim(rest).toUpperCase();
      } else if ((rest = stripPrefix(trimmed, 'to ')) !== null) {
        file.to = trim(rest).toUpperCase();
      } else if ((rest = stripPrefix(trimmed, 'source ')) !== null) {
        file.source = trimMatches(trim(rest), (/** @type {string} */ c) => c === "'" || c === '"');
      }
      file.content += `${line}\n`;
    }
    return file;
  });
}

/**
 * The newest date and the JSON object of latest rates of the files `pick` keys.
 * @param {RateFile[]} files
 * @param {(file: RateFile) => string | null} pick
 * @returns {[string, string]}
 */
function latestRates(files, pick) {
  /** @type {Map<string, number>} */
  const rates = new Map();
  let newest = '';
  for (const file of files) {
    const key = pick(file);
    const l = latest(file);
    if (key !== null && l !== null) {
      rates.set(key, l[1]);
      if (compareStrings(l[0], newest) > 0) {
        newest = l[0];
      }
    }
  }
  const sorted = [...rates].sort((a, b) => compareStrings(a[0], b[0]));
  return [newest, JSON.stringify(Object.fromEntries(sorted))];
}

/**
 * @param {RateFile[]} files
 * @param {any} scenario
 */
function newCalculator(files, scenario) {
  const bundled = [];
  for (const file of files) {
    const l = latest(file);
    if (l !== null) {
      bundled.push([file.from, file.to, file.source, l[0], l[1]]);
    }
  }
  const calculator = Calculator.new();
  calculator.parserMut()._currencyDb = CurrencyDatabase.withBundledRates(bundled);

  if (typeof scenario.timezoneOffsetMinutes === 'number') {
    calculator.setTimezoneOffset(scenario.timezoneOffsetMinutes);
  }
  if (scenario.history === true) {
    for (const file of files) {
      calculator.loadRatesFromConsolidatedLino(file.content);
    }
  }
  if (typeof scenario.ecbBase === 'string') {
    const base = scenario.ecbBase;
    const [date, rates] = latestRates(files, (f) => (f.from === base ? f.to.toLowerCase() : null));
    calculator.updateRatesFromApi(base, date, rates);
  }
  if (scenario.cbr === true) {
    const [date, rates] = latestRates(files, (f) => (f.to === 'RUB' ? f.from.toLowerCase() : null));
    calculator.updateCbrRatesFromApi(date, rates);
  }
  if (scenario.crypto !== undefined) {
    const { base, date, rates } = scenario.crypto;
    calculator.updateCryptoRatesFromApi(base, date, JSON.stringify(rates));
  }
  return calculator;
}

/** Sorts object keys, as the Rust test does. @param {any} value @returns {any} */
function canonical(value) {
  if (Array.isArray(value)) {
    return value.map(canonical);
  }
  if (value !== null && typeof value === 'object') {
    const keys = Object.keys(value).sort(compareStrings);
    return Object.fromEntries(keys.map((k) => [k, canonical(value[k])]));
  }
  return value;
}

/**
 * The `f64` that serde_json 1.0 (without the `float_roundtrip` feature, as the
 * Rust test links it) reads back from the shortest text of `x`: the decimal
 * significand times or divided by one power of ten from its table, which is
 * not always correctly rounded. The goldens hold these values, so the
 * JavaScript results take the same path (`f64_from_parts` of serde_json).
 * @param {number} x
 */
function serdeJsonRoundTrip(x) {
  if (!Number.isFinite(x) || (Number.isInteger(x) && Math.abs(x) <= Number.MAX_SAFE_INTEGER)) {
    return x;
  }
  const [mantissa, exp] = Math.abs(x).toExponential().split('e');
  const [intPart, frac = ''] = mantissa.split('.');
  let f = Number(intPart + frac);
  let exponent = Number(exp) - frac.length;
  for (;;) {
    if (Math.abs(exponent) <= 308) {
      f = exponent >= 0 ? f * Number(`1e${exponent}`) : f / Number(`1e${-exponent}`);
      break;
    }
    if (f === 0) {
      break;
    }
    f /= 1e308;
    exponent += 308;
  }
  return x < 0 ? -f : f;
}

/** @param {any} value @returns {any} */
function roundTripNumbers(value) {
  if (typeof value === 'number') {
    return serdeJsonRoundTrip(value);
  }
  if (Array.isArray(value)) {
    return value.map(roundTripNumbers);
  }
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, roundTripNumbers(v)]));
  }
  return value;
}

/** @param {string} text */
function parseOutput(text) {
  try {
    return canonical(roundTripNumbers(JSON.parse(text)));
  } catch {
    return text;
  }
}

/**
 * Evaluates every input with one calculator, as the web worker does.
 * @param {RateFile[]} files
 * @param {any} scenario
 * @param {string[]} inputs
 * @returns {[any, any][]}
 */
function evaluate(files, scenario, inputs) {
  let calculator = newCalculator(files, scenario);
  return inputs.map((input) => {
    if (verbose) {
      process.stderr.write(`parity: ${JSON.stringify(input)}\n`);
    }
    try {
      const plan = calculator.plan(input);
      const result = calculator.calculate(input);
      return [parseOutput(plan), parseOutput(result)];
    } catch (thrown) {
      calculator = newCalculator(files, scenario);
      const message = thrown instanceof Panic ? thrown.message : `JS error: ${thrown?.stack ?? thrown}`;
      return [{ panic: message }, { panic: message }];
    }
  });
}

/** Structural equality of parsed JSON. @param {any} a @param {any} b */
function same(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

const corpus = JSON.parse(readFileSync(join(root, 'parity/corpus.json'), 'utf8'));
const clock = DateTime.parseFromRfc3339(corpus.clock).withTimezone(Utc);
const files = readRateFiles(corpus.ratesCutoff);
/** @type {string[]} */
const inputs = corpus.inputs;

setFixedNow(clock);
const scenarios = corpus.scenarios;
const defaultCases = evaluate(files, scenarios.default, inputs);
let total = 0;
/** @type {string[]} */
const mismatches = [];
for (const [name, scenario] of Object.entries(scenarios)) {
  const golden = JSON.parse(readFileSync(join(root, 'parity/goldens', `${name}.json`), 'utf8'));
  const results = name === 'default' ? defaultCases : evaluate(files, scenario, inputs);
  /** @type {Map<string, [any, any]>} */
  const actual = new Map();
  inputs.forEach((input, i) => {
    if (name === 'default' || !same(defaultCases[i], results[i])) {
      actual.set(input, results[i]);
    }
  });
  const expectedInputs = new Set();
  for (const c of golden.cases) {
    expectedInputs.add(c.input);
    if (!c.input.includes(filter)) {
      continue;
    }
    total += 1;
    const got = actual.get(c.input) ?? (name === 'default' ? null : defaultCases[inputs.indexOf(c.input)]);
    if (got === null || !same([c.plan, c.result], got)) {
      const parts = [`[${name}] ${JSON.stringify(c.input)}`];
      if (got === null) {
        parts.push('  missing');
      } else {
        if (!same(c.plan, got[0])) {
          parts.push(`  plan expected ${JSON.stringify(c.plan)}`, `  plan actual   ${JSON.stringify(got[0])}`);
        }
        if (!same(c.result, got[1])) {
          parts.push(`  result expected ${JSON.stringify(c.result)}`, `  result actual   ${JSON.stringify(got[1])}`);
        }
      }
      mismatches.push(parts.join('\n'));
    }
  }
  // Cases the JavaScript engine differs on from default but the golden does not list.
  for (const input of actual.keys()) {
    if (!expectedInputs.has(input) && input.includes(filter)) {
      total += 1;
      const got = actual.get(input);
      const base = defaultCases[inputs.indexOf(input)];
      mismatches.push(`[${name}] ${JSON.stringify(input)}\n  expected same as default ${JSON.stringify(base)}\n  actual   ${JSON.stringify(got)}`);
    }
  }
}
setFixedNow(null);

const shown = verbose ? mismatches : mismatches.slice(0, limit);
for (const m of shown) {
  console.log(m);
}
console.log(`parity: ${total - mismatches.length}/${total} cases match (${inputs.length} inputs, ${Object.keys(scenarios).length} scenarios)`);
if (mismatches.length > 0) {
  if (shown.length < mismatches.length) {
    console.log(`(${mismatches.length - shown.length} more mismatches; use --verbose to print all)`);
  }
  process.exit(1);
}
