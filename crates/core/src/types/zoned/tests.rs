use super::*;
use chrono::NaiveDate;

fn ndt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(y, mo, d)
        .unwrap()
        .and_hms_opt(h, mi, 0)
        .unwrap()
}

fn berlin() -> ZoneId {
    ZoneId::Iana(chrono_tz::Europe::Berlin)
}

#[test]
fn berlin_summer_is_cest_plus_two() {
    let z = ZonedInstant::from_local(berlin(), ndt(2026, 7, 14, 11, 0), AmbiguousPolicy::Reject).unwrap();
    assert_eq!(z.offset_minutes(), 120);
    assert!(z.is_dst());
    assert_eq!(z.abbrev(), "CEST");
    assert_eq!(z.local(), ndt(2026, 7, 14, 11, 0));
    assert_eq!(z.to_rfc9557(), "2026-07-14T11:00:00+02:00[Europe/Berlin]");
}

#[test]
fn berlin_winter_is_cet_plus_one() {
    let z = ZonedInstant::from_local(berlin(), ndt(2026, 1, 14, 11, 0), AmbiguousPolicy::Reject).unwrap();
    assert_eq!(z.offset_minutes(), 60);
    assert!(!z.is_dst());
    assert_eq!(z.abbrev(), "CET");
}

#[test]
fn spring_forward_gap_rejected() {
    // 2026-03-29 02:30 Berlin does not exist (clocks jump 02:00 -> 03:00).
    let r = ZonedInstant::from_local(berlin(), ndt(2026, 3, 29, 2, 30), AmbiguousPolicy::Reject);
    assert!(r.is_none());
}

#[test]
fn spring_forward_gap_nonreject_resolves_into_cest() {
    let z = ZonedInstant::from_local(berlin(), ndt(2026, 3, 29, 2, 30), AmbiguousPolicy::Compatible).unwrap();
    assert_eq!(z.offset_minutes(), 120); // resolved past the gap into CEST
}

#[test]
fn fall_back_fold_reject_then_earliest_latest() {
    // 2026-10-25 02:30 Berlin happens twice (clocks fall 03:00 -> 02:00).
    assert!(ZonedInstant::from_local(berlin(), ndt(2026, 10, 25, 2, 30), AmbiguousPolicy::Reject).is_none());
    let early = ZonedInstant::from_local(berlin(), ndt(2026, 10, 25, 2, 30), AmbiguousPolicy::Earliest).unwrap();
    let late = ZonedInstant::from_local(berlin(), ndt(2026, 10, 25, 2, 30), AmbiguousPolicy::Latest).unwrap();
    assert_eq!(early.offset_minutes(), 120); // first occurrence still CEST
    assert_eq!(late.offset_minutes(), 60); // second occurrence CET
    assert_eq!(late.utc_nanos - early.utc_nanos, 3_600 * 1_000_000_000);
}

#[test]
fn fixed_offset_has_no_dst() {
    let z = ZonedInstant::from_local(ZoneId::Fixed(330), ndt(2026, 1, 1, 12, 0), AmbiguousPolicy::Reject).unwrap();
    assert_eq!(z.offset_minutes(), 330);
    assert!(!z.is_dst());
    assert_eq!(z.abbrev(), "+05:30");
    assert_eq!(z.to_rfc9557(), "2026-01-01T12:00:00+05:30");
}

#[test]
fn equality_is_structural_instant_and_zone() {
    let utc = ZonedInstant::from_instant(0, ZoneId::Iana(chrono_tz::Tz::UTC));
    let berlin_same_instant = ZonedInstant::from_instant(0, berlin());
    // Same instant, different zone: structurally distinct (engine `=` differs — see operator).
    assert_ne!(utc, berlin_same_instant);
    assert_eq!(utc, ZonedInstant::from_instant(0, ZoneId::Iana(chrono_tz::Tz::UTC)));
}

#[test]
fn parse_zone_variants() {
    assert_eq!(parse_zone("Europe/Berlin"), Some(berlin()));
    assert_eq!(parse_zone("UTC"), Some(ZoneId::Iana(chrono_tz::Tz::UTC)));
    assert_eq!(parse_zone("+05:30"), Some(ZoneId::Fixed(330)));
    assert_eq!(parse_zone("-08:00"), Some(ZoneId::Fixed(-480)));
    assert_eq!(parse_zone("Not/AZone"), None);
}

#[test]
fn rfc9557_round_trips() {
    let z = ZonedInstant::from_local(berlin(), ndt(2026, 7, 14, 11, 0), AmbiguousPolicy::Reject).unwrap();
    let back = parse_rfc9557(&z.to_rfc9557()).unwrap();
    assert_eq!(back.utc_nanos, z.utc_nanos);
    assert_eq!(back.zone, z.zone);
}
