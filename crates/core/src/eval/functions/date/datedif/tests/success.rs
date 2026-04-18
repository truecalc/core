use super::super::*;
use crate::types::Value;

fn num(serial: f64) -> Value {
    Value::Number(serial)
}

// Serials precomputed from epoch 1899-12-30:
//   DATE(2020,1,1) = 43831
//   DATE(2024,1,1) = 45292
//   DATE(2024,6,15) = 45458
//   DATE(2024,1,5) = 45296
//   DATE(2024,12,31) = 45657

#[test]
fn y_2020_to_2024_jun15() {
    // DATEDIF(DATE(2020,1,1), DATE(2024,6,15), "Y") = 4
    let args = [num(43831.0), num(45458.0), Value::Text("Y".into())];
    assert_eq!(datedif_fn(&args), Value::Number(4.0));
}

#[test]
fn m_2020_to_2024_jun15() {
    // DATEDIF(DATE(2020,1,1), DATE(2024,6,15), "M") = 53
    let args = [num(43831.0), num(45458.0), Value::Text("M".into())];
    assert_eq!(datedif_fn(&args), Value::Number(53.0));
}

#[test]
fn d_jan_to_jun_2024() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,6,15), "D") = 166
    let args = [num(45292.0), num(45458.0), Value::Text("D".into())];
    assert_eq!(datedif_fn(&args), Value::Number(166.0));
}

#[test]
fn md_days_ignoring_month_year() {
    // DATEDIF(DATE(2024,1,5), DATE(2024,6,15), "MD") = 10
    let args = [num(45296.0), num(45458.0), Value::Text("MD".into())];
    assert_eq!(datedif_fn(&args), Value::Number(10.0));
}

#[test]
fn ym_months_ignoring_year() {
    // DATEDIF(DATE(2020,1,1), DATE(2024,6,15), "YM") = 5
    let args = [num(43831.0), num(45458.0), Value::Text("YM".into())];
    assert_eq!(datedif_fn(&args), Value::Number(5.0));
}

#[test]
fn yd_days_ignoring_year() {
    // DATEDIF(DATE(2020,1,1), DATE(2024,6,15), "YD") = 166
    let args = [num(43831.0), num(45458.0), Value::Text("YD".into())];
    assert_eq!(datedif_fn(&args), Value::Number(166.0));
}

#[test]
fn y_within_same_year() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,12,31), "Y") = 0
    let args = [num(45292.0), num(45657.0), Value::Text("Y".into())];
    assert_eq!(datedif_fn(&args), Value::Number(0.0));
}

#[test]
fn unit_is_case_insensitive() {
    let args = [num(43831.0), num(45458.0), Value::Text("y".into())];
    assert_eq!(datedif_fn(&args), Value::Number(4.0));
}
