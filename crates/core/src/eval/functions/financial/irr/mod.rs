use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `IRR(values, [guess])` — internal rate of return.
///
/// `values` may be passed as an array literal `{-100,30,40,50}` (a single
/// `Value::Array` arg) or as individual numeric args.  An optional numeric
/// guess arg is accepted immediately after the values.
pub fn irr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 256) {
        return err;
    }

    let (cfs, guess) = match collect_cashflows_with_guess(args) {
        Ok(pair) => pair,
        Err(e) => return e,
    };

    if cfs.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }

    // Must have at least one positive and one negative cash flow
    let has_positive = cfs.iter().any(|&n| n > 0.0);
    let has_negative = cfs.iter().any(|&n| n < 0.0);
    if !has_positive || !has_negative {
        return Value::Error(ErrorKind::Num);
    }

    // Newton-Raphson iteration
    let mut rate = guess;
    for _ in 0..100 {
        let (npv, dnpv) = npv_and_derivative(&cfs, rate);
        if !npv.is_finite() || !dnpv.is_finite() || dnpv == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let new_rate = rate - npv / dnpv;
        if (new_rate - rate).abs() < 1e-7 {
            if !new_rate.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
            return Value::Number(new_rate);
        }
        rate = new_rate;
    }
    Value::Error(ErrorKind::Num)
}

/// Flatten args into cash flows, detecting an optional trailing guess.
///
/// Patterns:
///   IRR({-100,30,40,50})               → cfs from array, guess=0.1
///   IRR({-100,30,40,50}, 0.15)         → cfs from array, guess=0.15
///   IRR(-100, 30, 40, 50)              → all args as cfs, guess=0.1
fn collect_cashflows_with_guess(args: &[Value]) -> Result<(Vec<f64>, f64), Value> {
    // Case 1: first arg is an array
    if let Value::Array(items) = &args[0] {
        let cfs = flatten_values(items.clone())?;
        let guess = if args.len() > 1 {
            to_number(args[1].clone())?
        } else {
            0.1
        };
        return Ok((cfs, guess));
    }

    // Case 2: all args are scalars — treat them all as cash flows, no guess
    let mut cfs = Vec::with_capacity(args.len());
    for arg in args {
        cfs.push(to_number(arg.clone())?);
    }
    Ok((cfs, 0.1))
}

/// Flatten a `Vec<Value>` (may be nested arrays) into f64 cash flows.
fn flatten_values(items: Vec<Value>) -> Result<Vec<f64>, Value> {
    let mut out = Vec::new();
    for v in items {
        match v {
            Value::Array(inner) => {
                let sub = flatten_values(inner)?;
                out.extend(sub);
            }
            other => out.push(to_number(other)?),
        }
    }
    Ok(out)
}

fn npv_and_derivative(cfs: &[f64], rate: f64) -> (f64, f64) {
    let mut npv = 0.0;
    let mut dnpv = 0.0;
    for (i, &cf) in cfs.iter().enumerate() {
        let t = i as f64;
        let denom = (1.0 + rate).powf(t);
        npv  += cf / denom;
        dnpv -= t * cf / ((1.0 + rate).powf(t + 1.0));
    }
    (npv, dnpv)
}

#[cfg(test)]
mod tests;
