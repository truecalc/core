use super::super::*;
use crate::types::Value;

// Oracle: DAYS360(DATE(2024,6,15), DATE(2024,6,15)) = 0
#[test]
fn same_day_zero() {
    let args = [Value::Number(45458.0), Value::Number(45458.0)];
    assert_eq!(days360_fn(&args), Value::Number(0.0));
}

// Oracle: DAYS360(DATE(2024,1,31), DATE(2024,2,28), FALSE) = 28
// Jan 31 -> d1=30; Feb 28 is not 31 so d2 stays 28; result = 0*360+1*30+(28-30)=28
#[test]
fn us_jan31_to_feb28() {
    // DATE(2024,1,31)=45322, DATE(2024,2,28)=45350
    let args = [Value::Number(45322.0), Value::Number(45350.0), Value::Bool(false)];
    assert_eq!(days360_fn(&args), Value::Number(28.0));
}

// Oracle: DAYS360(DATE(2024,1,31), DATE(2024,2,28), TRUE) = 28
// Euro: d1=31->30, d2=28 (not 31, no change); result = 0*360+1*30+(28-30)=28
#[test]
fn euro_jan31_to_feb28() {
    let args = [Value::Number(45322.0), Value::Number(45350.0), Value::Bool(true)];
    assert_eq!(days360_fn(&args), Value::Number(28.0));
}

// Oracle: DAYS360(DATE(2024,4,1), DATE(2024,1,1)) = -90
#[test]
fn backward_negative() {
    let args = [Value::Number(45383.0), Value::Number(45292.0)];
    assert_eq!(days360_fn(&args), Value::Number(-90.0));
}

// US: Feb 29 (leap, last day of feb) as start, Feb 29 as end -> both set to 30 -> 0
#[test]
fn us_feb_end_to_feb_end_same_month() {
    // DATE(2024,2,29)=45351
    let args = [Value::Number(45351.0), Value::Number(45351.0), Value::Bool(false)];
    assert_eq!(days360_fn(&args), Value::Number(0.0));
}
