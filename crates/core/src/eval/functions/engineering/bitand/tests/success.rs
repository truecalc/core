use super::super::*;
use crate::types::Value;

#[test]
fn both_zero() {
    assert_eq!(bitand_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn basic_and() {
    assert_eq!(bitand_fn(&[Value::Number(12.0), Value::Number(10.0)]), Value::Number(8.0));
}

#[test]
fn all_ones() {
    assert_eq!(bitand_fn(&[Value::Number(255.0), Value::Number(255.0)]), Value::Number(255.0));
}

#[test]
fn no_overlap() {
    assert_eq!(bitand_fn(&[Value::Number(5.0), Value::Number(2.0)]), Value::Number(0.0));
}

#[test]
fn max_value() {
    let max = 281_474_976_710_655.0_f64;
    assert_eq!(bitand_fn(&[Value::Number(max), Value::Number(max)]), Value::Number(max));
}
