/**
 * Utility helpers exposed for downstream consumers.
 *
 * These helpers were originally crate-private; they are made public so that
 * Rust consumers of `link-calculator` can render the same issue-report links
 * and apply the same input truncation rules the calculator itself uses.
 */

import { charIndices, sliceBytes } from './rust/str.js';

/**
 * Generates a GitHub issue link for unrecognized input.
 * @param {string} input
 * @param {string} error
 * @returns {string}
 */
export function generateIssueLink(input, error) {
  const title = `Unrecognized input: ${truncate(input, 50)}`;
  const body = `## Input that failed to parse\n\n\`\`\`\n${input}\n\`\`\`\n\n## Error message\n\n\`\`\`\n${error}\n\`\`\`\n\n## Expected behavior\n\nPlease describe what you expected the calculator to do with this input.`;
  return `https://github.com/link-assistant/calculator/issues/new?title=${urlencodingEncode(title)}&body=${urlencodingEncode(body)}`;
}

/**
 * The first `maxChars` chars (code points) of `s`.
 * @param {string} s
 * @param {number} maxChars
 * @returns {string}
 */
export function truncate(s, maxChars) {
  const nth = charIndices(s)[maxChars] ?? null;
  if (nth !== null) {
    const [idx] = nth;
    return sliceBytes(s, 0, idx);
  }
  return s;
}

/** @param {string} s */
function urlencodingEncode(s) {
  let result = '';
  for (const c of s) {
    if (/^[A-Za-z0-9\-_.~]$/u.test(c)) {
      result += c;
    } else if (c === ' ') {
      result += '%20';
    } else if (c === '\n') {
      result += '%0A';
    } else {
      for (const byte of new TextEncoder().encode(c)) {
        result += `%${byte.toString(16).toUpperCase().padStart(2, '0')}`;
      }
    }
  }
  return result;
}
