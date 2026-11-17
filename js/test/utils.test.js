import { describe, expect, test } from 'vitest';
import { generateIssueLink, truncate } from '../src/utils.js';

describe('utils (src/utils.rs)', () => {
  test('issue link generation', () => {
    const link = generateIssueLink('invalid input', 'Parse error');
    expect(link).toContain('github.com');
    expect(link).toContain('issues/new');
  });

  test('truncate', () => {
    expect(truncate('hello world', 5)).toBe('hello');
    expect(truncate('hi', 10)).toBe('hi');
  });
});

// Values printed by the Rust module (experiments/issue-254/foundation-vectors).
describe('utils vectors (experiments/issue-254/foundation-vectors)', () => {
  test('the whole issue link', () => {
    expect(generateIssueLink('invalid input', 'Parse error')).toBe(
      'https://github.com/link-assistant/calculator/issues/new'
        + '?title=Unrecognized%20input%3A%20invalid%20input'
        + '&body=%23%23%20Input%20that%20failed%20to%20parse%0A%0A%60%60%60%0Ainvalid%20input%0A%60%60%60%0A%0A'
        + '%23%23%20Error%20message%0A%0A%60%60%60%0AParse%20error%0A%60%60%60%0A%0A'
        + '%23%23%20Expected%20behavior%0A%0APlease%20describe%20what%20you%20expected%20the%20calculator'
        + '%20to%20do%20with%20this%20input.',
    );
  });

  /**
   * The link for an input and an error whose percent-encodings are given.
   * @param {string} title
   * @param {string} input
   * @param {string} error
   */
  const link = (title, input, error) => 'https://github.com/link-assistant/calculator/issues/new'
    + `?title=Unrecognized%20input%3A%20${title}`
    + `&body=%23%23%20Input%20that%20failed%20to%20parse%0A%0A%60%60%60%0A${input}%0A%60%60%60%0A%0A`
    + `%23%23%20Error%20message%0A%0A%60%60%60%0A${error}%0A%60%60%60%0A%0A`
    + '%23%23%20Expected%20behavior%0A%0APlease%20describe%20what%20you%20expected%20the%20calculator'
    + '%20to%20do%20with%20this%20input.';

  test('UTF-8 and reserved characters are percent-encoded', () => {
    const input = '2 + \u{20ac} \u{d7} 3 \u{2014} \u{1f600} ~a_b.c-d';
    const encoded = '2%20%2B%20%E2%82%AC%20%C3%97%203%20%E2%80%94%20%F0%9F%98%80%20~a_b.c-d';
    expect(generateIssueLink(input, "Unexpected token '\u{20ac}'"))
      .toBe(link(encoded, encoded, 'Unexpected%20token%20%27%E2%82%AC%27'));
  });

  test('control characters are percent-encoded', () => {
    const encoded = 'line1%0Aline2%0D%0A%09%25%26%3F%3D%23%2B';
    expect(generateIssueLink('line1\nline2\r\n\t%&?=#+', 'Error: "x"'))
      .toBe(link(encoded, encoded, 'Error%3A%20%22x%22'));
  });

  test('the title keeps the first 50 chars', () => {
    expect(generateIssueLink('x'.repeat(60), 'e')).toBe(link('x'.repeat(50), 'x'.repeat(60), 'e'));
    expect(generateIssueLink('\u{e9}'.repeat(53), 'e'))
      .toBe(link('%C3%A9'.repeat(50), '%C3%A9'.repeat(53), 'e'));
    expect(generateIssueLink('\u{1f600}'.repeat(51), ''))
      .toBe(link('%F0%9F%98%80'.repeat(50), '%F0%9F%98%80'.repeat(51), ''));
    expect(generateIssueLink('', '')).toBe(link('', '', ''));
  });

  test.each([
    ['hello world', 5, 'hello'],
    ['hi', 10, 'hi'],
    ['', 0, ''],
    ['abc', 0, ''],
    ['abc', 3, 'abc'],
    ['\u{1f600}\u{e9}\u{6f22}x', 2, '\u{1f600}\u{e9}'],
    ['\u{1f600}\u{e9}\u{6f22}x', 3, '\u{1f600}\u{e9}\u{6f22}'],
    ['\u{1f600}\u{e9}\u{6f22}x', 4, '\u{1f600}\u{e9}\u{6f22}x'],
  ])('truncate(%j, %i)', (s, maxChars, expected) => {
    expect(truncate(s, maxChars)).toBe(expected);
  });
});
