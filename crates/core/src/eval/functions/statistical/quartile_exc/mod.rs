use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers_checked;
use super::percentile_exc::percentile_exc_calc;

/// `QUARTILE.EXC(array, quart)` — exclusive quartile. quart in {1,2,3} only.
pub fn quartile_exc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let quart = match &args[1] {
        Value::Number(n) => {
            let q = n.trunc();
            if !(1.0..=3.0).contains(&q) {
                return Value::Error(ErrorKind::Num);
            }
            q as u8
        }
        Value::Bool(b) => {
            // TRUE→1, FALSE→0 — 0 is out of range for EXC
            if !*b { return Value::Error(ErrorKind::Num); }
            1u8
        }
        _ => return Value::Error(ErrorKind::Num),
    };
    let mut nums = match collect_numbers_checked(&args[0]) {
        Ok(v) => v,
        Err(e) => return e,
    };
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
