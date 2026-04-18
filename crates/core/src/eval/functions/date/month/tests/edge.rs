use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn serial_zero_is_dec_1899() {
    // serial 0 = Dec 30 1899
    assert_eq!(month_fn(&[Value::Number(0.0)]), Value::Number(12.0));
}

#[test]
fn serial_1_is_dec_1899() {
    // oracle: MONTH(1) = 12
    assert_eq!(month_fn(&[Value::Number(1.0)]), Value::Number(12.0));
}

#[test]
fn serial_60_excel_bug() {
    // oracle: MONTH(60) = 2 (Feb 28 1900)
    assert_eq!(month_fn(&[Value::Number(60.0)]), Value::Number(2.0));
}

#[test]
fn negative_serial_returns_valid_month() {
    // serial -1 = Dec 29 1899 — negative serials are valid (pre-1900 dates)
    assert_eq!(month_fn(&[Value::Number(-1.0)]), Value::Number(12.0));
}
