use super::super::*;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;
use std::collections::HashMap;

fn run(formula: &str, vars: HashMap<String, Value>) -> Value {
    crate::Engine::sheets().evaluate(formula, &vars)
}

fn span() -> Span { Span::new(0, 1) }

fn run_counta_lazy(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    counta_lazy_fn(&args, &mut ctx)
}

#[test]
fn counta_lazy_array_arg_counts_elements() {
    // COUNTA with an array argument flattens and counts non-empty elements
    let args = vec![Expr::Array(
        vec![
            Expr::Number(1.0, span()),
            Expr::Number(2.0, span()),
            Expr::Number(3.0, span()),
        ],
        span(),
    )];
    assert_eq!(run_counta_lazy(args), Value::Number(3.0));
}

#[test]
fn counta_lazy_array_counts_text_skips_empty() {
    // COUNTA counts Text values, skips Value::Empty — empty string Text is counted
    let args = vec![Expr::Array(
        vec![
            Expr::Text("a".to_string(), span()),
            Expr::Text("b".to_string(), span()),
        ],
        span(),
    )];
    assert_eq!(run_counta_lazy(args), Value::Number(2.0));
}

#[test]
fn count_no_args_returns_zero() {
    assert_eq!(count_fn(&[]), Value::Number(0.0));
}

#[test]
fn counta_no_args_returns_zero() {
    assert_eq!(counta_fn(&[]), Value::Number(0.0));
}

#[test]
fn count_mixed_ignores_non_numeric() {
    // COUNT(1, "text", TRUE, 3) → 2
    assert_eq!(
        count_fn(&[
            Value::Number(1.0),
            Value::Text("text".to_string()),
            Value::Bool(true),
            Value::Number(3.0)
        ]),
        Value::Number(2.0)
    );
}

#[test]
fn counta_mixed_counts_all_non_empty() {
    // COUNTA(1, "text", TRUE, 3) → 4
    assert_eq!(
        counta_fn(&[
            Value::Number(1.0),
            Value::Text("text".to_string()),
            Value::Bool(true),
            Value::Number(3.0)
        ]),
        Value::Number(4.0)
    );
}

#[test]
fn counta_empty_values_not_counted() {
    assert_eq!(
        counta_fn(&[Value::Empty, Value::Number(1.0), Value::Empty]),
        Value::Number(1.0)
    );
}

#[test]
fn count_array_variable_counts_numbers() {
    // COUNT with a variable holding an array → recursively counts numbers
    let vars: HashMap<_, _> = [(
        "A".to_string(),
        Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]),
    )]
    .into();
    assert_eq!(run("=COUNT(A)", vars), Value::Number(3.0));
}

#[test]
fn count_array_variable_skips_non_numeric() {
    // COUNT with mixed array — only Numbers counted (Bool skipped in array context)
    let vars: HashMap<_, _> = [(
        "A".to_string(),
        Value::Array(vec![
            Value::Number(1.0),
            Value::Text("hello".to_string()),
            Value::Bool(true),
            Value::Number(2.0),
            Value::Empty,
        ]),
    )]
    .into();
    assert_eq!(run("=COUNT(A)", vars), Value::Number(2.0));
}

/// Regression for #584: COUNT over a range/variable skips booleans exactly
/// like array-literal context (both paths use count_in_array).
#[test]
fn count_range_variable_skips_bool_and_text() {
    // Mirrors Data!A1:D3 from workbook conformance: [10, 5, "hello", true, 20, 20, ...] → 6
    let vars: HashMap<_, _> = [(
        "R".to_string(),
        Value::Array(vec![
            Value::Number(10.0),
            Value::Number(5.0),
            Value::Text("hello".to_string()),
            Value::Bool(true),
            Value::Number(20.0),
            Value::Number(20.0),
        ]),
    )]
    .into();
    assert_eq!(run("=COUNT(R)", vars), Value::Number(4.0));
}

/// Regression for #780: COUNT counts dates, as a direct argument and inside an
/// array alike.
///
/// Captured from Google Sheets on the conformance-fixtures pipeline
/// (2026-08-04, locale `en_US`, timezone `Etc/GMT`), alongside the control
/// `=COUNT(5)` → 1 that proves the probe resolved:
///
/// ```text
/// =COUNT(DATE(2020,1,1))                     1
/// =COUNT(DATE(2020,1,1),DATE(2021,1,1))      2
/// =COUNT({DATE(2020,1,1),DATE(2021,1,1)})    2
/// =COUNT({DATE(2020,1,1),5})                 2
/// =COUNT(<a range of three dates>)           3
/// =COUNT(<a range of two dates and a 5>)     3
/// =COUNT({DATE(2020,1,1),TRUE})              1   (booleans still skipped in an array)
/// =COUNT({DATE(2020,1,1),"a"})               1   (text still skipped in an array)
/// =COUNT(DATE(2020,1,1),TRUE)                2   (a direct boolean still counts)
/// ```
///
/// The same list is on [`count_lazy_fn`]; both are the one 2026-08-04 capture.
///
/// Those rows are not in this repo: they come off the conformance-fixtures
/// pipeline and land in a separate fixtures-only PR, because CI rejects a PR
/// that mixes fixture TSVs with code. Read from this repo alone, this test
/// pins the behaviour, not the Sheets answer.
#[test]
fn count_counts_dates() {
    assert_eq!(run("=COUNT(DATE(2020,1,1))", HashMap::new()), Value::Number(1.0));
    assert_eq!(
        run("=COUNT(DATE(2020,1,1),DATE(2021,1,1))", HashMap::new()),
        Value::Number(2.0)
    );
    assert_eq!(
        run("=COUNT({DATE(2020,1,1),DATE(2021,1,1)})", HashMap::new()),
        Value::Number(2.0)
    );
    assert_eq!(
        run("=COUNT({DATE(2020,1,1),5})", HashMap::new()),
        Value::Number(2.0)
    );
    // The eager helper keeps the same rule as the registered lazy one.
    assert_eq!(count_fn(&[Value::Date(43831.0)]), Value::Number(1.0));
    assert_eq!(
        count_fn(&[Value::Date(43831.0), Value::Number(5.0)]),
        Value::Number(2.0)
    );
}

/// Regression for #780: a date arriving through a range — the shape
/// `=COUNT(Orders!A:A)` materializes as — is counted too, and the extremes
/// taken over a date range stay countable.
///
/// `=COUNT(MAX(<date range>))` answering 0 was the reported failure: a "how
/// many results did I get" guard silently reading zero.
#[test]
fn count_counts_dates_arriving_through_a_range() {
    let vars: HashMap<_, _> = [(
        "R".to_string(),
        Value::Array(vec![
            Value::Array(vec![Value::Date(43831.0)]),
            Value::Array(vec![Value::Date(44197.0)]),
            Value::Array(vec![Value::Number(5.0)]),
        ]),
    )]
    .into();
    assert_eq!(run("=COUNT(R)", vars.clone()), Value::Number(3.0));
    assert_eq!(run("=COUNT(MAX(R))", vars.clone()), Value::Number(1.0));
    assert_eq!(run("=COUNT(MIN(R))", vars.clone()), Value::Number(1.0));
    assert_eq!(run("=COUNT(MAXA(R))", vars.clone()), Value::Number(1.0));
    assert_eq!(run("=COUNT(MINA(R))", vars), Value::Number(1.0));
    assert_eq!(
        run("=COUNT(MAX({DATE(2020,1,1),5}))", HashMap::new()),
        Value::Number(1.0)
    );
}

/// Counting a date must not drag anything else into COUNT's scope: booleans
/// and text keep the direct-arg / array-context split they already had.
#[test]
fn counting_a_date_does_not_move_bools_or_text() {
    // Direct arguments: booleans counted (statistical.tsv: `=COUNT(TRUE,FALSE,1)`
    // is 3), and a date beside one is captured as 2.
    assert_eq!(
        run("=COUNT(DATE(2020,1,1),TRUE)", HashMap::new()),
        Value::Number(2.0)
    );
    // Array context: booleans and text are still skipped, so only the date counts.
    assert_eq!(
        run("=COUNT({DATE(2020,1,1),TRUE})", HashMap::new()),
        Value::Number(1.0)
    );
    assert_eq!(
        run("=COUNT({DATE(2020,1,1),\"a\"})", HashMap::new()),
        Value::Number(1.0)
    );
}

#[test]
fn counta_array_variable_counts_non_empty() {
    // COUNTA with a variable holding an array → recursively counts non-empty
    let vars: HashMap<_, _> = [(
        "A".to_string(),
        Value::Array(vec![
            Value::Number(1.0),
            Value::Text("hello".to_string()),
            Value::Empty,
            Value::Bool(false),
        ]),
    )]
    .into();
    assert_eq!(run("=COUNTA(A)", vars), Value::Number(3.0));
}

/// Sheets counts a direct text argument it can read as a number *or* as a
/// date/time. Every case here mirrors a recorded conformance row.
#[test]
fn direct_date_and_time_text_is_counted() {
    for formula in [
        "=COUNT(\"2020-01-01\")",
        "=COUNT(\"1/1/2020\")",
        "=COUNT(\"1-Jan-2020\")",
        "=COUNT(\"13:45:00\")",
        "=COUNT(\"13:45\")",
        "=COUNT(\"1:45 PM\")",
        "=COUNT(\"2020-01-01 13:45\")",
        // Surrounding whitespace is tolerated.
        "=COUNT(\" 2020-01-01 \")",
        // Controls: numeric text and a plain number already agreed.
        "=COUNT(\"5\")",
        "=COUNT(5)",
    ] {
        assert_eq!(run(formula, HashMap::new()), Value::Number(1.0), "{formula}");
    }
    assert_eq!(
        run("=COUNT(\"2020-01-01\",5)", HashMap::new()),
        Value::Number(2.0)
    );
}

/// The negative controls: this is real parsing, not a shape match. An invalid
/// month is not a date, and text that is not a number stays uncounted.
#[test]
fn direct_text_that_is_not_a_number_or_date_is_not_counted() {
    for formula in ["=COUNT(\"2020-13-01\")", "=COUNT(\"abc\")", "=COUNT(\"\")"] {
        assert_eq!(run(formula, HashMap::new()), Value::Number(0.0), "{formula}");
    }
}

/// Array context is captured as already correct and must not move: text of any
/// shape is skipped inside an array, date-shaped or not.
#[test]
fn array_context_still_skips_date_text() {
    assert_eq!(
        run("=COUNT({\"2020-01-01\"})", HashMap::new()),
        Value::Number(0.0)
    );
    assert_eq!(
        run("=COUNT({\"2020-01-01\",5})", HashMap::new()),
        Value::Number(1.0)
    );
}
