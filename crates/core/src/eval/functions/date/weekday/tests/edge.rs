use super::super::weekday_fn;
use crate::types::Value;

// DATE(2024,1,1) = 45292 (Monday)

#[test]
fn type11_monday_returns_1() {
    // =WEEKDAY(DATE(2024,1,1),11) → 1 (type 11 = Mon=1, same as type 2)
    let args = [Value::Number(45292.0), Value::Number(11.0)];
    assert_eq!(weekday_fn(&args), Value::Number(1.0));
}

#[test]
fn type17_is_same_as_type1() {
    // type 17: Sunday=1, Saturday=7, same as type 1
    let args1 = [Value::Number(45292.0), Value::Number(1.0)];
    let args17 = [Value::Number(45292.0), Value::Number(17.0)];
    assert_eq!(weekday_fn(&args1), weekday_fn(&args17));
}

#[test]
fn type12_tuesday_is_1() {
    // DATE(2024,1,2) = 45293 (Tuesday). type 12: Tue=1
    let args = [Value::Number(45293.0), Value::Number(12.0)];
    assert_eq!(weekday_fn(&args), Value::Number(1.0));
}

#[test]
fn type16_saturday_is_1() {
    // DATE(2024,1,6) = 45297 (Saturday). type 16: Sat=1
    let args = [Value::Number(45297.0), Value::Number(16.0)];
    assert_eq!(weekday_fn(&args), Value::Number(1.0));
}
