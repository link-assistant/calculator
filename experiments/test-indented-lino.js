/**
 * Test indented Links Notation format with linoToJson
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const require = createRequire(join(__dirname, '../web/package.json'));
const { linoToJson, jsonToLino } = require('lino-objects-codec');
const { Parser } = require('links-notation');

// Test the indented format
const indentedLino = `app:
  title 'Link Calculator'
  subtitle 'Grammar-based expression calculator'
input:
  label Expression
  placeholder 'Enter an expression'`;

console.log('=== Indented format input ===');
console.log(indentedLino);

console.log('\n=== Result of linoToJson ===');
const result = linoToJson({ lino: indentedLino });
console.log(JSON.stringify(result, null, 2));

// Use links-notation Parser directly
console.log('\n=== Using links-notation Parser directly ===');
const parser = new Parser();
const links = parser.parse(indentedLino);
console.log('Parsed links count:', links.length);
console.log('Links:', JSON.stringify(links, null, 2));

// Convert parsed links to i18n JSON format
function linksToI18nJson(links) {
  const result = {};

  for (const link of links) {
    if (!link.id) continue;

    const sectionKey = link.id;

    if (link.values && link.values.length > 0) {
      const nestedObj = {};

      for (const valueLink of link.values) {
        if (valueLink.values && valueLink.values.length >= 2) {
          const nestedKey = valueLink.values[0].id || '';
          const nestedValue = valueLink.values[1].id || '';
          nestedObj[nestedKey] = nestedValue;
        }
      }

      result[sectionKey] = nestedObj;
    }
  }

  return result;
}

const i18nResult = linksToI18nJson(links);
console.log('\n=== Custom linksToI18nJson result ===');
console.log(JSON.stringify(i18nResult, null, 2));

// Test parenthesized format for comparison
const parenthesizedLino = `(
  (app (
    (title 'Link Calculator')
    (subtitle 'Grammar-based expression calculator')
  ))
  (input (
    (label Expression)
    (placeholder 'Enter an expression')
  ))
)`;

console.log('\n=== Parenthesized format with linoToJson ===');
const result2 = linoToJson({ lino: parenthesizedLino });
console.log(JSON.stringify(result2, null, 2));
