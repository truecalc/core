use super::super::weekday_fn;
use crate::types::Value;

// Oracle: Google Sheets, Date.xlsx, WEEKDAY sheet
// DATE(2024,1,1) = 45292 (Monday), DATE(2024,1,7) = 45298 (Sunday), DATE(2024,1,6) = 45297 (Saturday)

#[test]
fn monday_type1_returns_2() {
    // =WEEKDAY(DATE(2024,1,1),1) → 2
    let args = [Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(weekday_fn(&args), Value::Number(2.0));
}

#[test]
fn monday_type2_returns_1() {
    // =WEEKDAY(DATE(2024,1,1),2) → 1
    let args = [Value::Number(45292.0), Value::Number(2.0)];
    assert_eq!(weekday_fn(&args), Value::Number(1.0));
}

#[test]
fn monday_type3_returns_0() {
    // =WEEKDAY(DATE(2024,1,1),3) → 0
    let args = [Value::Number(45292.0), Value::Number(3.0)];
    assert_eq!(weekday_fn(&args), Value::Number(0.0));
}

#[test]
fn sunday_type1_returns_1() {
    // =WEEKDAY(DATE(2024,1,7),1) → 1
    let args = [Value::Number(45298.0), Value::Number(1.0)];
    assert_eq!(weekday_fn(&args), Value::Number(1.0));
}

#[test]
fn sunday_type2_returns_7() {
    // =WEEKDAY(DATE(2024,1,7),2) → 7
    let args = [Value::Number(45298.0), Value::Number(2.0)];
    assert_eq!(weekday_fn(&args), Value::Number(7.0));
}

#[test]
fn saturday_type1_returns_7() {
    // =WEEKDAY(DATE(2024,1,6),1) → 7
    let args = [Value::Number(45297.0), Value::Number(1.0)];
    assert_eq!(weekday_fn(&args), Value::Number(7.0));
}

#[test]
fn saturday_type2_returns_6() {
    // =WEEKDAY(DATE(2024,1,6),2) → 6
    let args = [Value::Number(45297.0), Value::Number(2.0)];
    assert_eq!(weekday_fn(&args), Value::Number(6.0));
}

#[test]
fn default_type_omitted_equals_type1() {
    // =WEEKDAY(DATE(2024,1,1)) → 2 (same as type 1)
    let args = [Value::Number(45292.0)];
    assert_eq!(weekday_fn(&args), Value::Number(2.0));
}
