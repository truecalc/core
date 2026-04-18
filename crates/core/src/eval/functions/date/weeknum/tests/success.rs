use super::super::weeknum_fn;
use crate::types::Value;

// Oracle: Google Sheets, Date.xlsx, WEEKNUM sheet
// DATE(2024,1,1)=45292 (Mon), DATE(2024,1,7)=45298 (Sun), DATE(2024,1,8)=45299 (Mon)

#[test]
fn jan1_type1_is_week1() {
    // =WEEKNUM(DATE(2024,1,1),1) → 1
    let args = [Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan7_sunday_type1_is_week2() {
    // =WEEKNUM(DATE(2024,1,7),1) → 2
    let args = [Value::Number(45298.0), Value::Number(1.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(2.0));
}

#[test]
fn jan8_monday_type1_is_week2() {
    // =WEEKNUM(DATE(2024,1,8),1) → 2
    let args = [Value::Number(45299.0), Value::Number(1.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(2.0));
}

#[test]
fn jan1_type2_is_week1() {
    // =WEEKNUM(DATE(2024,1,1),2) → 1
    let args = [Value::Number(45292.0), Value::Number(2.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan7_sunday_type2_is_week1() {
    // =WEEKNUM(DATE(2024,1,7),2) → 1
    let args = [Value::Number(45298.0), Value::Number(2.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan8_monday_type2_is_week2() {
    // =WEEKNUM(DATE(2024,1,8),2) → 2
    let args = [Value::Number(45299.0), Value::Number(2.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(2.0));
}
