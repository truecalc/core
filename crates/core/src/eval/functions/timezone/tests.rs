use super::*;
use crate::types::{ErrorKind, Value};

fn num(n: f64) -> Value {
    Value::Number(n)
}
fn txt(s: &str) -> Value {
    Value::Text(s.to_string())
}

/// Build `TZDATETIME(y,m,d,h,mi,s,zone[,policy])`.
fn dt(args: &[Value]) -> Value {
    tzdatetime_fn(args)
}

#[test]
fn tzdatetime_builds_berlin_summer() {
    let v = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    match v {
        Value::Zoned(z) => {
            assert_eq!(z.offset_minutes(), 120);
            assert_eq!(z.to_rfc9557(), "2026-07-14T11:00:00+02:00[Europe/Berlin]");
        }
        other => panic!("expected Zoned, got {other:?}"),
    }
}

#[test]
fn tzdatetime_gap_rejected_by_default() {
    // 2026-03-29 02:30 Berlin is in the spring-forward gap.
    let v = dt(&[num(2026.0), num(3.0), num(29.0), num(2.0), num(30.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(v, Value::Error(ErrorKind::Value));
}

#[test]
fn tzdatetime_gap_compatible_resolves() {
    let v = dt(&[num(2026.0), num(3.0), num(29.0), num(2.0), num(30.0), num(0.0), txt("Europe/Berlin"), txt("compatible")]);
    match v {
        Value::Zoned(z) => assert_eq!(z.offset_minutes(), 120),
        other => panic!("expected Zoned, got {other:?}"),
    }
}

#[test]
fn tzdatetime_invalid_policy_is_value_error() {
    let v = dt(&[num(2026.0), num(1.0), num(1.0), num(0.0), num(0.0), num(0.0), txt("UTC"), txt("nonsense")]);
    assert_eq!(v, Value::Error(ErrorKind::Value));
}

#[test]
fn tzdatetime_invalid_zone_is_value_error() {
    let v = dt(&[num(2026.0), num(1.0), num(1.0), num(0.0), num(0.0), num(0.0), txt("Mars/Olympus")]);
    assert_eq!(v, Value::Error(ErrorKind::Value));
}

#[test]
fn tzdatetime_invalid_calendar_is_value_error() {
    let v = dt(&[num(2026.0), num(13.0), num(1.0), num(0.0), num(0.0), num(0.0), txt("UTC")]);
    assert_eq!(v, Value::Error(ErrorKind::Value));
}

#[test]
fn tzconvert_preserves_instant_changes_label() {
    let berlin = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let tokyo = tzconvert_fn(&[berlin.clone(), txt("Asia/Tokyo")]);
    match (&berlin, &tokyo) {
        (Value::Zoned(b), Value::Zoned(t)) => {
            assert_eq!(b.utc_nanos, t.utc_nanos); // same instant
            assert_eq!(t.offset_minutes(), 540); // Tokyo +09:00, no DST
        }
        _ => panic!("expected two Zoned values"),
    }
    assert_eq!(
        tzstring_fn(&[tokyo]),
        Value::Text("2026-07-14T18:00:00+09:00[Asia/Tokyo]".to_string())
    );
}

#[test]
fn tzoffset_and_tzstring_winter() {
    let z = dt(&[num(2026.0), num(1.0), num(14.0), num(9.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(tzoffset_fn(&[z.clone()]), Value::Number(60.0)); // CET
    assert_eq!(
        tzstring_fn(&[z]),
        Value::Text("2026-01-14T09:00:00+01:00[Europe/Berlin]".to_string())
    );
}

#[test]
fn tzserial_drops_zone_to_wallclock() {
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(12.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    match tzserial_fn(&[z]) {
        // Wall clock 12:00 -> fractional day 0.5.
        Value::Date(s) => assert_eq!(s.fract(), 0.5),
        other => panic!("expected Date, got {other:?}"),
    }
}

#[test]
fn tzvalid_classifies_zones() {
    assert_eq!(tzvalid_fn(&[txt("Europe/Berlin")]), Value::Bool(true));
    assert_eq!(tzvalid_fn(&[txt("+05:30")]), Value::Bool(true));
    assert_eq!(tzvalid_fn(&[txt("UTC")]), Value::Bool(true));
    assert_eq!(tzvalid_fn(&[txt("Not/AZone")]), Value::Bool(false));
    assert_eq!(tzvalid_fn(&[num(1.0)]), Value::Bool(false));
}

#[test]
fn iszoned_predicate() {
    let z = dt(&[num(2026.0), num(1.0), num(1.0), num(0.0), num(0.0), num(0.0), txt("UTC")]);
    assert_eq!(iszoned_fn(&[z]), Value::Bool(true));
    assert_eq!(iszoned_fn(&[num(1.0)]), Value::Bool(false));
    assert_eq!(iszoned_fn(&[txt("UTC")]), Value::Bool(false));
}

#[test]
fn tzdbversion_returns_text() {
    assert!(matches!(tzdbversion_fn(&[]), Value::Text(_)));
}

#[test]
fn introspection_rejects_non_zoned() {
    assert_eq!(tzoffset_fn(&[num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzstring_fn(&[txt("x")]), Value::Error(ErrorKind::Value));
    assert_eq!(tzserial_fn(&[num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzconvert_fn(&[num(1.0), txt("UTC")]), Value::Error(ErrorKind::Value));
}

#[test]
fn wrong_arity_returns_na() {
    assert_eq!(tzoffset_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(tzdbversion_fn(&[num(1.0)]), Value::Error(ErrorKind::NA));
    assert_eq!(tzdatetime_fn(&[num(2026.0)]), Value::Error(ErrorKind::NA));
}

// ── Batch 2: alt construction, introspection, extraction ──────────────────────

#[test]
fn tzlocalize_round_trips_with_tzserial() {
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let serial = match tzserial_fn(&[z.clone()]) {
        Value::Date(s) => s,
        other => panic!("expected Date, got {other:?}"),
    };
    let z2 = tzlocalize_fn(&[Value::Date(serial), txt("Europe/Berlin")]);
    match (&z, &z2) {
        (Value::Zoned(a), Value::Zoned(b)) => assert_eq!(a.utc_nanos, b.utc_nanos),
        _ => panic!("expected two Zoned values"),
    }
}

#[test]
fn tzlocalize_invalid_zone_is_value_error() {
    assert_eq!(tzlocalize_fn(&[num(46000.0), txt("Mars/Olympus")]), Value::Error(ErrorKind::Value));
}

#[test]
fn tzfromepoch_unix_zero_in_new_york() {
    // Unix 0 = 1970-01-01T00:00:00Z = 1969-12-31 19:00 EST.
    let z = tzfromepoch_fn(&[num(0.0), txt("America/New_York")]);
    assert_eq!(tzoffset_fn(&[z.clone()]), Value::Number(-300.0));
    assert_eq!(
        tzstring_fn(&[z]),
        Value::Text("1969-12-31T19:00:00-05:00[America/New_York]".to_string())
    );
}

#[test]
fn tzparse_handles_rfc9557_naive_and_offset_only() {
    // Bracketed RFC-9557.
    assert_eq!(
        tzoffset_fn(&[tzparse_fn(&[txt("2026-07-14T11:00:00+02:00[Europe/Berlin]")])]),
        Value::Number(120.0)
    );
    // Naive datetime + explicit zone.
    assert_eq!(
        tzstring_fn(&[tzparse_fn(&[txt("2026-07-14 11:00:00"), txt("Europe/Berlin")])]),
        Value::Text("2026-07-14T11:00:00+02:00[Europe/Berlin]".to_string())
    );
    // Offset-only -> fixed zone.
    assert_eq!(
        tzoffset_fn(&[tzparse_fn(&[txt("2026-01-01T12:00:00+05:30")])]),
        Value::Number(330.0)
    );
    // Naive without a zone, and garbage, are errors.
    assert_eq!(tzparse_fn(&[txt("2026-07-14 11:00:00")]), Value::Error(ErrorKind::Value));
    assert_eq!(tzparse_fn(&[txt("garbage")]), Value::Error(ErrorKind::Value));
}

#[test]
fn tzisdst_and_tzabbr_track_dst() {
    let summer = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let winter = dt(&[num(2026.0), num(1.0), num(14.0), num(9.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(tzisdst_fn(&[summer.clone()]), Value::Bool(true));
    assert_eq!(tzisdst_fn(&[winter.clone()]), Value::Bool(false));
    assert_eq!(tzabbr_fn(&[summer]), Value::Text("CEST".to_string()));
    assert_eq!(tzabbr_fn(&[winter]), Value::Text("CET".to_string()));
}

#[test]
fn tzoffsetdiff_between_zones() {
    let berlin = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let tokyo = tzconvert_fn(&[berlin.clone(), txt("Asia/Tokyo")]);
    // Tokyo +540, Berlin +120 -> +420.
    assert_eq!(tzoffsetdiff_fn(&[tokyo, berlin]), Value::Number(420.0));
}

#[test]
fn tzpart_extracts_local_fields() {
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(30.0), num(45.0), txt("Europe/Berlin")]);
    assert_eq!(tzpart_fn(&[z.clone(), txt("year")]), Value::Number(2026.0));
    assert_eq!(tzpart_fn(&[z.clone(), txt("month")]), Value::Number(7.0));
    assert_eq!(tzpart_fn(&[z.clone(), txt("hour")]), Value::Number(11.0));
    assert_eq!(tzpart_fn(&[z.clone(), txt("minute")]), Value::Number(30.0));
    assert_eq!(tzpart_fn(&[z.clone(), txt("second")]), Value::Number(45.0));
    // 2026-07-14 is a Tuesday -> Sunday=1 convention gives 3.
    assert_eq!(tzpart_fn(&[z.clone(), txt("weekday")]), Value::Number(3.0));
    assert_eq!(tzpart_fn(&[z, txt("bogus")]), Value::Error(ErrorKind::Value));
}

#[test]
fn batch2_rejects_non_zoned() {
    assert_eq!(tzisdst_fn(&[num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzabbr_fn(&[num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzpart_fn(&[num(1.0), txt("year")]), Value::Error(ErrorKind::Value));
    assert_eq!(tzoffsetdiff_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::Value));
}

// ── Batch 3: arithmetic (TZDIFF / TZADD) ──────────────────────────────────────

fn utc_dt(y: f64, mo: f64, d: f64, h: f64, mi: f64) -> Value {
    dt(&[num(y), num(mo), num(d), num(h), num(mi), num(0.0), txt("UTC")])
}

#[test]
fn tzdiff_units_and_sign() {
    let a = utc_dt(2026.0, 1.0, 15.0, 15.0, 0.0);
    let b = utc_dt(2026.0, 1.0, 15.0, 12.0, 0.0);
    assert_eq!(tzdiff_fn(&[a.clone(), b.clone()]), Value::Number(10_800.0)); // default seconds
    assert_eq!(tzdiff_fn(&[a.clone(), b.clone(), txt("hours")]), Value::Number(3.0));
    assert_eq!(tzdiff_fn(&[a.clone(), b.clone(), txt("minutes")]), Value::Number(180.0));
    assert_eq!(tzdiff_fn(&[b, a, txt("hours")]), Value::Number(-3.0)); // sign flips
}

#[test]
fn tzadd_absolute_hours_is_dst_immune() {
    let z = utc_dt(2026.0, 1.0, 15.0, 12.0, 0.0);
    let r = tzadd_fn(&[z, num(3.0), txt("hours")]);
    assert_eq!(tzpart_fn(&[r.clone(), txt("hour")]), Value::Number(15.0));
    assert!(matches!(r, Value::Zoned(_)));
}

#[test]
fn tzadd_calendar_day_keeps_wallclock_across_dst() {
    // 2026-03-29 is the Berlin spring-forward day (the 02:00->03:00 jump).
    let start = dt(&[num(2026.0), num(3.0), num(28.0), num(12.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let next = tzadd_fn(&[start.clone(), num(1.0), txt("days")]);
    // Calendar add keeps the wall clock at 12:00 the next day...
    assert_eq!(tzpart_fn(&[next.clone(), txt("hour")]), Value::Number(12.0));
    assert_eq!(tzpart_fn(&[next.clone(), txt("day")]), Value::Number(29.0));
    // ...but that calendar day was only 23 hours long on the absolute timeline.
    assert_eq!(tzdiff_fn(&[next, start, txt("hours")]), Value::Number(23.0));
}

#[test]
fn tzadd_absolute_24h_lands_one_hour_ahead_across_spring_forward() {
    let start = dt(&[num(2026.0), num(3.0), num(28.0), num(12.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let later = tzadd_fn(&[start, num(24.0), txt("hours")]);
    // +24h absolute over a 23h day -> wall clock is 13:00, not 12:00.
    assert_eq!(tzpart_fn(&[later, txt("hour")]), Value::Number(13.0));
}

#[test]
fn tzadd_months_and_years() {
    let z = utc_dt(2026.0, 1.0, 15.0, 12.0, 0.0);
    let m = tzadd_fn(&[z.clone(), num(1.0), txt("months")]);
    assert_eq!(tzpart_fn(&[m.clone(), txt("month")]), Value::Number(2.0));
    assert_eq!(tzpart_fn(&[m, txt("day")]), Value::Number(15.0));
    let y = tzadd_fn(&[z.clone(), num(2.0), txt("years")]);
    assert_eq!(tzpart_fn(&[y, txt("year")]), Value::Number(2028.0));
    // Negative amounts subtract.
    let back = tzadd_fn(&[z, num(-1.0), txt("days")]);
    assert_eq!(tzpart_fn(&[back, txt("day")]), Value::Number(14.0));
}

#[test]
fn tzadd_month_end_clamps() {
    // Jan 31 + 1 month clamps to Feb 28 (2026 is not a leap year).
    let z = utc_dt(2026.0, 1.0, 31.0, 9.0, 0.0);
    let m = tzadd_fn(&[z, num(1.0), txt("months")]);
    assert_eq!(tzpart_fn(&[m.clone(), txt("month")]), Value::Number(2.0));
    assert_eq!(tzpart_fn(&[m, txt("day")]), Value::Number(28.0));
}

#[test]
fn arithmetic_error_cases() {
    let z = utc_dt(2026.0, 1.0, 15.0, 12.0, 0.0);
    assert_eq!(tzadd_fn(&[z.clone(), num(1.0), txt("fortnights")]), Value::Error(ErrorKind::Value));
    assert_eq!(tzadd_fn(&[num(1.0), num(1.0), txt("days")]), Value::Error(ErrorKind::Value));
    assert_eq!(tzdiff_fn(&[z, num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzdiff_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::Value));
}
