use super::super::*;
use crate::types::Value;

// Oracle: DAYS360(DATE(2024,1,1), DATE(2024,12,31), FALSE) = 360
#[test]
fn us_full_year() {
    // DATE(2024,1,1)=45292, DATE(2024,12,31)=45657
    let args = [Value::Number(45292.0), Value::Number(45657.0), Value::Bool(false)];
    assert_eq!(days360_fn(&args), Value::Number(360.0));
}

// Oracle: DAYS360(DATE(2024,1,1), DATE(2024,12,31), TRUE) = 359
#[test]
fn euro_full_year() {
    let args = [Value::Number(45292.0), Value::Number(45657.0), Value::Bool(true)];
    assert_eq!(days360_fn(&args), Value::Number(359.0));
}

// Oracle: DAYS360(DATE(2024,1,1), DATE(2024,4,1), FALSE) = 90
#[test]
fn us_jan_to_apr() {
    // DATE(2024,4,1)=45383
    let args = [Value::Number(45292.0), Value::Number(45383.0), Value::Bool(false)];
    assert_eq!(days360_fn(&args), Value::Number(90.0));
}

// Oracle: DAYS360(DATE(2024,1,1), DATE(2024,4,1)) = 90 (method omitted, defaults to US)
#[test]
fn default_us_method() {
    let args = [Value::Number(45292.0), Value::Number(45383.0)];
    assert_eq!(days360_fn(&args), Value::Number(90.0));
}
