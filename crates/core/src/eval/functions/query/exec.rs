//! Execution of a parsed QUERY against the (header-stripped, rectangular)
//! data rows: WHERE filter, GROUP BY aggregation, ORDER BY, LIMIT, SELECT
//! projection and LABEL/header construction.

use super::parser::{AggFunc, BoolJoin, CondOp, OrderItem, ParsedQuery, SelectItem, SortDir};
use crate::types::{ErrorKind, Value};

/// Type-aware ascending comparison for sorting/grouping. Same-type pairs
/// compare by value; cross-type pairs are treated as equal (stable sort
/// preserves their relative order) since QUERY's ORDER BY does not define a
/// cross-type ordering.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (Value::Empty, Value::Empty) => Ordering::Equal,
        _ => Ordering::Equal,
    }
}

fn is_empty_cell(v: &Value) -> bool {
    matches!(v, Value::Empty)
}

fn condition_matches(cond: &super::parser::Condition, row: &[Value]) -> bool {
    let cell = row.get(cond.col).unwrap_or(&Value::Empty);
    match cond.op {
        CondOp::IsNull => is_empty_cell(cell),
        CondOp::IsNotNull => !is_empty_cell(cell),
        CondOp::Eq | CondOp::Ne | CondOp::Lt | CondOp::Le | CondOp::Gt | CondOp::Ge => {
            let rhs = cond.value.as_ref().expect("comparison conditions always carry a value");
            match cond.op {
                CondOp::Eq => values_equal(cell, rhs),
                CondOp::Ne => !values_equal(cell, rhs),
                _ => {
                    let ord = match (cell, rhs) {
                        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y),
                        (Value::Text(x), Value::Text(y)) => Some(x.cmp(y)),
                        (Value::Bool(x), Value::Bool(y)) => Some(x.cmp(y)),
                        _ => None,
                    };
                    match ord {
                        Some(o) => match cond.op {
                            CondOp::Lt => o.is_lt(),
                            CondOp::Le => o.is_le(),
                            CondOp::Gt => o.is_gt(),
                            CondOp::Ge => o.is_ge(),
                            _ => unreachable!(),
                        },
                        None => false,
                    }
                }
            }
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Text(x), Value::Text(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

fn row_matches_where(where_clause: &Option<super::parser::WhereClause>, row: &[Value]) -> bool {
    match where_clause {
        None => true,
        Some(w) => {
            if w.conditions.is_empty() {
                return true;
            }
            match w.join {
                BoolJoin::And => w.conditions.iter().all(|c| condition_matches(c, row)),
                BoolJoin::Or => w.conditions.iter().any(|c| condition_matches(c, row)),
            }
        }
    }
}

fn compute_agg(func: AggFunc, col: usize, rows: &[&Vec<Value>]) -> Value {
    let nums: Vec<f64> = rows
        .iter()
        .filter_map(|r| match r.get(col) {
            Some(Value::Number(n)) => Some(*n),
            _ => None,
        })
        .collect();
    match func {
        AggFunc::Count => {
            let count = rows.iter().filter(|r| !matches!(r.get(col), None | Some(Value::Empty))).count();
            Value::Number(count as f64)
        }
        AggFunc::Sum => Value::Number(nums.iter().sum()),
        AggFunc::Avg => {
            if nums.is_empty() {
                Value::Error(ErrorKind::DivByZero)
            } else {
                Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        }
        AggFunc::Max => {
            if nums.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
            }
        }
        AggFunc::Min => {
            if nums.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(nums.iter().cloned().fold(f64::INFINITY, f64::min))
            }
        }
    }
}

/// Default output label for a select item when no LABEL override applies.
/// `header` is the corresponding column's header-row cell, when the caller
/// supplied one or more header rows.
fn default_label(item: SelectItem, header_row: Option<&[Value]>) -> String {
    let col_text = |c: usize| -> String {
        match header_row.and_then(|h| h.get(c)) {
            Some(Value::Text(s)) => s.clone(),
            Some(Value::Number(n)) => format!("{n}"),
            _ => format!("Col{}", c + 1),
        }
    };
    match item {
        SelectItem::Column(c) => col_text(c),
        SelectItem::Agg(func, c) => format!("{} {}", func.label_word(), col_text(c)),
    }
}

fn label_for(item: SelectItem, query: &ParsedQuery, header_row: Option<&[Value]>) -> String {
    query
        .labels
        .iter()
        .find(|(target, _)| *target == item)
        .map(|(_, text)| text.clone())
        .unwrap_or_else(|| default_label(item, header_row))
}

/// Run the parsed query over `data_rows` (already stripped of header rows).
/// Header text for output labels is handled separately by [`build_header`].
pub fn execute(query: &ParsedQuery, data_rows: &[Vec<Value>]) -> Result<Vec<Vec<Value>>, Value> {
    let filtered: Vec<&Vec<Value>> = data_rows.iter().filter(|row| row_matches_where(&query.where_clause, row)).collect();

    let has_agg = query.select.iter().any(|s| matches!(s, SelectItem::Agg(..)));

    let mut result_rows: Vec<Vec<Value>> = if !query.group_by.is_empty() {
        // GROUP BY: bucket rows by their group-by column values, in first-seen order.
        let mut groups: Vec<(Vec<Value>, Vec<&Vec<Value>>)> = Vec::new();
        for row in filtered.iter().copied() {
            let key: Vec<Value> = query.group_by.iter().map(|&c| row.get(c).cloned().unwrap_or(Value::Empty)).collect();
            let existing = groups
                .iter()
                .position(|(k, _)| k.len() == key.len() && k.iter().zip(&key).all(|(a, b)| values_equal(a, b)));
            match existing {
                Some(idx) => groups[idx].1.push(row),
                None => groups.push((key, vec![row])),
            }
        }
        // Default order: ascending by group-by key columns, left to right.
        groups.sort_by(|a, b| {
            for (x, y) in a.0.iter().zip(b.0.iter()) {
                let ord = compare_values(x, y);
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });

        let mut rows_out = Vec::with_capacity(groups.len());
        for (key, members) in &groups {
            let mut out_row = Vec::with_capacity(query.select.len());
            for item in &query.select {
                let val = match item {
                    SelectItem::Column(c) => {
                        let pos = query.group_by.iter().position(|g| g == c).expect("validated at parse time");
                        key[pos].clone()
                    }
                    SelectItem::Agg(func, c) => compute_agg(*func, *c, members),
                };
                out_row.push(val);
            }
            rows_out.push(out_row);
        }

        if !query.order_by.is_empty() {
            sort_grouped_rows(&mut rows_out, &groups, query)?;
        }
        rows_out
    } else if has_agg {
        // No GROUP BY but aggregates present: collapse to a single summary row
        // (parser guarantees every select item is an aggregate in this case).
        let out_row: Vec<Value> = query
            .select
            .iter()
            .map(|item| match item {
                SelectItem::Agg(func, c) => compute_agg(*func, *c, &filtered),
                SelectItem::Column(_) => unreachable!("parser forbids mixing bare columns with aggregates sans GROUP BY"),
            })
            .collect();
        vec![out_row]
    } else {
        let mut rows_sorted = filtered;
        if !query.order_by.is_empty() {
            sort_flat_rows(&mut rows_sorted, &query.order_by);
        }
        rows_sorted
            .into_iter()
            .map(|row| {
                query
                    .select
                    .iter()
                    .map(|item| match item {
                        SelectItem::Column(c) => row.get(*c).cloned().unwrap_or(Value::Empty),
                        SelectItem::Agg(..) => unreachable!("has_agg is false in this branch"),
                    })
                    .collect()
            })
            .collect()
    };

    if let Some(limit) = query.limit {
        result_rows.truncate(limit);
    }

    Ok(result_rows)
}

fn sort_flat_rows(rows: &mut [&Vec<Value>], order_by: &[OrderItem]) {
    // Stable-sort from the least-significant key to the most-significant so
    // the final pass (the first ORDER BY key) dominates.
    for item in order_by.iter().rev() {
        rows.sort_by(|a, b| {
            let va = a.get(item.col).unwrap_or(&Value::Empty);
            let vb = b.get(item.col).unwrap_or(&Value::Empty);
            let ord = compare_values(va, vb);
            if item.dir == SortDir::Desc { ord.reverse() } else { ord }
        });
    }
}

/// Sort already-projected grouped output rows by ORDER BY items. Each item
/// must reference either a GROUP BY column (read from the group key) or a
/// column that appears as an aggregate in SELECT (read from the projected
/// output at that select position) — the same restriction Google Sheets
/// applies to ORDER BY under GROUP BY.
fn sort_grouped_rows(rows_out: &mut [Vec<Value>], groups: &[(Vec<Value>, Vec<&Vec<Value>>)], query: &ParsedQuery) -> Result<(), Value> {
    // Precompute, for each ORDER BY item, how to read its sort key.
    enum KeySource {
        GroupKey(usize),
        OutputCol(usize),
    }
    let mut sources = Vec::with_capacity(query.order_by.len());
    for item in &query.order_by {
        if let Some(pos) = query.group_by.iter().position(|g| *g == item.col) {
            sources.push(KeySource::GroupKey(pos));
            continue;
        }
        if let Some(pos) = query.select.iter().position(|s| matches!(s, SelectItem::Agg(_, c) if *c == item.col)) {
            sources.push(KeySource::OutputCol(pos));
            continue;
        }
        return Err(Value::ErrorMsg(
            ErrorKind::Value,
            format!(
                "Unable to parse query string for Function QUERY parameter 2: ORDER BY Col{} must appear in GROUP BY or be aggregated",
                item.col + 1
            ),
        ));
    }

    let mut indices: Vec<usize> = (0..rows_out.len()).collect();
    for (item, source) in query.order_by.iter().zip(sources.iter()).rev() {
        indices.sort_by(|&i, &j| {
            let (va, vb) = match source {
                KeySource::GroupKey(pos) => (&groups[i].0[*pos], &groups[j].0[*pos]),
                KeySource::OutputCol(pos) => (&rows_out[i][*pos], &rows_out[j][*pos]),
            };
            let ord = compare_values(va, vb);
            if item.dir == SortDir::Desc { ord.reverse() } else { ord }
        });
    }

    let reordered: Vec<Vec<Value>> = indices.into_iter().map(|i| rows_out[i].clone()).collect();
    rows_out.clone_from_slice(&reordered);
    Ok(())
}

/// Build the header row (labels) for the result, when one should be emitted.
pub fn build_header(query: &ParsedQuery, header_row: Option<&[Value]>) -> Vec<Value> {
    query.select.iter().map(|&item| Value::Text(label_for(item, query, header_row))).collect()
}
