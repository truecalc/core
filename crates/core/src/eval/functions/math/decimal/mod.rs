use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `DECIMAL(text, base)` — converts a string representation of a number in the given base (2–36)
/// to a decimal number. Case-insensitive.
pub fn decimal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let text = match &args[0] {
        Value::Text(s) => s.clone(),
        Value::Number(n) => (n.trunc() as i64).to_string(),
        other => match to_number(other.clone()) {
            Ok(n) => (n.trunc() as i64).to_string(),
            Err(e) => return e,
        },
    };

    let base = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let base_int = base.trunc() as i64;
    if !(2..=36).contains(&base_int) {
        return Value::Error(ErrorKind::Num);
    }
    let base_u32 = base_int as u32;

    let upper = text.to_uppercase();
    let mut result: u64 = 0;
    for ch in upper.chars() {
        let digit = match ch {
            '0'..='9' => (ch as u32) - ('0' as u32),
            'A'..='Z' => (ch as u32) - ('A' as u32) + 10,
            _ => return Value::Error(ErrorKind::Num),
        };
        if digit >= base_u32 {
            return Value::Error(ErrorKind::Num);
        }
        result = result * base_u32 as u64 + digit as u64;
    }

    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
