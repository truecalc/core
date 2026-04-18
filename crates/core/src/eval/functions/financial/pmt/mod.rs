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

/// `PMT(rate, nper, pv, [fv], [type])` — payment for a loan.
pub fn pmt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 5) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let nper = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let fv   = match opt_number(args, 3, 0.0)   { Ok(n) => n, Err(e) => return e };
    let typ  = match opt_number(args, 4, 0.0)   { Ok(n) => n, Err(e) => return e };

    if nper == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = if rate == 0.0 {
        -(pv + fv) / nper
    } else {
        let factor = (1.0 + rate).powf(nper);
        let denom = factor - 1.0;
        if denom == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        -(pv * rate * factor + fv * rate) / denom / (1.0 + rate * typ)
    };

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
