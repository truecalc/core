use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;

/// `DEVSQ(value1, ...)` — sum of squared deviations from the mean.
/// Ignores text, bool, empty. Returns `#NUM!` if no numeric values.
pub fn devsq_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    let n = nums.len();
    if n == 0 {
        return Value::Error(ErrorKind::Num);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let devsq = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>();
    if !devsq.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(devsq)
}
