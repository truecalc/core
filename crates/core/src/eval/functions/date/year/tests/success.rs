use super::super::*;
use crate::types::Value;

#[test]
fn year_2024_jun() {
    // DATE(2024,6,15) = serial 45458
    assert_eq!(year_fn(&[Value::Number(45458.0)]), Value::Number(2024.0));
}

#[test]
fn year_2024_jan() {
    // DATE(2024,1,1) = serial 45292
    assert_eq!(year_fn(&[Value::Number(45292.0)]), Value::Number(2024.0));
}

#[test]
fn year_9999() {
    // DATE(9999,12,31) = serial 2958465
    assert_eq!(year_fn(&[Value::Number(2958465.0)]), Value::Number(9999.0));
}

#[test]
fn year_1900_jan1() {
    // DATE(1900,1,1) = serial 2
    assert_eq!(year_fn(&[Value::Number(2.0)]), Value::Number(1900.0));
}
