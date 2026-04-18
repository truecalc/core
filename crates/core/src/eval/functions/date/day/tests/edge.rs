use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn serial_zero_is_day_30() {
    // serial 0 = Dec 30 1899
    assert_eq!(day_fn(&[Value::Number(0.0)]), Value::Number(30.0));
}

#[test]
fn serial_1_is_day_31() {
    // oracle: DAY(1) = 31 (Dec 31 1899)
    assert_eq!(day_fn(&[Value::Number(1.0)]), Value::Number(31.0));
}

#[test]
fn serial_60_excel_bug() {
    // oracle: DAY(60) = 28 (Feb 28 1900, phantom Feb 29)
    assert_eq!(day_fn(&[Value::Number(60.0)]), Value::Number(28.0));
}

#[test]
fn negative_serial_returns_valid_day() {
    // serial -1 = Dec 29 1899 — negative serials are valid (pre-1900 dates)
    assert_eq!(day_fn(&[Value::Number(-1.0)]), Value::Number(29.0));
}
