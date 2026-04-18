use super::super::*;
use crate::types::Value;

// Oracle: EDATE(DATE(2024,1,31), 1) = 45351 = DATE(2024,2,29) (leap year snap)
#[test]
fn jan31_plus_one_snaps_to_feb29_leap() {
    let args = [Value::Number(45322.0), Value::Number(1.0)];
    assert_eq!(edate_fn(&args), Value::Number(45351.0));
}

// Oracle: EDATE(DATE(2024,1,31), 2) = 45382 = DATE(2024,3,31)
#[test]
fn jan31_plus_two_is_mar31() {
    let args = [Value::Number(45322.0), Value::Number(2.0)];
    assert_eq!(edate_fn(&args), Value::Number(45382.0));
}

// Oracle: EDATE(DATE(2024,3,31), -1) = 45351 = DATE(2024,2,29) (leap year snap)
#[test]
fn mar31_minus_one_snaps_to_feb29_leap() {
    let args = [Value::Number(45382.0), Value::Number(-1.0)];
    assert_eq!(edate_fn(&args), Value::Number(45351.0));
}

// Oracle: EDATE(DATE(2024,1,15), 13) = 45703 = DATE(2025,2,15) (year rollover)
#[test]
fn year_rollover_plus_13_months() {
    let args = [Value::Number(45306.0), Value::Number(13.0)];
    assert_eq!(edate_fn(&args), Value::Number(45703.0));
}

// Fractional months: truncated (1.9 treated as 1)
#[test]
fn fractional_months_truncated() {
    let args_int = [Value::Number(45306.0), Value::Number(1.0)];
    let args_frac = [Value::Number(45306.0), Value::Number(1.9)];
    assert_eq!(edate_fn(&args_int), edate_fn(&args_frac));
}
