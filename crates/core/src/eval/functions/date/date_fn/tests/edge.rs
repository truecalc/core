use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn month_13_overflow() {
    // DATE(2024,13,1) -> Jan 1 2025 = 45658
    let args = [Value::Number(2024.0), Value::Number(13.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Date(45658.0));
}

#[test]
fn day_32_overflow() {
    // DATE(2024,1,32) -> Feb 1 2024 = 45323
    let args = [Value::Number(2024.0), Value::Number(1.0), Value::Number(32.0)];
    assert_eq!(date_fn(&args), Value::Date(45323.0));
}

#[test]
fn month_0_underflow() {
    // DATE(2024,0,1) -> Dec 1 2023 = 45261
    let args = [Value::Number(2024.0), Value::Number(0.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Date(45261.0));
}

#[test]
fn day_0_prev_month() {
    // DATE(2024,3,0) -> last day of Feb 2024 = 45351
    let args = [Value::Number(2024.0), Value::Number(3.0), Value::Number(0.0)];
    assert_eq!(date_fn(&args), Value::Date(45351.0));
}

#[test]
fn negative_day() {
    // DATE(2024,3,-1) = 45350
    let args = [Value::Number(2024.0), Value::Number(3.0), Value::Number(-1.0)];
    assert_eq!(date_fn(&args), Value::Date(45350.0));
}

#[test]
fn negative_month_rolls_back_across_epoch_to_negative_serial() {
    // DATE(1900,-1,1) -> Nov 1, 1899 = -59 (issue #849). Equivalent to
    // DATE(1900,1,1)-61: DATE(1900,1,1)=2, 2-61=-59. The engine already
    // supports negative (pre-epoch) serials via arithmetic, YEAR/MONTH/DAY,
    // and TEXT — DATE()'s own month rollover must be consistent with that.
    let args = [Value::Number(1900.0), Value::Number(-1.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Date(-59.0));
}

#[test]
fn month_zero_rolls_back_across_epoch_to_negative_serial() {
    // DATE(1900,0,1) -> Dec 1, 1899 = -29, i.e. DATE(1900,1,1) (=2) minus
    // December 1899's 31 days: 2 - 31 = -29 (issue #849).
    let args = [Value::Number(1900.0), Value::Number(0.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Date(-29.0));
}

#[test]
fn max_date() {
    // DATE(9999,12,31) = 2958465
    let args = [Value::Number(9999.0), Value::Number(12.0), Value::Number(31.0)];
    assert_eq!(date_fn(&args), Value::Date(2958465.0));
}

#[test]
fn decimal_inputs_truncated() {
    // DATE(2024.9,6.9,15.9) -> same as DATE(2024,6,15) = 45458
    let args = [Value::Number(2024.9), Value::Number(6.9), Value::Number(15.9)];
    assert_eq!(date_fn(&args), Value::Date(45458.0));
}

#[test]
fn extreme_negative_day_rollover_below_year_zero_errors() {
    // DATE(1900,1,-5000000): day argument rolls the date back roughly 13,700
    // years, past year 0 — this must still error like the symmetric extreme
    // positive case (below), not silently succeed just because the result
    // has a negative serial. Guards against reintroducing an unbounded
    // negative range while fixing issue #849's near-epoch rollover case.
    let args = [Value::Number(1900.0), Value::Number(1.0), Value::Number(-5000000.0)];
    assert_eq!(date_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn extreme_positive_day_rollover_above_year_9999_errors() {
    // DATE(1900,1,5000000) -> year far beyond 9999, still rejected.
    let args = [Value::Number(1900.0), Value::Number(1.0), Value::Number(5000000.0)];
    assert_eq!(date_fn(&args), Value::Error(ErrorKind::Num));
}
