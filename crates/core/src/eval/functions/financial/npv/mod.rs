use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `NPV(rate, value1, [value2, ...])` — net present value of cash flows.
///
/// `value1` may be an array literal `{100,200,300}` which is flattened.
pub fn npv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 256) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };

    // Collect cash flows, flattening any arrays
    let mut cfs: Vec<f64> = Vec::new();
    for arg in &args[1..] {
        match arg {
            Value::Array(items) => {
                match flatten_values(items.clone()) {
                    Ok(vals) => cfs.extend(vals),
                    Err(e) => return e,
                }
            }
            other => match to_number(other.clone()) {
                Ok(n) => cfs.push(n),
                Err(e) => return e,
            }
        }
    }

    let mut npv = 0.0_f64;
    for (i, cf) in cfs.iter().enumerate() {
        npv += cf / (1.0 + rate).powi(i as i32 + 1);
    }
    if !npv.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(npv)
}

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

#[cfg(test)]
mod tests;
