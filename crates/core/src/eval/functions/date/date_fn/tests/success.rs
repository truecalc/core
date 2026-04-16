use super::super::*;
use crate::types::Value;

#[test]
fn basic_date_2024_01_15() {
    let args = [Value::Number(2024.0), Value::Number(1.0), Value::Number(15.0)];
    assert_eq!(date_fn(&args), Value::Date(45306.0));
}

#[test]
fn basic_date_2024_06_15() {
    let args = [Value::Number(2024.0), Value::Number(6.0), Value::Number(15.0)];
    assert_eq!(date_fn(&args), Value::Date(45458.0));
}

#[test]
fn jan_1_1900() {
    let args = [Value::Number(1900.0), Value::Number(1.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Date(2.0));
}

#[test]
fn leap_day_2000() {
    let args = [Value::Number(2000.0), Value::Number(2.0), Value::Number(29.0)];
    assert_eq!(date_fn(&args), Value::Date(36585.0));
}

#[test]
fn leap_day_2024() {
    let args = [Value::Number(2024.0), Value::Number(2.0), Value::Number(29.0)];
    assert_eq!(date_fn(&args), Value::Date(45351.0));
}
