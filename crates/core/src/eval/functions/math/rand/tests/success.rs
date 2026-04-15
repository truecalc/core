use super::super::*;
use crate::types::Value;

#[test]
fn rand_returns_number_in_range() {
    let result = rand_fn(&[]);
    if let Value::Number(n) = result {
        assert!(n >= 0.0 && n < 1.0, "RAND() must be in [0, 1), got {}", n);
    } else {
        panic!("Expected Number from RAND()");
    }
}

#[test]
fn randbetween_returns_number_in_range() {
    let result = randbetween_fn(&[Value::Number(1.0), Value::Number(10.0)]);
    if let Value::Number(n) = result {
        assert!(n >= 1.0 && n <= 10.0, "RANDBETWEEN must be in [1,10], got {}", n);
        assert_eq!(n, n.floor(), "RANDBETWEEN must return an integer");
    } else {
        panic!("Expected Number from RANDBETWEEN()");
    }
}

#[test]
fn randbetween_same_low_high() {
    let result = randbetween_fn(&[Value::Number(5.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(5.0));
}
