//! Tests for the flavor-locked date serial systems (P1.4, issue #526).
//!
//! Sheets system: day 0 = 1899-12-30, no leap-year bug.
//! Excel 1900 system: serial 1 = 1900-01-01, serial 60 = fictitious 1900-02-29.

use chrono::NaiveDate;
use super::serial::{
    date_to_serial, excel_serial_to_ymd, excel_ymd_to_serial, serial_to_date,
};

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// ── Sheets epoch (pipeline-verified anchors, workbook.tsv PR #559) ──────────

#[test]
fn sheets_serial_zero_is_1899_12_30() {
    // workbook.tsv: =TEXT(0,"yyyy-mm-dd") → 1899-12-30
    assert_eq!(date_to_serial(ymd(1899, 12, 30)), 0.0);
    assert_eq!(serial_to_date(0.0), Some(ymd(1899, 12, 30)));
}

#[test]
fn sheets_serial_one_is_1899_12_31() {
    // workbook.tsv: =TEXT(1,"yyyy-mm-dd") → 1899-12-31
    assert_eq!(date_to_serial(ymd(1899, 12, 31)), 1.0);
}

#[test]
fn sheets_has_no_leap_year_bug() {
    // workbook.tsv: =N(DATE(1900,2,28)) → 60, =N(DATE(1900,3,1)) → 61.
    // 1900 is not a leap year; 60 and 61 are adjacent days.
    assert_eq!(date_to_serial(ymd(1900, 2, 28)), 60.0);
    assert_eq!(date_to_serial(ymd(1900, 3, 1)), 61.0);
}

#[test]
fn sheets_negative_serials_are_pre_1900_dates() {
    assert_eq!(date_to_serial(ymd(1899, 12, 29)), -1.0);
    assert_eq!(serial_to_date(-1.0), Some(ymd(1899, 12, 29)));
}

// ── Excel 1900 system (flavor-locked; evaluation still stubbed) ─────────────

#[test]
fn excel_serial_one_is_1900_01_01() {
    assert_eq!(excel_ymd_to_serial(1900, 1, 1), Some(1.0));
    assert_eq!(excel_serial_to_ymd(1.0), Some((1900, 1, 1)));
}

#[test]
fn excel_serial_59_is_1900_02_28() {
    assert_eq!(excel_ymd_to_serial(1900, 2, 28), Some(59.0));
    assert_eq!(excel_serial_to_ymd(59.0), Some((1900, 2, 28)));
}

#[test]
fn excel_serial_60_is_the_fictitious_leap_day() {
    // The Lotus 1-2-3 leap-year bug: 1900-02-29 does not exist, but the
    // Excel 1900 system assigns it serial 60.
    assert_eq!(excel_ymd_to_serial(1900, 2, 29), Some(60.0));
    assert_eq!(excel_serial_to_ymd(60.0), Some((1900, 2, 29)));
}

#[test]
fn excel_serial_61_is_1900_03_01_and_coincides_with_sheets() {
    assert_eq!(excel_ymd_to_serial(1900, 3, 1), Some(61.0));
    assert_eq!(excel_serial_to_ymd(61.0), Some((1900, 3, 1)));
    // From 1900-03-01 onward the two systems share serials.
    assert_eq!(date_to_serial(ymd(1900, 3, 1)), 61.0);
}

#[test]
fn excel_and_sheets_diverge_only_before_1900_03_01() {
    // Divergence zone: Excel = Sheets − 1 for 1900-01-01..=1900-02-28.
    for (y, m, d) in [(1900, 1, 1), (1900, 1, 31), (1900, 2, 28)] {
        let sheets = date_to_serial(ymd(y, m, d));
        let excel = excel_ymd_to_serial(y, m, d).unwrap();
        assert_eq!(excel, sheets - 1.0, "{y}-{m}-{d}");
    }
    // Coincidence zone: a modern date (2026-06-07, Sheets serial 46180).
    let sheets = date_to_serial(ymd(2026, 6, 7));
    assert_eq!(sheets, 46180.0);
    assert_eq!(excel_ymd_to_serial(2026, 6, 7), Some(46180.0));
    assert_eq!(excel_serial_to_ymd(46180.0), Some((2026, 6, 7)));
}

#[test]
fn excel_rejects_dates_before_1900() {
    assert_eq!(excel_ymd_to_serial(1899, 12, 31), None);
    assert_eq!(excel_serial_to_ymd(0.0), None);
    assert_eq!(excel_serial_to_ymd(-5.0), None);
}
