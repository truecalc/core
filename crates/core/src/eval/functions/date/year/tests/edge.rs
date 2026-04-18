use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn serial_zero_is_dec_30_1899() {
    // serial 0 = Dec 30 1899
    assert_eq!(year_fn(&[Value::Number(0.0)]), Value::Number(1899.0));
}

#[test]
fn serial_1_is_dec_31_1899() {
    // oracle: YEAR(1) = 1899
    assert_eq!(year_fn(&[Value::Number(1.0)]), Value::Number(1899.0));
}

#[test]
fn serial_60_excel_bug() {
    // oracle: YEAR(60) = 1900 (Feb 28 1900, phantom Feb 29)
    assert_eq!(year_fn(&[Value::Number(60.0)]), Value::Number(1900.0));
}

#[test]
fn negative_serial_returns_valid_year() {
    // serial -1 = Dec 29 1899 — negative serials are valid (pre-1900 dates)
    assert_eq!(year_fn(&[Value::Number(-1.0)]), Value::Number(1899.0));
}
