use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;

/// `AVEDEV(value1, ...)` — average of absolute deviations from the mean.
/// Ignores text, bool, empty. Returns `#DIV/0!` if no numeric values.
pub fn avedev_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = collect_nums(args);
    let n = nums.len();
    if n == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let avedev = nums.iter().map(|x| (x - mean).abs()).sum::<f64>() / n as f64;
    if !avedev.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(avedev)
}
