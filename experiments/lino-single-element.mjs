import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

// Test single element array
const single = ['lc_cache_v2_2 + 3'];
const encoded = jsonToLino({ json: single });
console.log("Single element array encoded:", encoded);
const decoded = linoToJson({ lino: encoded });
console.log("Decoded:", JSON.stringify(decoded));
console.log("Is array?", Array.isArray(decoded));
// -> This produces "lc_cache_v2_2 + 3" (string, not array!)

// Two element array  
const two = ['lc_cache_v2_2 + 3', 'lc_cache_v2_100 USD'];
const encoded2 = jsonToLino({ json: two });
console.log("\nTwo element array encoded:", encoded2);
const decoded2 = linoToJson({ lino: encoded2 });
console.log("Decoded:", JSON.stringify(decoded2));
console.log("Is array?", Array.isArray(decoded2));

// Empty array
const empty = [];
const encodedEmpty = jsonToLino({ json: empty });
console.log("\nEmpty array encoded:", encodedEmpty);
const decodedEmpty = linoToJson({ lino: encodedEmpty });
console.log("Decoded:", JSON.stringify(decodedEmpty));
console.log("Is array?", Array.isArray(decodedEmpty));
