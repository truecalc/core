use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_a;
use super::var_s::sample_variance;

/// `VARA(value1, ...)` — sample variance, text/FALSE=0, TRUE=1.
pub fn vara_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    if args.iter().any(|a| matches!(a, Value::Text(_))) {
        return Value::Error(ErrorKind::Value);
    }
    let nums = collect_nums_a(args);
    sample_variance(&nums)
}

#[cfg(test)]
mod tests;
