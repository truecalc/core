use crate::types::{ErrorKind, Value};

fn collect_nums_into(args: &[Value], out: &mut Vec<f64>) {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Array(inner) => collect_nums_into(inner, out),
            _ => {}
        }
    }
}

/// `MEDIAN(value1, ...)` — middle value of numeric arguments.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty, Error.
/// Even count: average of two middle values. Odd count: middle value.
/// Returns `Value::Error(ErrorKind::NA)` if no args, `#NUM!` if no numeric values.
pub fn median_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut nums: Vec<f64> = Vec::new();
    collect_nums_into(args, &mut nums);
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
