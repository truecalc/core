use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn average_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut sum = 0.0_f64;
    let mut count = 0usize;
    for arg in args {
        match arg {
            Value::Number(n) => { sum += n; count += 1; }
            Value::Date(n)   => { sum += n; count += 1; }
            Value::Bool(b)   => { sum += if *b { 1.0 } else { 0.0 }; count += 1; }
            Value::Text(s) => {
                // Direct text: empty or non-parseable -> #VALUE!
                let t = s.trim();
                if t.is_empty() {
                    return Value::Error(ErrorKind::Value);
                }
                match t.parse::<f64>() {
                    Ok(v) if v.is_finite() => { sum += v; count += 1; }
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Empty => {} // skip
            Value::Zoned(_) => return Value::Error(ErrorKind::Value),
            Value::Error(_) | Value::ErrorMsg(_, _) => return arg.clone(),
            Value::Array(elems) => {
                // Array context: skip bool/text, include numbers
                for elem in elems {
                    match elem {
                        Value::Number(n) => { sum += n; count += 1; }
                        Value::Date(n)   => { sum += n; count += 1; }
                        Value::Error(_) | Value::ErrorMsg(_, _) => return elem.clone(),
                        _ => {} // Bool, Text, Empty all skipped in array
                    }
                }
            }
        }
    }
    if count == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let result = sum / count as f64;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
