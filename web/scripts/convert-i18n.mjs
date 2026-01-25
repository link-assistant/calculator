/**
 * Script to convert i18n JSON files to Links Notation format
 */
import { linoToJson } from 'lino-objects-codec';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function escapeValue(str) {
  if (str === null || str === undefined) return 'null';
  if (typeof str !== 'string') return String(str);

  const s = String(str);
  // Check if needs quoting
  const needsQuoting = s.includes(' ') || s.includes(':') || s.includes('(') ||
                       s.includes(')') || s.includes("'") || s.includes('"') ||
                       s.includes('\n') || s.includes('\t') || s === '';

  if (!needsQuoting) return s;

  // Choose quote character
  const hasSingle = s.includes("'");
  const hasDouble = s.includes('"');

  if (hasDouble && !hasSingle) {
    return "'" + s + "'";
  }
  if (hasSingle && !hasDouble) {
    return '"' + s + '"';
  }
  if (hasSingle && hasDouble) {
    // Escape single quotes
    return "'" + s.replace(/'/g, "\\'") + "'";
  }
  return "'" + s + "'";
}

// Convert a nested object to Links Notation key-value pairs
function objectToLinoPairs(obj, indent = 0) {
  const prefix = '  '.repeat(indent);
  const lines = [];

  for (const [key, value] of Object.entries(obj)) {
    const escapedKey = escapeValue(key);

    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      // Nested object: (key ((nested pairs)))
      const nestedPairs = objectToLinoPairs(value, indent + 1);
      lines.push(prefix + '(' + escapedKey + ' (');
      lines.push(nestedPairs);
      lines.push(prefix + '))');
    } else {
      // Simple value: (key value)
      lines.push(prefix + '(' + escapedKey + ' ' + escapeValue(value) + ')');
    }
  }

  return lines.join('\n');
}

// Wrap entire object in parentheses for proper root structure
function jsonToLinoFile(obj) {
  const content = objectToLinoPairs(obj, 1);
  return '(\n' + content + '\n)\n';
}

// Read and convert all JSON files
const localesDir = path.join(__dirname, '../src/i18n/locales');
const files = fs.readdirSync(localesDir).filter(f => f.endsWith('.json'));

for (const file of files) {
  const jsonPath = path.join(localesDir, file);
  const linoPath = path.join(localesDir, file.replace('.json', '.lino'));

  console.log(`Converting ${file}...`);

  const json = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));
  const lino = jsonToLinoFile(json);

  // Verify round-trip
  const backJson = linoToJson({ lino });
  const original = JSON.stringify(json);
  const roundTrip = JSON.stringify(backJson);

  if (original !== roundTrip) {
    console.error(`  ERROR: Round-trip mismatch for ${file}`);
    console.error('  Original:', original.substring(0, 100));
    console.error('  RoundTrip:', roundTrip.substring(0, 100));
    continue;
  }

  fs.writeFileSync(linoPath, lino, 'utf-8');
  console.log(`  Created ${file.replace('.json', '.lino')}`);
}

console.log('\nDone!');
