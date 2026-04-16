use super::super::workday_fn;
use crate::types::Value;

// Serials: DATE(2024,1,1)=45292 Mon, DATE(2024,1,5)=45296 Fri
// Oracle: WORKDAY(DATE(2024,1,1),5)=45299, WORKDAY(DATE(2024,1,1),1)=45293
//         WORKDAY(DATE(2024,1,5),1)=45299, WORKDAY(DATE(2024,1,1),-1)=45289
//         WORKDAY(DATE(2024,1,5),0)=45296

#[test]
fn mon_plus_5() {
    // WORKDAY(DATE(2024,1,1), 5) = 45299  (Jan 8, next Monday)
    let args = [Value::Number(45292.0), Value::Number(5.0)];
    assert_eq!(workday_fn(&args), Value::Number(45299.0));
}

#[test]
fn mon_plus_1() {
    // WORKDAY(DATE(2024,1,1), 1) = 45293  (Jan 2, Tuesday)
    let args = [Value::Number(45292.0), Value::Number(1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45293.0));
}

#[test]
fn fri_plus_1_skips_weekend() {
    // WORKDAY(DATE(2024,1,5), 1) = 45299  (Jan 8, Monday)
    let args = [Value::Number(45296.0), Value::Number(1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45299.0));
}

#[test]
fn mon_minus_1_prior_fri() {
    // WORKDAY(DATE(2024,1,1), -1) = 45289  (Dec 29, 2023 Friday)
    let args = [Value::Number(45292.0), Value::Number(-1.0)];
    assert_eq!(workday_fn(&args), Value::Number(45289.0));
}
