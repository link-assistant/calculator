import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, test } from 'vitest';
import {
  REPO_ROOT,
  collectBundledRates,
  parseLatestRate,
  renderBundledRates,
  staleFiles,
  templateLiteral,
} from '../scripts/generate-data.mjs';
import { ARITHMETIC_WORDS_LINO } from '../src/generated/arithmetic-words-lino.js';
import { BUNDLED_LATEST_RATES } from '../src/generated/bundled-rates.js';
import { UNITS_LINO } from '../src/generated/units-lino.js';

describe('generated data (js/scripts/generate-data.mjs)', () => {
  test('the committed files are up to date', () => {
    expect(staleFiles()).toEqual([]);
  });

  test('the text modules equal the data files', () => {
    expect(UNITS_LINO).toBe(readFileSync(join(REPO_ROOT, 'data/units/units.lino'), 'utf8'));
    expect(ARITHMETIC_WORDS_LINO).toBe(readFileSync(join(REPO_ROOT, 'data/words/arithmetic-words.lino'), 'utf8'));
  });

  test('bundled rates are frozen tuples', () => {
    expect(Object.isFrozen(BUNDLED_LATEST_RATES)).toBe(true);
    for (const entry of BUNDLED_LATEST_RATES) {
      expect(Object.isFrozen(entry)).toBe(true);
      expect(entry).toHaveLength(5);
      expect(Number.isFinite(entry[4]) && entry[4] > 0).toBe(true);
    }
  });

  test('template literals keep the exact text', () => {
    const text = 'a`b${c}\\d\r\n\u{a0}\u{feff}\u{1}\t\u{1f600}';
    expect(new Function(`return ${templateLiteral(text)};`)()).toBe(text);
  });
});

describe('parseLatestRate (build.rs parse_latest_rate)', () => {
  test('rates: form with a quoted source', () => {
    expect(parseLatestRate(
      "conversion:\n  from usd\n  to hkd\n  source 'frankfurter.dev (ECB)'\n  rates:\n"
        + '    2026-10-01 7.8\n    2026-10-02 7.8471\n',
    )).toEqual({ from: 'USD', to: 'HKD', source: 'frankfurter.dev (ECB)', date: '2026-10-02', rate: 7.8471 });
  });

  test('data: form, double quotes, CRLF and the default source', () => {
    expect(parseLatestRate('rates:\r\n  from CHF\r\n  to RUB\r\n  data:\r\n    2021-03-23 80.3192\r\n'))
      .toBe(null); // the leading `rates:` starts the rate section, so from/to are never read
    expect(parseLatestRate('x:\r\n  from CHF\r\n  to RUB\r\n  source "cbr"\r\n  data:\r\n    2021-03-23 80.3192\r\n'))
      .toEqual({ from: 'CHF', to: 'RUB', source: 'cbr', date: '2021-03-23', rate: 80.3192 });
    expect(parseLatestRate('from A\nto B\nrates:\n2020-01-01 1\n')?.source).toBe('unknown');
  });

  test('non-finite, non-positive and malformed rates are ignored', () => {
    const content = 'from A\nto B\nrates:\n2020-01-01 2\n2020-01-02 inf\n2020-01-03 NaN\n'
      + '2020-01-04 0\n2020-01-05 -1\n2020-01-06 x\n2020-01-07\n';
    expect(parseLatestRate(content)?.date).toBe('2020-01-01');
    expect(parseLatestRate('from A\nto B\nrates:\n2020-01-01 0\n')).toBe(null);
    expect(parseLatestRate('to B\nrates:\n2020-01-01 1\n')).toBe(null);
  });

  test('newest date wins, ties keep the first, order does not matter', () => {
    const rate = parseLatestRate('from A\nto B\nrates:\n2020-01-02 2\n2020-01-03 3\n2020-01-01 1\n2020-01-03 4\n');
    expect([rate?.date, rate?.rate]).toEqual(['2020-01-03', 3]);
  });
});

describe('collectBundledRates (build.rs main)', () => {
  test('sorted .lino files only, absent dir gives an empty table', () => {
    const dir = mkdtempSync(join(tmpdir(), 'rates-'));
    writeFileSync(join(dir, 'b.lino'), 'from B\nto C\nrates:\n2020-01-01 2\n');
    writeFileSync(join(dir, 'a.lino'), 'from A\nto C\nrates:\n2020-01-01 1e-7\n');
    writeFileSync(join(dir, 'c.txt'), 'from X\nto C\nrates:\n2020-01-01 1\n');
    writeFileSync(join(dir, '.lino'), 'from Y\nto C\nrates:\n2020-01-01 1\n');
    writeFileSync(join(dir, 'd.lino'), Buffer.from([0xff, 0xfe]));
    mkdirSync(join(dir, 'e.lino'));
    const rates = collectBundledRates(dir);
    expect(rates.map((r) => r.from)).toEqual(['A', 'B']);
    expect(renderBundledRates(rates)).toContain('rate(["A", "C", "unknown", "2020-01-01", 1e-7]),');
    expect(collectBundledRates(join(dir, 'missing'))).toEqual([]);
  });
});
