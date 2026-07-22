//! `QUERY(data, query, [headers])` — run a small pseudo-SQL query language
//! over a 2D array/range, per Google Sheets' documented QUERY semantics.
//!
//! ## Implemented clauses
//!
//! `select`, `where`, `group by`, `order by`, `limit`, `label`.
//!
//! - `select`: bare column references (`Col1`, `Col2`, ...) and the
//!   aggregate functions `sum`, `count`, `avg`/`average`, `max`, `min`
//!   applied to a single column (e.g. `sum(Col2)`). No arithmetic
//!   expressions, no `select *`.
//! - `where`: comparisons (`=`, `!=`/`<>`, `<`, `<=`, `>`, `>=`) and
//!   `is null` / `is not null`, combined with a single, uniform `and` or
//!   `or` (mixing both, and parenthesised grouping, is not supported).
//!   The right-hand side is a literal (quoted text, number, or `true`/
//!   `false`) — column-to-column comparisons are not supported.
//! - `group by`: one or more bare columns; every non-aggregated `select`
//!   column must be a `group by` key (Google Sheets' own rule).
//! - `order by`: one or more columns with optional `asc`/`desc`.
//! - `limit`: a row cap applied after ordering.
//! - `label`: overrides the default output header text for a `select`
//!   item, e.g. `label Col2 'Total'`.
//!
//! ## Explicitly deferred (not implemented in this PR)
//!
//! - `format` and `options` clauses (issue #760 calls these out as an
//!   acceptable follow-up).
//! - `pivot`.
//! - `offset`.
//! - Arithmetic/string expressions in `select` or `where` beyond a bare
//!   column or a single-column aggregate.
//! - Mixed `and`/`or` precedence and parenthesised grouping in `where`.
//! - Column-to-column comparisons in `where`.
//! - Date/time literals in `where`.
//! - Plain spreadsheet column letters (`A`, `B`, ...) as identifiers —
//!   only `ColN` is supported (see the parser module doc for why: this
//!   function only ever sees a plain evaluated array, never the original
//!   cell coordinates a live range would carry).
//!
//! ## Header rows and the output header
//!
//! `headers` (3rd argument, default `0`) is the count of leading rows in
//! `data` treated as header rows and excluded from the query. Only the
//! *first* header row is used as the source of default output labels; any
//! further header rows are still excluded from the data but otherwise
//! ignored — Google Sheets' fuller multi-header-row label-merging behavior
//! is not replicated.
//!
//! A result header row is emitted iff `headers >= 1` **or** a `label`
//! clause is present. This is a deliberate simplification: real Google
//! Sheets applies a header-count auto-guess when `headers` is omitted,
//! which is not implemented here (omitting `headers` behaves exactly like
//! passing `0`) — see issue #760's scope note. Passing `headers` explicitly
//! is recommended for header-aware queries.

mod exec;
mod parser;

use super::super::{FunctionMeta, Registry};
use super::check_arity;
use crate::eval::functions::lookup::array_utils::flatten_to_rows;
use crate::types::{ErrorKind, Value};

pub fn query_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let query_str = match &args[1] {
        Value::Text(s) => s.clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    let headers: usize = match args.get(2) {
        None | Some(Value::Empty) => 0,
        Some(Value::Number(n)) if *n >= 0.0 => *n as usize,
        _ => return Value::Error(ErrorKind::Value),
    };

    let rows = flatten_to_rows(&args[0]);
    if rows.is_empty() || headers > rows.len() {
        return Value::Error(ErrorKind::Value);
    }
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return Value::Error(ErrorKind::Value);
    }

    let pad = |mut r: Vec<Value>| -> Vec<Value> {
        r.resize(ncols, Value::Empty);
        r
    };
    let mut all_rows: Vec<Vec<Value>> = rows.into_iter().map(pad).collect();
    let data_rows: Vec<Vec<Value>> = all_rows.split_off(headers);
    let header_row: Option<Vec<Value>> = if headers >= 1 { all_rows.into_iter().next() } else { None };

    let parsed = match parser::parse(&query_str, ncols) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let result_rows = match exec::execute(&parsed, &data_rows) {
        Ok(r) => r,
        Err(e) => return e,
    };

    let emit_header = headers >= 1 || !parsed.labels.is_empty();
    let mut out_rows: Vec<Value> = Vec::with_capacity(result_rows.len() + 1);
    if emit_header {
        out_rows.push(Value::Array(exec::build_header(&parsed, header_row.as_deref())));
    }
    out_rows.extend(result_rows.into_iter().map(Value::Array));

    if out_rows.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    Value::Array(out_rows)
}

#[cfg(test)]
mod tests;

pub fn register_query(registry: &mut Registry) {
    registry.register_eager(
        "QUERY",
        query_fn,
        FunctionMeta {
            category: "query",
            signature: "QUERY(data, query, [headers])",
            description: "Run a pseudo-SQL query (select/where/group by/order by/limit/label) over a range",
        },
    );
}
