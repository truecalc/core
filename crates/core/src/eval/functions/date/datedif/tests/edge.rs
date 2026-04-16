use super::super::*;
use crate::types::Value;

fn num(serial: f64) -> Value {
    Value::Number(serial)
}

// Serials:
//   DATE(2024,6,14) = 45457
//   DATE(2024,6,15) = 45458
//   DATE(2024,2,1)  = 45323
//   DATE(2024,3,1)  = 45352
//   DATE(2023,3,1)  = 44986
//   DATE(2025,3,1)  = 45717
//   DATE(2024,1,1)  = 45292
//   DATE(2024,1,31) = 45322

#[test]
fn d_same_day_is_zero() {
    // DATEDIF(DATE(2024,6,15), DATE(2024,6,15), "D") = 0
    let args = [num(45458.0), num(45458.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Number(0.0));
}

#[test]
fn d_one_day_apart() {
    // DATEDIF(DATE(2024,6,14), DATE(2024,6,15), "D") = 1
    let args = [num(45457.0), num(45458.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Number(1.0));
}

#[test]
fn d_across_leap_day_feb_2024() {
    // DATEDIF(DATE(2024,2,1), DATE(2024,3,1), "D") = 29
    let args = [num(45323.0), num(45352.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Number(29.0));
}

#[test]
fn y_crossing_leap_year() {
    // DATEDIF(DATE(2023,3,1), DATE(2025,3,1), "Y") = 2
    let args = [num(44986.0), num(45717.0), Value::Text("Y".into())];
    assert_eq!(datedif_fn(&args), Value::Number(2.0));
}

#[test]
fn d_30_day_span() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,1,31), "D") = 30
    let args = [num(45292.0), num(45322.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Number(30.0));
}
