use super::super::*;
use crate::types::Value;

/// Serial for 2024-01-01 = 45292.
const MIN_SERIAL: f64 = 45292.0;

#[test]
fn returns_a_number() {
    assert!(matches!(now_fn(&[]), Value::Number(_)));
}

#[test]
fn result_is_after_2024_jan_01() {
    if let Value::Number(n) = now_fn(&[]) {
        assert!(n >= MIN_SERIAL, "now serial {n} should be >= {MIN_SERIAL}");
    } else {
        panic!("expected Number");
    }
}

#[test]
fn fractional_part_is_between_0_and_1() {
    if let Value::Number(n) = now_fn(&[]) {
        let frac = n.fract();
        assert!(frac >= 0.0 && frac < 1.0, "fractional part {frac} out of range");
    } else {
        panic!("expected Number");
    }
}
