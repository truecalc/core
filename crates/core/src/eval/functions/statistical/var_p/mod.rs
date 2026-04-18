use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;

/// `VAR.P(value1, ...)` — population variance: Σ(x-mean)²/n.
/// Requires n≥1. Returns `#DIV/0!` if no numeric values.
pub fn var_p_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    pop_variance(&nums)
}

/// Compute population variance from a slice of f64 values.
pub(crate) fn pop_variance(nums: &[f64]) -> Value {
    let n = nums.len();
    if n == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    if !var.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(var)
}

#[cfg(test)]
mod tests;
