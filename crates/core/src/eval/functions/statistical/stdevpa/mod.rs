use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_a;
use super::var_p::pop_variance;

/// `STDEVPA(value1, ...)` — population standard deviation, text/FALSE=0, TRUE=1.
pub fn stdevpa_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    if args.iter().any(|a| matches!(a, Value::Text(_))) {
        return Value::Error(ErrorKind::Value);
    }
    let nums = collect_nums_a(args);
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
