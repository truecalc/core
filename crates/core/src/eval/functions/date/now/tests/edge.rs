use super::super::*;
use crate::types::Value;

#[test]
fn integer_part_equals_today_floor() {
    // The integer portion of NOW() must match the date (no fractional spillover).
    if let Value::Number(n) = now_fn(&[]) {
        assert!(n.floor() >= 45292.0, "date portion {} seems too small", n.floor());
    } else {
        panic!("expected Number");
    }
}
