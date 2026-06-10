use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_direct;

/// `MEDIAN(value1, ...)` — middle value of numeric arguments.
/// Direct args: Bool/text coerced; array elements: Numbers only, errors propagate.
pub fn median_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut nums = match collect_nums_direct(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = nums.len() / 2;
    let result = if nums.len().is_multiple_of(2) {
        (nums[mid - 1] + nums[mid]) / 2.0
    } else {
        nums[mid]
    };
    Value::Number(result)
}

#[cfg(test)]
mod tests;
