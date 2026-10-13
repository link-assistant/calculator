import { describe, expect, test } from 'vitest';
import { debugF64, debugList, debugMap, debugOption, debugStruct, debugTuple, fmtF64 } from '../../src/rust/fmt.js';
import { debugStr } from '../../src/rust/str.js';
import { f64FromBits, loadVectors } from '../vectors.js';

const vectors = loadVectors('std');

describe('f64 formatting against Rust', () => {
  test('`{}` and `{:?}`', () => {
    const mismatches = [];
    for (const c of vectors.floats) {
      const x = f64FromBits(c.bits);
      if (fmtF64(x) !== c.display) {
        mismatches.push({ bits: c.bits, format: '{}', expected: c.display, actual: fmtF64(x) });
      }
      if (debugF64(x) !== c.debug) {
        mismatches.push({ bits: c.bits, format: '{:?}', expected: c.debug, actual: debugF64(x) });
      }
    }
    expect(mismatches).toEqual([]);
  });

  test('the forms JavaScript prints differently', () => {
    expect(fmtF64(1e21)).toBe('1000000000000000000000');
    expect(fmtF64(1.5e-7)).toBe('0.00000015');
    expect(fmtF64(-0)).toBe('-0');
    expect(fmtF64(Number.NEGATIVE_INFINITY)).toBe('-inf');
    expect(debugF64(1)).toBe('1.0');
    expect(debugF64(1e16)).toBe('1e16');
    expect(debugF64(1.5e-5)).toBe('1.5e-5');
    expect(debugF64(-0)).toBe('-0.0');
  });
});

describe('derived Debug shapes', () => {
  test('structs, tuples, lists, options and maps', () => {
    expect(debugStruct('Token', [['kind', 'Eof'], ['start', '5'], ['end', '5'], ['text', debugStr('')]]))
      .toBe('Token { kind: Eof, start: 5, end: 5, text: "" }');
    expect(debugStruct('Unit', [])).toBe('Unit');
    expect(debugTuple('Number', [debugStr('1781299013388')])).toBe('Number("1781299013388")');
    expect(debugTuple('None', [])).toBe('None');
    expect(debugList(['1', '2'])).toBe('[1, 2]');
    expect(debugList([])).toBe('[]');
    expect(debugOption(null, debugStr)).toBe('None');
    expect(debugOption('x', debugStr)).toBe('Some("x")');
    expect(debugMap([[debugStr('a'), '1.0']])).toBe('{"a": 1.0}');
    expect(debugMap([])).toBe('{}');
  });
});
