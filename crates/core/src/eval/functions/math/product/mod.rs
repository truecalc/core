use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn product_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut product = 1.0_f64;
    for arg in args {
        match to_number(arg.clone()) {
            Err(e) => return e,
            Ok(n) => product *= n,
        }
    }
    if !product.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(product)
}

#[cfg(test)]
mod tests;
