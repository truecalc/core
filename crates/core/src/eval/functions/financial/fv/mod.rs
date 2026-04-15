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

/// `FV(rate, nper, pmt, [pv], [type])` — future value of an investment.
pub fn fv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 5) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let nper = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pmt  = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match opt_number(args, 3, 0.0)   { Ok(n) => n, Err(e) => return e };
    let typ  = match opt_number(args, 4, 0.0)   { Ok(n) => n, Err(e) => return e };

    let result = if rate == 0.0 {
        -(pv + pmt * nper)
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(pv * factor + pmt * (1.0 + rate * typ) * (factor - 1.0) / rate)
    };

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
