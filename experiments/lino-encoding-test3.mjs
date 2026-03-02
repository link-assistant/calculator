// Understand what the lino encoding looks like for cache entries
import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

// Test index encoding
const index = ['lc_cache_v2_2 + 3', 'lc_cache_v2_100 USD in EUR'];
const encodedIndex = jsonToLino({ json: index });
console.log("Index encoded:");
console.log(encodedIndex);
const decodedIndex = linoToJson({ lino: encodedIndex });
console.log("Index decoded:", JSON.stringify(decodedIndex));
console.log("Match:", JSON.stringify(decodedIndex) === JSON.stringify(index));

// Test cache entry encoding
const entry = {
  expression: "2 + 3",
  result: {
    result: "5",
    lino_interpretation: "((2) + (3))",
    steps: ["Input expression: 2 + 3"],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};

const encodedEntry = jsonToLino({ json: entry });
console.log("\nCache entry encoded:");
console.log(encodedEntry);

const decoded = linoToJson({ lino: encodedEntry });
console.log("\nDecoded:");
console.log(JSON.stringify(decoded, null, 2));

// What does a numeric result look like?
const numericResult = {
  expression: "42",
  result: {
    result: "42",
    lino_interpretation: "42",
    steps: [],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};

const encodedNumeric = jsonToLino({ json: numericResult });
console.log("\nNumeric result encoded:");
console.log(encodedNumeric);
const decodedNumeric = linoToJson({ lino: encodedNumeric });
const r = decodedNumeric;
console.log("result.result type after decode:", typeof r.result.result);
console.log("result.result value:", r.result.result);
console.log("After coerce:", String(r.result.result));

// Empty array
const emptyEntry = {
  expression: "2 + 3",
  result: {
    result: "5",
    lino_interpretation: "5",
    steps: [],
    success: true,
  },
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};
const encodedEmpty = jsonToLino({ json: emptyEntry });
console.log("\nEntry with empty steps array:");
console.log(encodedEmpty);
const decodedEmpty = linoToJson({ lino: encodedEmpty });
console.log("Decoded:", JSON.stringify(decodedEmpty));
