use super::super::*;
use crate::types::Value;

#[test]
fn mode_simple() {
    // MODE(1,2,3,2,4) = 2
    assert_eq!(
        mode_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(2.0),
            Value::Number(4.0)
        ]),
        Value::Number(2.0)
    );
}

#[test]
fn mode_tie_returns_smallest() {
    // MODE(1,1,2,2,3) → 1 (tie: pick smallest)
    assert_eq!(
        mode_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ]),
        Value::Number(1.0)
    );
}
