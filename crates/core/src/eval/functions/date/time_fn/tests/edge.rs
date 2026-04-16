use super::super::*;
use crate::types::Value;

fn approx(a: Value, b: f64) -> bool {
    if let Value::Number(n) = a { (n - b).abs() < 1e-9 } else { false }
}

#[test]
fn second_overflow_wraps() {
    // TIME(0,0,60) = 1 minute = 60/86400
    let args = [Value::Number(0.0), Value::Number(0.0), Value::Number(60.0)];
    assert!(approx(time_fn(&args), 60.0 / 86400.0));
}

#[test]
fn minute_overflow_wraps() {
    // TIME(0,60,0) = 1 hour = 3600/86400
    let args = [Value::Number(0.0), Value::Number(60.0), Value::Number(0.0)];
    assert!(approx(time_fn(&args), 3600.0 / 86400.0));
}

#[test]
fn hour_24_wraps_to_zero() {
    // TIME(24,0,0) = 0 (wraps)
    let args = [Value::Number(24.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Number(0.0));
}

#[test]
fn decimal_hour_truncated() {
    // TIME(14.9,0,0) = TIME(14,0,0)
    let args_frac = [Value::Number(14.9), Value::Number(0.0), Value::Number(0.0)];
    let args_int  = [Value::Number(14.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args_frac), time_fn(&args_int));
}
