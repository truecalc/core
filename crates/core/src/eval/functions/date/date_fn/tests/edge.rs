use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn month_13_overflow() {
    // DATE(2024,13,1) -> Jan 1 2025 = 45658
    let args = [Value::Number(2024.0), Value::Number(13.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Number(45658.0));
}

#[test]
fn day_32_overflow() {
    // DATE(2024,1,32) -> Feb 1 2024 = 45323
    let args = [Value::Number(2024.0), Value::Number(1.0), Value::Number(32.0)];
    assert_eq!(date_fn(&args), Value::Number(45323.0));
}

#[test]
fn month_0_underflow() {
    // DATE(2024,0,1) -> Dec 1 2023 = 45261
    let args = [Value::Number(2024.0), Value::Number(0.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Number(45261.0));
}

#[test]
fn day_0_prev_month() {
    // DATE(2024,3,0) -> last day of Feb 2024 = 45351
    let args = [Value::Number(2024.0), Value::Number(3.0), Value::Number(0.0)];
    assert_eq!(date_fn(&args), Value::Number(45351.0));
}

#[test]
fn negative_day() {
    // DATE(2024,3,-1) = 45350
    let args = [Value::Number(2024.0), Value::Number(3.0), Value::Number(-1.0)];
    assert_eq!(date_fn(&args), Value::Number(45350.0));
}

#[test]
fn max_date() {
    // DATE(9999,12,31) = 2958465
    let args = [Value::Number(9999.0), Value::Number(12.0), Value::Number(31.0)];
    assert_eq!(date_fn(&args), Value::Number(2958465.0));
}

#[test]
fn decimal_inputs_truncated() {
    // DATE(2024.9,6.9,15.9) -> same as DATE(2024,6,15) = 45458
    let args = [Value::Number(2024.9), Value::Number(6.9), Value::Number(15.9)];
    assert_eq!(date_fn(&args), Value::Number(45458.0));
}
