use crate::types::{ErrorKind, Value};

/// `MEDIAN(value1, ...)` — middle value of numeric arguments.
/// Ignores Text, Bool, Empty, Error. Even count: average of two middle values. Odd count: middle value.
/// Returns `Value::Error(ErrorKind::Num)` if no numeric values are present.
pub fn median_fn(args: &[Value]) -> Value {
    let mut nums: Vec<f64> = Vec::new();
    for arg in args {
        if let Value::Number(n) = arg {
            nums.push(*n);
        }
    }
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
