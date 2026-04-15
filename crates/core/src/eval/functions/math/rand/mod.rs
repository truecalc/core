use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `RAND()` — returns a random number in [0, 1).
pub fn rand_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 0, 0) {
        return err;
    }
    // Use a simple LCG seeded from system time for no-dependency randomness.
    // For a real spreadsheet engine a proper RNG would be used, but this avoids
    // adding external crates.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345);
    // LCG: multiplier 1664525, addend 1013904223 (Numerical Recipes)
    let val = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let f = (val as f64) / (u32::MAX as f64 + 1.0);
    Value::Number(f)
}

/// `RANDBETWEEN(low, high)` — returns a random integer in [low, high] inclusive.
pub fn randbetween_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let low = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let high = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let lo = low.ceil() as i64;
    let hi = high.floor() as i64;
    if lo > hi {
        return Value::Error(ErrorKind::Num);
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345);
    let val = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let range = (hi - lo + 1) as u64;
    let result = lo + (val as u64 % range) as i64;
    Value::Number(result as f64)
}

#[cfg(test)]
mod tests;
