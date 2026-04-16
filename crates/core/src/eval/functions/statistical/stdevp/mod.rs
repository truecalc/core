use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;
use super::var_p::pop_variance;

/// `STDEVP(value1, ...)` — population standard deviation (same as STDEV.P).
pub fn stdevp_fn(args: &[Value]) -> Value {
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
