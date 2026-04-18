use crate::eval::functions::check_arity;
use crate::types::Value;

fn count_blanks_in(values: &[Value]) -> usize {
    let mut count = 0;
    for v in values {
        match v {
            Value::Empty => count += 1,
            Value::Text(s) if s.is_empty() => count += 1,
            Value::Array(arr) => count += count_blanks_in(arr),
            _ => {}
        }
    }
    count
}

/// `COUNTBLANK(range)` — counts empty/blank values.
pub fn countblank_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    Value::Number(count_blanks_in(args) as f64)
}

#[cfg(test)]
mod tests;
