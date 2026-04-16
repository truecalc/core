use super::super::networkdays_intl_fn;
use crate::types::Value;

#[test]
fn same_workday_returns_one() {
    // NETWORKDAYS.INTL(DATE(2024,1,1), DATE(2024,1,1), 1) = 1  (Monday)
    let args = [Value::Number(45292.0), Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(1.0));
}

#[test]
fn default_weekend_equals_code_1() {
    // Omitting weekend arg should behave same as code 1
    let with_default = networkdays_intl_fn(&[Value::Number(45292.0), Value::Number(45296.0)]);
    let with_code1   = networkdays_intl_fn(&[Value::Number(45292.0), Value::Number(45296.0), Value::Number(1.0)]);
    assert_eq!(with_default, with_code1);
}

#[test]
fn code_15_thu_only_skipped() {
    // NETWORKDAYS.INTL(DATE(2024,1,1), DATE(2024,1,5), 15) = 4
    // Code 15 = Thu only weekend; Mon-Fri has one Thu → count = 4
    let args = [Value::Number(45292.0), Value::Number(45296.0), Value::Number(15.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(4.0));
}

#[test]
fn backward_negative() {
    // start > end → negative result
    let args = [Value::Number(45296.0), Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(networkdays_intl_fn(&args), Value::Number(-5.0));
}
