use super::super::*;
use crate::types::Value;

// Oracle: EDATE(DATE(2024,1,15), 1) = 45337 = DATE(2024,2,15)
#[test]
fn add_one_month() {
    let args = [Value::Number(45306.0), Value::Number(1.0)];
    assert_eq!(edate_fn(&args), Value::Number(45337.0));
}

// Oracle: EDATE(DATE(2024,1,15), 6) = 45488 = DATE(2024,7,15)
#[test]
fn add_six_months() {
    let args = [Value::Number(45306.0), Value::Number(6.0)];
    assert_eq!(edate_fn(&args), Value::Number(45488.0));
}

// Oracle: EDATE(DATE(2024,6,15), -1) = 45427 = DATE(2024,5,15)
#[test]
fn subtract_one_month() {
    // DATE(2024,5,15) = 45427
    let args = [Value::Number(45458.0), Value::Number(-1.0)];
    assert_eq!(edate_fn(&args), Value::Number(45427.0));
}

// Oracle: EDATE(DATE(2024,6,15), -12) = 45092 = DATE(2023,6,15)
#[test]
fn subtract_twelve_months() {
    let args = [Value::Number(45458.0), Value::Number(-12.0)];
    assert_eq!(edate_fn(&args), Value::Number(45092.0));
}

// Oracle: EDATE(DATE(2024,6,15), 0) = 45458 (same date)
#[test]
fn zero_months_same_date() {
    let args = [Value::Number(45458.0), Value::Number(0.0)];
    assert_eq!(edate_fn(&args), Value::Number(45458.0));
}
