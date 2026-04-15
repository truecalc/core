use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `IRR(value1, value2, ..., [guess])` — internal rate of return.
///
/// All positional args are treated as cash flows except the last if there
/// are only 2 args total (ambiguous). We require at least 2 cash flow values,
/// so min arity is 2. The optional guess is NOT a separate named arg here —
/// instead we treat all args as cash flows by default. To pass a guess, callers
/// should be aware this implementation uses 0.1 as the default guess.
///
/// Per the spec: min 2 args (cash flows). A guess arg is accepted as the
/// last arg when arity allows, but for simplicity and matching spreadsheet
/// behavior, all args are treated as cash flows and guess defaults to 0.1.
pub fn irr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 255) {
        return err;
    }

    // Collect cash flows
    let mut cfs = Vec::with_capacity(args.len());
    for arg in args {
        match to_number(arg.clone()) {
            Ok(n)  => cfs.push(n),
            Err(e) => return e,
        }
    }

    // Must have at least one positive and one negative cash flow
    let has_positive = cfs.iter().any(|&n| n > 0.0);
    let has_negative = cfs.iter().any(|&n| n < 0.0);
    if !has_positive || !has_negative {
        return Value::Error(ErrorKind::Num);
    }

    // Newton-Raphson iteration
    let mut rate = 0.1_f64;
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
