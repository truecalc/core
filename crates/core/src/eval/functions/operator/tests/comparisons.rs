use super::super::{eq_fn, gt_fn, gte_fn, lt_fn, lte_fn, ne_fn, unary_percent_fn, uplus_fn};
use crate::types::{ErrorKind, Value};

fn n(v: f64) -> Value { Value::Number(v) }
fn t(s: &str) -> Value { Value::Text(s.to_string()) }
fn b(v: bool) -> Value { Value::Bool(v) }

// ── EQ ───────────────────────────────────────────────────────────────────────

#[test]
fn eq_equal_numbers() { assert_eq!(eq_fn(&[n(1.0), n(1.0)]), b(true)); }

#[test]
fn eq_unequal_numbers() { assert_eq!(eq_fn(&[n(1.0), n(2.0)]), b(false)); }

#[test]
fn eq_equal_text() { assert_eq!(eq_fn(&[t("a"), t("a")]), b(true)); }

#[test]
fn eq_text_case_insensitive() { assert_eq!(eq_fn(&[t("A"), t("a")]), b(true)); }

#[test]
fn eq_different_types_returns_false() { assert_eq!(eq_fn(&[n(1.0), t("1")]), b(false)); }

#[test]
fn eq_wrong_arity_returns_na() { assert_eq!(eq_fn(&[n(1.0)]), Value::Error(ErrorKind::NA)); }

// ── NE ───────────────────────────────────────────────────────────────────────

#[test]
fn ne_equal_numbers() { assert_eq!(ne_fn(&[n(1.0), n(1.0)]), b(false)); }

#[test]
fn ne_unequal_numbers() { assert_eq!(ne_fn(&[n(1.0), n(2.0)]), b(true)); }

#[test]
fn ne_different_types_returns_true() { assert_eq!(ne_fn(&[n(1.0), t("1")]), b(true)); }

// ── GT ───────────────────────────────────────────────────────────────────────

#[test]
fn gt_greater() { assert_eq!(gt_fn(&[n(2.0), n(1.0)]), b(true)); }

#[test]
fn gt_equal() { assert_eq!(gt_fn(&[n(1.0), n(1.0)]), b(false)); }

#[test]
fn gt_less() { assert_eq!(gt_fn(&[n(0.0), n(1.0)]), b(false)); }

#[test]
fn gt_text_comparison() { assert_eq!(gt_fn(&[t("b"), t("a")]), b(true)); }

// ── GTE ──────────────────────────────────────────────────────────────────────

#[test]
fn gte_greater() { assert_eq!(gte_fn(&[n(2.0), n(1.0)]), b(true)); }

#[test]
fn gte_equal() { assert_eq!(gte_fn(&[n(1.0), n(1.0)]), b(true)); }

#[test]
fn gte_less() { assert_eq!(gte_fn(&[n(0.0), n(1.0)]), b(false)); }

// ── LT ───────────────────────────────────────────────────────────────────────

#[test]
fn lt_less() { assert_eq!(lt_fn(&[n(1.0), n(2.0)]), b(true)); }

#[test]
fn lt_equal() { assert_eq!(lt_fn(&[n(1.0), n(1.0)]), b(false)); }

#[test]
fn lt_greater() { assert_eq!(lt_fn(&[n(2.0), n(1.0)]), b(false)); }

// ── LTE ──────────────────────────────────────────────────────────────────────

#[test]
fn lte_less() { assert_eq!(lte_fn(&[n(1.0), n(2.0)]), b(true)); }

#[test]
fn lte_equal() { assert_eq!(lte_fn(&[n(1.0), n(1.0)]), b(true)); }

#[test]
fn lte_greater() { assert_eq!(lte_fn(&[n(2.0), n(1.0)]), b(false)); }

// ── Cross-type ordering (Number < Text < Bool) ────────────────────────────────

#[test]
fn gt_number_vs_text_number_is_less() {
    // Number rank < Text rank → Number < Text
    assert_eq!(gt_fn(&[n(999.0), t("a")]), b(false));
}

#[test]
fn lt_number_vs_text_number_is_less() {
    assert_eq!(lt_fn(&[n(999.0), t("a")]), b(true));
}

#[test]
fn gt_text_vs_bool_text_is_less() {
    // Text rank < Bool rank → Text < Bool
    assert_eq!(gt_fn(&[t("z"), b(false)]), b(false));
}

// ── UPLUS ────────────────────────────────────────────────────────────────────

#[test]
fn uplus_numeric_string_coerces() { assert_eq!(uplus_fn(&[t("42")]), n(42.0)); }

#[test]
fn uplus_non_numeric_string_passthrough() { assert_eq!(uplus_fn(&[t("abc")]), t("abc")); }

#[test]
fn uplus_number_passthrough() { assert_eq!(uplus_fn(&[n(5.0)]), n(5.0)); }

#[test]
fn uplus_bool_passthrough() { assert_eq!(uplus_fn(&[b(true)]), b(true)); }

#[test]
fn uplus_wrong_arity_returns_na() { assert_eq!(uplus_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA)); }

// ── UNARY PERCENT ─────────────────────────────────────────────────────────────

#[test]
fn percent_divides_by_100() { assert_eq!(unary_percent_fn(&[n(50.0)]), n(0.5)); }

#[test]
fn percent_of_100() { assert_eq!(unary_percent_fn(&[n(100.0)]), n(1.0)); }

#[test]
fn percent_wrong_arity_returns_na() { assert_eq!(unary_percent_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA)); }

#[test]
fn percent_non_numeric_returns_error() { assert_eq!(unary_percent_fn(&[t("abc")]), Value::Error(ErrorKind::Value)); }

// ── Date comparison (issue #671 — =DATE=DATE returned FALSE) ───────────────────

fn d(v: f64) -> Value { Value::Date(v) }

#[test]
fn eq_equal_dates() { assert_eq!(eq_fn(&[d(43831.0), d(43831.0)]), b(true)); }

#[test]
fn eq_unequal_dates() { assert_eq!(eq_fn(&[d(43831.0), d(43832.0)]), b(false)); }

#[test]
fn ne_equal_dates() { assert_eq!(ne_fn(&[d(43831.0), d(43831.0)]), b(false)); }

#[test]
fn eq_date_equals_serial_number() {
    // Google Sheets treats a date as its serial number.
    assert_eq!(eq_fn(&[d(43831.0), n(43831.0)]), b(true));
}

#[test]
fn lt_dates() { assert_eq!(lt_fn(&[d(43831.0), d(43832.0)]), b(true)); }

#[test]
fn gt_dates() { assert_eq!(gt_fn(&[d(43832.0), d(43831.0)]), b(true)); }

#[test]
fn gte_equal_dates() { assert_eq!(gte_fn(&[d(43831.0), d(43831.0)]), b(true)); }

// ── Zoned comparison (Model B: equality/ordering on the instant only) ─────────

use crate::types::{ZoneId, ZonedInstant};

fn z_utc(nanos: i64) -> Value {
    Value::Zoned(Box::new(ZonedInstant::from_instant(nanos, ZoneId::Iana(chrono_tz::Tz::UTC))))
}
fn z_berlin(nanos: i64) -> Value {
    Value::Zoned(Box::new(ZonedInstant::from_instant(nanos, ZoneId::Iana(chrono_tz::Europe::Berlin))))
}

#[test]
fn eq_zoned_same_instant_different_zone() {
    // Engine `=` compares the absolute instant, ignoring the zone label.
    assert_eq!(eq_fn(&[z_utc(0), z_berlin(0)]), b(true));
}

#[test]
fn eq_zoned_different_instant() { assert_eq!(eq_fn(&[z_utc(0), z_utc(1)]), b(false)); }

#[test]
fn lt_zoned_by_instant() { assert_eq!(lt_fn(&[z_utc(0), z_utc(1)]), b(true)); }

#[test]
fn eq_zoned_vs_number_is_value_error() {
    assert_eq!(eq_fn(&[z_utc(0), n(0.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn gt_zoned_vs_text_is_value_error() {
    assert_eq!(gt_fn(&[z_utc(0), t("x")]), Value::Error(ErrorKind::Value));
}
