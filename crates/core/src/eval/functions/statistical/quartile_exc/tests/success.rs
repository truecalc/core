use super::super::*;
use crate::types::Value;

#[test]
fn quartile_exc_q1() {
    // QUARTILE.EXC([1,2,3,4,5], 1) uses exclusive formula k=0.25, idx=0.25*6-1=0.5
    // sorted=[1,2,3,4,5], lo=0 (1.0), hi=1 (2.0), frac=0.5 → 1.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(1.0)]), Value::Number(1.5));
}

#[test]
fn quartile_exc_q2() {
    // QUARTILE.EXC([1,2,3,4,5], 2) k=0.5, idx=0.5*6-1=2.0 → sorted[2]=3.0
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(2.0)]), Value::Number(3.0));
}

#[test]
fn quartile_exc_q3() {
    // QUARTILE.EXC([1,2,3,4,5], 3) k=0.75, idx=0.75*6-1=3.5
    // lo=3 (4.0), hi=4 (5.0), frac=0.5 → 4.5
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(3.0)]), Value::Number(4.5));
}
