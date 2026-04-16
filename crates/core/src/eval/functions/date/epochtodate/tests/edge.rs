use super::super::*;
use crate::types::Value;

#[test]
fn fractional_seconds_produce_fractional_serial() {
    // 43200 seconds = half a day → serial 25569.5
    let args = [Value::Number(43200.0), Value::Number(1.0)];
    if let Value::Number(n) = epochtodate_fn(&args) {
        assert!((n - 25569.5).abs() < 1e-9, "expected 25569.5, got {n}");
    } else {
        panic!("expected Number");
    }
}

#[test]
fn negative_timestamp_before_epoch() {
    // -86400 seconds → one day before 1970-01-01 → serial 25568
    let args = [Value::Number(-86400.0), Value::Number(1.0)];
    assert_eq!(epochtodate_fn(&args), Value::Number(25568.0));
}
