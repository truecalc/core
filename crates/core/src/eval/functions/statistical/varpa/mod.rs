use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_a_direct;
use super::var_p::pop_variance;

/// `VARPA(value1, ...)` - population variance including text (→0 in arrays) and bool (→1/0).
/// Direct non-parseable text → #VALUE!.
pub fn varpa_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_a_direct(args) { Ok(v) => v, Err(e) => return e };
    pop_variance(&nums)
}

#[cfg(test)]
mod tests;
