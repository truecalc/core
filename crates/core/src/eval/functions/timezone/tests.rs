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

// ── Batch 4: TZNOW, TZINWINDOW, TZCANONICAL ───────────────────────────────────

/// Call the lazy `TZNOW` with a pinned (or absent) UTC instant.
fn tznow(zone: &str, now_utc_nanos: Option<i64>) -> Value {
    use crate::eval::functions::{EvalCtx, Registry};
    use crate::eval::Context;
    use crate::parser::ast::{Expr, Span};
    let registry = Registry::new();
    let mut ctx = Context::empty();
    ctx.now_utc_nanos = now_utc_nanos;
    let mut eval_ctx = EvalCtx::new(ctx, &registry);
    tznow_fn(&[Expr::Text(zone.to_string(), Span::new(0, 0))], &mut eval_ctx)
}

#[test]
fn tznow_uses_pinned_utc_instant() {
    // Pinned to the Unix epoch.
    assert_eq!(
        tzstring_fn(&[tznow("UTC", Some(0))]),
        Value::Text("1970-01-01T00:00:00+00:00[UTC]".to_string())
    );
    assert_eq!(tzoffset_fn(&[tznow("America/New_York", Some(0))]), Value::Number(-300.0));
}

#[test]
fn tznow_ambient_when_unpinned_is_zoned() {
    assert!(matches!(tznow("UTC", None), Value::Zoned(_)));
}

#[test]
fn tznow_invalid_zone_is_value_error() {
    assert_eq!(tznow("Mars/Olympus", Some(0)), Value::Error(ErrorKind::Value));
}

#[test]
fn tzinwindow_business_hours() {
    // 09:00 = 0.375, 18:00 = 0.75.
    let day = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    let evening = dt(&[num(2026.0), num(7.0), num(14.0), num(20.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(tzinwindow_fn(&[day, num(0.375), num(0.75)]), Value::Bool(true));
    assert_eq!(tzinwindow_fn(&[evening, num(0.375), num(0.75)]), Value::Bool(false));
}

#[test]
fn tzinwindow_overnight_wraps_past_midnight() {
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(2.0), num(0.0), num(0.0), txt("UTC")]);
    // 22:00 -> 06:00 window; 02:00 is inside.
    assert_eq!(tzinwindow_fn(&[z, num(22.0 / 24.0), num(6.0 / 24.0)]), Value::Bool(true));
}

#[test]
fn tzinwindow_days_mask_excludes_weekday() {
    // 2026-07-14 is a Tuesday -> Sunday-indexed position 2.
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(tzinwindow_fn(&[z.clone(), num(0.375), num(0.75), txt("1101111")]), Value::Bool(false));
    assert_eq!(tzinwindow_fn(&[z, num(0.375), num(0.75), txt("1111111")]), Value::Bool(true));
}

#[test]
fn tzcanonical_validates_and_normalizes() {
    assert_eq!(tzcanonical_fn(&[txt("Europe/Berlin")]), Value::Text("Europe/Berlin".to_string()));
    assert_eq!(tzcanonical_fn(&[txt("+05:30")]), Value::Text("+05:30".to_string()));
    assert_eq!(tzcanonical_fn(&[txt("Mars/Olympus")]), Value::Error(ErrorKind::Value));
}

#[test]
fn batch4_rejects_bad_inputs() {
    assert_eq!(tzinwindow_fn(&[num(1.0), num(0.0), num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzcanonical_fn(&[num(1.0)]), Value::Error(ErrorKind::Value));
    assert_eq!(tzinwindow_fn(&[txt("x")]), Value::Error(ErrorKind::NA));
}

// ── Phase 4: flagship + display (TZLOCALSTATUS, TZTEXT, TZBOARD, TZWORLDCLOCK) ─

#[test]
fn tzlocalstatus_classifies_dst_seams() {
    let unique = [num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), txt("Europe/Berlin")];
    let gap = [num(2026.0), num(3.0), num(29.0), num(2.0), num(30.0), txt("Europe/Berlin")];
    let fold = [num(2026.0), num(10.0), num(25.0), num(2.0), num(30.0), txt("Europe/Berlin")];
    assert_eq!(tzlocalstatus_fn(&unique), Value::Text("unique".to_string()));
    assert_eq!(tzlocalstatus_fn(&gap), Value::Text("gap".to_string()));
    assert_eq!(tzlocalstatus_fn(&fold), Value::Text("fold".to_string()));
    // Fixed offsets never have a gap/fold.
    let fixed = [num(2026.0), num(3.0), num(29.0), num(2.0), num(30.0), txt("+05:30")];
    assert_eq!(tzlocalstatus_fn(&fixed), Value::Text("unique".to_string()));
}

#[test]
fn tztext_default_and_custom_format() {
    let z = dt(&[num(2026.0), num(7.0), num(14.0), num(11.0), num(0.0), num(0.0), txt("Europe/Berlin")]);
    assert_eq!(tztext_fn(&[z.clone()]), Value::Text("Tue Jul 14 2026 11:00 CEST".to_string()));
    assert_eq!(tztext_fn(&[z.clone(), txt("%Y-%m-%d")]), Value::Text("2026-07-14".to_string()));
    // A malformed strftime string is rejected rather than panicking.
    assert_eq!(tztext_fn(&[z, txt("%")]), Value::Error(ErrorKind::Value));
}

#[test]
fn tzboard_rows_are_internally_consistent() {
    // Anchor instant = 2026-07-14T09:00:00Z; base defaults to the anchor zone (UTC).
    let anchor = dt(&[num(2026.0), num(7.0), num(14.0), num(9.0), num(0.0), num(0.0), txt("UTC")]);
    let zones = Value::Array(vec![txt("Europe/Berlin"), txt("Asia/Tokyo")]);
    match tzboard_fn(&[anchor, zones]) {
        Value::Array(rows) => {
            assert_eq!(rows.len(), 2);
            match &rows[0] {
                Value::Array(r) => {
                    assert_eq!(r[0], txt("Europe/Berlin"));
                    assert_eq!(r[1], txt("2026-07-14T11:00:00+02:00[Europe/Berlin]"));
                    assert_eq!(r[2], num(120.0)); // offset
                    assert_eq!(r[3], num(120.0)); // delta vs UTC base
                    assert_eq!(r[4], num(0.0)); // rollover
                    assert_eq!(r[5], txt("CEST"));
                    assert_eq!(r[6], Value::Bool(true));
                }
                other => panic!("expected row array, got {other:?}"),
            }
            match &rows[1] {
                Value::Array(r) => {
                    assert_eq!(r[2], num(540.0)); // Tokyo +09:00
                    assert_eq!(r[6], Value::Bool(false));
                }
                other => panic!("expected row array, got {other:?}"),
            }
        }
        other => panic!("expected array, got {other:?}"),
    }
}

#[test]
fn tzboard_base_override_changes_delta() {
    let anchor = dt(&[num(2026.0), num(7.0), num(14.0), num(9.0), num(0.0), num(0.0), txt("UTC")]);
    let zones = Value::Array(vec![txt("Europe/Berlin")]);
    match tzboard_fn(&[anchor, zones, txt("Asia/Tokyo")]) {
        Value::Array(rows) => match &rows[0] {
            // Berlin +120 vs Tokyo +540 base = -420.
            Value::Array(r) => assert_eq!(r[3], num(-420.0)),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn tzworldclock_friendly_sentence() {
    // Anchor 02:00 Los Angeles (PDT, -7h) on a Tuesday = 09:00Z.
    let anchor = dt(&[num(2026.0), num(7.0), num(14.0), num(2.0), num(0.0), num(0.0), txt("America/Los_Angeles")]);
    let zones = Value::Array(vec![txt("Europe/Berlin"), txt("Asia/Tokyo"), txt("Asia/Kathmandu")]);
    assert_eq!(
        tzworldclock_fn(&[anchor, zones]),
        Value::Text(
            "It is 02:00 Tue for you (America/Los_Angeles). \
             Europe/Berlin is 9h ahead (11:00). \
             Asia/Tokyo is 16h ahead (18:00). \
             Asia/Kathmandu is 12:45 ahead (14:45)."
                .to_string()
        )
    );
}

#[test]
fn flagship_aliases_and_registration() {
    use crate::eval::functions::Registry;
    let r = Registry::new();
    for name in ["TZBOARD", "TZTABLE", "TZWORLDCLOCK", "TZCOMPARETEXT", "TZLOCALSTATUS", "TZTEXT"] {
        assert!(r.functions.contains_key(name), "{name} should be registered");
    }
}

#[test]
fn flagship_rejects_bad_inputs() {
    let zones = Value::Array(vec![txt("Europe/Berlin")]);
    assert_eq!(tzboard_fn(&[num(1.0), zones.clone()]), Value::Error(ErrorKind::Value)); // non-zoned anchor
    let anchor = dt(&[num(2026.0), num(1.0), num(1.0), num(0.0), num(0.0), num(0.0), txt("UTC")]);
    assert_eq!(
        tzboard_fn(&[anchor.clone(), Value::Array(vec![txt("Mars/Olympus")])]),
        Value::Error(ErrorKind::Value)
    );
    assert_eq!(tzworldclock_fn(&[anchor, num(5.0)]), Value::Error(ErrorKind::Value)); // zones not text/array
}

// ── TZOVERLAP — working-hours overlap across N zones ──────────────────────────

/// Date serial for a UTC calendar date at midnight.
fn date_serial(y: f64, mo: f64, d: f64) -> Value {
    match tzserial_fn(&[dt(&[num(y), num(mo), num(d), num(0.0), num(0.0), num(0.0), txt("UTC")])]) {
        Value::Date(s) => Value::Date(s),
        other => panic!("expected Date, got {other:?}"),
    }
}

#[test]
fn tzoverlap_new_york_berlin_winter() {
    // 09:00-18:00 local in both. Winter: NY EST(-5), Berlin CET(+1) -> 6h apart.
    // Overlap = 14:00Z-17:00Z = NY 09:00-12:00.
    let zones = Value::Array(vec![txt("America/New_York"), txt("Europe/Berlin")]);
    let res = tzoverlap_fn(&[zones, num(9.0 / 24.0), num(18.0 / 24.0), date_serial(2026.0, 1.0, 15.0)]);
    match res {
        Value::Array(iv) => {
            assert_eq!(iv.len(), 2);
            assert_eq!(tzstring_fn(&[iv[0].clone()]), txt("2026-01-15T09:00:00-05:00[America/New_York]"));
            assert_eq!(tzstring_fn(&[iv[1].clone()]), txt("2026-01-15T12:00:00-05:00[America/New_York]"));
        }
        other => panic!("expected overlap interval, got {other:?}"),
    }
}

#[test]
fn tzoverlap_none_for_opposite_zones() {
    // UTC and a fixed +12 zone never share 09:00-18:00 local.
    let zones = Value::Array(vec![txt("UTC"), txt("+12:00")]);
    assert_eq!(
        tzoverlap_fn(&[zones, num(9.0 / 24.0), num(18.0 / 24.0), date_serial(2026.0, 1.0, 15.0)]),
        Value::Text("No overlap".to_string())
    );
}

#[test]
fn tzoverlap_error_cases() {
    let s = date_serial(2026.0, 1.0, 15.0);
    // Invalid zone.
    assert_eq!(
        tzoverlap_fn(&[Value::Array(vec![txt("Mars/Olympus")]), num(0.375), num(0.75), s.clone()]),
        Value::Error(ErrorKind::Value)
    );
    // Granularity below 1 minute.
    assert_eq!(
        tzoverlap_fn(&[Value::Array(vec![txt("UTC")]), num(0.375), num(0.75), s, num(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}
