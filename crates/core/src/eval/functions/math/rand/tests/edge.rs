use super::super::*;
use crate::types::Value;

#[test]
fn rand_result_is_finite() {
    let result = rand_fn(&[]);
    if let Value::Number(n) = result {
        assert!(n.is_finite());
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn randbetween_negative_range() {
    let result = randbetween_fn(&[Value::Number(-5.0), Value::Number(-1.0)]);
    if let Value::Number(n) = result {
        assert!(n >= -5.0 && n <= -1.0);
        assert_eq!(n, n.floor());
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn randbetween_zero_range() {
    let result = randbetween_fn(&[Value::Number(0.0), Value::Number(0.0)]);
    assert_eq!(result, Value::Number(0.0));
}
