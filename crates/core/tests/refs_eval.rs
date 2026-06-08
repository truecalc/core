//! P1.2 reference grammar — evaluation plumbing tests (issue #524).
//!
//! Resolution stays delegated (core = language): a sheet-qualified reference
//! evaluates by looking up its canonical text (e.g. `Data!A1`,
//! `'Quoted Name'!A1:B2`) in the caller-supplied variables, exactly as bare
//! `A1` / `A1:D4` variables always have. Real workbook semantics (e.g.
//! `#REF!` for a missing sheet) arrive with the P1.3 resolver and are
//! pipeline-verified by the P1.5 workbook fixtures; nothing here
//! self-confirms Google Sheets values.

use std::collections::HashMap;
use truecalc_core::{Engine, Value};

fn eval(formula: &str, vars: &[(&str, Value)]) -> Value {
    let map: HashMap<String, Value> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    Engine::sheets().evaluate(formula, &map)
}

#[test]
fn sheet_qualified_cell_resolves_from_variables() {
    let v = eval("=Data!A1", &[("Data!A1", Value::Number(10.0))]);
    assert_eq!(v, Value::Number(10.0));
}

#[test]
fn sheet_qualified_refs_in_arithmetic() {
    let v = eval(
        "=Data!A1+Data!B1",
        &[
            ("Data!A1", Value::Number(10.0)),
            ("Data!B1", Value::Number(5.0)),
        ],
    );
    assert_eq!(v, Value::Number(15.0));
}

#[test]
fn lookup_is_case_insensitive_like_variables() {
    let v = eval("=data!a1", &[("DATA!A1", Value::Number(10.0))]);
    assert_eq!(v, Value::Number(10.0));
}

#[test]
fn quoted_sheet_resolves_by_canonical_text() {
    let v = eval(
        "='Quoted Name'!A1",
        &[("'Quoted Name'!A1", Value::Number(100.0))],
    );
    assert_eq!(v, Value::Number(100.0));
}

#[test]
fn optional_quotes_normalize_to_unquoted_canonical_form() {
    // 'Data' is a bare identifier, so its canonical form is unquoted: Data!A1.
    let v = eval("='Data'!A1", &[("Data!A1", Value::Number(10.0))]);
    assert_eq!(v, Value::Number(10.0));
}

#[test]
fn sheet_qualified_range_resolves_to_bound_array() {
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
    ]);
    let v = eval("=SUM(Data!A1:A3)", &[("Data!A1:A3", arr)]);
    assert_eq!(v, Value::Number(60.0));
}

#[test]
fn ref_in_if_condition() {
    let v = eval(
        "=IF(Data!A1>5,\"big\",\"small\")",
        &[("Data!A1", Value::Number(10.0))],
    );
    assert_eq!(v, Value::Text("big".into()));
}

#[test]
fn unbound_sheet_ref_is_empty_without_resolver() {
    // Same delegation contract as bare variables today: unbound → Empty.
    // Workbook semantics (#REF! for a missing sheet) land with the P1.3
    // resolver, pipeline-verified by the P1.5 workbook fixtures.
    let v = eval("=MissingSheet!A1", &[]);
    assert_eq!(v, Value::Empty);
}

#[test]
fn bare_variable_lookup_unchanged() {
    let v = eval("=A1", &[("A1", Value::Number(5.0))]);
    assert_eq!(v, Value::Number(5.0));
}

// ── structural functions on sheet-qualified refs ────────────────────────────

#[test]
fn isref_is_true_for_sheet_qualified_refs() {
    assert_eq!(eval("=ISREF(Data!A1)", &[]), Value::Bool(true));
    assert_eq!(eval("=ISREF('Quoted Name'!A1:B2)", &[]), Value::Bool(true));
}

#[test]
fn row_and_column_read_the_address() {
    assert_eq!(eval("=ROW(Sheet1!B7)", &[]), Value::Number(7.0));
    assert_eq!(eval("=COLUMN(Sheet1!B7)", &[]), Value::Number(2.0));
    assert_eq!(eval("=ROW(Data!A2:C5)", &[]), Value::Number(2.0));
    assert_eq!(eval("=COLUMN(Data!B2:C5)", &[]), Value::Number(2.0));
}

#[test]
fn rows_and_columns_match_bare_range_behavior() {
    // ROWS/COLUMNS are eager (content-based) functions today; a
    // sheet-qualified range behaves exactly like the equivalent bare range
    // (zero behavior change). Extent-from-address semantics arrive with the
    // P1.3 resolver and are pipeline-verified by the P1.5 fixtures.
    let arr = || {
        Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])
    };
    assert_eq!(
        eval("=ROWS(Data!A1:A3)", &[("Data!A1:A3", arr())]),
        eval("=ROWS(A1:A3)", &[("A1:A3", arr())]),
    );
    assert_eq!(
        eval("=COLUMNS(Data!A1:A3)", &[("Data!A1:A3", arr())]),
        eval("=COLUMNS(A1:A3)", &[("A1:A3", arr())]),
    );
    // Unbound refs also match bare-range behavior.
    assert_eq!(eval("=ROWS(Data!A2:C5)", &[]), eval("=ROWS(A2:C5)", &[]));
    assert_eq!(
        eval("=COLUMNS(Data!A2:C5)", &[]),
        eval("=COLUMNS(A2:C5)", &[]),
    );
}
