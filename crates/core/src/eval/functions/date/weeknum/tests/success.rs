use super::super::weeknum_fn;
use crate::types::Value;

// gs: Google Sheets, Date.xlsx, WEEKNUM sheet
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

// Return types 11-17 and 21
// DATE(2024,1,1)=45292 (Mon), DATE(2024,1,7)=45298 (Sun)

#[test]
fn type_11_week_starts_monday_same_as_2() {
    // Type 11 = Mon start (same as type 2); Jan 7 2024 (Sun) is still in week 1
    let args = [Value::Number(45298.0), Value::Number(11.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_12_week_starts_tuesday() {
    // Jan 1 2024 (Mon): Tue-start means offset=6, week=(0+6)/7+1=1
    let args = [Value::Number(45292.0), Value::Number(12.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_13_week_starts_wednesday() {
    let args = [Value::Number(45292.0), Value::Number(13.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_14_week_starts_thursday() {
    let args = [Value::Number(45292.0), Value::Number(14.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_15_week_starts_friday() {
    let args = [Value::Number(45292.0), Value::Number(15.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_16_week_starts_saturday() {
    let args = [Value::Number(45292.0), Value::Number(16.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_17_week_starts_sunday_same_as_1() {
    // Type 17 = Sun start (same as type 1); Jan 1 2024 (Mon), offset=1, week=(0+1)/7+1=1
    let args = [Value::Number(45292.0), Value::Number(17.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}

#[test]
fn type_21_iso_week_for_jan1_2024() {
    // 2024-01-01 (Mon) is ISO week 1
    let args = [Value::Number(45292.0), Value::Number(21.0)];
    assert_eq!(weeknum_fn(&args), Value::Number(1.0));
}
