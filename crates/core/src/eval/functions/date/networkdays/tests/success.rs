use super::super::networkdays_fn;
use crate::types::Value;

// DATE(2024,1,1) = 45292 (Monday), DATE(2024,1,5) = 45296 (Friday)

#[test]
fn mon_to_fri_five_days() {
    // NETWORKDAYS(DATE(2024,1,1), DATE(2024,1,5)) = 5
    let args = [Value::Number(45292.0), Value::Number(45296.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(5.0));
}

#[test]
fn mon_to_sun_five_workdays() {
    // NETWORKDAYS(DATE(2024,1,1), DATE(2024,1,7)) = 5  (Sat+Sun skipped)
    let args = [Value::Number(45292.0), Value::Number(45298.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(5.0));
}

#[test]
fn two_full_weeks() {
    // NETWORKDAYS(DATE(2024,1,1), DATE(2024,1,14)) = 10
    let args = [Value::Number(45292.0), Value::Number(45305.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(10.0));
}

#[test]
fn mon_to_next_mon_six() {
    // NETWORKDAYS(DATE(2024,1,1), DATE(2024,1,8)) = 6
    let args = [Value::Number(45292.0), Value::Number(45299.0)];
    assert_eq!(networkdays_fn(&args), Value::Number(6.0));
}
