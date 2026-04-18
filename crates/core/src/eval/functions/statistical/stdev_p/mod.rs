use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;
use super::var_p::pop_variance;

/// `STDEV.P(value1, ...)` — population standard deviation: sqrt(population variance).
/// Requires n≥1. Returns `#DIV/0!` if no numeric values.
pub fn stdev_p_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    match pop_variance(&nums) {
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
