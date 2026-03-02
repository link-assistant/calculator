// Experiment to understand how lino-objects-codec encodes CalculationResult

import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

// Example CalculationResult
const exampleResult = {
  result: "5",
  lino_interpretation: "((2) + (3))",
  steps: ["Input expression: 2 + 3", "Compute: 2 + 3", "= 5"],
  success: true,
};

// Cache entry structure
const cacheEntry = {
  expression: "2 + 3",
  result: exampleResult,
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};

console.log("=== Encoding CacheEntry to lino ===");
const encoded = jsonToLino({ json: cacheEntry });
console.log("Encoded:");
console.log(encoded);
console.log();

console.log("=== Decoding back to JSON ===");
const decoded = linoToJson({ lino: encoded });
console.log("Decoded:");
console.log(JSON.stringify(decoded, null, 2));
console.log();

console.log("=== Round-trip check ===");
console.log("Match:", JSON.stringify(decoded) === JSON.stringify(cacheEntry));
console.log("Original:", JSON.stringify(cacheEntry));
console.log("Decoded:", JSON.stringify(decoded));

// Example with currency/USD result
const currencyResult = {
  result: "91.50 EUR",
  lino_interpretation: "((100 USD) in EUR)",
  steps: [
    "Input expression: 100 USD in EUR",
    "Exchange rate: 1 USD = 0.915 EUR",
    "= 91.5 EUR"
  ],
  success: true,
};

const currencyCacheEntry = {
  expression: "100 USD in EUR",
  result: currencyResult,
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};

console.log("\n=== Currency Result Encoding ===");
const encodedCurrency = jsonToLino({ json: currencyCacheEntry });
console.log("Encoded:");
console.log(encodedCurrency);
