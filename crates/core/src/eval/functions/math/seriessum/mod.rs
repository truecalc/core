use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SERIESSUM(x, n, m, coefficients)` — power series sum.
/// Computes sum(coefficients[i] * x^(n + i*m)) for i = 0..len(coefficients)-1.
pub fn seriessum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 4) {
        return err;
    }
    let x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let n = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let m = match to_number(args[2].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };

    // Collect coefficients from the 4th argument (array or scalar)
    let coeffs: Vec<f64> = match &args[3] {
        Value::Array(elems) => {
            let mut cs = Vec::with_capacity(elems.len());
            for elem in elems {
                match to_number(elem.clone()) {
                    Err(e) => return e,
                    Ok(v) => cs.push(v),
                }
            }
            cs
        }
        other => match to_number(other.clone()) {
            Err(e) => return e,
            Ok(v) => vec![v],
        },
    };

    let mut total = 0.0_f64;
    for (i, &coeff) in coeffs.iter().enumerate() {
        let power = n + (i as f64) * m;
        let term = coeff * x.powf(power);
        if !term.is_finite() {
            return Value::Error(ErrorKind::Num);
        }
        total += term;
    }
    if !total.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(total)
}

#[cfg(test)]
mod tests;
