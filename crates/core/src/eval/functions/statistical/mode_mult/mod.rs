use crate::types::Value;
use super::mode::mode_fn;

/// `MODE.MULT(value1, ...)` — in scalar context, behaves like MODE.SNGL.
/// Returns the most frequently occurring value (smallest if tied).
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// Returns `#N/A` if all values unique or no values.
pub fn mode_mult_fn(args: &[Value]) -> Value {
    mode_fn(args)
}

#[cfg(test)]
mod tests;
