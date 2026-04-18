use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;
use super::var_s::sample_variance;

/// `VAR(value1, ...)` — sample variance (same as VAR.S).
pub fn var_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    sample_variance(&nums)
}

#[cfg(test)]
mod tests;
