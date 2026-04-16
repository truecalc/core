use super::super::*;
use crate::types::Value;

#[test]
fn trimmean_basic() {
    // TRIMMEAN({1,2,3,4,5,6,7,8,9,10}, 0.2)
    // trim = floor(10 * 0.1) = 1 from each end → [2,3,4,5,6,7,8,9] mean = 5.5
    let data = Value::Array(
        (1..=10).map(|i| Value::Number(i as f64)).collect(),
    );
    let result = trimmean_fn(&[data, Value::Number(0.2)]);
    assert_eq!(result, Value::Number(5.5));
}

#[test]
fn trimmean_no_trim() {
    // TRIMMEAN({1,2,3}, 0) → mean of all = 2
    let data = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(trimmean_fn(&[data, Value::Number(0.0)]), Value::Number(2.0));
}
