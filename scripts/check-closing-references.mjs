#!/usr/bin/env node

/**
 * Check that a pull request description closes exactly the issues it says it
 * closes (issue #247).
 *
 * PR #228 was a planning PR: its description said "Merging this PR closes
 * only #227. The open sub-issues #236 to #246 stay open", yet ended with a
 * `Fixes` line for each of #229 to #246. Automated link repair appended those
 * lines after every working session, and the merge closed 11 issues that had
 * not been implemented. GitHub honours the keywords, not the prose, so this
 * check fails when the two disagree:
 *
 * 1. `keep-open`: a sentence says an issue stays open, remains open, is not
 *    closed, and so on, but a closing keyword also references it.
 * 2. `closes-only`: a sentence says the PR "closes only #N" but closing
 *    keywords reference other issues too.
 * 3. `part-of`: the same issue is referenced as `Part of #N` (or `Refs`,
 *    `Related to`) and with a closing keyword.
 * 4. `missing-keyword`: `Fixes #1, #2` or `Fixes #1 and #2` closes only #1;
 *    GitHub needs one keyword per issue.
 * 5. `negated-keyword`: "does not close #N" or "won't fix #N" still closes #N,
 *    because GitHub ignores the negation (verified on PR #261, see
 *    docs/case-studies/issue-247/data/negated-keyword-probe.md).
 *
 * Closing keywords inside fenced code blocks, inline code and HTML comments
 * are ignored, as GitHub ignores them; so are statements quoted there.
 *
 * Usage:
 *   node scripts/check-closing-references.mjs [--body-file <path>] [--verbose]
 *
 * Without --body-file the description is read from the pull_request payload
 * at $GITHUB_EVENT_PATH. Exits with code 1 and one ::error annotation per
 * contradiction. --verbose (or CLOSING_REFERENCES_VERBOSE=1) prints every
 * closing and keep-open reference it parsed.
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const KEYWORD = String.raw`(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)`;
// `#12`, `owner/repo#12` or a full issue URL, captured as owner/repo and number.
const REFERENCE = String.raw`(?:https://github\.com/([\w.-]+/[\w.-]+)/(?:issues|pull)/(\d+)|(?:([\w.-]+/[\w.-]+))?#(\d+))`;
const KEEP_OPEN =
  /\b(?:stays?|remains?|kept|keeps?|left|leaves?|staying|remaining)\b[^.;\n]{0,40}\bopen\b|\b(?:do(?:es)?\s+not|doesn't|don't|never|without)\s+(?:close|closing|fix|fixing|resolve|resolving)\b|\bnot\s+(?:be\s+)?(?:closed|fixed|resolved)\b|\bremain\s+unimplemented\b/i;
const CLOSES_ONLY = new RegExp(String.raw`\b(?:closes|fixes|resolves)\s+only\s+${REFERENCE}|\bonly\s+(?:closes|fixes|resolves)\s+${REFERENCE}`, 'i');
const PART_OF = new RegExp(String.raw`\b(?:part\s+of|refs?|related\s+to|relates\s+to|see\s+also)\b:?\s+${REFERENCE}`, 'gi');
// Words before a closing keyword that GitHub does not read as a negation.
const NEGATION = /\b(?:not|never|cannot|can't|won't|doesn't|don't|didn't|isn't|shouldn't|wouldn't)[ \t]+(?:\w+[ \t]+)?$/i;
const MAX_RANGE = 500;

/** Remove the parts of Markdown GitHub does not scan for closing keywords. */
export function stripIgnoredMarkdown(body) {
  return body
    .replace(/<!--[\s\S]*?-->/g, ' ')
    .replace(/^(\s*)(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\2[^\n]*$/gm, ' ')
    .replace(/`[^`\n]*`/g, ' ');
}

/** `owner/repo#N` from the four capture groups of REFERENCE, or null. */
function key([urlRepo, urlNumber, shortRepo, shortNumber], repository) {
  const number = urlNumber ?? shortNumber;
  if (number === undefined) {
    return null;
  }
  return `${(urlRepo ?? shortRepo ?? repository).toLowerCase()}#${number}`;
}

function lineOf(text, index) {
  return text.slice(0, index).split('\n').length;
}

/**
 * Closing references: what GitHub will close on merge, and bare references
 * that follow a keyword in a list and therefore will not be closed.
 */
export function parseClosingReferences(body, repository) {
  const text = stripIgnoredMarkdown(body);
  const closes = new Map();
  const missingKeyword = [];
  const negated = [];
  const pattern = new RegExp(String.raw`\b${KEYWORD}:?\s+(${REFERENCE})`, 'gi');
  for (const match of text.matchAll(pattern)) {
    const reference = key(match.slice(2, 6), repository);
    if (!closes.has(reference)) {
      closes.set(reference, lineOf(text, match.index));
    }
    const start = text.lastIndexOf('\n', match.index - 1) + 1;
    if (NEGATION.test(text.slice(start, match.index))) {
      negated.push({ reference, line: lineOf(text, match.index), text: text.slice(start, match.index + match[0].length) });
    }
    // `Fixes #1, #2 and #3`: GitHub closes only #1.
    const tail = text.slice(match.index + match[0].length);
    const list = tail.match(new RegExp(String.raw`^(?:\s*(?:,|\band\b|&)\s*(?:${REFERENCE}))+`, 'i'));
    if (list) {
      for (const bare of list[0].matchAll(new RegExp(REFERENCE, 'gi'))) {
        missingKeyword.push({
          reference: key(bare.slice(1, 5), repository),
          line: lineOf(text, match.index),
          text: match[0] + list[0],
        });
      }
    }
  }
  return { closes, missingKeyword, negated };
}

/** Issue references in a sentence, with `#A to #B` style ranges expanded. */
function sentenceReferences(sentence, repository) {
  const references = new Set();
  const closingTargets = new Set();
  for (const match of sentence.matchAll(new RegExp(String.raw`\b${KEYWORD}:?\s+(${REFERENCE})`, 'gi'))) {
    closingTargets.add(key(match.slice(2, 6), repository));
  }
  const range = new RegExp(String.raw`${REFERENCE}\s*(?:to|through|thru|until|-|–|—|\.\.)\s*${REFERENCE}`, 'gi');
  for (const match of sentence.matchAll(range)) {
    const from = key(match.slice(1, 5), repository);
    const to = key(match.slice(5, 9), repository);
    const [fromRepo, fromNumber] = from.split('#');
    const [toRepo, toNumber] = to.split('#');
    const low = Number(fromNumber);
    const high = Number(toNumber);
    if (fromRepo === toRepo && high > low && high - low <= MAX_RANGE) {
      for (let number = low; number <= high; number += 1) {
        references.add(`${fromRepo}#${number}`);
      }
    }
  }
  for (const match of sentence.matchAll(new RegExp(REFERENCE, 'gi'))) {
    references.add(key(match.slice(1, 5), repository));
  }
  // `Fixes #1 so #2 stays open`: the closing target is not the one kept open.
  return [...references].filter((reference) => !closingTargets.has(reference));
}

function sentences(text) {
  const result = [];
  let start = 0;
  // A sentence ends at `.`, `!`, `?` or `;` followed by whitespace, or at a blank line / list item.
  const boundary = /[.!?;](?=\s)|\n\s*\n|\n(?=\s*(?:[-*+]|\d+\.)\s)/g;
  for (const match of text.matchAll(boundary)) {
    const end = match.index + match[0].length;
    result.push({ text: text.slice(start, end), index: start });
    start = end;
  }
  result.push({ text: text.slice(start), index: start });
  return result;
}

/** Issues the prose says this PR leaves open, and "closes only" statements. */
export function parseOpenDeclarations(body, repository) {
  const text = stripIgnoredMarkdown(body);
  const keepOpen = new Map();
  const closesOnly = [];
  const partOf = new Map();
  for (const sentence of sentences(text)) {
    if (KEEP_OPEN.test(sentence.text)) {
      for (const reference of sentenceReferences(sentence.text, repository)) {
        if (!keepOpen.has(reference)) {
          keepOpen.set(reference, { line: lineOf(text, sentence.index), text: sentence.text.trim() });
        }
      }
    }
    const only = sentence.text.match(CLOSES_ONLY);
    if (only) {
      const allowed = key(only.slice(1, 5), repository) ?? key(only.slice(5, 9), repository);
      closesOnly.push({ allowed, line: lineOf(text, sentence.index), text: sentence.text.trim() });
    }
  }
  for (const match of text.matchAll(PART_OF)) {
    const reference = key(match.slice(1, 5), repository);
    if (!partOf.has(reference)) {
      partOf.set(reference, { line: lineOf(text, match.index), text: match[0] });
    }
  }
  return { keepOpen, closesOnly, partOf };
}

function short(reference, repository) {
  return reference.startsWith(`${repository.toLowerCase()}#`) ? reference.slice(reference.indexOf('#')) : reference;
}

/**
 * @param {string} body pull request description
 * @param {string} repository `owner/repo` the PR targets
 * @returns {{closes: string[], errors: {rule: string, reference: string, line: number, message: string}[]}}
 */
export function checkClosingReferences(body, repository) {
  const { closes, missingKeyword, negated } = parseClosingReferences(body ?? '', repository);
  const { keepOpen, closesOnly, partOf } = parseOpenDeclarations(body ?? '', repository);
  const errors = [];
  for (const [reference, line] of closes) {
    const name = short(reference, repository);
    const declaration = keepOpen.get(reference);
    if (declaration) {
      errors.push({
        rule: 'keep-open',
        reference,
        line,
        message: `${name} has a closing keyword, but line ${declaration.line} says it stays open: "${declaration.text}". Remove the keyword (write "Part of ${name}") or change the statement.`,
      });
    }
    for (const only of closesOnly) {
      if (only.allowed !== reference) {
        errors.push({
          rule: 'closes-only',
          reference,
          line,
          message: `${name} has a closing keyword, but line ${only.line} says this PR closes only ${short(only.allowed, repository)}: "${only.text}".`,
        });
      }
    }
    const related = partOf.get(reference);
    if (related) {
      errors.push({
        rule: 'part-of',
        reference,
        line,
        message: `${name} has a closing keyword and is also referenced as "${related.text}" on line ${related.line}; pick one.`,
      });
    }
  }
  for (const missing of missingKeyword) {
    if (!closes.has(missing.reference)) {
      const name = short(missing.reference, repository);
      errors.push({
        rule: 'missing-keyword',
        reference: missing.reference,
        line: missing.line,
        message: `"${missing.text.trim()}" does not close ${name}: GitHub needs one keyword per issue ("Fixes ${name}").`,
      });
    }
  }
  for (const negation of negated) {
    const name = short(negation.reference, repository);
    errors.push({
      rule: 'negated-keyword',
      reference: negation.reference,
      line: negation.line,
      message: `"${negation.text.trim()}" closes ${name}: GitHub ignores the negation. Write "${name} stays open" without a closing keyword.`,
    });
  }
  return { closes: [...closes.keys()], keepOpen: [...keepOpen.keys()], errors };
}

function parseArgs(argv) {
  const options = { bodyFile: null, verbose: process.env.CLOSING_REFERENCES_VERBOSE === '1' };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--body-file') {
      options.bodyFile = argv[++i];
    } else if (argv[i] === '--verbose') {
      options.verbose = true;
    } else {
      throw new Error(`Unknown argument: ${argv[i]}`);
    }
  }
  return options;
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const repository = process.env.GITHUB_REPOSITORY ?? 'link-assistant/calculator';
  let body;
  if (options.bodyFile) {
    body = readFileSync(options.bodyFile, 'utf8');
  } else if (process.env.GITHUB_EVENT_PATH) {
    const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
    if (!event.pull_request) {
      console.log('::notice::No pull_request in the event payload; nothing to check.');
      return;
    }
    body = event.pull_request.body ?? '';
  } else {
    throw new Error('Pass --body-file <path> or run in a pull_request workflow.');
  }
  const result = checkClosingReferences(body, repository);
  if (options.verbose) {
    console.log(`Closing references: ${result.closes.map((r) => short(r, repository)).join(', ') || '(none)'}`);
    console.log(`Declared open: ${result.keepOpen.map((r) => short(r, repository)).join(', ') || '(none)'}`);
  }
  for (const error of result.errors) {
    console.log(`::error title=Closing reference (${error.rule})::line ${error.line}: ${error.message}`);
  }
  if (result.errors.length > 0) {
    console.error(
      `${result.errors.length} contradictory closing reference(s). Merging would close issues the description says stay open; see CONTRIBUTING.md#closing-issues.`
    );
    process.exitCode = 1;
  } else {
    console.log(`Closing references are consistent (${result.closes.length} issue(s) closed on merge).`);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`check-closing-references: ${error.message}`);
    process.exitCode = 1;
  }
}
