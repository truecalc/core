use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn pi_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 0, 0) {
        return err;
    }
    Value::Number(std::f64::consts::PI)
}

pub fn sin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.sin();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn cos_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.cos();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn tan_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.tan();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// Issue #56 — Inverse trig + angle conversion

pub fn acos_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.acos();
    if result.is_nan() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn asin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.asin();
    if result.is_nan() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn atan_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(n.atan())
}

pub fn atan2_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    // Excel ATAN2(x, y) — first arg is x, second is y (opposite of math convention)
    let n_x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let n_y = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n_x == 0.0 && n_y == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(n_y.atan2(n_x))
}

pub fn degrees_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(n * 180.0 / std::f64::consts::PI)
}

pub fn radians_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(n * std::f64::consts::PI / 180.0)
}

// Issue #57 — Hyperbolic trig

pub fn sinh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.sinh();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn cosh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.cosh();
    // GS/Excel max representable value ≈ 9.99e307; anything larger → #NUM!
    if !result.is_finite() || result > 9.99e307 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn tanh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(n.tanh())
}

pub fn acosh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.acosh();
    if result.is_nan() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn asinh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(n.asinh())
}

pub fn atanh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let result = n.atanh();
    if !result.is_finite() || result.is_nan() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// Issue #58 — Reciprocal/cotangent trig

pub fn cot_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let tan = n.tan();
    if tan == 0.0 || !tan.is_finite() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(1.0 / tan)
}

pub fn coth_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let tanh = n.tanh();
    if tanh == 0.0 || !tanh.is_finite() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(1.0 / tanh)
}

pub fn csc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let sin = n.sin();
    if sin == 0.0 || !sin.is_finite() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(1.0 / sin)
}

pub fn csch_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let sinh = n.sinh();
    if sinh == 0.0 || !sinh.is_finite() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(1.0 / sinh)
}

pub fn sec_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let cos = n.cos();
    if cos == 0.0 || !cos.is_finite() {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(1.0 / cos)
}

pub fn sech_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    // cosh is always >= 1, so no div-by-zero possible
    let cosh = n.cosh();
    if !cosh.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(1.0 / cosh)
}

pub fn acot_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    // ACOT(x) = atan(1/x); matches Google Sheets behavior
    Value::Number((1.0 / n).atan())
}

pub fn acoth_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    // ACOTH(x) = atanh(1/x); domain requires |x| > 1
    if n.abs() <= 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = (1.0 / n).atanh();
    if !result.is_finite() || result.is_nan() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
