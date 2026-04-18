use super::super::*;
use crate::types::Value;

#[test]
fn month_june() {
    // DATE(2024,6,15) = serial 45458
    assert_eq!(month_fn(&[Value::Number(45458.0)]), Value::Number(6.0));
}

#[test]
fn month_january() {
    // DATE(2024,1,1) = serial 45292
    assert_eq!(month_fn(&[Value::Number(45292.0)]), Value::Number(1.0));
}

#[test]
fn month_december() {
    // DATE(2024,12,31) = serial 45657
    assert_eq!(month_fn(&[Value::Number(45657.0)]), Value::Number(12.0));
}

#[test]
fn month_february_leap() {
    // DATE(2024,2,29) = serial 45351
    assert_eq!(month_fn(&[Value::Number(45351.0)]), Value::Number(2.0));
}
