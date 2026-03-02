// Generate realistic localStorage examples for the PR description
// Shows the indented Links Notation format used for cache entries,
// matching the currency rates convention used in this application.
import { Parser } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/links-notation/src/index.js';
import { escapeReference } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

const parser = new Parser();

function escapeRef(value) {
  return escapeReference({ value });
}

function serializeCacheEntry(entry) {
  const lines = [];

  lines.push('cache-entry:');
  lines.push(`  expression ${escapeRef(entry.expression)}`);
  lines.push(`  appVersion ${escapeRef(entry.appVersion)}`);
  lines.push(`  timestamp ${entry.timestamp}`);

  lines.push('result:');
  lines.push(`  result ${escapeRef(entry.result.result)}`);
  lines.push(`  lino_interpretation ${escapeRef(entry.result.lino_interpretation)}`);
  lines.push(`  success ${entry.result.success}`);

  if (entry.result.error !== undefined) {
    lines.push(`  error ${escapeRef(entry.result.error)}`);
  }
  if (entry.result.latex_input !== undefined) {
    lines.push(`  latex_input ${escapeRef(entry.result.latex_input)}`);
  }
  if (entry.result.latex_result !== undefined) {
    lines.push(`  latex_result ${escapeRef(entry.result.latex_result)}`);
  }

  lines.push('steps:');
  for (const step of entry.result.steps) {
    lines.push(`  ${escapeRef(step)}`);
  }

  if (entry.result.alternative_lino && entry.result.alternative_lino.length > 0) {
    lines.push('alternative-lino:');
    for (const alt of entry.result.alternative_lino) {
      lines.push(`  ${escapeRef(alt)}`);
    }
  }

  return lines.join('\n');
}

function serializeCacheIndex(keys) {
  const lines = ['cache-index:'];
  for (const key of keys) {
    lines.push(`  ${escapeRef(key)}`);
  }
  return lines.join('\n');
}

console.log("=== localStorage example entries (indented Links Notation format) ===\n");

// Example 1: Simple arithmetic
const entry1 = {
  expression: "2 + 3",
  result: {
    result: "5",
    lino_interpretation: "((2) + (3))",
    steps: ["Input expression: 2 + 3", "Compute: 2 + 3", "= 5"],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740925200000,
};
console.log("Key: lc_cache_v3_2 + 3");
console.log("Value:");
console.log(serializeCacheEntry(entry1));

// Example 2: Currency conversion
const entry2 = {
  expression: "100 USD in EUR",
  result: {
    result: "91.50 EUR",
    lino_interpretation: "((100 USD) in EUR)",
    steps: [
      "Input expression: 100 USD in EUR",
      "Exchange rate: 1 USD = 0.915 EUR (source: frankfurter.dev (ECB), date: 2026-03-01)",
      "= 91.5 EUR",
    ],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740925200000,
};
console.log("\nKey: lc_cache_v3_100 USD in EUR");
console.log("Value:");
console.log(serializeCacheEntry(entry2));

// Example 3: Repeating decimal with alternative_lino
const entry3 = {
  expression: "1 / 3",
  result: {
    result: "0.(3)",
    lino_interpretation: "(1/3)",
    alternative_lino: ["0.333...", "0.3\u0305"],
    steps: [],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740925200000,
};
console.log("\nKey: lc_cache_v3_1 / 3");
console.log("Value:");
console.log(serializeCacheEntry(entry3));

// Example 4: Cache index
const index = [
  "lc_cache_v3_2 + 3",
  "lc_cache_v3_100 USD in EUR",
  "lc_cache_v3_1 / 3",
];
console.log("\nKey: lc_cache_index_v3");
console.log("Value:");
console.log(serializeCacheIndex(index));

// Verify round-trip
console.log("\n=== Round-trip verification ===");
const encoded1 = serializeCacheEntry(entry1);
const links = parser.parse(encoded1);

// Find sections
let entrySection, resultSection, stepsSection;
for (const link of links) {
  if (link.id === 'cache-entry') entrySection = link;
  else if (link.id === 'result') resultSection = link;
  else if (link.id === 'steps') stepsSection = link;
}

function extractPairs(section) {
  const data = {};
  for (const val of section.values || []) {
    if (val.values && val.values.length === 2) {
      data[val.values[0].id] = val.values[1].id;
    }
  }
  return data;
}

function extractList(section) {
  if (!section) return [];
  const items = [];
  for (const val of section.values || []) {
    if (val.id !== null && val.id !== undefined && (val.values?.length ?? 0) === 0) {
      items.push(String(val.id));
    }
  }
  return items;
}

const entryData = extractPairs(entrySection);
const resultData = extractPairs(resultSection);
const steps = extractList(stepsSection);

console.log("expression:", entryData.expression);
console.log("appVersion:", entryData.appVersion);
console.log("result.result:", resultData.result, "(type:", typeof resultData.result, ")");
console.log("After String():", String(resultData.result));
console.log("result.lino_interpretation:", resultData.lino_interpretation);
console.log("result.success:", resultData.success, "-> parsed:", resultData.success === 'true');
console.log("steps count:", steps.length);
console.log("steps[0]:", steps[0]);
