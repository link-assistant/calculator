import { describe, expect, test } from 'vitest';
import {
  DateTime, FixedOffset, Months, NaiveDate, NaiveDateTime, NaiveTime, ParseError, SecondsFormat,
  TimeDelta, Utc,
} from '../../src/rust/chrono.js';
import { Err, Panic } from '../../src/rust/core.js';
import { loadVectors } from '../vectors.js';

/**
 * @typedef {{ ok: string } | { err: string }} ParseOutcome
 * @typedef {{
 *   dt: number[], rfc3339: string, rfc3339z: string, rfc3339ms: string, rfc3339us: string,
 *   rfc3339ns: string, rfc3339s: string, display: string, debug: string, utcDisplay: string,
 *   utcDebug: string, naiveDebug: string, naiveDisplay: string, local: string, time: string,
 *   ymd: number[], hms: number[], timestamp: number, millis: number,
 *   out: (string | null)[], naiveOut: (string | null)[], dateOut: (string | null)[],
 *   timeOut: (string | null)[],
 * }} FormatCase
 * @typedef {{
 *   base: string, ymdHms: number[], months: (string | null)[][], secs: [boolean, string | null, string | null][],
 *   since: [number, number, number, boolean][],
 * }} ArithCase
 */

/**
 * @type {{
 *   parse: [string, string, string, ParseOutcome][],
 *   rfc3339: [string, ParseOutcome, [string, number, number, string] | null][],
 *   format: FormatCase[], formats: string[],
 *   offsets: [number, string | null, string | null, number | null][],
 *   arith: ArithCase[], months: number[], secs: string[],
 *   millis: [string, string | null][],
 *   ymd: [number, number, number, string | null][],
 *   hms: [number, number, number, string | null][],
 * }}
 */
const vectors = loadVectors('chrono');

/**
 * Runs a fallible parse, rendering it like the vectors do.
 * @param {() => { fmtDebug(): string }} f
 * @returns {ParseOutcome}
 */
function parseOutcome(f) {
  try {
    return { ok: f().fmtDebug() };
  } catch (e) {
    if (e instanceof Err && e.error instanceof ParseError) {
      return { err: e.error.toString() };
    }
    throw e;
  }
}

/**
 * A `format(..).to_string()` whose `fmt::Error` panic becomes `null`.
 * @param {() => string} f
 * @returns {string | null}
 */
function tryFormat(f) {
  try {
    return f();
  } catch (e) {
    if (e instanceof Panic && /Display implementation returned an error/.test(e.message)) {
      return null;
    }
    throw e;
  }
}

/**
 * The vector fields other than the date-time and the format outputs.
 * @param {FormatCase} c
 */
function scalars(c) {
  const { dt: _dt, out: _out, naiveOut: _n, dateOut: _d, timeOut: _t, ...rest } = c;
  return rest;
}

/** @param {number[]} dt */
function build(dt) {
  const [y, mo, d, h, mi, s, ns, off] = dt;
  const naive = NaiveDate.fromYmdOpt(y, mo, d)?.andHmsNanoOpt(h, mi, s, ns);
  const offset = FixedOffset.eastOpt(off);
  if (!naive || !offset) {
    throw new Error(`bad vector ${dt}`);
  }
  return naive.andUtc().withTimezone(offset);
}

describe('chrono parse_from_str', () => {
  const parsers = {
    date: (/** @type {string} */ s, /** @type {string} */ f) => NaiveDate.parseFromStr(s, f),
    time: (/** @type {string} */ s, /** @type {string} */ f) => NaiveTime.parseFromStr(s, f),
    datetime: (/** @type {string} */ s, /** @type {string} */ f) => NaiveDateTime.parseFromStr(s, f),
  };
  test(`${vectors.parse.length} vectors`, () => {
    for (const [kind, fmt, s, want] of vectors.parse) {
      const parser = parsers[/** @type {keyof typeof parsers} */ (kind)];
      expect(parseOutcome(() => parser(s, fmt)), `${kind} ${JSON.stringify(fmt)} ${JSON.stringify(s)}`)
        .toEqual(want);
    }
  });
});

describe('chrono DateTime::parse_from_rfc3339', () => {
  test(`${vectors.rfc3339.length} vectors`, () => {
    for (const [s, want, extra] of vectors.rfc3339) {
      expect(parseOutcome(() => DateTime.parseFromRfc3339(s)), JSON.stringify(s)).toEqual(want);
      if (extra) {
        const d = DateTime.parseFromRfc3339(s);
        expect([d.toRfc3339(), d.timezone().localMinusUtc(), d.timestampMillis(), d.toString()])
          .toEqual([extra[0], extra[1], extra[2], extra[3]]);
      }
    }
  });
});

describe('chrono formatting', () => {
  test(`${vectors.format.length} date-times x ${vectors.formats.length} formats`, () => {
    for (const c of vectors.format) {
      const v = build(c.dt);
      const naive = v.naiveUtc();
      const utc = naive.andUtc();
      const label = JSON.stringify(c.dt);
      expect({
        rfc3339: v.toRfc3339(),
        rfc3339z: v.toRfc3339Opts(SecondsFormat.AutoSi, true),
        rfc3339ms: v.toRfc3339Opts(SecondsFormat.Millis, false),
        rfc3339us: v.toRfc3339Opts(SecondsFormat.Micros, false),
        rfc3339ns: v.toRfc3339Opts(SecondsFormat.Nanos, true),
        rfc3339s: v.toRfc3339Opts(SecondsFormat.Secs, false),
        display: v.toString(),
        debug: v.fmtDebug(),
        utcDisplay: utc.toString(),
        utcDebug: utc.fmtDebug(),
        naiveDebug: naive.fmtDebug(),
        naiveDisplay: naive.toString(),
        local: v.overflowingNaiveLocal().fmtDebug(),
        time: v.time().fmtDebug(),
        ymd: [v.year(), v.month(), v.day(), v.ordinal()],
        hms: [v.hour(), v.minute(), v.second(), v.nanosecond()],
        timestamp: v.timestamp(),
        millis: v.timestampMillis(),
      }, label).toEqual(scalars(c));
      vectors.formats.forEach((fmt, i) => {
        const at = `${label} ${JSON.stringify(fmt)}`;
        expect(tryFormat(() => v.format(fmt)), at).toBe(c.out[i]);
        expect(tryFormat(() => naive.format(fmt)), `naive ${at}`).toBe(c.naiveOut[i]);
        expect(tryFormat(() => naive.date().format(fmt)), `date ${at}`).toBe(c.dateOut[i]);
        expect(tryFormat(() => naive.time().format(fmt)), `time ${at}`).toBe(c.timeOut[i]);
      });
    }
  });
});

describe('chrono FixedOffset', () => {
  test(`${vectors.offsets.length} vectors`, () => {
    for (const [secs, debug, display, west] of vectors.offsets) {
      const o = FixedOffset.eastOpt(secs);
      expect([o?.fmtDebug() ?? null, o?.toString() ?? null,
        FixedOffset.westOpt(secs)?.localMinusUtc() ?? null], String(secs))
        .toEqual([debug, display, west]);
    }
  });
});

describe('chrono arithmetic', () => {
  const bases = vectors.arith.map(c => {
    const [y, mo, d, h, mi, s] = c.ymdHms;
    return Utc.withYmdAndHms(y, mo, d, h, mi, s).unwrap();
  });
  test(`${bases.length} bases`, () => {
    vectors.arith.forEach((c, i) => {
      const b = bases[i];
      vectors.months.forEach((m, j) => {
        const n = b.naiveUtc();
        expect([
          b.checkedAddMonths(new Months(m))?.toRfc3339() ?? null,
          b.checkedSubMonths(new Months(m))?.toRfc3339() ?? null,
          (n.checkedAddMonths(new Months(m)) ?? n).andUtc().toRfc3339(),
        ], `${c.base} months ${m}`).toEqual(c.months[j]);
      });
      vectors.secs.forEach((s, j) => {
        const d = TimeDelta.trySeconds(BigInt(s));
        expect([
          d !== null,
          d && (b.checkedAddSigned(d)?.toRfc3339() ?? null),
          d && (b.checkedSubSigned(d)?.toRfc3339() ?? null),
        ], `${c.base} secs ${s}`).toEqual(c.secs[j]);
      });
      bases.forEach((o, j) => {
        const d = b.signedDurationSince(o);
        let std = 0n;
        try {
          std = d.toStd().asSecs();
        } catch (e) {
          if (!(e instanceof Err)) {
            throw e;
          }
        }
        const [secs, stdSecs, ord, eq] = c.since[j];
        expect([d.numSeconds(), std, b.cmp(o), b.equals(o)], `${c.base} since ${o}`)
          .toEqual([BigInt(secs), BigInt(stdSecs), ord, eq]);
      });
    });
  });
});

describe('chrono constructors', () => {
  test('Utc.timestamp_millis_opt', () => {
    for (const [ms, want] of vectors.millis) {
      // `ms` went through JSON.parse; re-read it exactly from the source text.
      expect(Utc.timestampMillisOpt(BigInt(ms)).single()?.toRfc3339() ?? null, String(ms))
        .toBe(want);
    }
  });
  test('NaiveDate::from_ymd_opt', () => {
    for (const [y, m, d, want] of vectors.ymd) {
      expect(NaiveDate.fromYmdOpt(y, m, d)?.fmtDebug() ?? null).toBe(want);
    }
  });
  test('NaiveTime::from_hms_opt', () => {
    for (const [h, m, s, want] of vectors.hms) {
      expect(NaiveTime.fromHmsOpt(h, m, s)?.fmtDebug() ?? null).toBe(want);
    }
  });
});
