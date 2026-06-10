use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;

/// `DEVSQ(value1, ...)` — sum of squared deviations from the mean.
/// Array text/Bool → skip. Direct Bool/text → coerce. Empty set → 0.0.
pub fn devsq_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let nums = match collect_nums_direct(args) { Ok(v) => v, Err(e) => return e };
    let n = nums.len();
    if n == 0 { return Value::Number(0.0); }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let devsq = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    if devsq.is_finite() { Value::Number(devsq) } else { Value::Error(ErrorKind::Num) }
}

#[cfg(test)]
mod tests;
