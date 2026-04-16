use super::super::*;
use crate::types::Value;

#[test]
fn epoch_zero_seconds_is_25569() {
    let args = [Value::Number(0.0), Value::Number(1.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25569.0));
}

#[test]
fn one_day_in_seconds_is_25570() {
    let args = [Value::Number(86400.0), Value::Number(1.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25570.0));
}

#[test]
fn known_timestamp_jan_15_2024() {
    // 1705276800 seconds → serial 45306
    let args = [Value::Number(1705276800.0), Value::Number(1.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(45306.0));
}

#[test]
fn milliseconds_unit() {
    // 86400 * 1000 ms → 1 day → serial 25570
    let args = [Value::Number(86_400_000.0), Value::Number(2.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25570.0));
}

#[test]
fn microseconds_unit() {
    // 86400 * 1_000_000 µs → 1 day → serial 25570
    let args = [Value::Number(86_400_000_000.0), Value::Number(3.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25570.0));
}

#[test]
fn default_unit_is_seconds() {
    // Without unit arg, defaults to seconds
    let args = [Value::Number(0.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25569.0));
}
