use super::super::workday_fn;
use crate::types::Value;

#[test]
fn zero_days_returns_start() {
    // WORKDAY(DATE(2024,1,5), 0) = 45296  (same date returned)
    let args = [Value::Number(45296.0), Value::Number(0.0)];
    assert_eq!(workday_fn(&args), Value::Number(45296.0));
}

#[test]
fn sat_start_plus_1_is_mon() {
    // WORKDAY(DATE(2024,1,6), 1) = 45299  (Saturday start → next Monday)
    let args = [Value::Number(45297.0), Value::Number(1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45299.0));
}

#[test]
fn sun_start_plus_1_is_mon() {
    // WORKDAY(DATE(2024,1,7), 1) = 45299  (Sunday start → next Monday)
    let args = [Value::Number(45298.0), Value::Number(1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45299.0));
}

#[test]
fn negative_days() {
    // WORKDAY(DATE(2024,1,1), -1) = 45289
    let args = [Value::Number(45292.0), Value::Number(-1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45289.0));
}
