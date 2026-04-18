use super::super::*;
use crate::types::Value;

// Oracle: DAYS(DATE(2024,6,15), DATE(2024,6,15)) = 0
#[test]
fn same_day_zero() {
    let args = [Value::Number(45458.0), Value::Number(45458.0)];
    assert_eq!(days_fn(&args), Value::Number(0.0));
}

// Oracle: DAYS(DATE(2024,3,1), DATE(2024,2,1)) = 29 (leap year)
#[test]
fn across_leap_day_feb_2024() {
    // DATE(2024,3,1) = 45352, DATE(2024,2,1) = 45323
    let args = [Value::Number(45352.0), Value::Number(45323.0)];
    assert_eq!(days_fn(&args), Value::Number(29.0));
}

// Oracle: DAYS(DATE(2025,1,1), DATE(2024,1,1)) = 366 (leap year 2024)
#[test]
fn full_leap_year_2024() {
    // DATE(2025,1,1) = 45658, DATE(2024,1,1) = 45292
    let args = [Value::Number(45658.0), Value::Number(45292.0)];
    assert_eq!(days_fn(&args), Value::Number(366.0));
}

// Oracle: DAYS(DATE(2024,1,1), DATE(2023,1,1)) = 365
#[test]
fn full_non_leap_year_2023() {
    // DATE(2024,1,1) = 45292, DATE(2023,1,1) = 44927
    let args = [Value::Number(45292.0), Value::Number(44927.0)];
    assert_eq!(days_fn(&args), Value::Number(365.0));
}

// Fractional serials: floor is applied
#[test]
fn fractional_serials_floored() {
    // 45458.9 and 45292.1 should use floors 45458 and 45292
    let args = [Value::Number(45458.9), Value::Number(45292.1)];
    assert_eq!(days_fn(&args), Value::Number(166.0));
}
