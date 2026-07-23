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
        Value::Bool(true) => 1usize,
        Value::Bool(false) => return Value::Error(ErrorKind::Num),
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
            collect_numbers_checked_into(arr, &mut nums)?;
            Ok(nums)
        }
        Value::Number(n) => Ok(vec![*n]),
        Value::Error(e) => Err(Value::Error(e.clone())),
        Value::ErrorMsg(e, m) => Err(Value::ErrorMsg(e.clone(), m.clone())),
        _ => Ok(vec![]),
    }
}

/// Recurse into nested arrays (e.g. a vertical range materializes as nested
/// one-element row arrays) so every cell is visited.
fn collect_numbers_checked_into(arr: &[Value], out: &mut Vec<f64>) -> Result<(), Value> {
    for x in arr {
        match x {
            Value::Number(n) => out.push(*n),
            Value::Array(inner) => collect_numbers_checked_into(inner, out)?,
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
