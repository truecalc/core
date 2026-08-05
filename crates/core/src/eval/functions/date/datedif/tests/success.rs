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

// MD: days remaining after complete months are stripped
#[test]
fn unit_md_partial_month_days() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,1,20), "MD") = 19
    // 45311 = 45292 + 19 = 2024-01-20
    let args = [num(45292.0), num(45311.0), Value::Text("MD".into())];
    assert_eq!(datedif_fn(&args), Value::Number(19.0));
}

#[test]
fn unit_md_zero_when_same_day_of_month() {
    // DATEDIF(DATE(2024,1,15), DATE(2024,2,15), "MD") = 0
    // 45306 = 45292+14 = 2024-01-15, 45337 = 45306+31 = 2024-02-15
    let args = [num(45306.0), num(45337.0), Value::Text("MD".into())];
    assert_eq!(datedif_fn(&args), Value::Number(0.0));
}

// YM: months remaining after complete years
#[test]
fn unit_ym_same_year_months() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,7,1), "YM") = 6
    // 45474 = 45292+182 = 2024-07-01
    let args = [num(45292.0), num(45474.0), Value::Text("YM".into())];
    assert_eq!(datedif_fn(&args), Value::Number(6.0));
}

#[test]
fn unit_ym_strips_full_years() {
    // DATEDIF(DATE(2023,1,1), DATE(2024,7,1), "YM") = 6
    // 44927 = 2023-01-01, 45474 = 2024-07-01
    let args = [num(44927.0), num(45474.0), Value::Text("YM".into())];
    assert_eq!(datedif_fn(&args), Value::Number(6.0));
}

// YD: days from start to end as if in same year
#[test]
fn unit_yd_same_year() {
    // DATEDIF(DATE(2024,1,1), DATE(2024,4,1), "YD") = 91
    // 45383 = 45292+91 = 2024-04-01
    let args = [num(45292.0), num(45383.0), Value::Text("YD".into())];
    assert_eq!(datedif_fn(&args), Value::Number(91.0));
}
