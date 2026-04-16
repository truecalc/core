use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < 1e-9 } else { false }
}

#[test]
fn noon_hms() {
    let args = [Value::Text("12:00:00".to_string())];
    assert_eq!(timevalue_fn(&args), Value::Number(0.5));
}

#[test]
fn six_am() {
    let args = [Value::Text("6:00:00".to_string())];
    assert!(approx(timevalue_fn(&args), 6.0 * 3600.0 / 86400.0));
}

#[test]
fn midnight() {
    let args = [Value::Text("00:00:00".to_string())];
    assert_eq!(timevalue_fn(&args), Value::Number(0.0));
}

#[test]
fn fourteen_thirty() {
    let args = [Value::Text("14:30:00".to_string())];
    assert!(approx(timevalue_fn(&args), (14.0 * 3600.0 + 30.0 * 60.0) / 86400.0));
}

#[test]
fn pm_notation() {
    // 2:30 PM = 14:30:00
    let args = [Value::Text("2:30 PM".to_string())];
    assert!(approx(timevalue_fn(&args), (14.0 * 3600.0 + 30.0 * 60.0) / 86400.0));
}
