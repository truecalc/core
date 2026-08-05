use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::{percentile_inc_calc, collect_numbers_checked};

/// `QUARTILE(array, quart)` — quartile (same as QUARTILE.INC). quart in {0,1,2,3,4}.
pub fn quartile_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let quart = match &args[1] {
        Value::Number(n) => {
            let q = n.trunc();
            if !(0.0..=4.0).contains(&q) {
                return Value::Error(ErrorKind::Num);
            }
            q as u8
        }
        Value::Bool(b) => if *b { 1u8 } else { 0u8 },
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
    Value::Number(percentile_inc_calc(&nums, k))
}

#[cfg(test)]
mod tests;
