use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums_a_direct;

/// `AVERAGEA(value1, ...)` — average including booleans and text.
/// - Numbers/Dates: included at face value.
/// - Booleans: TRUE=1, FALSE=0 (always, direct and array).
/// - Direct text that parses as number: coerced to that number.
/// - Direct text that does NOT parse: treated as 0 (counted).
/// - Array text: treated as 0 (counted).
/// - Empty: skipped.
/// - No args → `#N/A`. No numeric values → `#DIV/0!`.
pub fn averagea_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let nums = match collect_nums_a_direct(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if nums.is_empty() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
}

#[cfg(test)]
mod tests;
