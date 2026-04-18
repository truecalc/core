use super::super::*;
use crate::types::Value;

#[test]
fn day_15th() {
    // DATE(2024,6,15) = serial 45458
    assert_eq!(day_fn(&[Value::Number(45458.0)]), Value::Number(15.0));
}

#[test]
fn day_first() {
    // DATE(2024,1,1) = serial 45292
    assert_eq!(day_fn(&[Value::Number(45292.0)]), Value::Number(1.0));
}

#[test]
fn day_31st() {
    // DATE(2024,1,31) = serial 45322
    assert_eq!(day_fn(&[Value::Number(45322.0)]), Value::Number(31.0));
}

#[test]
fn day_feb29_leap() {
    // DATE(2024,2,29) = serial 45351
    assert_eq!(day_fn(&[Value::Number(45351.0)]), Value::Number(29.0));
}
