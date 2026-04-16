use super::super::yearfrac_fn;
use crate::types::Value;

// Oracle: Google Sheets, Date.xlsx, YEARFRAC sheet
// DATE(2024,1,1)=45292, DATE(2025,1,1)=45658

fn approx(a: Value, b: f64, tol: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < tol } else { false }
}

#[test]
fn basis0_full_year_is_1() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1),0) → 1
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(0.0)];
    assert!(approx(yearfrac_fn(&args), 1.0, 1e-9));
}

#[test]
fn basis1_full_year_is_1() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1),1) → 1
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(1.0)];
    assert!(approx(yearfrac_fn(&args), 1.0, 1e-9));
}

#[test]
fn basis2_full_year_2024() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1),2) → 1.016666667
    // 2024 is a leap year: 366 actual days / 360
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(2.0)];
    assert!(approx(yearfrac_fn(&args), 366.0 / 360.0, 1e-7));
}

#[test]
fn basis3_full_year_2024() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1),3) → 1.002739726
    // 366 actual days / 365
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(3.0)];
    assert!(approx(yearfrac_fn(&args), 366.0 / 365.0, 1e-7));
}

#[test]
fn basis4_full_year_is_1() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1),4) → 1
    let args = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(4.0)];
    assert!(approx(yearfrac_fn(&args), 1.0, 1e-9));
}

#[test]
fn default_basis_omitted_equals_basis0() {
    // =YEARFRAC(DATE(2024,1,1),DATE(2025,1,1)) → 1
    let with_basis = [Value::Number(45292.0), Value::Number(45658.0), Value::Number(0.0)];
    let without_basis = [Value::Number(45292.0), Value::Number(45658.0)];
    assert_eq!(yearfrac_fn(&with_basis), yearfrac_fn(&without_basis));
}
