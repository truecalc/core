use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;
use super::var_p::pop_variance;

/// `VARP(value1, ...)` — population variance (same as VAR.P).
pub fn varp_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    pop_variance(&nums)
}

#[cfg(test)]
mod tests;
