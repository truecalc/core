use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// LCG PRNG seeded from system time (Numerical Recipes constants).
/// Returns successive values by stepping the state forward on each call.
fn lcg_sequence(count: usize) -> Vec<f64> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);
    // Mix in a higher-entropy seed using the full nanosecond count
    let mut state = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Use upper 32 bits for better quality
        let val = (state >> 32) as u32;
        out.push((val as f64) / (u32::MAX as f64 + 1.0));
    }
    out
}

/// `RANDARRAY([rows], [cols], [min], [max], [integer])`
///
/// Returns an array of random numbers.
/// With no args: returns a single random number (equivalent to RAND()).
/// With rows/cols: returns a nested 2D array (rows × cols).
pub fn randarray_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        // No args: return single random number
        let vals = lcg_sequence(1);
        return Value::Number(vals[0]);
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

    let total = rows * cols;
    let nums = lcg_sequence(total);
    // Always return nested 2D array so ROWS/COLUMNS work correctly
    let outer: Vec<Value> = (0..rows)
        .map(|r| {
            let row: Vec<Value> = (0..cols)
                .map(|c| Value::Number(nums[r * cols + c]))
                .collect();
            Value::Array(row)
        })
        .collect();
    Value::Array(outer)
}

#[cfg(test)]
mod tests;
