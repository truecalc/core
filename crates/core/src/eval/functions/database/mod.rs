use super::super::{FunctionMeta, Registry};
use crate::eval::functions::math::criterion::{matches_criterion, parse_criterion};
use crate::types::{ErrorKind, Value};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract rows from a 2D Value::Array.
/// The outer array contains rows (each row is a Value::Array).
/// Returns None if the structure is not a valid 2D array.
fn extract_rows(v: &Value) -> Option<Vec<&[Value]>> {
    match v {
        Value::Array(outer) => {
            let rows: Option<Vec<&[Value]>> = outer
                .iter()
                .map(|row| match row {
                    Value::Array(r) => Some(r.as_slice()),
                    _ => None,
                })
                .collect();
            rows
        }
        _ => None,
    }
}

/// Resolve field argument to a 0-based column index.
/// field can be a 1-based number or a column name string matching headers.
fn resolve_field(field: &Value, headers: &[Value]) -> Option<usize> {
    match field {
        Value::Number(n) => {
            let idx = *n as usize;
            if idx >= 1 && idx <= headers.len() {
                Some(idx - 1)
            } else {
                None
            }
        }
        Value::Text(name) => {
            let name_lower = name.to_lowercase();
            headers.iter().position(|h| match h {
                Value::Text(s) => s.to_lowercase() == name_lower,
                _ => false,
            })
        }
        _ => None,
    }
}

/// Check whether a data row matches all criteria.
/// criteria_rows: [header_row, criteria_row]
/// data_headers: the database headers
/// data_row: the data row to check
fn row_matches_criteria(
    criteria_rows: &[&[Value]],
    data_headers: &[Value],
    data_row: &[Value],
) -> bool {
    if criteria_rows.len() < 2 {
        return false;
    }
    let crit_headers = criteria_rows[0];
    let crit_values = criteria_rows[1];

    for (crit_col, crit_val) in crit_headers.iter().zip(crit_values.iter()) {
        // Skip empty criteria
        match crit_val {
            Value::Empty => continue,
            Value::Text(s) if s.is_empty() => continue,
            _ => {}
        }

        // Find the matching data column
        let col_name = match crit_col {
            Value::Text(s) => s.to_lowercase(),
            _ => continue,
        };
        let col_idx = data_headers.iter().position(|h| match h {
            Value::Text(s) => s.to_lowercase() == col_name,
            _ => false,
        });
        let col_idx = match col_idx {
            Some(i) => i,
            None => return false, // criteria column not found in database
        };

        let cell_val = data_row.get(col_idx).unwrap_or(&Value::Empty);
        let criterion = parse_criterion(crit_val);
        if !matches_criterion(cell_val, &criterion) {
            return false;
        }
    }
    true
}

/// Parse the database and criteria arguments, collect the field values for
/// matching rows.
///
/// Returns `Err(Value)` on structural errors (wrong types, field not found).
fn collect_matching_values(args: &[Value]) -> Result<Vec<Value>, Value> {
    if args.len() != 3 {
        return Err(Value::Error(ErrorKind::NA));
    }
    let db_rows = extract_rows(&args[0])
        .ok_or(Value::Error(ErrorKind::Value))?;

    if db_rows.len() < 2 {
        // Need at least header row + one data row
        return Ok(vec![]);
    }

    let headers = db_rows[0];
    let field_idx = resolve_field(&args[1], headers)
        .ok_or(Value::Error(ErrorKind::Value))?;

    let crit_rows = extract_rows(&args[2])
        .ok_or(Value::Error(ErrorKind::Value))?;

    if crit_rows.len() < 2 {
        return Err(Value::Error(ErrorKind::Value));
    }

    let mut values = Vec::new();
    for data_row in &db_rows[1..] {
        if row_matches_criteria(&crit_rows, headers, data_row) {
            let val = data_row.get(field_idx).cloned().unwrap_or(Value::Empty);
            values.push(val);
        }
    }
    Ok(values)
}

// ── D* functions ──────────────────────────────────────────────────────────────

/// `DSUM(database, field, criteria)` — sum of field values for matching rows.
pub fn dsum_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let mut sum = 0.0_f64;
            for v in &values {
                if let Value::Number(n) = v {
                    sum += n;
                }
            }
            Value::Number(sum)
        }
    }
}

/// `DAVERAGE(database, field, criteria)` — average of field values for matching rows.
pub fn daverage_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            if nums.is_empty() {
                return Value::Error(ErrorKind::DivByZero);
            }
            Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
        }
    }
}

/// `DCOUNT(database, field, criteria)` — count of numeric field values for matching rows.
pub fn dcount_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let count = values
                .iter()
                .filter(|v| matches!(v, Value::Number(_)))
                .count();
            Value::Number(count as f64)
        }
    }
}

/// `DCOUNTA(database, field, criteria)` — count of non-empty field values for matching rows.
pub fn dcounta_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let count = values
                .iter()
                .filter(|v| !matches!(v, Value::Empty))
                .count();
            Value::Number(count as f64)
        }
    }
}

/// `DGET(database, field, criteria)` — returns the single matching value, or error.
pub fn dget_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            if values.len() == 1 {
                values.into_iter().next().unwrap()
            } else if values.is_empty() {
                Value::Error(ErrorKind::Value)
            } else {
                // Multiple matches
                Value::Error(ErrorKind::Num)
            }
        }
    }
}

/// `DMAX(database, field, criteria)` — max of field values for matching rows.
pub fn dmax_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        }
    }
}

/// `DMIN(database, field, criteria)` — min of field values for matching rows.
pub fn dmin_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.iter().cloned().fold(f64::INFINITY, f64::min))
        }
    }
}

/// `DPRODUCT(database, field, criteria)` — product of field values for matching rows.
pub fn dproduct_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            if nums.is_empty() {
                return Value::Number(0.0);
            }
            Value::Number(nums.iter().product())
        }
    }
}

/// `DSTDEV(database, field, criteria)` — sample standard deviation of matching numeric values.
pub fn dstdev_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            let n = nums.len();
            if n < 2 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
            Value::Number(var.sqrt())
        }
    }
}

/// `DSTDEVP(database, field, criteria)` — population standard deviation of matching numeric values.
pub fn dstdevp_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            let n = nums.len();
            if n == 0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
            Value::Number(var.sqrt())
        }
    }
}

/// `DVAR(database, field, criteria)` — sample variance of matching numeric values.
pub fn dvar_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            let n = nums.len();
            if n < 2 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
            Value::Number(var)
        }
    }
}

/// `DVARP(database, field, criteria)` — population variance of matching numeric values.
pub fn dvarp_fn(args: &[Value]) -> Value {
    match collect_matching_values(args) {
        Err(e) => e,
        Ok(values) => {
            let nums: Vec<f64> = values
                .iter()
                .filter_map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .collect();
            let n = nums.len();
            if n == 0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
            Value::Number(var)
        }
    }
}

// ── Registration ──────────────────────────────────────────────────────────────

pub fn register_database(registry: &mut Registry) {
    registry.register_eager("DSUM",     dsum_fn,     FunctionMeta { category: "database", signature: "DSUM(database, field, criteria)",     description: "Sum of field values for rows matching criteria" });
    registry.register_eager("DAVERAGE", daverage_fn, FunctionMeta { category: "database", signature: "DAVERAGE(database, field, criteria)", description: "Average of field values for rows matching criteria" });
    registry.register_eager("DCOUNT",   dcount_fn,   FunctionMeta { category: "database", signature: "DCOUNT(database, field, criteria)",   description: "Count of numeric field values for rows matching criteria" });
    registry.register_eager("DCOUNTA",  dcounta_fn,  FunctionMeta { category: "database", signature: "DCOUNTA(database, field, criteria)",  description: "Count of non-empty field values for rows matching criteria" });
    registry.register_eager("DGET",     dget_fn,     FunctionMeta { category: "database", signature: "DGET(database, field, criteria)",     description: "Single field value for rows matching criteria" });
    registry.register_eager("DMAX",     dmax_fn,     FunctionMeta { category: "database", signature: "DMAX(database, field, criteria)",     description: "Maximum field value for rows matching criteria" });
    registry.register_eager("DMIN",     dmin_fn,     FunctionMeta { category: "database", signature: "DMIN(database, field, criteria)",     description: "Minimum field value for rows matching criteria" });
    registry.register_eager("DPRODUCT", dproduct_fn, FunctionMeta { category: "database", signature: "DPRODUCT(database, field, criteria)", description: "Product of field values for rows matching criteria" });
    registry.register_eager("DSTDEV",   dstdev_fn,   FunctionMeta { category: "database", signature: "DSTDEV(database, field, criteria)",   description: "Sample standard deviation of field values for rows matching criteria" });
    registry.register_eager("DSTDEVP",  dstdevp_fn,  FunctionMeta { category: "database", signature: "DSTDEVP(database, field, criteria)",  description: "Population standard deviation of field values for rows matching criteria" });
    registry.register_eager("DVAR",     dvar_fn,     FunctionMeta { category: "database", signature: "DVAR(database, field, criteria)",     description: "Sample variance of field values for rows matching criteria" });
    registry.register_eager("DVARP",    dvarp_fn,    FunctionMeta { category: "database", signature: "DVARP(database, field, criteria)",    description: "Population variance of field values for rows matching criteria" });
}

#[cfg(test)]
mod tests;
