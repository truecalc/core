use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;

/// `VAR.S(value1, ...)` — sample variance.
pub fn var_s_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    sample_variance(&nums)
}

pub(crate) fn sample_variance(nums: &[f64]) -> Value {
    let n = nums.len();
    if n < 2 { return Value::Error(ErrorKind::DivByZero); }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    if !var.is_finite() { Value::Error(ErrorKind::Num) } else { Value::Number(var) }
}

#[cfg(test)]
mod tests;
