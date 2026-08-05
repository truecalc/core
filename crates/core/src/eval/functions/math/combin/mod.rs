use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// Helper: compute C(n, k) = n! / (k! * (n-k)!) for non-negative integers.
fn combinations(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k); // take smaller k for efficiency
    let mut result = 1.0f64;
    for i in 0..k {
        result *= (n - i) as f64;
        result /= (i + 1) as f64;
    }
    result
}

/// `COMBIN(n, k)` — number of combinations (without repetition): C(n,k) = n! / (k! * (n-k)!)
pub fn combin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let k = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n < 0.0 || k < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n_int = n.trunc() as u64;
    let k_int = k.trunc() as u64;
    if k_int > n_int {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(combinations(n_int, k_int))
}

#[cfg(test)]
mod tests;
