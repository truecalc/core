use super::super::query_fn;
use super::{employees, num, table, text};
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity() {
    assert_eq!(query_fn(&[]).error_kind(), Some(&ErrorKind::NA));
    assert_eq!(query_fn(&[employees()]).error_kind(), Some(&ErrorKind::NA));
    assert_eq!(query_fn(&[employees(), text("select Col1"), num(1.0), num(1.0)]).error_kind(), Some(&ErrorKind::NA));
}

#[test]
fn query_argument_must_be_text() {
    assert_eq!(query_fn(&[employees(), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn headers_greater_than_row_count_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1"), num(999.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn unsupported_select_expression_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1 + Col2"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn invalid_where_condition_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1 where Col2 nonsense"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn non_numeric_limit_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1 limit abc"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn mixing_bare_and_aggregate_without_group_by_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1, sum(Col3)"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn mixing_and_or_errors() {
    assert_eq!(
        query_fn(&[employees(), text("select Col1 where Col2 = 'Eng' and Col3 > 100 or Col3 < 60"), num(1.0)]).error_kind(),
        Some(&ErrorKind::Value)
    );
}

#[test]
fn select_column_not_in_group_by_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1, sum(Col3) group by Col2"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn order_by_column_not_grouped_or_aggregated_errors() {
    assert_eq!(
        query_fn(&[employees(), text("select Col2, sum(Col3) group by Col2 order by Col1"), num(1.0)]).error_kind(),
        Some(&ErrorKind::Value)
    );
}

#[test]
fn column_reference_out_of_range_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col9"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn zero_matching_rows_without_header_returns_na() {
    let result = query_fn(&[employees(), text("select Col1 where Col2 = 'Nonexistent'")]);
    assert_eq!(result.error_kind(), Some(&ErrorKind::NA));
}

#[test]
fn zero_matching_rows_with_header_returns_header_only() {
    let result = query_fn(&[employees(), text("select Col1 where Col2 = 'Nonexistent'"), num(1.0)]);
    assert_eq!(result, Value::Array(vec![Value::Array(vec![text("Name")])]));
}

#[test]
fn label_target_not_in_select_errors() {
    assert_eq!(query_fn(&[employees(), text("select Col1 label Col2 'X'"), num(1.0)]).error_kind(), Some(&ErrorKind::Value));
}

#[test]
fn empty_data_array_errors() {
    let empty = table(vec![]);
    assert_eq!(query_fn(&[empty, text("select Col1")]).error_kind(), Some(&ErrorKind::Value));
}
