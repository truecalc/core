use super::super::*;
use crate::types::Value;

// Oracle: EOMONTH(DATE(2024,1,15), 0) = 45322 = Jan 31, 2024
#[test]
fn zero_months_end_of_jan() {
    let args = [Value::Number(45306.0), Value::Number(0.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45322.0));
}

// Oracle: EOMONTH(DATE(2024,2,1), 0) = 45351 = Feb 29, 2024 (leap)
#[test]
fn zero_months_end_of_feb_leap() {
    let args = [Value::Number(45323.0), Value::Number(0.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45351.0));
}

// Oracle: EOMONTH(DATE(2023,2,1), 0) = 44985 = Feb 28, 2023 (non-leap)
#[test]
fn zero_months_end_of_feb_non_leap() {
    let args = [Value::Number(44958.0), Value::Number(0.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(44985.0));
}

// Oracle: EOMONTH(DATE(2024,1,15), 1) = 45351 = Feb 29, 2024
#[test]
fn plus_one_jan_to_end_feb_leap() {
    let args = [Value::Number(45306.0), Value::Number(1.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45351.0));
}

// Oracle: EOMONTH(DATE(2024,3,15), -1) = 45351 = Feb 29, 2024
#[test]
fn minus_one_mar_to_end_feb_leap() {
    let args = [Value::Number(45366.0), Value::Number(-1.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45351.0));
}

// Oracle: EOMONTH(DATE(2024,12,1), 0) = 45657 = Dec 31, 2024
#[test]
fn zero_months_end_of_dec() {
    let args = [Value::Number(45627.0), Value::Number(0.0)];
    assert_eq!(eomonth_fn(&args), Value::Number(45657.0));
}
