use crate::types::Value;

/// Flatten a Value into a list of rows (Vec of Vec<Value>).
/// A 2D array (Array of Arrays) → each inner Array is a row.
/// A 1D array (Array of scalars) → one row containing all elements.
/// A scalar → one row with one element.
pub fn flatten_to_rows(v: &Value) -> Vec<Vec<Value>> {
    match v {
        Value::Array(outer) => {
            // Check if 2D: any element is itself an Array
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                outer.iter().map(|row| match row {
                    Value::Array(cols) => cols.clone(),
                    other => vec![other.clone()],
                }).collect()
            } else {
                // 1D: one row
                vec![outer.clone()]
            }
        }
        other => vec![vec![other.clone()]],
    }
}

/// Flatten all values in a Value (including nested Arrays) into a single Vec<Value>.
pub fn flatten_to_flat(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(elems) => elems.iter().flat_map(flatten_to_flat).collect(),
        other => vec![other.clone()],
    }
}

/// Compare two values for equality (case-insensitive for text).
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Text(x), Value::Text(y)) => x.to_uppercase() == y.to_uppercase(),
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

/// Compare two values for ordering (for approximate match).
/// Returns None if the types are incomparable.
pub fn value_compare(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y),
        (Value::Text(x), Value::Text(y)) => Some(x.to_uppercase().cmp(&y.to_uppercase())),
        _ => None,
    }
}
