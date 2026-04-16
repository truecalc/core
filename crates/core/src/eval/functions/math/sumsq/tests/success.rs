use super::super::sumsq_fn;
use crate::types::Value;

#[test]
fn sumsq_two_args() {
    // SUMSQ(3, 4) = 9 + 16 = 25
    assert_eq!(sumsq_fn(&[Value::Number(3.0), Value::Number(4.0)]), Value::Number(25.0));
}

#[test]
fn sumsq_three_args() {
    // SUMSQ(1, 2, 3) = 1 + 4 + 9 = 14
    assert_eq!(
        sumsq_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(14.0)
    );
}

#[test]
fn sumsq_array() {
    // SUMSQ({1,2,3,4}) = 1+4+9+16 = 30
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
    ]);
    assert_eq!(sumsq_fn(&[arr]), Value::Number(30.0));
}

#[test]
fn sumsq_single() {
    // SUMSQ(5) = 25
    assert_eq!(sumsq_fn(&[Value::Number(5.0)]), Value::Number(25.0));
}

#[test]
fn sumsq_zero() {
    // SUMSQ(0) = 0
    assert_eq!(sumsq_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn sumsq_negatives() {
    // SUMSQ(-3, -4) = 9 + 16 = 25
    assert_eq!(sumsq_fn(&[Value::Number(-3.0), Value::Number(-4.0)]), Value::Number(25.0));
}
