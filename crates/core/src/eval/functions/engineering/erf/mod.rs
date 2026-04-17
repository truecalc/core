use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// Error function (Abramowitz & Stegun approximation 7.1.26, max error < 1.5e-7).
pub(crate) fn erf(x: f64) -> f64 {
    if x < 0.0 { return -erf(-x); }
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t * (0.254829592
        + t * (-0.284496736
            + t * (1.421413741
                + t * (-1.453152027
                    + t * 1.061405429))));
    1.0 - poly * (-x * x).exp()
}

pub(crate) fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
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
