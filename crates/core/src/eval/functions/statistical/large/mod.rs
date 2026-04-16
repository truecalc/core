use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `LARGE(array, k)` — k-th largest value in the data set.
/// k is 1-based. Ignores Text, Bool, Empty. Error if k < 1 or k > n: `#NUM!`.
pub fn large_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let mut nums = collect_numbers(&args[0]);
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    let k = match &args[1] {
        Value::Number(n) => {
            let k = *n;
            if k < 1.0 || k != k.floor() {
                return Value::Error(ErrorKind::Num);
            }
            k as usize
        }
        _ => return Value::Error(ErrorKind::Num),
    };
    if k > nums.len() {
        return Value::Error(ErrorKind::Num);
    }
    // Sort descending, return index k-1
    nums.sort_by(|a, b| b.partial_cmp(a).unwrap());
    Value::Number(nums[k - 1])
}

fn collect_numbers(v: &Value) -> Vec<f64> {
    match v {
        Value::Array(arr) => arr.iter().filter_map(|x| {
            if let Value::Number(n) = x { Some(*n) } else { None }
        }).collect(),
        Value::Number(n) => vec![*n],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests;
