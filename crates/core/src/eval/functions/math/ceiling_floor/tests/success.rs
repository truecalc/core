use super::super::*;
use crate::types::Value;

#[test]
fn ceiling_basic() {
    assert_eq!(
        ceiling_fn(&[Value::Number(4.2), Value::Number(1.0)]),
        Value::Number(5.0)
    );
}

#[test]
fn ceiling_to_nearest_5() {
    assert_eq!(
        ceiling_fn(&[Value::Number(11.0), Value::Number(5.0)]),
        Value::Number(15.0)
    );
}

#[test]
fn floor_basic() {
    assert_eq!(
        floor_fn(&[Value::Number(4.8), Value::Number(1.0)]),
        Value::Number(4.0)
    );
}

#[test]
fn floor_to_nearest_5() {
    assert_eq!(
        floor_fn(&[Value::Number(11.0), Value::Number(5.0)]),
        Value::Number(10.0)
    );
}
