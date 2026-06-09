use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn power_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let base = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let exp = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    // GS: POWER(-8, 1/3) = -2.0 (real odd-root for negative base)
    // When base < 0 and exp is rational p/q with q odd, use real root.
    let result = if base < 0.0 && exp.fract() != 0.0 {
        // Check if 1/exp is close to an odd integer -> odd root
        let inv = 1.0 / exp;
        let inv_round = inv.round();
        if (inv - inv_round).abs() < 1e-9 && (inv_round.abs() as i64) % 2 == 1 {
            let mag = base.abs().powf(exp);
            -mag
        } else {
            // non-real result -> #NUM!
            return Value::Error(ErrorKind::Num);
        }
    } else {
        base.powf(exp)
    };
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
