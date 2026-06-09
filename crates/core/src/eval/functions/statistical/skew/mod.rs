use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;

/// `SKEW(value1, ...)` — Fisher's sample skewness.
pub fn skew_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    let n = nums.len();
    if n == 0 { return Value::Error(ErrorKind::NA); }
    if n < 3 { return Value::Error(ErrorKind::DivByZero); }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let s = variance.sqrt();
    if s == 0.0 { return Value::Error(ErrorKind::DivByZero); }
    let nf = n as f64;
    let sum3 = nums.iter().map(|&x| ((x - mean) / s).powi(3)).sum::<f64>();
    let result = (nf / ((nf - 1.0) * (nf - 2.0))) * sum3;
    if result.is_finite() { Value::Number(result) } else { Value::Error(ErrorKind::Num) }
}

#[cfg(test)]
mod tests;
