use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SMALL(array, k)` — k-th smallest value in the data set.
/// k is 1-based. Ignores Text, Bool, Empty. Error if k < 1 or k > n: `#NUM!`.
pub fn small_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let nums = match collect_numbers_checked(&args[0]) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    let k = match &args[1] {
        Value::Number(n) => {
            let k = n.trunc();
            if k < 1.0 {
                return Value::Error(ErrorKind::Num);
            }
            k as usize
        }
        Value::Bool(b) => if *b { 1usize } else { return Value::Error(ErrorKind::Num); },
        _ => return Value::Error(ErrorKind::Num),
    };
    let mut nums = nums;
    if k > nums.len() {
        return Value::Error(ErrorKind::Num);
    }
    // Sort ascending, return index k-1
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Value::Number(nums[k - 1])
}

fn collect_numbers_checked(v: &Value) -> Result<Vec<f64>, Value> {
    match v {
        Value::Array(arr) => {
            let mut nums = Vec::new();
            for x in arr {
                match x {
                    Value::Number(n) => nums.push(*n),
                    Value::Error(e) => return Err(Value::Error(e.clone())),
                    _ => {}
                }
            }
            Ok(nums)
        }
        Value::Number(n) => Ok(vec![*n]),
        Value::Error(e) => Err(Value::Error(e.clone())),
        _ => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests;
