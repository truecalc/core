use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;
use super::var_s::sample_variance;

/// `STDEV(value1, ...)` — sample standard deviation (same as STDEV.S).
pub fn stdev_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    match sample_variance(&nums) {
        Value::Number(v) => {
            let s = v.sqrt();
            if !s.is_finite() {
                Value::Error(ErrorKind::Num)
            } else {
                Value::Number(s)
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests;
