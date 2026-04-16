use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn average_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut sum = 0.0_f64;
    let mut count = 0usize;
    for arg in args {
        let scalars: Vec<&Value> = match arg {
            Value::Array(elems) => elems.iter().collect(),
            other => vec![other],
        };
        for scalar in scalars {
            match to_number(scalar.clone()) {
                Err(e) => return e,
                Ok(n) => {
                    sum += n;
                    count += 1;
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
