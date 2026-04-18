use super::super::*;
use crate::types::Value;

#[test]
fn product_of_three() {
    assert_eq!(
        product_fn(&[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]),
        Value::Number(24.0)
    );
}

#[test]
fn product_single() {
    assert_eq!(product_fn(&[Value::Number(7.0)]), Value::Number(7.0));
}

#[test]
fn product_with_one() {
    assert_eq!(
        product_fn(&[Value::Number(1.0), Value::Number(5.0)]),
        Value::Number(5.0)
    );
}
