// Test encode/decode which preserves types

import { encode, decode } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/codec.js';
import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

const exampleResult = {
  result: "5",
  lino_interpretation: "((2) + (3))",
  steps: ["Input expression: 2 + 3", "Compute: 2 + 3", "= 5"],
  success: true,
};

const cacheEntry = {
  expression: "2 + 3",
  result: exampleResult,
  appVersion: "0.5.1",
  timestamp: 1740000000000,
};

console.log("=== encode/decode (type-preserving) ===");
const encoded = encode({ obj: cacheEntry });
console.log("Encoded:");
console.log(encoded);
console.log();
const decoded = decode({ notation: encoded });
console.log("Decoded match:", JSON.stringify(decoded) === JSON.stringify(cacheEntry));

console.log("\n=== jsonToLino format (human-readable, NOT type-preserving for string numbers) ===");
const lino = jsonToLino({ json: cacheEntry });
console.log("Encoded:");
console.log(lino);
console.log();

console.log("\n=== What does 0.5.1 look like encoded? ===");
// The issue is "0.5.1" is valid but might be a problem
const versionEncoded = jsonToLino({ json: "0.5.1" });
console.log("Version encoded:", versionEncoded);
const versionDecoded = linoToJson({ lino: versionEncoded });
console.log("Version decoded:", versionDecoded);
