//! Test vectors for js/src/rust/chrono.js (issue #254): chrono 0.4.45 with the
//! features the calculator enables.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin chrono > js/test/vectors/chrono.json
//!
//! Encoding: a parse result is {"ok": Debug} or {"err": Display}; a failed
//! format (`fmt::Error`, which makes `to_string` panic) is `null`; `None` is
//! `null`. A date-time is given as [year, month, day, hour, min, sec, nano,
//! offset seconds] and built as `NaiveDate::from_ymd_opt(..).and_hms_nano_opt(..)
//! .and_utc().with_timezone(&FixedOffset::east_opt(offset))`.

use std::fmt::Write;

use chrono::{
    DateTime, Datelike, Duration, FixedOffset, Months, NaiveDate, NaiveDateTime, NaiveTime,
    SecondsFormat, TimeZone, Timelike, Utc,
};
use serde_json::{json, Value};

const DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y", "%d.%m.%Y", "%m.%d.%Y", "%Y.%m.%d", "%d-%m-%Y",
    "%m-%d-%Y", "%b %d, %Y", "%B %d, %Y", "%d %b %Y", "%d %B %Y", "%b %d %Y", "%B %d %Y",
    // beyond the crate: other fields the port parses
    "%y-%m-%d", "%C%y-%m-%d", "%Y-%j", "%a %Y-%m-%d", "%A, %e %h %Y", "%u %Y-%m-%d",
    "%w %Y-%m-%d", "%q %Y-%m-%d", "%F", "%D", "%v", "%x", "%Y-%m", "%Y", "%m-%d", "%Y-%m-%d%",
    "%Y-%m-%d %Q",
];
const TIME_FORMATS: &[&str] = &[
    "%H:%M", "%H:%M:%S", "%I:%M %p", "%l:%M:%S %P", "%H:%M:%S%.f", "%H:%M:%S%.3f",
    "%H:%M:%S.%3f", "%H:%M:%S %z", "%H:%M:%S %:z", "%H:%M:%S %#z", "%H:%M:%S %Z", "%T", "%R",
    "%X", "%r", "%H", "%H:%M:%S%.6f", "%H:%M:%S%.9f", "%k:%M", "%H:%M:%S %::z", "%H:%M:%S %:::z",
];
const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M:%S%.f%:z", "%c", "%Y-%m-%d %I %p",
];

const DATE_INPUTS: &[&str] = &[
    "2024-03-15", "2024-3-5", "2024-02-29", "2023-02-29", "2024-02-30", "2024-13-01", "2024-00-10",
    "2024-01-32", "2024-04-31", "0000-01-01", "-0001-12-31", "+12345-06-07", "12345-06-07",
    "-262143-01-01", "262142-12-31", "262143-01-01", "99999999999-01-01", " 2024-03-15",
    "2024 - 03 - 15", "2024-03-15 ", "2024-03-15x", "2024-03", "2024-", "", "  ", "2024/03/15",
    "03/15/2024", "3/5/2024", "15/03/2024", "31/12/1999", "12/31/1999", "15.03.2024", "03.15.2024",
    "2024.03.15", "15-03-2024", "03-15-2024", "3-15-24", "Mar 15, 2024", "mar 15, 2024",
    "MARCH 15, 2024", "March 15, 2024", "Sept 15, 2024", "Sep 15, 2024", "September 15, 2024",
    "Septem 15, 2024", "15 Mar 2024", "15 March 2024", "15 march 2024", "Mar 15 2024",
    "March 15 2024", "Ma 15 2024", "Feb 29, 2023", "Feb 29 2024", "Jan 1 0", "Dec 31 -5",
    "Jun 05, 2024", "Jun  5, 2024", "Jun 5,2024", "Jun 5 , 2024", "\u{a0}Jun 5, 2024",
    "Jun 5, 2024\u{a0}", "Jun\t5,\n2024", "Jux 5, 2024", "Junee 5, 2024", "24-03-15",
    "69-01-01", "70-01-01", "20-24-03-15", "2024-075", "2024-366", "2023-366", "2024-000",
    "Fri 2024-03-15", "Fri 2024-03-16", "Friday, 15 Mar 2024", "friday, 15 mar 2024",
    "Fri, 15 Mar 2024", "5 2024-03-15", "4 2024-03-15", "8 2024-03-15", "1 2024-03-15",
    "4 2024-03-17", "0 2024-03-17", "1 2024-03-15", "2 2024-04-01", "03/15/24", "15-Mar-2024",
    "2024-03-15%", "2024-03-15 x", "2024", "03-15", "2024-03-15T", "２０２４-03-15", "2024-0３-15",
    "2024—03—15", "+2024-03-15", "-2024-03-15", "00002024-03-15", "2024-003-15", "2024-03-015",
];
const TIME_INPUTS: &[&str] = &[
    "12:34", "12:34:56", "1:2", "01:02:03", "23:59", "24:00", "23:60", "23:59:60", "23:59:61",
    "12:34:5", " 12:34", "12 : 34", "12:34 ", "12:34x", "12:", "12", "", "12:34:56.789",
    "12:34:56.123456789", "12:34:56.1234567891234", "12:34:56.", "12:34:56.12", "12:34:56.1234",
    "12:34:56.123", "12:34 AM", "12:34 pm", "1:34 PM", "01:34 am", "13:34 PM", "00:34 AM",
    "12:34 XM", "12:34 A", " 9:05:00 pm", "12:34:56 +0530", "12:34:56 +05:30", "12:34:56 -05",
    "12:34:56 Z", "12:34:56 z", "12:34:56 +05:60", "12:34:56 +05:99", "12:34:56 \u{2212}0530",
    "12:34:56 +5:30", "12:34:56 UTC", "12:34:56 +05:30:00", "12:34:56 +05 30", "12:34:56 +05::30",
    "12:34:56 EST more", "01:02:03 AM", "01:02:03 PM", "9:05", "１２:34", "12:34:56 +0530 ",
    "12:34:56 +053", "12:34:56 +", "12:34:56 +05:", "12:34:56 +05:3",
];
const DATETIME_INPUTS: &[&str] = &[
    "2024-03-15 12:34:56", "2024-03-15 12:34", "2024-03-15 12:34:60", "2024-02-30 25:00:00",
    "2024-02-30 12:00:00", "2024-03-15 25:00:00", "2024-03-15", "12:34:56", "2024-03-15T12:34:56",
    "2024-03-15  12:34:56", "2024-03-15 12:34:56 ", "2024-03-15 12:34:56.5",
    "2024-03-15T12:34:56.5+05:30", "2024-03-15T12:34:56Z", "Fri Mar 15 12:34:56 2024",
    "Sat Mar 15 12:34:56 2024", "2024-03-15 1 PM", "2024-03-15 12 AM", "2024-03-15 13 PM",
    "-0005-03-15 00:00:00", "2024-3-5 1:2:3",
];
const RFC3339_INPUTS: &[&str] = &[
    "2024-03-15T12:34:56Z", "2024-03-15T12:34:56z", "2024-03-15t12:34:56+00:00",
    "2024-03-15 12:34:56-05:00", "2024-03-15T12:34:56.123+05:30", "2024-03-15T12:34:56.1-01:00",
    "2024-03-15T12:34:56.123456789Z", "2024-03-15T12:34:56.1234567891Z", "2024-03-15T12:34:56.Z",
    "2024-03-15T23:59:60Z", "2024-03-15T23:59:61Z", "2024-03-15T24:00:00Z", "2024-02-30T00:00:00Z",
    "2024-03-15T12:34:56", "2024-03-15T12:34:56+0530", "2024-03-15T12:34:56+05", "2024-03-15T12:34:56+05:3",
    "2024-03-15T12:34:56+05:60", "2024-03-15T12:34:56+23:59", "2024-03-15T12:34:56+24:00",
    "2024-03-15T12:34:56+99:59", "2024-03-15T12:34:56\u{2212}05:00", "2024-03-15T12:34:56Z ",
    "2024-03-15T12:34:56Zx", "2024-03-15X12:34:56Z", "2024-3-15T12:34:56Z", "2024-03-15T12:34",
    "", "0000-01-01T00:00:00Z", "9999-12-31T23:59:59.999999999Z", "0000-01-01T00:00:00+00:01",
    "9999-12-31T23:59:59-00:01", "2024-03-15T12:34:56+05:30:00", "2024-03-15T12:34:56 +05:30",
    "2024-03-15T12:34:56+05 30", "2024-03-15T12:34:56+05:30\u{e9}", "２024-03-15T12:34:56Z",
    "2024/03/15T12:34:56Z", "2024-03-15T12-34-56Z", "2024-03-15T12:34:56.+05:30",
];
const FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S UTC", "%Y-%m-%d", "%H:%M:%S UTC", "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M:%S %:z", "%H:%M:%S", "%H:%M:%S %:z", "%Y-%m-%d %H:%M",
    "%C %y %q %e %-d %_m %0e %-Y %_Y %j %-j %_j", "%G %g %V %U %W %u %w %-V %_U",
    "%a %A %b %B %h", "%I %l %p %P %k %-I %_H %-M %0S", "%f %.f %.3f %.6f %.9f %3f %6f %9f %-f",
    "%Z|%z|%:z|%::z|%:::z", "%#z", "%+", "%s", "%c", "%x", "%X", "%r", "%D", "%F", "%R", "%T",
    "%v", "%%|%n|%t", "%", "%Q", "%.4f", "%-a", "%#Y", "%:", "%::", "%.", "%3", "%3x", "x%-",
];
const DATES: &[(i32, u32, u32)] = &[
    (2026, 3, 15), (2024, 2, 29), (2024, 12, 31), (2023, 1, 1), (2021, 1, 3), (2020, 12, 31),
    (2015, 12, 31), (2016, 1, 1), (2010, 1, 3), (2009, 12, 31), (2008, 12, 29), (1970, 1, 1),
    (1999, 12, 31), (1000, 1, 1), (999, 12, 31), (100, 6, 7), (5, 7, 8), (0, 1, 1), (0, 12, 31),
    (-1, 12, 31), (-5, 3, 1), (-100, 1, 1), (-9999, 1, 1), (-10000, 1, 1), (9999, 12, 31),
    (10000, 1, 1), (12345, 6, 7), (262142, 12, 31), (-262143, 1, 1), (1900, 3, 1), (2000, 2, 29),
];
const TIMES: &[(u32, u32, u32, u32)] = &[
    (0, 0, 0, 0), (12, 34, 56, 0), (23, 59, 59, 999_999_999), (1, 2, 3, 4), (13, 0, 0, 120_000_000),
    (11, 59, 59, 500_000), (12, 0, 0, 1_000), (23, 59, 59, 1_500_000_000), (9, 5, 7, 123_456_789),
];
const OFFSETS: &[i32] = &[0, 3600, -18000, 19800, -12345, 45, -45, 86399, -86399, 30, 29, -30, 36000];

fn parse_result<T: std::fmt::Debug>(r: Result<T, chrono::ParseError>) -> Value {
    match r {
        Ok(v) => json!({ "ok": format!("{v:?}") }),
        Err(e) => json!({ "err": e.to_string() }),
    }
}

fn try_format<D: std::fmt::Display>(d: D) -> Value {
    let mut s = String::new();
    match write!(s, "{d}") {
        Ok(()) => json!(s),
        Err(_) => Value::Null,
    }
}

fn dt(y: i32, mo: u32, d: u32, t: (u32, u32, u32, u32), off: i32) -> DateTime<FixedOffset> {
    NaiveDate::from_ymd_opt(y, mo, d)
        .unwrap()
        .and_hms_nano_opt(t.0, t.1, t.2, t.3)
        .unwrap()
        .and_utc()
        .with_timezone(&FixedOffset::east_opt(off).unwrap())
}

fn main() {
    let mut parse = Vec::new();
    for fmt in DATE_FORMATS {
        for s in DATE_INPUTS {
            parse.push(json!(["date", fmt, s, parse_result(NaiveDate::parse_from_str(s, fmt))]));
        }
    }
    for fmt in TIME_FORMATS {
        for s in TIME_INPUTS {
            parse.push(json!(["time", fmt, s, parse_result(NaiveTime::parse_from_str(s, fmt))]));
        }
    }
    for fmt in DATETIME_FORMATS {
        for s in DATETIME_INPUTS {
            parse.push(json!(["datetime", fmt, s, parse_result(NaiveDateTime::parse_from_str(s, fmt))]));
        }
    }

    let mut rfc3339 = Vec::new();
    for s in RFC3339_INPUTS {
        let r = DateTime::parse_from_rfc3339(s);
        let extra = r.as_ref().ok().map(|d| {
            json!([d.to_rfc3339(), d.timezone().local_minus_utc(), d.timestamp_millis(), d.to_string()])
        });
        rfc3339.push(json!([s, parse_result(r), extra]));
    }

    // Formatting: every date with a few times and offsets, every format.
    let mut format = Vec::new();
    for (i, &(y, mo, d)) in DATES.iter().enumerate() {
        for (j, &t) in TIMES.iter().enumerate() {
            if (i + j) % 3 != 0 && j != 1 {
                continue;
            }
            let off = OFFSETS[(i * 7 + j) % OFFSETS.len()];
            let value = dt(y, mo, d, t, off);
            let naive = value.naive_utc();
            let local = value.overflowing_naive_local_public();
            let outs: Vec<Value> = FORMATS.iter().map(|f| try_format(value.format(f))).collect();
            let naive_outs: Vec<Value> = FORMATS.iter().map(|f| try_format(naive.format(f))).collect();
            let date_outs: Vec<Value> = FORMATS.iter().map(|f| try_format(naive.date().format(f))).collect();
            let time_outs: Vec<Value> = FORMATS.iter().map(|f| try_format(naive.time().format(f))).collect();
            let utc = naive.and_utc();
            format.push(json!({
                "dt": [y, mo, d, t.0, t.1, t.2, t.3, off],
                "rfc3339": value.to_rfc3339(),
                "rfc3339z": value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                "rfc3339ms": value.to_rfc3339_opts(SecondsFormat::Millis, false),
                "rfc3339us": value.to_rfc3339_opts(SecondsFormat::Micros, false),
                "rfc3339ns": value.to_rfc3339_opts(SecondsFormat::Nanos, true),
                "rfc3339s": value.to_rfc3339_opts(SecondsFormat::Secs, false),
                "display": value.to_string(),
                "debug": format!("{value:?}"),
                "utcDisplay": utc.to_string(),
                "utcDebug": format!("{utc:?}"),
                "naiveDebug": format!("{naive:?}"),
                "naiveDisplay": naive.to_string(),
                "local": format!("{local:?}"),
                "time": format!("{:?}", value.time()),
                "ymd": [value.year(), value.month(), value.day(), value.ordinal()],
                "hms": [value.hour(), value.minute(), value.second(), value.nanosecond()],
                "timestamp": value.timestamp(),
                "millis": value.timestamp_millis(),
                "out": outs, "naiveOut": naive_outs, "dateOut": date_outs, "timeOut": time_outs,
            }));
        }
    }

    // Offsets.
    let offsets: Vec<Value> = [0, 1, -1, 59, 3600, -3600, 19800, -12345, 86399, -86399, 86400, -86400, 100000]
        .iter()
        .map(|&s| {
            let o = FixedOffset::east_opt(s);
            json!([s, o.map(|o| format!("{o:?}")), o.map(|o| o.to_string()),
                FixedOffset::west_opt(s).map(|o| o.local_minus_utc())])
        })
        .collect();

    // Arithmetic on UTC date-times.
    let base_parts = [
        (2024, 1, 31, 12, 0, 0), (2024, 2, 29, 0, 0, 0), (2023, 3, 31, 23, 59, 59), (2000, 12, 31, 1, 2, 3),
        (1970, 1, 1, 0, 0, 0), (0, 1, 1, 0, 0, 0), (-1, 3, 31, 6, 0, 0), (262142, 12, 31, 23, 59, 59),
        (-262143, 1, 1, 0, 0, 0), (2026, 3, 15, 12, 34, 56),
    ];
    let bases: Vec<DateTime<Utc>> = base_parts
    .iter()
    .map(|&(y, mo, d, h, mi, s)| Utc.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap())
    .collect();
    let months: &[u32] = &[0, 1, 2, 11, 12, 13, 25, 1200, 3_000_000, 2_147_483_647, 2_147_483_648, u32::MAX];
    let secs: &[i64] = &[0, 1, -1, 59, 86399, 86400, -86400, 2_592_000, -31_536_000, 10_000_000_000,
        -10_000_000_000, 8_000_000_000_000, i64::MAX / 1000, -i64::MAX / 1000];
    let mut arith = Vec::new();
    for (b, p) in bases.iter().zip(&base_parts) {
        let add_m: Vec<Value> = months.iter().map(|&m| {
            json!([b.checked_add_months(Months::new(m)).map(|d| d.to_rfc3339()),
                b.checked_sub_months(Months::new(m)).map(|d| d.to_rfc3339()),
                b.naive_utc().checked_add_months(Months::new(m)).unwrap_or(b.naive_utc()).and_utc().to_rfc3339()])
        }).collect();
        let add_s: Vec<Value> = secs.iter().map(|&s| {
            let d = Duration::try_seconds(s);
            json!([d.is_some(), d.and_then(|d| b.checked_add_signed(d)).map(|d| d.to_rfc3339()),
                d.and_then(|d| b.checked_sub_signed(d)).map(|d| d.to_rfc3339())])
        }).collect();
        let since: Vec<Value> = bases.iter().map(|o| {
            let d = b.signed_duration_since(*o);
            json!([d.num_seconds(), d.to_std().unwrap_or_default().as_secs(), b.cmp(o) as i8, b == o])
        }).collect();
        arith.push(json!({ "base": b.to_rfc3339(), "ymdHms": [p.0, p.1, p.2, p.3, p.4, p.5], "months": add_m, "secs": add_s, "since": since }));
    }

    // Timestamps in milliseconds.
    let millis: Vec<Value> = [0i64, 1, -1, 999, -999, 1_000, 1_773_578_096_789, -62_135_596_800_000,
        -62_167_219_200_001, 253_402_300_799_999, 8_210_266_876_799_999, 8_210_266_876_800_000,
        -8_334_601_228_800_000, -8_334_601_228_800_001]
        .iter()
        .map(|&ms| json!([ms.to_string(), Utc.timestamp_millis_opt(ms).single().map(|d| d.to_rfc3339())]))
        .collect();

    let ymd: Vec<Value> = [(2024, 2, 29), (2023, 2, 29), (2024, 4, 31), (2024, 13, 1), (2024, 0, 1),
        (2024, 1, 0), (262142, 12, 31), (262143, 1, 1), (-262143, 1, 1), (-262144, 12, 31), (0, 2, 29),
        (-4, 2, 29), (-1, 2, 29), (1900, 2, 29)]
        .iter()
        .map(|&(y, m, d)| json!([y, m, d, NaiveDate::from_ymd_opt(y, m, d).map(|d| format!("{d:?}"))]))
        .collect();
    let hms: Vec<Value> = [(0, 0, 0), (23, 59, 59), (24, 0, 0), (12, 60, 0), (12, 0, 60)]
        .iter()
        .map(|&(h, m, s)| json!([h, m, s, NaiveTime::from_hms_opt(h, m, s).map(|t| format!("{t:?}"))]))
        .collect();

    let out = json!({
        "parse": parse, "rfc3339": rfc3339, "format": format, "formats": FORMATS,
        "offsets": offsets, "arith": arith, "months": months, "secs": secs.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "millis": millis, "ymd": ymd, "hms": hms,
    });
    println!("{out}");
}

/// `overflowing_naive_local` is crate-private; this is its public equivalent
/// for the in-range values used here.
trait LocalExt {
    fn overflowing_naive_local_public(&self) -> NaiveDateTime;
}

impl LocalExt for DateTime<FixedOffset> {
    fn overflowing_naive_local_public(&self) -> NaiveDateTime {
        self.naive_utc().overflowing_add_offset_public(*self.offset())
    }
}

trait OffsetExt {
    fn overflowing_add_offset_public(&self, off: FixedOffset) -> NaiveDateTime;
}

impl OffsetExt for NaiveDateTime {
    fn overflowing_add_offset_public(&self, off: FixedOffset) -> NaiveDateTime {
        self.checked_add_offset(off).unwrap_or(*self)
    }
}
