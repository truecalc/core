use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;
use super::var_p::pop_variance;

/// `STDEV.P(value1, ...)` — population standard deviation.
pub fn stdev_p_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    match pop_variance(&nums) {
        Value::Number(v) => {
            let s = libm::sqrt(v);
            if s.is_finite() { Value::Number(s) } else { Value::Error(ErrorKind::Num) }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests;
