use crate::types::Value;
use super::mode::mode_fn;

/// `MODE.SNGL(value1, ...)` — most frequently occurring value (same as MODE).
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// Returns `#N/A` if all values unique or no values.
pub fn mode_sngl_fn(args: &[Value]) -> Value {
    mode_fn(args)
}

#[cfg(test)]
mod tests;
