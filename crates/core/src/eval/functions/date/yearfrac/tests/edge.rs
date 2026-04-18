use super::super::yearfrac_fn;
use crate::types::Value;

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

// DATE(2024,1,1)=45292, DATE(2024,7,1)=45474, DATE(2024,6,15)=45458
// DATE(2020,1,1)=43831, DATE(2021,1,1)=44197

#[test]
fn basis0_half_year_is_0_5() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2024,7,1),0) → 0.5
    // US 30/360: (2024-2024)*360 + (7-1)*30 + (1-1) = 180, /360 = 0.5
    let args = [Value::Number(45292.0), Value::Number(45474.0), Value::Number(0.0)];
    assert!(approx(yearfrac_fn(&args), 0.5, 1e-9));
}

#[test]
fn basis0_same_day_is_zero() {
    // =YEARFRAC(DATE(2024,6,15),DATE(2024,6,15),0) → 0
    let args = [Value::Number(45458.0), Value::Number(45458.0), Value::Number(0.0)];
    assert!(approx(yearfrac_fn(&args), 0.0, 1e-9));
}

#[test]
fn basis1_leap_year_2020_to_2021() {
    // DATE(2020,1,1)=43831, DATE(2021,1,1)=44197
    // 2020 is a leap year (366 days). 366/366 = 1.0
    let args = [Value::Number(43831.0), Value::Number(44197.0), Value::Number(1.0)];
    assert!(approx(yearfrac_fn(&args), 1.0, 1e-9));
}

#[test]
fn basis1_non_leap_year_2023_to_2024() {
    // DATE(2023,1,1)=44927, DATE(2024,1,1)=45292
    // 2023 is not a leap year (365 days). 365/365 = 1.0
    let args = [Value::Number(44927.0), Value::Number(45292.0), Value::Number(1.0)];
    assert!(approx(yearfrac_fn(&args), 1.0, 1e-9));
}

#[test]
fn basis1_partial_leap_year() {
    // DATE(2024,1,1)=45292 to DATE(2024,7,1)=45474
    // 2024 is a leap year, 182 actual days. 182/366 ≈ 0.4973
    let args = [Value::Number(45292.0), Value::Number(45474.0), Value::Number(1.0)];
    assert!(approx(yearfrac_fn(&args), 182.0 / 366.0, 1e-9));
}

#[test]
fn basis1_same_day_is_zero() {
    let args = [Value::Number(45458.0), Value::Number(45458.0), Value::Number(1.0)];
    assert!(approx(yearfrac_fn(&args), 0.0, 1e-9));
}
