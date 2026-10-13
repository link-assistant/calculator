// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3baa7636edbfa8ccf9ec1bf864d4020986941b95584b6659dd72974726ba4b9e bytes=1717

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Utility helpers exposed for downstream consumers.
// |  *
// |  * These helpers were originally crate-private; they are made public so that
// |  * Rust consumers of `link-calculator` can render the same issue-report links
// |  * and apply the same input truncation rules the calculator itself uses.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=b1310cabec0948aa5e58d54f833b776fdea735597e2de9a4d778ca7e53381602
// | import { charIndices, sliceBytes } from './rust/str.js';
use crate::rust::str_::{charIndices, sliceBytes};

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * Generates a GitHub issue link for unrecognized input.
// |  * @param {string} input
// |  * @param {string} error
// |  * @returns {string}
// |  */
// | export function generateIssueLink(input, error) {
// |   const title = `Unrecognized input: ${truncate(input, 50)}`;
// |   const body = `## Input that failed to parse\n\n\`\`\`\n${input}\n\`\`\`\n\n## Error message\n\n\`\`\`\n${error}\n\`\`\`\n\n## Expected behavior\n\nPlease describe what you expected the calculator to do with this input.`;
// |   return `https://github.com/link-assistant/calculator/issues/new?title=${urlencodingEncode(title)}&body=${urlencodingEncode(body)}`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The first `maxChars` chars (code points) of `s`.
// |  * @param {string} s
// |  * @param {number} maxChars
// |  * @returns {string}
// |  */
// | export function truncate(s, maxChars) {
// |   const nth = charIndices(s)[maxChars] ?? null;
// |   if (nth !== null) {
// |     const [idx] = nth;
// |     return sliceBytes(s, 0, idx);
// |   }
// |   return s;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {string} s */
// | function urlencodingEncode(s) {
// |   let result = '';
// |   for (const c of s) {
// |     if (/^[A-Za-z0-9\-_.~]$/u.test(c)) {
// |       result += c;
// |     } else if (c === ' ') {
// |       result += '%20';
// |     } else if (c === '\n') {
// |       result += '%0A';
// |     } else {
// |       for (const byte of new TextEncoder().encode(c)) {
// |         result += `%${byte.toString(16).toUpperCase().padStart(2, '0')}`;
// |       }
// |     }
// |   }
// |   return result;
// | }
