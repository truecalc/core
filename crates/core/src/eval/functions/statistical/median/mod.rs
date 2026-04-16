use crate::types::{ErrorKind, Value};

/// `MEDIAN(value1, ...)` — middle value of numeric arguments.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty, Error.
/// Even count: average of two middle values. Odd count: middle value.
/// Returns `Value::Error(ErrorKind::NA)` if no args; `ErrorKind::Num` if no numeric values.
pub fn median_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut nums: Vec<f64> = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Array(arr) => {
                for v in arr {
                    if let Value::Number(n) = v {
                        nums.push(*n);
                    }
                }
            }
            _ => {}
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
