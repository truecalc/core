use super::super::*;
use crate::types::Value;

/// Serial for 2024-01-01 = 45292. TODAY() must be at or above that.
const MIN_SERIAL: f64 = 45292.0;

#[test]
fn returns_a_number() {
    assert!(matches!(today_fn(&[]), Value::Number(_)));
}

#[test]
fn result_is_after_2024_jan_01() {
    if let Value::Number(n) = today_fn(&[]) {
        assert!(n >= MIN_SERIAL, "today serial {n} should be >= {MIN_SERIAL}");
    } else {
        panic!("expected Number");
    }
}

#[test]
fn result_has_no_fractional_part() {
    if let Value::Number(n) = today_fn(&[]) {
        assert_eq!(n.fract(), 0.0, "TODAY() should be a whole number, got {n}");
    } else {
        panic!("expected Number");
    }
}
