use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `MULTINOMIAL(value1, value2, ...)` — (sum of args)! / product of factorials of each arg.
pub fn multinomial_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut values: Vec<u64> = Vec::with_capacity(args.len());
    for arg in args {
        let n = match to_number(arg.clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        if n < 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        values.push(n.trunc() as u64);
    }
    let sum: u64 = values.iter().sum();
    // Compute sum! / product(i!) using the identity:
    // MULTINOMIAL = C(sum, v1) * C(sum-v1, v2) * ...
    let mut result = 1.0f64;
    let mut remaining = sum;
    for &v in &values {
        result *= combinations(remaining, v);
        remaining -= v;
    }
    Value::Number(result)
}

fn combinations(n: u64, k: u64) -> f64 {
    if k == 0 || k == n {
        return 1.0;
    }
    let k = k.min(n - k);
    let mut result = 1.0f64;
    for i in 0..k {
        result *= (n - i) as f64;
        result /= (i + 1) as f64;
    }
    result
}

#[cfg(test)]
mod tests;
