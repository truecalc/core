use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < 1e-9 } else { false }
}

#[test]
fn noon() {
    let args = [Value::Number(12.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Number(0.5));
}

#[test]
fn midnight() {
    let args = [Value::Number(0.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Number(0.0));
}

#[test]
fn fourteen_thirty() {
    // TIME(14,30,0) = 0.6041666...
    let args = [Value::Number(14.0), Value::Number(30.0), Value::Number(0.0)];
    assert!(approx(time_fn(&args), 14.0 * 3600.0 / 86400.0 + 30.0 * 60.0 / 86400.0));
}

#[test]
fn twenty_three_fifty_nine_fifty_nine() {
    let args = [Value::Number(23.0), Value::Number(59.0), Value::Number(59.0)];
    assert!(approx(time_fn(&args), (23.0 * 3600.0 + 59.0 * 60.0 + 59.0) / 86400.0));
}
