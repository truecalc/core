use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `TRIMMEAN(data, percent)` — mean of data after trimming percent/2 from each end.
/// `percent` must be in [0, 1). Trims `floor(n * percent / 2)` items from each end.
/// Flattens `Value::Array` in first arg. Ignores Text, Bool, Empty.
/// Returns `#NUM!` if percent < 0 or percent ≥ 1.
pub fn trimmean_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let percent = match &args[1] {
        Value::Number(p) => *p,
        Value::Text(s) => match s.trim().parse::<f64>() {
            Ok(v) if v.is_finite() => v,
            _ => return Value::Error(ErrorKind::Value),
        },
        _ => return Value::Error(ErrorKind::Value),
    };
    if !(0.0..1.0).contains(&percent) {
        return Value::Error(ErrorKind::Num);
    }

    let mut nums: Vec<f64> = Vec::new();
    match &args[0] {
        Value::Number(n) => nums.push(*n),
        Value::Error(e) => return Value::Error(e.clone()),
        Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
        Value::Array(arr) => {
            for v in arr {
                match v {
                    Value::Number(n) => nums.push(*n),
                    Value::Error(e) => return Value::Error(e.clone()),
                    Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
                    _ => {}
                }
            }
        }
        _ => {}
    }

    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }

    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let trim = (nums.len() as f64 * percent / 2.0).floor() as usize;
    let trimmed = &nums[trim..nums.len() - trim];

    if trimmed.is_empty() {
        return Value::Error(ErrorKind::Num);
    }

    let mean = trimmed.iter().sum::<f64>() / trimmed.len() as f64;
    if !mean.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(mean)
}

#[cfg(test)]
mod tests;
