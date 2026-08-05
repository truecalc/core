use super::super::*;
use crate::types::Value;

#[test]
fn trimmean_percent_zero_no_trimming() {
    // TRIMMEAN([1,2,3,4,5], 0) = mean of all = 3
    let data = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(trimmean_fn(&[data, Value::Number(0.0)]), Value::Number(3.0));
}

#[test]
fn trimmean_percent_near_one() {
    // TRIMMEAN([1..10], 0.8) — trim = floor(10 * 0.4) = 4 from each end → [5,6] mean = 5.5
    let data = Value::Array(
        (1..=10).map(|i| Value::Number(i as f64)).collect(),
    );
    let result = trimmean_fn(&[data, Value::Number(0.8)]);
    assert_eq!(result, Value::Number(5.5));
}

#[test]
fn trimmean_odd_count() {
    // TRIMMEAN([1,2,3,4,5], 0.4) — trim = floor(5 * 0.2) = 1 from each end → [2,3,4] mean = 3
    let data = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(trimmean_fn(&[data, Value::Number(0.4)]), Value::Number(3.0));
}
