//! Array and matrix functions for Google Sheets compatibility.

use crate::eval::{evaluate_expr, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

use super::{check_arity, check_arity_len, FunctionMeta, Registry};

// ── 2D array helpers ──────────────────────────────────────────────────────────

/// Convert a Value into a 2D grid (Vec<Vec<Value>>).
/// - Nested Array (2D): outer = rows, inner = cols
/// - Flat Array (1D): one row
/// - Scalar: 1x1
pub fn to_2d(v: &Value) -> Vec<Vec<Value>> {
    match v {
        Value::Array(outer) => {
            if outer.iter().any(|e| matches!(e, Value::Array(_))) {
                outer
                    .iter()
                    .map(|row| match row {
                        Value::Array(cols) => cols.clone(),
                        other => vec![other.clone()],
                    })
                    .collect()
            } else {
                vec![outer.clone()] // 1-D flat array → single row
            }
        }
        other => vec![vec![other.clone()]], // scalar → 1×1
    }
}

/// Convert a 2D grid back to a Value.
/// - Empty grid → empty Array
/// - Single row → flat Array
/// - Multiple rows → nested Array of row Arrays
pub fn from_2d(rows: Vec<Vec<Value>>) -> Value {
    if rows.is_empty() {
        return Value::Array(vec![]);
    }
    if rows.len() == 1 {
        return Value::Array(rows.into_iter().next().unwrap());
    }
    Value::Array(rows.into_iter().map(Value::Array).collect())
}

/// Flatten a Value to a 1D Vec<Value> (row-major order).
pub fn flatten_val(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(outer) => {
            if outer.iter().any(|e| matches!(e, Value::Array(_))) {
                outer
                    .iter()
                    .flat_map(|row| match row {
                        Value::Array(cols) => cols.clone(),
                        other => vec![other.clone()],
                    })
                    .collect()
            } else {
                outer.clone()
            }
        }
        other => vec![other.clone()],
    }
}

/// Convert a Value to f64 for numeric computations.
fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Case-insensitive equality for scalar Values (used by UNIQUE).
fn values_equal_1d(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Text(x), Value::Text(y)) => x.to_uppercase() == y.to_uppercase(),
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

// ── ROWS ─────────────────────────────────────────────────────────────────────

pub(crate) fn rows_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let grid = to_2d(&args[0]);
    Value::Number(grid.len() as f64)
}

// ── COLUMNS ───────────────────────────────────────────────────────────────────

pub(crate) fn columns_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let cols = grid.first().map(|r| r.len()).unwrap_or(0);
    Value::Number(cols as f64)
}

// ── INDEX ─────────────────────────────────────────────────────────────────────
// INDEX(array, row, [col]) — 1-based indices

fn index_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 3) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let nrows = grid.len();
    let ncols = grid.first().map(|r| r.len()).unwrap_or(0);

    let row_idx = if args.len() >= 2 {
        match to_f64(&args[1]) {
            Some(n) => {
                let r = n as isize;
                if r < 0 {
                    (nrows as isize + r + 1) as usize
                } else {
                    r as usize
                }
            }
            None => return Value::Error(ErrorKind::Value),
        }
    } else {
        0 // 0 means return whole row/col
    };

    let col_idx = if args.len() >= 3 {
        match to_f64(&args[2]) {
            Some(n) => {
                let c = n as isize;
                if c < 0 {
                    (ncols as isize + c + 1) as usize
                } else {
                    c as usize
                }
            }
            None => return Value::Error(ErrorKind::Value),
        }
    } else {
        0 // 0 means return whole column
    };

    // Single element
    if row_idx > 0 && col_idx > 0 {
        if row_idx > nrows || col_idx > ncols {
            return Value::Error(ErrorKind::Ref);
        }
        return grid[row_idx - 1][col_idx - 1].clone();
    }

    // Return whole row (col_idx == 0, row_idx > 0)
    if row_idx > 0 && col_idx == 0 {
        // For a 1D row vector (1 row, N cols), treat row_idx as col index
        if nrows == 1 && ncols > 1 {
            if row_idx > ncols {
                return Value::Error(ErrorKind::Ref);
            }
            return grid[0][row_idx - 1].clone();
        }
        if row_idx > nrows {
            return Value::Error(ErrorKind::Ref);
        }
        let row = grid[row_idx - 1].clone();
        if row.len() == 1 {
            return row.into_iter().next().unwrap();
        }
        return Value::Array(row);
    }

    // Return whole column (row_idx == 0, col_idx > 0)
    if row_idx == 0 && col_idx > 0 {
        if col_idx > ncols {
            return Value::Error(ErrorKind::Ref);
        }
        let col: Vec<Value> = grid.iter().map(|r| r[col_idx - 1].clone()).collect();
        if col.len() == 1 {
            return col.into_iter().next().unwrap();
        }
        return from_2d(col.into_iter().map(|v| vec![v]).collect());
    }

    // Both zero → return full array
    args[0].clone()
}

// ── TRANSPOSE ─────────────────────────────────────────────────────────────────

pub(crate) fn transpose_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let grid = to_2d(&args[0]);
    if grid.is_empty() {
        return Value::Array(vec![]);
    }
    let nrows = grid.len();
    let ncols = grid[0].len();
    let transposed: Vec<Vec<Value>> = (0..ncols)
        .map(|c| (0..nrows).map(|r| grid[r][c].clone()).collect())
        .collect();
    from_2d(transposed)
}

// ── ARRAY_CONSTRAIN ───────────────────────────────────────────────────────────

pub(crate) fn array_constrain_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 3, 3) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let num_rows = match to_f64(&args[1]) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let num_cols = match to_f64(&args[2]) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let rows_to_take = num_rows.min(grid.len());
    let result: Vec<Vec<Value>> = grid[..rows_to_take]
        .iter()
        .map(|row| {
            let cols_to_take = num_cols.min(row.len());
            row[..cols_to_take].to_vec()
        })
        .collect();
    from_2d(result)
}

// ── CHOOSECOLS ────────────────────────────────────────────────────────────────

fn choosecols_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, usize::MAX) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let ncols = grid.first().map(|r| r.len()).unwrap_or(0);
    let mut selected_cols: Vec<usize> = Vec::new();
    for col_arg in &args[1..] {
        match to_f64(col_arg) {
            Some(0.0) => return Value::Error(ErrorKind::Value),
            Some(n) => {
                let idx = if n < 0.0 {
                    let i = (ncols as isize + n as isize) as usize;
                    if n as isize + (ncols as isize) < 0 {
                        return Value::Error(ErrorKind::Value);
                    }
                    i
                } else {
                    let i = n as usize - 1;
                    if i >= ncols {
                        return Value::Error(ErrorKind::Value);
                    }
                    i
                };
                selected_cols.push(idx);
            }
            None => return Value::Error(ErrorKind::Value),
        }
    }
    let result: Vec<Vec<Value>> = grid
        .iter()
        .map(|row| {
            selected_cols
                .iter()
                .map(|&c| row.get(c).cloned().unwrap_or(Value::Empty))
                .collect()
        })
        .collect();
    from_2d(result)
}

// ── CHOOSEROWS ────────────────────────────────────────────────────────────────

fn chooserows_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, usize::MAX) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let nrows = grid.len();
    let mut selected_rows: Vec<usize> = Vec::new();
    for row_arg in &args[1..] {
        match to_f64(row_arg) {
            Some(0.0) => return Value::Error(ErrorKind::Value),
            Some(n) => {
                let idx = if n < 0.0 {
                    let i = (nrows as isize + n as isize) as usize;
                    if n as isize + (nrows as isize) < 0 {
                        return Value::Error(ErrorKind::Value);
                    }
                    i
                } else {
                    let i = n as usize - 1;
                    if i >= nrows {
                        return Value::Error(ErrorKind::Value);
                    }
                    i
                };
                selected_rows.push(idx);
            }
            None => return Value::Error(ErrorKind::Value),
        }
    }
    let result: Vec<Vec<Value>> = selected_rows
        .iter()
        .map(|&r| grid.get(r).cloned().unwrap_or_default())
        .collect();
    from_2d(result)
}

// ── FLATTEN ───────────────────────────────────────────────────────────────────
// Returns a single-column (ROWS=n, COLS=1) array

pub(crate) fn flatten_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let flat = flatten_val(&args[0]);
    // Return as column vector (nested array of single-element rows)
    let col: Vec<Vec<Value>> = flat.into_iter().map(|v| vec![v]).collect();
    from_2d(col)
}

// ── HSTACK ────────────────────────────────────────────────────────────────────

fn hstack_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, usize::MAX) {
        return e;
    }
    let grids: Vec<Vec<Vec<Value>>> = args.iter().map(to_2d).collect();
    let nrows = grids.iter().map(|g| g.len()).max().unwrap_or(0);
    let result: Vec<Vec<Value>> = (0..nrows)
        .map(|r| {
            grids
                .iter()
                .flat_map(|g| {
                    g.get(r).cloned().unwrap_or_default()
                })
                .collect()
        })
        .collect();
    from_2d(result)
}

// ── VSTACK ────────────────────────────────────────────────────────────────────

fn vstack_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, usize::MAX) {
        return e;
    }
    let mut result: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        let grid = to_2d(arg);
        result.extend(grid);
    }
    from_2d(result)
}

// ── TOCOL ─────────────────────────────────────────────────────────────────────
// Converts array to column vector (many rows, 1 col)

fn tocol_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 3) {
        return e;
    }
    let flat = flatten_val(&args[0]);
    let col: Vec<Vec<Value>> = flat.into_iter().map(|v| vec![v]).collect();
    from_2d(col)
}

// ── TOROW ─────────────────────────────────────────────────────────────────────
// Converts array to row vector (1 row, many cols)

fn torow_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 3) {
        return e;
    }
    let flat = flatten_val(&args[0]);
    Value::Array(flat)
}

// ── WRAPCOLS ──────────────────────────────────────────────────────────────────
// WRAPCOLS(vector, wrap_count) — split into columns of wrap_count rows
// Result: ceil(n/wrap_count) columns, wrap_count rows (pad last col with Empty)

fn wrapcols_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 3) {
        return e;
    }
    let flat = flatten_val(&args[0]);
    let wrap_count = match to_f64(&args[1]) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let pad = args.get(2).cloned().unwrap_or(Value::Empty);

    // Split into columns of wrap_count elements each
    let ncols = flat.len().div_ceil(wrap_count);
    let nrows = wrap_count;

    // Build column-major layout, then transpose to row-major
    let grid: Vec<Vec<Value>> = (0..nrows)
        .map(|r| {
            (0..ncols)
                .map(|c| {
                    let idx = c * wrap_count + r;
                    flat.get(idx).cloned().unwrap_or_else(|| pad.clone())
                })
                .collect()
        })
        .collect();
    from_2d(grid)
}

// ── WRAPROWS ──────────────────────────────────────────────────────────────────
// WRAPROWS(vector, wrap_count) — split into rows of wrap_count cols

fn wraprows_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 3) {
        return e;
    }
    let flat = flatten_val(&args[0]);
    let wrap_count = match to_f64(&args[1]) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let pad = args.get(2).cloned().unwrap_or(Value::Empty);

    let nrows = flat.len().div_ceil(wrap_count);
    let grid: Vec<Vec<Value>> = (0..nrows)
        .map(|r| {
            (0..wrap_count)
                .map(|c| {
                    let idx = r * wrap_count + c;
                    flat.get(idx).cloned().unwrap_or_else(|| pad.clone())
                })
                .collect()
        })
        .collect();
    from_2d(grid)
}

// ── SORT ──────────────────────────────────────────────────────────────────────

pub(crate) fn sort_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 4) {
        return e;
    }
    let is_1d = matches!(&args[0], Value::Array(outer) if !outer.iter().any(|e| matches!(e, Value::Array(_))));
    let mut grid = to_2d(&args[0]);
    let sort_col = if args.len() >= 2 {
        match to_f64(&args[1]) {
            Some(n) if n >= 1.0 => n as usize - 1,
            Some(_) => return Value::Error(ErrorKind::Value),
            None => 0,
        }
    } else {
        0
    };
    let ascending = if args.len() >= 3 {
        match &args[2] {
            Value::Number(n) => *n >= 0.0,
            Value::Bool(b) => *b,
            _ => true,
        }
    } else {
        true
    };

    if is_1d {
        // 1D: sort the elements within the single row
        let mut elems = grid.into_iter().next().unwrap_or_default();
        elems.sort_by(|a, b| {
            let cmp = compare_values_sort(a, b);
            if ascending { cmp } else { cmp.reverse() }
        });
        return Value::Array(elems);
    }

    grid.sort_by(|a, b| {
        let va = a.get(sort_col).unwrap_or(&Value::Empty);
        let vb = b.get(sort_col).unwrap_or(&Value::Empty);
        let cmp = compare_values_sort(va, vb);
        if ascending { cmp } else { cmp.reverse() }
    });
    from_2d(grid)
}

fn compare_values_sort(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

// ── SORTBY ────────────────────────────────────────────────────────────────────

fn sortby_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, usize::MAX) {
        return e;
    }
    let is_1d = matches!(&args[0], Value::Array(outer) if !outer.iter().any(|e| matches!(e, Value::Array(_))));

    if is_1d {
        // 1D: treat each element as a separate item to sort
        let elems = flatten_val(&args[0]);
        let n = elems.len();

        let mut sort_keys: Vec<(Vec<Value>, bool)> = Vec::new();
        let mut i = 1;
        while i < args.len() {
            let key_vals = flatten_val(&args[i]);
            if key_vals.len() != n {
                return Value::Error(ErrorKind::Value);
            }
            let ascending = if i + 1 < args.len() {
                match to_f64(&args[i + 1]) {
                    Some(v) => v >= 0.0,
                    None => true,
                }
            } else {
                true
            };
            sort_keys.push((key_vals, ascending));
            i += 2;
        }

        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&ra, &rb| {
            for (keys, asc) in &sort_keys {
                let va = keys.get(ra).unwrap_or(&Value::Empty);
                let vb = keys.get(rb).unwrap_or(&Value::Empty);
                let cmp = compare_values_sort(va, vb);
                if cmp != std::cmp::Ordering::Equal {
                    return if *asc { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        return Value::Array(indices.iter().map(|&r| elems[r].clone()).collect());
    }

    let grid = to_2d(&args[0]);
    let nrows = grid.len();

    // Collect (sort_key_array, order) pairs
    let mut sort_keys: Vec<(Vec<Value>, bool)> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let key_vals = flatten_val(&args[i]);
        if key_vals.len() != nrows && nrows > 1 {
            return Value::Error(ErrorKind::Value);
        }
        let ascending = if i + 1 < args.len() {
            match to_f64(&args[i + 1]) {
                Some(n) => n >= 0.0,
                None => true,
            }
        } else {
            true
        };
        sort_keys.push((key_vals, ascending));
        i += 2;
    }

    let mut indices: Vec<usize> = (0..nrows).collect();
    indices.sort_by(|&ra, &rb| {
        for (keys, asc) in &sort_keys {
            let va = keys.get(ra).unwrap_or(&Value::Empty);
            let vb = keys.get(rb).unwrap_or(&Value::Empty);
            let cmp = compare_values_sort(va, vb);
            if cmp != std::cmp::Ordering::Equal {
                return if *asc { cmp } else { cmp.reverse() };
            }
        }
        std::cmp::Ordering::Equal
    });

    let sorted: Vec<Vec<Value>> = indices.iter().map(|&r| grid[r].clone()).collect();
    drop(grid);
    from_2d(sorted)
}

// ── UNIQUE ────────────────────────────────────────────────────────────────────

pub(crate) fn unique_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 3) {
        return e;
    }
    let is_1d = matches!(&args[0], Value::Array(outer) if !outer.iter().any(|e| matches!(e, Value::Array(_))));
    let grid = to_2d(&args[0]);
    // by_col defaults to false (deduplicate rows)
    let by_col = args.get(1).map(|v| matches!(v, Value::Bool(true))).unwrap_or(false);
    let exactly_once = args.get(2).map(|v| matches!(v, Value::Bool(true))).unwrap_or(false);

    // For 1D arrays, deduplicate individual elements (not rows)
    if is_1d && !by_col {
        let elems = flatten_val(&args[0]);
        let mut seen: Vec<Value> = Vec::new();
        let mut counts: Vec<usize> = Vec::new();
        for elem in &elems {
            if let Some(pos) = seen.iter().position(|s| values_equal_1d(s, elem)) {
                counts[pos] += 1;
            } else {
                seen.push(elem.clone());
                counts.push(1);
            }
        }
        let result: Vec<Value> = seen.into_iter().zip(counts)
            .filter(|(_, cnt)| !exactly_once || *cnt == 1)
            .map(|(v, _)| v)
            .collect();
        return Value::Array(result);
    }

    if by_col {
        // Deduplicate columns
        let nrows = grid.len();
        if nrows == 0 {
            return from_2d(vec![]);
        }
        let ncols = grid[0].len();
        // Build column-major representation
        let columns: Vec<Vec<Value>> = (0..ncols)
            .map(|c| grid.iter().map(|row| row[c].clone()).collect())
            .collect();
        let mut seen_cols: Vec<Vec<Value>> = Vec::new();
        let mut counts: Vec<usize> = Vec::new();
        for col in columns {
            if let Some(pos) = seen_cols.iter().position(|sc| sc == &col) {
                counts[pos] += 1;
            } else {
                seen_cols.push(col);
                counts.push(1);
            }
        }
        let result_cols: Vec<Vec<Value>> = seen_cols
            .into_iter()
            .zip(counts)
            .filter(|(_, cnt)| !exactly_once || *cnt == 1)
            .map(|(col, _)| col)
            .collect();
        // Transpose back to row-major
        let ncols2 = result_cols.len();
        let result: Vec<Vec<Value>> = (0..nrows)
            .map(|r| (0..ncols2).map(|c| result_cols[c][r].clone()).collect())
            .collect();
        return from_2d(result);
    }

    // Deduplicate rows
    let mut seen_rows: Vec<Vec<Value>> = Vec::new();
    let mut counts: Vec<usize> = Vec::new();
    for row in &grid {
        if let Some(pos) = seen_rows.iter().position(|sr| sr == row) {
            counts[pos] += 1;
        } else {
            seen_rows.push(row.clone());
            counts.push(1);
        }
    }
    let result: Vec<Vec<Value>> = seen_rows
        .into_iter()
        .zip(counts)
        .filter(|(_, cnt)| !exactly_once || *cnt == 1)
        .map(|(row, _)| row)
        .collect();
    from_2d(result)
}

// ── SUMPRODUCT ────────────────────────────────────────────────────────────────

pub(crate) fn sumproduct_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, usize::MAX) {
        return e;
    }
    let arrays: Vec<Vec<Value>> = args.iter().map(flatten_val).collect();
    let len = arrays[0].len();
    // All arrays must have the same length
    for arr in &arrays[1..] {
        if arr.len() != len {
            return Value::Error(ErrorKind::Value);
        }
    }
    let mut sum = 0.0;
    for i in 0..len {
        let mut prod = 1.0;
        for arr in &arrays {
            match to_f64(&arr[i]) {
                Some(n) => prod *= n,
                None => return Value::Error(ErrorKind::Value),
            }
        }
        sum += prod;
    }
    Value::Number(sum)
}

// ── SUMXMY2 ───────────────────────────────────────────────────────────────────

fn sumxmy2_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let xs = flatten_val(&args[0]);
    let ys = flatten_val(&args[1]);
    if xs.len() != ys.len() {
        return Value::Error(ErrorKind::Value);
    }
    let mut sum = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let xn = match to_f64(x) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        let yn = match to_f64(y) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        sum += (xn - yn).powi(2);
    }
    Value::Number(sum)
}

// ── SUMX2MY2 ──────────────────────────────────────────────────────────────────

fn sumx2my2_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let xs = flatten_val(&args[0]);
    let ys = flatten_val(&args[1]);
    if xs.len() != ys.len() {
        return Value::Error(ErrorKind::Value);
    }
    let mut sum = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let xn = match to_f64(x) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        let yn = match to_f64(y) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        sum += xn * xn - yn * yn;
    }
    Value::Number(sum)
}

// ── SUMX2PY2 ──────────────────────────────────────────────────────────────────

fn sumx2py2_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let xs = flatten_val(&args[0]);
    let ys = flatten_val(&args[1]);
    if xs.len() != ys.len() {
        return Value::Error(ErrorKind::Value);
    }
    let mut sum = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let xn = match to_f64(x) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        let yn = match to_f64(y) { Some(n) => n, None => return Value::Error(ErrorKind::Value) };
        sum += xn * xn + yn * yn;
    }
    Value::Number(sum)
}

// ── MMULT ─────────────────────────────────────────────────────────────────────

fn mmult_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let a = to_2d(&args[0]);
    let b = to_2d(&args[1]);
    let n = a.first().map(|r| r.len()).unwrap_or(0);
    let p = b.first().map(|r| r.len()).unwrap_or(0);
    if b.len() != n {
        return Value::Error(ErrorKind::Value);
    }
    // Convert to f64 matrices for computation
    let af: Vec<Vec<f64>> = a.iter().map(|row| {
        row.iter().map(|v| to_f64(v).unwrap_or(f64::NAN)).collect()
    }).collect();
    let bf: Vec<Vec<f64>> = b.iter().map(|row| {
        row.iter().map(|v| to_f64(v).unwrap_or(f64::NAN)).collect()
    }).collect();
    if af.iter().any(|r| r.iter().any(|v| v.is_nan())) || bf.iter().any(|r| r.iter().any(|v| v.is_nan())) {
        return Value::Error(ErrorKind::Value);
    }
    let result: Vec<Vec<Value>> = af.iter().map(|row_a| {
        (0..p).map(|j| {
            let sum: f64 = row_a.iter().enumerate().map(|(k, &av)| av * bf[k][j]).sum();
            Value::Number(sum)
        }).collect()
    }).collect();
    from_2d(result)
}

// ── MDETERM ───────────────────────────────────────────────────────────────────

fn mdeterm_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let n = grid.len();
    if n == 0 {
        return Value::Error(ErrorKind::Value);
    }
    for row in &grid {
        if row.len() != n {
            return Value::Error(ErrorKind::Value);
        }
    }
    // Convert to f64 matrix
    let mut mat: Vec<Vec<f64>> = Vec::with_capacity(n);
    for row in &grid {
        let mut r = Vec::with_capacity(n);
        for v in row {
            match to_f64(v) {
                Some(x) => r.push(x),
                None => return Value::Error(ErrorKind::Value),
            }
        }
        mat.push(r);
    }
    Value::Number(determinant(&mat))
}

fn determinant(mat: &[Vec<f64>]) -> f64 {
    let n = mat.len();
    if n == 1 {
        return mat[0][0];
    }
    if n == 2 {
        return mat[0][0] * mat[1][1] - mat[0][1] * mat[1][0];
    }
    let mut det = 0.0;
    for c in 0..n {
        let minor: Vec<Vec<f64>> = (1..n)
            .map(|r| {
                (0..n)
                    .filter(|&cc| cc != c)
                    .map(|cc| mat[r][cc])
                    .collect()
            })
            .collect();
        let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
        det += sign * mat[0][c] * determinant(&minor);
    }
    det
}

// ── MINVERSE ──────────────────────────────────────────────────────────────────

fn minverse_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let grid = to_2d(&args[0]);
    let n = grid.len();
    if n == 0 {
        return Value::Error(ErrorKind::Value);
    }
    for row in &grid {
        if row.len() != n {
            return Value::Error(ErrorKind::Value);
        }
    }
    let mut mat: Vec<Vec<f64>> = Vec::with_capacity(n);
    for row in &grid {
        let mut r = Vec::with_capacity(n);
        for v in row {
            match to_f64(v) {
                Some(x) => r.push(x),
                None => return Value::Error(ErrorKind::Value),
            }
        }
        mat.push(r);
    }
    match invert_matrix(mat) {
        Some(inv) => from_2d(inv.into_iter().map(|r| r.into_iter().map(Value::Number).collect()).collect()),
        None => Value::Error(ErrorKind::Num),
    }
}

fn invert_matrix(mut mat: Vec<Vec<f64>>) -> Option<Vec<Vec<f64>>> {
    let n = mat.len();
    // Augment with identity
    let mut inv: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for col in 0..n {
        // Find pivot
        let pivot = (col..n).max_by(|&a, &b| mat[a][col].abs().partial_cmp(&mat[b][col].abs()).unwrap_or(std::cmp::Ordering::Equal))?;
        if mat[pivot][col].abs() < 1e-12 {
            return None; // singular
        }
        mat.swap(col, pivot);
        inv.swap(col, pivot);
        let div = mat[col][col];
        for j in 0..n {
            mat[col][j] /= div;
            inv[col][j] /= div;
        }
        for r in 0..n {
            if r != col {
                let factor = mat[r][col];
                for j in 0..n {
                    mat[r][j] -= factor * mat[col][j];
                    inv[r][j] -= factor * inv[col][j];
                }
            }
        }
    }
    Some(inv)
}

// ── FREQUENCY ─────────────────────────────────────────────────────────────────
// Array-spill function; Google Sheets returns #REF! in scalar (non-array-formula) context.

fn frequency_fn(_args: &[Value]) -> Value {
    Value::Error(ErrorKind::Ref)
}

// ── LINEST ────────────────────────────────────────────────────────────────────
// LINEST(known_y, [known_x], [const], [stats]) → returns 1-row array [slope, intercept, ...]

fn linest_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 4) {
        return e;
    }
    let ys = flatten_val(&args[0]);
    let n = ys.len();
    let xs: Vec<f64> = if args.len() >= 2 {
        flatten_val(&args[1]).iter().filter_map(to_f64).collect()
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    if xs.len() != n || n < 2 {
        return Value::Error(ErrorKind::Value);
    }
    let y_vals: Vec<f64> = ys.iter().filter_map(to_f64).collect();
    if y_vals.len() != n {
        return Value::Error(ErrorKind::Value);
    }
    let (slope, intercept) = simple_linear_regression(&xs, &y_vals);
    Value::Array(vec![Value::Number(slope), Value::Number(intercept)])
}

fn simple_linear_regression(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let sum_x: f64 = xs.iter().sum();
    let sum_y: f64 = ys.iter().sum();
    let sum_xy: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum();
    let sum_xx: f64 = xs.iter().map(|x| x * x).sum();
    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        let intercept = sum_y / n;
        return (0.0, intercept);
    }
    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;
    (slope, intercept)
}

// ── LOGEST ────────────────────────────────────────────────────────────────────
// LOGEST(known_y, [known_x], [const], [stats]) → returns 1-row array [base, intercept, ...]

fn logest_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 4) {
        return e;
    }
    let ys = flatten_val(&args[0]);
    let n = ys.len();
    let xs: Vec<f64> = if args.len() >= 2 {
        flatten_val(&args[1]).iter().filter_map(to_f64).collect()
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    if xs.len() != n || n < 2 {
        return Value::Error(ErrorKind::Value);
    }
    let y_vals: Vec<f64> = ys.iter().filter_map(to_f64).collect();
    if y_vals.len() != n {
        return Value::Error(ErrorKind::Value);
    }
    // Take log of y values
    let log_y: Vec<f64> = y_vals.iter().map(|&y| y.ln()).collect();
    if log_y.iter().any(|v| v.is_nan() || v.is_infinite()) {
        return Value::Error(ErrorKind::Num);
    }
    let (log_base, log_intercept) = simple_linear_regression(&xs, &log_y);
    let base = log_base.exp();
    let intercept = log_intercept.exp();
    Value::Array(vec![Value::Number(base), Value::Number(intercept)])
}

// ── TREND ─────────────────────────────────────────────────────────────────────
// TREND(known_y, [known_x], [new_x], [const]) → array of fitted/predicted values

fn trend_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 4) {
        return e;
    }
    let ys = flatten_val(&args[0]);
    let n = ys.len();
    let xs: Vec<f64> = if args.len() >= 2 {
        flatten_val(&args[1]).iter().filter_map(to_f64).collect()
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    if xs.len() != n || n < 2 {
        return Value::Error(ErrorKind::Value);
    }
    let y_vals: Vec<f64> = ys.iter().filter_map(to_f64).collect();
    if y_vals.len() != n {
        return Value::Error(ErrorKind::Value);
    }
    let new_xs: Vec<f64> = if args.len() >= 3 {
        flatten_val(&args[2]).iter().filter_map(to_f64).collect()
    } else {
        xs.clone()
    };
    let (slope, intercept) = simple_linear_regression(&xs, &y_vals);
    let result: Vec<Value> = new_xs.iter().map(|&x| Value::Number(slope * x + intercept)).collect();
    Value::Array(result)
}

// ── GROWTH ────────────────────────────────────────────────────────────────────
// GROWTH(known_y, [known_x], [new_x], [const]) → exponential predictions

fn growth_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 4) {
        return e;
    }
    let ys = flatten_val(&args[0]);
    let n = ys.len();
    let xs: Vec<f64> = if args.len() >= 2 {
        flatten_val(&args[1]).iter().filter_map(to_f64).collect()
    } else {
        (1..=n).map(|i| i as f64).collect()
    };
    if xs.len() != n || n < 2 {
        return Value::Error(ErrorKind::Value);
    }
    let y_vals: Vec<f64> = ys.iter().filter_map(to_f64).collect();
    if y_vals.len() != n {
        return Value::Error(ErrorKind::Value);
    }
    let log_y: Vec<f64> = y_vals.iter().map(|&y| y.ln()).collect();
    if log_y.iter().any(|v| v.is_nan() || v.is_infinite()) {
        return Value::Error(ErrorKind::Num);
    }
    let new_xs: Vec<f64> = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        let vals: Vec<f64> = flatten_val(&args[2]).iter().filter_map(to_f64).collect();
        if vals.is_empty() { xs.clone() } else { vals }
    } else {
        xs.clone()
    };
    // Ignore b param (args[3]) — not fully implemented; flag error if b=FALSE
    if args.len() >= 4 {
        let b_false = match &args[3] {
            Value::Bool(b) => !b,
            Value::Number(n) => *n == 0.0,
            _ => false,
        };
        if b_false {
            return Value::Error(ErrorKind::Value);
        }
    }
    let (log_base, log_intercept) = simple_linear_regression(&xs, &log_y);
    let result: Vec<Value> = new_xs
        .iter()
        .map(|&x| Value::Number((log_base * x + log_intercept).exp()))
        .collect();
    Value::Array(result)
}

// ── Higher-order functions (LazyFn) ───────────────────────────────────────────

/// Apply a LAMBDA expression with bound parameter values.
/// `lambda_expr` should be `Expr::FunctionCall { name: "LAMBDA", args: [p1, ..., body] }`
/// `bound_args` are the Values to bind to p1, p2, ...
fn apply_lambda(lambda_expr: &Expr, bound_args: &[Value], ctx: &mut EvalCtx<'_>) -> Option<Value> {
    match lambda_expr {
        Expr::FunctionCall { name, args, .. } if name == "LAMBDA" => {
            if args.is_empty() {
                return None;
            }
            let body = &args[args.len() - 1];
            let params = &args[..args.len() - 1];
            if params.len() != bound_args.len() {
                return None;
            }
            // Bind each parameter in context
            let mut saved: Vec<(String, Value)> = Vec::new();
            for (param_expr, val) in params.iter().zip(bound_args.iter()) {
                if let Expr::Variable(name, _) = param_expr {
                    let old = ctx.ctx.get(name);
                    saved.push((name.clone(), old));
                    ctx.ctx.set(name.clone(), val.clone());
                } else {
                    return None;
                }
            }
            let result = evaluate_expr(body, ctx);
            // Restore context
            for (name, old_val) in saved {
                ctx.ctx.set(name, old_val);
            }
            Some(result)
        }
        _ => None,
    }
}

// ── BYROW ─────────────────────────────────────────────────────────────────────

pub fn byrow_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 2, 2) {
        return e;
    }
    let arr_val = evaluate_expr(&args[0], ctx);
    if matches!(arr_val, Value::Error(_)) {
        return arr_val;
    }
    let grid = to_2d(&arr_val);
    let lambda_expr = &args[1];
    let mut results: Vec<Value> = Vec::with_capacity(grid.len());
    for row in &grid {
        let row_val = Value::Array(row.clone());
        match apply_lambda(lambda_expr, &[row_val], ctx) {
            Some(v) => results.push(v),
            None => return Value::Error(ErrorKind::Value),
        }
    }
    // Return as column vector (one result per row)
    let col: Vec<Vec<Value>> = results.into_iter().map(|v| vec![v]).collect();
    from_2d(col)
}

// ── BYCOL ─────────────────────────────────────────────────────────────────────

pub fn bycol_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 2, 2) {
        return e;
    }
    let arr_val = evaluate_expr(&args[0], ctx);
    if matches!(arr_val, Value::Error(_)) {
        return arr_val;
    }
    let grid = to_2d(&arr_val);
    let ncols = grid.first().map(|r| r.len()).unwrap_or(0);
    // Build columns first to avoid range-loop indexing
    let columns: Vec<Vec<Value>> = (0..ncols)
        .map(|c| grid.iter().map(|row| row[c].clone()).collect())
        .collect();
    let lambda_expr = &args[1];
    let mut results: Vec<Value> = Vec::with_capacity(ncols);
    for col in columns {
        // Pass flat array so SUM/MAX/MIN etc can iterate over elements
        let col_val = Value::Array(col);
        match apply_lambda(lambda_expr, &[col_val], ctx) {
            Some(v) => results.push(v),
            None => return Value::Error(ErrorKind::Value),
        }
    }
    // Return as row vector (one result per col)
    Value::Array(results)
}

// ── MAP ───────────────────────────────────────────────────────────────────────

pub fn map_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 2, usize::MAX) {
        return e;
    }
    // Last arg is LAMBDA, all prior are arrays
    let lambda_expr = &args[args.len() - 1];
    let arr_count = args.len() - 1;
    let arrays: Vec<Vec<Value>> = args[..arr_count]
        .iter()
        .map(|a| {
            let v = evaluate_expr(a, ctx);
            flatten_val(&v)
        })
        .collect();
    let len = arrays[0].len();
    for arr in &arrays[1..] {
        if arr.len() != len {
            return Value::Error(ErrorKind::Value);
        }
    }
    let mut results: Vec<Value> = Vec::with_capacity(len);
    for i in 0..len {
        let bound: Vec<Value> = arrays.iter().map(|a| a[i].clone()).collect();
        match apply_lambda(lambda_expr, &bound, ctx) {
            Some(v) => results.push(v),
            None => return Value::Error(ErrorKind::Value),
        }
    }
    // Preserve shape of first array
    let first_grid = to_2d(&evaluate_expr(&args[0], ctx));
    if first_grid.len() > 1 {
        // 2D → reshape results
        let ncols = first_grid[0].len();
        let nrows = first_grid.len();
        let grid: Vec<Vec<Value>> = (0..nrows)
            .map(|r| (0..ncols).map(|c| results[r * ncols + c].clone()).collect())
            .collect();
        from_2d(grid)
    } else {
        Value::Array(results)
    }
}

// ── REDUCE ────────────────────────────────────────────────────────────────────

pub fn reduce_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 3, 3) {
        return e;
    }
    let initial = evaluate_expr(&args[0], ctx);
    if matches!(initial, Value::Error(_)) {
        return initial;
    }
    let arr_val = evaluate_expr(&args[1], ctx);
    if matches!(arr_val, Value::Error(_)) {
        return arr_val;
    }
    let items = flatten_val(&arr_val);
    let lambda_expr = &args[2];
    let mut acc = initial;
    for item in &items {
        match apply_lambda(lambda_expr, &[acc.clone(), item.clone()], ctx) {
            Some(v) => acc = v,
            None => return Value::Error(ErrorKind::Value),
        }
    }
    acc
}

// ── SCAN ──────────────────────────────────────────────────────────────────────

pub fn scan_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 3, 3) {
        return e;
    }
    let initial = evaluate_expr(&args[0], ctx);
    if matches!(initial, Value::Error(_)) {
        return initial;
    }
    let arr_val = evaluate_expr(&args[1], ctx);
    if matches!(arr_val, Value::Error(_)) {
        return arr_val;
    }
    let grid = to_2d(&arr_val);
    let items = flatten_val(&arr_val);
    let lambda_expr = &args[2];
    let mut acc = initial;
    let mut results: Vec<Value> = Vec::with_capacity(items.len());
    for item in &items {
        match apply_lambda(lambda_expr, &[acc.clone(), item.clone()], ctx) {
            Some(v) => {
                acc = v.clone();
                results.push(v);
            }
            None => return Value::Error(ErrorKind::Value),
        }
    }
    // Preserve shape of input array
    if grid.len() > 1 {
        let ncols = grid[0].len();
        let nrows = grid.len();
        let result_grid: Vec<Vec<Value>> = (0..nrows)
            .map(|r| (0..ncols).map(|c| results[r * ncols + c].clone()).collect())
            .collect();
        from_2d(result_grid)
    } else {
        Value::Array(results)
    }
}

// ── MAKEARRAY ─────────────────────────────────────────────────────────────────

pub fn makearray_lazy_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 3, 3) {
        return e;
    }
    let rows_val = evaluate_expr(&args[0], ctx);
    let cols_val = evaluate_expr(&args[1], ctx);
    if matches!(rows_val, Value::Error(_)) {
        return rows_val;
    }
    if matches!(cols_val, Value::Error(_)) {
        return cols_val;
    }
    let nrows = match to_f64(&rows_val) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let ncols = match to_f64(&cols_val) {
        Some(n) if n >= 1.0 => n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let lambda_expr = &args[2];
    let mut grid: Vec<Vec<Value>> = Vec::with_capacity(nrows);
    for r in 1..=nrows {
        let mut row = Vec::with_capacity(ncols);
        for c in 1..=ncols {
            let rv = Value::Number(r as f64);
            let cv = Value::Number(c as f64);
            match apply_lambda(lambda_expr, &[rv, cv], ctx) {
                Some(v) => row.push(v),
                None => return Value::Error(ErrorKind::Value),
            }
        }
        grid.push(row);
    }
    from_2d(grid)
}

// ── Registration ─────────────────────────────────────────────────────────────

pub fn register_array(registry: &mut Registry) {
    registry.register_eager("ROWS", rows_fn, FunctionMeta {
        category: "array",
        signature: "ROWS(array)",
        description: "Returns the number of rows in an array or range",
    });
    registry.register_eager("COLUMNS", columns_fn, FunctionMeta {
        category: "array",
        signature: "COLUMNS(array)",
        description: "Returns the number of columns in an array or range",
    });
    registry.register_eager("INDEX", index_fn, FunctionMeta {
        category: "array",
        signature: "INDEX(array, row, [col])",
        description: "Returns the value at the given row and column of an array",
    });
    registry.register_eager("TRANSPOSE", transpose_fn, FunctionMeta {
        category: "array",
        signature: "TRANSPOSE(array)",
        description: "Transposes the rows and columns of an array",
    });
    registry.register_eager("ARRAY_CONSTRAIN", array_constrain_fn, FunctionMeta {
        category: "array",
        signature: "ARRAY_CONSTRAIN(input, num_rows, num_cols)",
        description: "Constrains an array to a given number of rows and columns",
    });
    registry.register_eager("CHOOSECOLS", choosecols_fn, FunctionMeta {
        category: "array",
        signature: "CHOOSECOLS(array, col_num1, ...)",
        description: "Returns selected columns from an array",
    });
    registry.register_eager("CHOOSEROWS", chooserows_fn, FunctionMeta {
        category: "array",
        signature: "CHOOSEROWS(array, row_num1, ...)",
        description: "Returns selected rows from an array",
    });
    registry.register_eager("FLATTEN", flatten_fn, FunctionMeta {
        category: "array",
        signature: "FLATTEN(array)",
        description: "Flattens an array into a single column",
    });
    registry.register_eager("HSTACK", hstack_fn, FunctionMeta {
        category: "array",
        signature: "HSTACK(array1, ...)",
        description: "Horizontally stacks arrays",
    });
    registry.register_eager("VSTACK", vstack_fn, FunctionMeta {
        category: "array",
        signature: "VSTACK(array1, ...)",
        description: "Vertically stacks arrays",
    });
    registry.register_eager("TOCOL", tocol_fn, FunctionMeta {
        category: "array",
        signature: "TOCOL(array, [ignore], [scan_by_col])",
        description: "Converts an array to a single column",
    });
    registry.register_eager("TOROW", torow_fn, FunctionMeta {
        category: "array",
        signature: "TOROW(array, [ignore], [scan_by_col])",
        description: "Converts an array to a single row",
    });
    registry.register_eager("WRAPCOLS", wrapcols_fn, FunctionMeta {
        category: "array",
        signature: "WRAPCOLS(vector, wrap_count, [pad_with])",
        description: "Wraps a vector into columns of the given length",
    });
    registry.register_eager("WRAPROWS", wraprows_fn, FunctionMeta {
        category: "array",
        signature: "WRAPROWS(vector, wrap_count, [pad_with])",
        description: "Wraps a vector into rows of the given length",
    });
    registry.register_eager("SORT", sort_fn, FunctionMeta {
        category: "array",
        signature: "SORT(array, [sort_index], [sort_order], [by_col])",
        description: "Sorts an array",
    });
    registry.register_eager("SORTBY", sortby_fn, FunctionMeta {
        category: "array",
        signature: "SORTBY(array, by_array1, [sort_order1], ...)",
        description: "Sorts an array based on the values in corresponding arrays",
    });
    registry.register_eager("UNIQUE", unique_fn, FunctionMeta {
        category: "array",
        signature: "UNIQUE(array, [by_col], [exactly_once])",
        description: "Returns unique rows or columns from an array",
    });
    registry.register_eager("SUMPRODUCT", sumproduct_fn, FunctionMeta {
        category: "array",
        signature: "SUMPRODUCT(array1, [array2], ...)",
        description: "Returns the sum of products of corresponding elements",
    });
    registry.register_eager("SUMXMY2", sumxmy2_fn, FunctionMeta {
        category: "array",
        signature: "SUMXMY2(array_x, array_y)",
        description: "Returns sum of squares of differences",
    });
    registry.register_eager("SUMX2MY2", sumx2my2_fn, FunctionMeta {
        category: "array",
        signature: "SUMX2MY2(array_x, array_y)",
        description: "Returns sum of (x^2 - y^2)",
    });
    registry.register_eager("SUMX2PY2", sumx2py2_fn, FunctionMeta {
        category: "array",
        signature: "SUMX2PY2(array_x, array_y)",
        description: "Returns sum of (x^2 + y^2)",
    });
    registry.register_eager("MMULT", mmult_fn, FunctionMeta {
        category: "array",
        signature: "MMULT(array1, array2)",
        description: "Returns the matrix product of two arrays",
    });
    registry.register_eager("MDETERM", mdeterm_fn, FunctionMeta {
        category: "array",
        signature: "MDETERM(array)",
        description: "Returns the matrix determinant",
    });
    registry.register_eager("MINVERSE", minverse_fn, FunctionMeta {
        category: "array",
        signature: "MINVERSE(array)",
        description: "Returns the matrix inverse",
    });
    registry.register_eager("FREQUENCY", frequency_fn, FunctionMeta {
        category: "array",
        signature: "FREQUENCY(data, bins)",
        description: "Calculates the frequency distribution of values",
    });
    registry.register_eager("LINEST", linest_fn, FunctionMeta {
        category: "array",
        signature: "LINEST(known_y, [known_x], [const], [stats])",
        description: "Returns linear regression statistics",
    });
    registry.register_eager("LOGEST", logest_fn, FunctionMeta {
        category: "array",
        signature: "LOGEST(known_y, [known_x], [const], [stats])",
        description: "Returns exponential regression statistics",
    });
    registry.register_eager("TREND", trend_fn, FunctionMeta {
        category: "array",
        signature: "TREND(known_y, [known_x], [new_x], [const])",
        description: "Returns values along a linear trend",
    });
    registry.register_eager("GROWTH", growth_fn, FunctionMeta {
        category: "array",
        signature: "GROWTH(known_y, [known_x], [new_x], [const])",
        description: "Returns values along an exponential trend",
    });
    registry.register_lazy("BYROW", byrow_lazy_fn, FunctionMeta {
        category: "array",
        signature: "BYROW(array, lambda)",
        description: "Applies a LAMBDA to each row of an array",
    });
    registry.register_lazy("BYCOL", bycol_lazy_fn, FunctionMeta {
        category: "array",
        signature: "BYCOL(array, lambda)",
        description: "Applies a LAMBDA to each column of an array",
    });
    registry.register_lazy("MAP", map_lazy_fn, FunctionMeta {
        category: "array",
        signature: "MAP(array1, [array2, ...], lambda)",
        description: "Maps a LAMBDA over one or more arrays",
    });
    registry.register_lazy("REDUCE", reduce_lazy_fn, FunctionMeta {
        category: "array",
        signature: "REDUCE(initial_value, array, lambda)",
        description: "Reduces an array to a single value using a LAMBDA",
    });
    registry.register_lazy("SCAN", scan_lazy_fn, FunctionMeta {
        category: "array",
        signature: "SCAN(initial_value, array, lambda)",
        description: "Returns running accumulation using a LAMBDA",
    });
    registry.register_lazy("MAKEARRAY", makearray_lazy_fn, FunctionMeta {
        category: "array",
        signature: "MAKEARRAY(rows, cols, lambda)",
        description: "Creates an array using a LAMBDA for each cell value",
    });
}

#[cfg(test)]
mod tests;
