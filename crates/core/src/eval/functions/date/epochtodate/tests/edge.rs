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
fn negative_timestamp_returns_num_error() {
    // Negative timestamps are not supported → #NUM!
    let args = [Value::Number(-86400.0), Value::Number(1.0)];
    assert_eq!(epochtodate_fn(&args), Value::Error(crate::types::ErrorKind::Num));
}
