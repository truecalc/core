use super::super::networkdays_fn;
use crate::types::Value;

#[test]
fn same_day_workday() {
    // NETWORKDAYS(DATE(2024,1,1), DATE(2024,1,1)) = 1  (Monday)
    let args = [Value::Number(45292.0), Value::Number(45292.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(1.0));
}

#[test]
fn same_day_saturday() {
    // NETWORKDAYS(DATE(2024,1,6), DATE(2024,1,6)) = 0
    let args = [Value::Number(45297.0), Value::Number(45297.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(0.0));
}

#[test]
fn same_day_sunday() {
    // NETWORKDAYS(DATE(2024,1,7), DATE(2024,1,7)) = 0
    let args = [Value::Number(45298.0), Value::Number(45298.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(0.0));
}

#[test]
fn backward_negative() {
    // NETWORKDAYS(DATE(2024,1,8), DATE(2024,1,1)) = -6
    let args = [Value::Number(45299.0), Value::Number(45292.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(-6.0));
}

#[test]
fn sat_to_sun_zero() {
    // NETWORKDAYS(DATE(2024,1,6), DATE(2024,1,7)) = 0
    let args = [Value::Number(45297.0), Value::Number(45298.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(0.0));
}
