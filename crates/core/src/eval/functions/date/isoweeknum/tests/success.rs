use super::super::isoweeknum_fn;
use crate::types::Value;

// Oracle: Google Sheets, Date.xlsx, ISOWEEKNUM sheet
// DATE(2024,1,1)=45292 (Mon), DATE(2024,1,7)=45298 (Sun), DATE(2024,1,8)=45299 (Mon)
// DATE(2024,6,15)=45458 (Sat), DATE(2024,12,28)=45654

#[test]
fn jan1_2024_monday_is_week1() {
    // =ISOWEEKNUM(DATE(2024,1,1)) → 1
    let args = [Value::Number(45292.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan7_2024_sunday_is_week1() {
    // =ISOWEEKNUM(DATE(2024,1,7)) → 1
    let args = [Value::Number(45298.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn jan8_2024_monday_is_week2() {
    // =ISOWEEKNUM(DATE(2024,1,8)) → 2
    let args = [Value::Number(45299.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(2.0));
}

#[test]
fn jun15_2024_is_week24() {
    // =ISOWEEKNUM(DATE(2024,6,15)) → 24
    let args = [Value::Number(45458.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(24.0));
}

#[test]
fn dec28_2024_is_week52() {
    // =ISOWEEKNUM(DATE(2024,12,28)) → 52
    // DATE(2024,12,28) = 45654
    let args = [Value::Number(45654.0)];
    assert_eq!(isoweeknum_fn(&args), Value::Number(52.0));
}
