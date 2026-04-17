use super::super::*;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

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
