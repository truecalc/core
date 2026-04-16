use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers;
use super::percentile_exc::percentile_exc_calc;

/// `QUARTILE.EXC(array, quart)` — exclusive quartile. quart in {1,2,3} only.
pub fn quartile_exc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let quart = match &args[1] {
        Value::Number(n) => {
            let q = *n;
            if !(1.0..=3.0).contains(&q) || q != q.floor() {
                return Value::Error(ErrorKind::Num);
            }
            q as u8
        }
        _ => return Value::Error(ErrorKind::Num),
    };
    let mut nums = collect_numbers(&args[0]);
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let k = quart as f64 / 4.0;
    match percentile_exc_calc(&nums, k) {
        Some(v) => Value::Number(v),
        None => Value::Error(ErrorKind::Num),
    }
}

#[cfg(test)]
mod tests;
