import { describe, expect, test } from 'vitest';
import {
  clone,
  cmp,
  compareStrings,
  eq,
  Err,
  err,
  expect as expectOption,
  expectResult,
  isOk,
  ok,
  Panic,
  panic,
  tryResult,
  unreachable,
  unwrap,
  unwrapResult,
} from '../../src/rust/core.js';
import { I32 } from '../../src/rust/num.js';

describe('Result and Option', () => {
  test('err throws an Err that tryResult, ok and isOk catch', () => {
    const fails = () => err('bad');
    expect(fails).toThrow(Err);
    expect(tryResult(fails)).toEqual({ ok: false, error: 'bad' });
    expect(tryResult(() => 5)).toEqual({ ok: true, value: 5 });
    expect(ok(fails)).toBeNull();
    expect(ok(() => 0)).toBe(0);
    expect(isOk(fails)).toBe(false);
  });

  test('panics and JavaScript errors are not swallowed', () => {
    expect(() => tryResult(() => panic('boom'))).toThrow(Panic);
    expect(() => ok(() => unreachable())).toThrow('internal error: entered unreachable code');
    expect(() => isOk(() => { throw new TypeError('bug'); })).toThrow(TypeError);
    expect(() => unreachable('why')).toThrow('internal error: entered unreachable code: why');
  });

  test('unwrap and expect print the messages Rust prints', () => {
    expect(unwrap(0)).toBe(0);
    expect(() => unwrap(null)).toThrow('called `Option::unwrap()` on a `None` value');
    expect(expectOption('', 'set')).toBe('');
    expect(() => expectOption(null, 'non-EOF token should exist')).toThrow(/^non-EOF token should exist$/);
    expect(unwrapResult(() => 1)).toBe(1);
    expect(() => unwrapResult(() => I32.parse('x')))
      .toThrow('called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }');
    expect(() => expectResult(() => I32.parse(''), 'valid'))
      .toThrow('valid: ParseIntError { kind: Empty }');
    expect(() => expectResult(() => err('plain'), 'valid')).toThrow('valid: plain');
  });
});

describe('derived traits', () => {
  class Point {
    /** @param {number} x @param {number[]} ys */
    constructor(x, ys) {
      this.x = x;
      this.ys = ys;
    }
  }

  test('eq is structural, and NaN is not equal to itself', () => {
    expect(eq(new Point(1, [2]), new Point(1, [2]))).toBe(true);
    expect(eq(new Point(1, [2]), new Point(1, [3]))).toBe(false);
    expect(eq(new Point(1, [2]), { x: 1, ys: [2] })).toBe(false);
    expect(eq(Number.NaN, Number.NaN)).toBe(false);
    expect(eq(new Map([['a', [1]]]), new Map([['a', [1]]]))).toBe(true);
    expect(eq(new Set([1, 2]), new Set([2, 1]))).toBe(true);
    expect(eq({ equals: () => true }, {})).toBe(true);
    expect(eq({ equals: () => true }, 5)).toBe(false);
    expect(eq(null, null)).toBe(true);
    expect(eq(null, {})).toBe(false);
  });

  test('clone copies mutable containers and shares frozen values', () => {
    const point = new Point(1, [2]);
    const copy = clone(point);
    copy.ys.push(3);
    expect(point.ys).toEqual([2]);
    expect(copy).toBeInstanceOf(Point);
    const frozen = Object.freeze({ tag: 'A' });
    expect(clone(frozen)).toBe(frozen);
    expect(clone(new Map([['k', [1]]])).get('k')).toEqual([1]);
  });

  test('cmp and compareStrings order as Rust does', () => {
    expect(cmp(1, 2)).toBe(-1);
    expect(cmp(2n, 2n)).toBe(0);
    expect(cmp('b', 'a')).toBe(1);
    expect(cmp({ cmp: () => 1 }, null)).toBe(1);
    // U+FF61 sorts before U+1F600 by code point, after it by UTF-16 unit.
    expect(compareStrings('｡', '😀')).toBe(-1);
    expect('｡' < '😀').toBe(false);
    expect(compareStrings('😀a', '😀b')).toBe(-1);
    expect(compareStrings('ab', 'a')).toBe(1);
    expect(compareStrings('', '')).toBe(0);
  });
});
