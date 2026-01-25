/**
 * Script to convert i18n locale files to indented Links Notation (.lino) format
 */

import { createRequire } from 'module';
import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Create require function to load from web/node_modules
const require = createRequire(join(__dirname, '../web/package.json'));
const { escapeReference, linoToJson } = require('lino-objects-codec');

const localesDir = join(__dirname, '../web/src/i18n/locales');

/**
 * Convert a JSON object to indented Links Notation format.
 * This creates a highly readable format where each section is a link
 * with key-value doublets as children.
 */
function jsonToIndentedLino(obj) {
  const lines = [];

  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      // Nested object - use indented format with colon
      const escapedKey = escapeReference({ value: key });
      lines.push(`${escapedKey}:`);

      for (const [nestedKey, nestedValue] of Object.entries(value)) {
        const escapedNestedKey = escapeReference({ value: nestedKey });
        const escapedNestedValue = escapeReference({ value: String(nestedValue) });
        lines.push(`  ${escapedNestedKey} ${escapedNestedValue}`);
      }
    } else {
      // Simple value
      const escapedKey = escapeReference({ value: key });
      const escapedValue = escapeReference({ value: String(value) });
      lines.push(`${escapedKey} ${escapedValue}`);
    }
  }

  return lines.join('\n');
}

// Read existing .lino files (which are in parenthesized format) and convert to indented format
const files = readdirSync(localesDir).filter(f => f.endsWith('.lino'));

for (const file of files) {
  const linoPath = join(localesDir, file);

  // Read current lino content and parse it to JSON
  const linoContent = readFileSync(linoPath, 'utf-8');
  const json = linoToJson({ lino: linoContent });

  // Convert JSON to indented format
  const indentedLino = jsonToIndentedLino(json);

  // Write back the indented format
  writeFileSync(linoPath, indentedLino + '\n');
  console.log(`Converted ${file} to indented format`);
}

console.log('\nDone! Converted all .lino files to indented format.');
