use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Complementary error function — high precision (< 1e-4 relative error).
/// Uses Taylor series for |x| < 1, continued fraction for |x| >= 1.
pub(crate) fn erfc(x: f64) -> f64 {
    if x < 0.0 { return 2.0 - erfc(-x); }
    if x == 0.0 { return 1.0; }
    if x > 26.0 { return 0.0; }
    if x < 1.0 {
        // erf via Maclaurin series (12 terms, accurate to ~1e-15 for |x|<1)
        let x2 = x * x;
        let mut total = x;
        let mut term = x;
        for n in 1u32..13 {
            term *= -x2 * (2 * n - 1) as f64 / (n as f64 * (2 * n + 1) as f64);
            total += term;
        }
        return 1.0 - (2.0 / std::f64::consts::PI.sqrt()) * total;
    }
    // Continued fraction for x >= 1: erfc(x) = exp(-x^2)/sqrt(pi) * 1/CF
    // Evaluate from the tail inward (50 terms)
    let mut cf = 0.0f64;
    for k in (1u32..=50).rev() {
        cf = (k as f64 * 0.5) / (x + cf);
    }
    (-x * x).exp() / (std::f64::consts::PI.sqrt() * (x + cf))
}

/// Error function: erf(x) = 1 - erfc(x).
pub(crate) fn erf(x: f64) -> f64 {
    if x < 0.0 { return -erf(-x); }
    1.0 - erfc(x)
}

pub fn erf_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) { return err; }
    let lower = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    if args.len() == 2 {
        let upper = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
        Value::Number(erf(upper) - erf(lower))
    } else {
        Value::Number(erf(lower))
    }
}

pub fn erf_precise_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    let x = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    Value::Number(erf(x))
}

pub fn erfc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    let x = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    Value::Number(erfc(x))
}

pub fn erfc_precise_fn(args: &[Value]) -> Value {
    erfc_fn(args)
}

#[cfg(test)]
mod tests;
