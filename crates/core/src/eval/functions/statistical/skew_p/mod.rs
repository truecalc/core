use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;

/// `SKEW.P(value1, ...)` — population skewness.
pub fn skew_p_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    let n = nums.len();
    if n == 0 { return Value::Error(ErrorKind::NA); }
    if n < 3 { return Value::Error(ErrorKind::DivByZero); }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let pop_variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let sigma = libm::sqrt(pop_variance);
    if sigma == 0.0 { return Value::Error(ErrorKind::DivByZero); }
    let nf = n as f64;
    let sum3 = nums.iter().map(|&x| ((x - mean) / sigma).powi(3)).sum::<f64>();
    let result = sum3 / nf;
    if result.is_finite() { Value::Number(result) } else { Value::Error(ErrorKind::Num) }
}

#[cfg(test)]
mod tests;
