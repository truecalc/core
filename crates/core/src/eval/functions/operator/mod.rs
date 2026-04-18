use crate::eval::coercion::{to_bool, to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

use super::{FunctionMeta, Registry};

// ── Coercion helpers ──────────────────────────────────────────────────────────

/// Like `to_number`, but treats empty string as 0.0 (Excel arithmetic behavior).
fn to_number_arith(v: Value) -> Result<f64, Value> {
    match &v {
        Value::Text(s) if s.is_empty() => return Ok(0.0),
        _ => {}
    }
    to_number(v)
}

// ── Arity helpers ─────────────────────────────────────────────────────────────

fn check_exact(args: &[Value], n: usize) -> Option<Value> {
    if args.len() != n {
        Some(Value::Error(ErrorKind::NA))
    } else {
        None
    }
}

// ── Issue #51 — Arithmetic aliases ────────────────────────────────────────────

pub fn add_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let a = match to_number_arith(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let b = match to_number_arith(args[1].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let result = a + b;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn minus_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let a = match to_number_arith(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let b = match to_number_arith(args[1].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let result = a - b;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn multiply_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let a = match to_number_arith(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let b = match to_number_arith(args[1].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let result = a * b;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn divide_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let a = match to_number_arith(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let b = match to_number_arith(args[1].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if b == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let result = a / b;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ── Issue #52 — Comparison aliases ────────────────────────────────────────────

/// Type rank for cross-type ordered comparisons: Number < Text < Bool
fn type_rank(v: &Value) -> u8 {
    match v {
        Value::Number(_) | Value::Empty => 0,
        Value::Text(_) => 1,
        Value::Bool(_) => 2,
        _ => 255,
    }
}

/// Compare two values using Excel-compatible rules.
/// Returns Some(Ordering) when both values are the same type, None for cross-type.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => type_rank(a).cmp(&type_rank(b)),
    }
}

/// Returns true when a and b are the same type (for EQ/NE same-type check)
fn same_type(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Number(_), Value::Number(_))
            | (Value::Text(_), Value::Text(_))
            | (Value::Bool(_), Value::Bool(_))
            | (Value::Empty, Value::Empty)
    )
}

pub fn eq_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let (a, b) = (&args[0], &args[1]);
    if !same_type(a, b) {
        return Value::Bool(false);
    }
    Value::Bool(compare_values(a, b) == std::cmp::Ordering::Equal)
}

pub fn ne_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let (a, b) = (&args[0], &args[1]);
    if !same_type(a, b) {
        return Value::Bool(true);
    }
    Value::Bool(compare_values(a, b) != std::cmp::Ordering::Equal)
}

pub fn gt_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    Value::Bool(compare_values(&args[0], &args[1]) == std::cmp::Ordering::Greater)
}

pub fn gte_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    Value::Bool(compare_values(&args[0], &args[1]) != std::cmp::Ordering::Less)
}

pub fn lt_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    Value::Bool(compare_values(&args[0], &args[1]) == std::cmp::Ordering::Less)
}

pub fn lte_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    Value::Bool(compare_values(&args[0], &args[1]) != std::cmp::Ordering::Greater)
}

// ── Issue #53 — Unary/power aliases ───────────────────────────────────────────

pub fn pow_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let base = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let exp = match to_number(args[1].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let result = base.powf(exp);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

pub fn concat_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 2) {
        return e;
    }
    let a = match to_string_val(args[0].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let b = match to_string_val(args[1].clone()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    Value::Text(format!("{}{}", a, b))
}

pub fn uminus_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 1) {
        return e;
    }
    let n = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Value::Number(-n)
}

/// UPLUS coerces numeric-parseable text to Number; other values pass through unchanged.
pub fn uplus_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 1) {
        return e;
    }
    match &args[0] {
        Value::Text(s) => {
            if let Ok(n) = s.parse::<f64>() {
                Value::Number(n)
            } else {
                args[0].clone()
            }
        }
        _ => args[0].clone(),
    }
}

pub fn unary_percent_fn(args: &[Value]) -> Value {
    if let Some(e) = check_exact(args, 1) {
        return e;
    }
    let n = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    Value::Number(n / 100.0)
}

// ── Issue #336 — ISBETWEEN ────────────────────────────────────────────────────

/// ISBETWEEN(value, lower, upper, [lower_inclusive], [upper_inclusive])
///
/// Returns TRUE if value is between lower and upper.
/// lower_inclusive defaults to TRUE; upper_inclusive defaults to TRUE.
pub fn isbetween_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 5) {
        return err;
    }
    let value = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let lower = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let upper = match to_number(args[2].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let lower_inclusive = if args.len() >= 4 {
        match to_bool(args[3].clone()) {
            Err(e) => return e,
            Ok(b) => b,
        }
    } else {
        true
    };
    let upper_inclusive = if args.len() >= 5 {
        match to_bool(args[4].clone()) {
            Err(e) => return e,
            Ok(b) => b,
        }
    } else {
        true
    };
    let lower_ok = if lower_inclusive { value >= lower } else { value > lower };
    let upper_ok = if upper_inclusive { value <= upper } else { value < upper };
    Value::Bool(lower_ok && upper_ok)
}

// ── Issue #336 — UNIQUE ───────────────────────────────────────────────────────

/// UNIQUE(array, [by_col], [exactly_once])
///
/// Returns unique rows (by_col=FALSE, default) or unique columns (by_col=TRUE)
/// from the array. For a flat 1D row array, by_col=FALSE treats the whole array
/// as a single row — so UNIQUE always returns it unchanged (there is only 1 row).
/// exactly_once=TRUE (only with by_col=TRUE) returns only elements that appear
/// exactly once.
pub fn unique_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 3) {
        return err;
    }
    let by_col = if args.len() >= 2 {
        match to_bool(args[1].clone()) {
            Err(e) => return e,
            Ok(b) => b,
        }
    } else {
        false
    };
    let exactly_once = if args.len() >= 3 {
        match to_bool(args[2].clone()) {
            Err(e) => return e,
            Ok(b) => b,
        }
    } else {
        false
    };

    match &args[0] {
        Value::Array(items) => {
            if by_col {
                // by_col=TRUE: deduplicate individual elements.
                unique_elements(items, exactly_once)
            } else {
                // by_col=FALSE: the flat array is treated as a single row.
                // One row is always unique (and appears exactly once).
                Value::Array(items.clone())
            }
        }
        other => other.clone(),
    }
}

/// Deduplicate a flat slice of values (used for by_col=TRUE).
fn unique_elements(items: &[Value], exactly_once: bool) -> Value {
    if exactly_once {
        let mut counts: Vec<(Value, usize)> = Vec::new();
        for item in items {
            if let Some(entry) = counts.iter_mut().find(|(v, _)| v == item) {
                entry.1 += 1;
            } else {
                counts.push((item.clone(), 1));
            }
        }
        let result: Vec<Value> = counts
            .into_iter()
            .filter(|(_, count)| *count == 1)
            .map(|(v, _)| v)
            .collect();
        Value::Array(result)
    } else {
        let mut seen: Vec<Value> = Vec::new();
        for item in items {
            if !seen.contains(item) {
                seen.push(item.clone());
            }
        }
        Value::Array(seen)
    }
}

#[cfg(test)]
mod tests;

// ── Registration ──────────────────────────────────────────────────────────────
// Operator aliases are compiler-internal; they must not appear in list_functions().

pub fn register_operator(registry: &mut Registry) {
    // Arithmetic
    registry.register_internal("ADD", add_fn);
    registry.register_internal("MINUS", minus_fn);
    registry.register_internal("MULTIPLY", multiply_fn);
    registry.register_internal("DIVIDE", divide_fn);
    // Comparison
    registry.register_internal("EQ", eq_fn);
    registry.register_internal("NE", ne_fn);
    registry.register_internal("GT", gt_fn);
    registry.register_internal("GTE", gte_fn);
    registry.register_internal("LT", lt_fn);
    registry.register_internal("LTE", lte_fn);
    // Unary / power
    registry.register_internal("POW", pow_fn);
    registry.register_internal("CONCAT", concat_fn);
    registry.register_internal("UMINUS", uminus_fn);
    registry.register_internal("UPLUS", uplus_fn);
    registry.register_internal("UNARY_PERCENT", unary_percent_fn);
    // User-facing functions
    registry.register_eager("ISBETWEEN", isbetween_fn, FunctionMeta {
        category: "operator",
        signature: "ISBETWEEN(value, lower, upper, [lower_inclusive], [upper_inclusive])",
        description: "Returns TRUE if value is between lower and upper bounds",
    });
    registry.register_eager("UNIQUE", unique_fn, FunctionMeta {
        category: "operator",
        signature: "UNIQUE(array, [by_col], [exactly_once])",
        description: "Returns unique rows or columns from an array",
    });
}
