use super::super::*;
use crate::types::Value;

#[test]
fn median_odd_count() {
    // MEDIAN(1, 2, 3) → 2
    assert_eq!(
        median_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn median_even_count() {
    // MEDIAN(1, 2, 3, 4) → 2.5
    assert_eq!(
        median_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0)
        ]),
        Value::Number(2.5)
    );
}

#[test]
fn median_single_value() {
    // MEDIAN(1) → 1
    assert_eq!(median_fn(&[Value::Number(1.0)]), Value::Number(1.0));
}
