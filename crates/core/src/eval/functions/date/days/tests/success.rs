use super::super::*;
use crate::types::Value;

// Oracle: DAYS(DATE(2024,6,15), DATE(2024,1,1)) = 166
#[test]
fn forward_jan_to_jun_2024() {
    let args = [Value::Number(45458.0), Value::Number(45292.0)];
    assert_eq!(days_fn(&args), Value::Number(166.0));
}

// Oracle: DAYS(DATE(2024,1,1), DATE(2024,6,15)) = -166
#[test]
fn backward_negative() {
    let args = [Value::Number(45292.0), Value::Number(45458.0)];
    assert_eq!(days_fn(&args), Value::Number(-166.0));
}

// Oracle: DAYS(DATE(2024,1,2), DATE(2024,1,1)) = 1
#[test]
fn one_day() {
    let args = [Value::Number(45293.0), Value::Number(45292.0)];
    assert_eq!(days_fn(&args), Value::Number(1.0));
}
