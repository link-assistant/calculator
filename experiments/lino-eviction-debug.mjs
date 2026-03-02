// Debug: why is eviction not working?
import { jsonToLino, linoToJson } from '/tmp/gh-issue-solver-1772485931035/web/node_modules/lino-objects-codec/src/format.js';

// Simulate what getCacheIndex does
function getCacheIndex(store) {
  const raw = store['lc_cache_index_v2'];
  if (!raw) return [];
  const parsed = linoToJson({ lino: raw });
  console.log("  Index raw:", raw);
  console.log("  Index parsed:", JSON.stringify(parsed));
  return Array.isArray(parsed) ? parsed : [];
}

function saveCacheIndex(store, index) {
  store['lc_cache_index_v2'] = jsonToLino({ json: index });
  console.log("  Saved index:", store['lc_cache_index_v2']);
}

function getCacheKey(expression) {
  return 'lc_cache_v2_' + expression.trim();
}

const store = {};

// Add 3 entries
for (let i = 1; i <= 3; i++) {
  const key = getCacheKey(`expression ${i}`);
  store[key] = jsonToLino({ json: { expression: `expression ${i}`, result: { result: String(i), lino_interpretation: String(i), steps: [], success: true }, appVersion: '1.0.0', timestamp: Date.now() }});
  
  const index = getCacheIndex(store);
  console.log(`After adding expression ${i}, index:`, JSON.stringify(index));
  
  const existingIdx = index.indexOf(key);
  if (existingIdx !== -1) {
    index.splice(existingIdx, 1);
  }
  index.push(key);
  
  // Evict with limit of 2
  while (index.length > 2) {
    const evicted = index.shift();
    if (evicted) {
      delete store[evicted];
      console.log(`Evicted: ${evicted}`);
    }
  }
  
  saveCacheIndex(store, index);
  console.log(`After saving, keys in store:`, Object.keys(store));
  console.log("---");
}

console.log("Final store keys:", Object.keys(store));
console.log("expression 1 exists:", !!store['lc_cache_v2_expression 1']);
console.log("expression 2 exists:", !!store['lc_cache_v2_expression 2']);
console.log("expression 3 exists:", !!store['lc_cache_v2_expression 3']);
