use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::{percentile_inc_calc, collect_numbers};

/// `PERCENTILE(array, k)` — k-th percentile (inclusive, same as PERCENTILE.INC).
pub fn percentile_fn(args: &[Value]) -> Value {
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
    let mut nums = collect_numbers(&args[0]);
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Value::Number(percentile_inc_calc(&nums, k))
}

#[cfg(test)]
mod tests;
