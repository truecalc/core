use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn large_sum_stays_finite() {
    assert_eq!(
        sum_fn(&[Value::Number(1e308), Value::Number(0.0)]),
        Value::Number(1e308)
    );
}

#[test]
fn negative_sum() {
    assert_eq!(
        sum_fn(&[Value::Number(-1.0), Value::Number(-2.0)]),
        Value::Number(-3.0)
    );
}

#[test]
fn mixed_bool_text_number() {
    // TRUE=1, "2"=2, 3=3 → 6
    assert_eq!(
        sum_fn(&[
            Value::Bool(true),
            Value::Text("2".to_string()),
            Value::Number(3.0)
        ]),
        Value::Number(6.0)
    );
}

#[test]
fn empty_arg_treated_as_zero() {
    assert_eq!(sum_fn(&[Value::Empty, Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn overflow_to_infinity_returns_num_error() {
    assert_eq!(
        sum_fn(&[Value::Number(1e308_f64), Value::Number(1e308_f64)]),
        Value::Error(ErrorKind::Num)
    );
}
