use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn opt_number(args: &[Value], idx: usize, default: f64) -> Result<f64, Value> {
    if idx < args.len() {
        to_number(args[idx].clone())
    } else {
        Ok(default)
    }
}

/// `RATE(nper, pmt, pv, [fv], [type], [guess])` — interest rate per period.
pub fn rate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 6) {
        return err;
    }
    let nper  = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let pmt   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pv    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let fv    = match opt_number(args, 3, 0.0)   { Ok(n) => n, Err(e) => return e };
    let typ   = match opt_number(args, 4, 0.0)   { Ok(n) => n, Err(e) => return e };
    let guess = match opt_number(args, 5, 0.1)   { Ok(n) => n, Err(e) => return e };

    // Newton-Raphson: find rate such that PV equation = 0
    // f(r) = pv*(1+r)^nper + pmt*(1+r*type)*((1+r)^nper - 1)/r + fv = 0
    let mut rate = guess;
    for _ in 0..100 {
        let (f, df) = rate_f_and_df(rate, nper, pmt, pv, fv, typ);
        if !f.is_finite() || !df.is_finite() || df == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let new_rate = rate - f / df;
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

fn rate_f_and_df(r: f64, nper: f64, pmt: f64, pv: f64, fv: f64, typ: f64) -> (f64, f64) {
    if r == 0.0 {
        // Degenerate: use limit
        let f = pv + pmt * nper + fv;
        let df = pv * nper + pmt * nper * (nper - 1.0) / 2.0;
        return (f, df);
    }
    let factor = (1.0 + r).powf(nper);
    let annuity = pmt * (1.0 + r * typ) * (factor - 1.0) / r;
    let f = pv * factor + annuity + fv;

    // Derivative wrt r
    let dfactor = nper * (1.0 + r).powf(nper - 1.0);
    let dann = pmt * (typ * (factor - 1.0) / r
        + (1.0 + r * typ) * (dfactor * r - (factor - 1.0)) / (r * r));
    let df = pv * dfactor + dann;
    (f, df)
}

#[cfg(test)]
mod tests;
