use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `PERCENTILE.INC(array, k)` — k-th percentile (inclusive), k in [0,1].
/// Interpolates linearly between adjacent sorted values.
pub fn percentile_inc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let k = match &args[1] {
        Value::Number(n) => *n,
        _ => return Value::Error(ErrorKind::Num),
    };
    if !(0.0..=1.0).contains(&k) {
        return Value::Error(ErrorKind::Num);
    }
    let mut nums = match collect_numbers_checked(&args[0]) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Value::Number(percentile_inc_calc(&nums, k))
}

/// Calculate inclusive percentile for a sorted slice.
pub fn percentile_inc_calc(sorted: &[f64], k: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let idx = k * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = idx - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

pub fn collect_numbers(v: &Value) -> Vec<f64> {
    match v {
        Value::Array(arr) => arr.iter().filter_map(|x| {
            if let Value::Number(n) = x { Some(*n) } else { None }
        }).collect(),
        Value::Number(n) => vec![*n],
        _ => vec![],
    }
}

pub fn collect_numbers_checked(v: &Value) -> Result<Vec<f64>, Value> {
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
