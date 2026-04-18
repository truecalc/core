use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn random_number() -> f64 {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345);
    let val = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (val as f64) / (u32::MAX as f64 + 1.0)
}

/// `RANDARRAY([rows], [cols], [min], [max], [integer])`
///
/// Returns an array of random numbers.
/// With no args: returns a single random number (equivalent to RAND()).
/// With rows/cols: returns a nested 2D array (rows × cols).
pub fn randarray_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        // No args: return single random number
        return Value::Number(random_number());
    }
    if let Some(err) = check_arity(args, 1, 5) {
        return err;
    }
    let rows = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if rows <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let rows = rows as usize;
    let cols = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => {
                if v <= 0.0 {
                    return Value::Error(ErrorKind::Num);
                }
                v as usize
            }
        }
    } else {
        1
    };

    // Always return nested 2D array so ROWS/COLUMNS work correctly
    let outer: Vec<Value> = (0..rows)
        .map(|_| {
            let row: Vec<Value> = (0..cols).map(|_| Value::Number(random_number())).collect();
            Value::Array(row)
        })
        .collect();
    Value::Array(outer)
}
