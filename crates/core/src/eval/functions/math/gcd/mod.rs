use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn collect_gcd_values(v: &Value, out: &mut Vec<Value>) {
    match v {
        Value::Array(elems) => {
            for elem in elems { collect_gcd_values(elem, out); }
        }
        Value::Text(s) if s.is_empty() => out.push(Value::Error(ErrorKind::Value)),
        other => out.push(other.clone()),
    }
}

/// `GCD(value1, value2, ...)` — greatest common divisor.
/// Array args flattened; empty string => #VALUE!.
pub fn gcd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut nums: Vec<Value> = Vec::new();
    for arg in args { collect_gcd_values(arg, &mut nums); }
    let mut result: u64 = 0;
    for v in &nums {
        if let Value::Error(_) = v { return v.clone(); }
        let n = match to_number(v.clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        if n < 0.0 { return Value::Error(ErrorKind::Num); }
        result = gcd(result, n.trunc() as u64);
    }
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
