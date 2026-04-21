use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Recursively collect numeric values from a Value, flattening arrays.
fn collect_numbers(v: &Value, out: &mut Vec<f64>) {
    match v {
        Value::Array(elems) => {
            for elem in elems {
                collect_numbers(elem, out);
            }
        }
        Value::Number(n) => out.push(*n),
        _ => {}
    }
}

/// Recursively count non-empty values from a Value, flattening arrays.
fn count_nonempty(v: &Value) -> usize {
    match v {
        Value::Array(elems) => elems.iter().map(count_nonempty).sum(),
        Value::Empty => 0,
        _ => 1,
    }
}

/// `SUBTOTAL(function_code, ref1, ...)` — applies an aggregate function.
///
/// function_code 1 (or 101): AVERAGE
/// function_code 2 (or 102): COUNT (numeric values)
/// function_code 3 (or 103): COUNTA (non-empty values)
/// function_code 4 (or 104): MAX
/// function_code 5 (or 105): MIN
/// function_code 9 (or 109): SUM
pub fn subtotal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 255) {
        return err;
    }

    let code = match &args[0] {
        Value::Number(n) => *n as i64,
        _ => return Value::Error(ErrorKind::Value),
    };

    // Normalize: codes 101–111 behave the same as 1–11 for inline arrays.
    let func = code % 100;

    let rest = &args[1..];
    let mut nums: Vec<f64> = Vec::new();
    for arg in rest {
        collect_numbers(arg, &mut nums);
    }

    match func {
        9 => {
            // SUM
            let sum: f64 = nums.iter().sum();
            if !sum.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
            Value::Number(sum)
        }
        1 => {
            // AVERAGE
            if nums.is_empty() {
                return Value::Error(ErrorKind::DivByZero);
            }
            Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
        }
        2 => {
            // COUNT (numeric values only)
            Value::Number(nums.len() as f64)
        }
        3 => {
            // COUNTA (non-empty values)
            let total: usize = rest.iter().map(count_nonempty).sum();
            Value::Number(total as f64)
        }
        4 => {
            // MAX
            match nums.iter().cloned().reduce(f64::max) {
                Some(v) => Value::Number(v),
                None => Value::Error(ErrorKind::Num),
            }
        }
        5 => {
            // MIN
            match nums.iter().cloned().reduce(f64::min) {
                Some(v) => Value::Number(v),
                None => Value::Error(ErrorKind::Num),
            }
        }
        _ => Value::Error(ErrorKind::Value),
    }
}
