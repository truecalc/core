use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_a;
use super::var_p::pop_variance;

/// `VARPA(value1, ...)` — population variance, text/FALSE=0, TRUE=1.
pub fn varpa_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    if args.iter().any(|a| matches!(a, Value::Text(_))) {
        return Value::Error(ErrorKind::Value);
    }
    let nums = collect_nums_a(args);
    pop_variance(&nums)
}

#[cfg(test)]
mod tests;
