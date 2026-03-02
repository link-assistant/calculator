// Generate realistic localStorage examples for the PR description
import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

console.log("=== localStorage example entries ===\n");

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
console.log("Key: lc_cache_v2_2 + 3");
console.log("Value:");
console.log(jsonToLino({ json: entry1 }));

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
console.log("\nKey: lc_cache_v2_100 USD in EUR");
console.log("Value:");
console.log(jsonToLino({ json: entry2 }));

// Example 3: Index
const index = ["lc_cache_v2_2 + 3", "lc_cache_v2_100 USD in EUR"];
console.log("\nKey: lc_cache_index_v2");
console.log("Value:");
console.log(jsonToLino({ json: index }));

// Verify round-trip
const encoded1 = jsonToLino({ json: entry1 });
const decoded1 = linoToJson({ lino: encoded1 });
console.log("\n=== Round-trip verification ===");
console.log("expression:", decoded1.expression);
console.log("result.result:", decoded1.result.result, "(type:", typeof decoded1.result.result, ")");
console.log("After coerce:", String(decoded1.result.result));
console.log("result.lino_interpretation:", decoded1.result.lino_interpretation);
console.log("result.success:", decoded1.result.success);
console.log("appVersion:", decoded1.appVersion);
