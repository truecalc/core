use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::{percentile_inc_calc, collect_numbers};

/// `QUARTILE.INC(array, quart)` — inclusive quartile. quart in {0,1,2,3,4}.
pub fn quartile_inc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let quart = match &args[1] {
        Value::Number(n) => {
            let q = *n;
            if !(0.0..=4.0).contains(&q) || q != q.floor() {
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
    Value::Number(percentile_inc_calc(&nums, k))
}

#[cfg(test)]
mod tests;
