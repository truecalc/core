use super::super::switch_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    switch_fn(&args, &mut ctx)
}

#[test]
fn bool_match() {
    let args = vec![
        Expr::Bool(true, span()),
        Expr::Bool(false, span()),
        Expr::Text("false".to_string(), span()),
        Expr::Bool(true, span()),
        Expr::Text("true".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Text("true".to_string()));
}

#[test]
fn text_match() {
    let args = vec![
        Expr::Text("b".to_string(), span()),
        Expr::Text("a".to_string(), span()),
        Expr::Number(1.0, span()),
        Expr::Text("b".to_string(), span()),
        Expr::Number(2.0, span()),
    ];
    assert_eq!(run(args), Value::Number(2.0));
}

#[test]
fn single_case_with_default_no_match_uses_default() {
    // SWITCH(5, 1, "one", "default") — rest=[1,"one","default"], odd=true => default="default"
    let args = vec![
        Expr::Number(5.0, span()),
        Expr::Number(1.0, span()),
        Expr::Text("one".to_string(), span()),
        Expr::Text("default".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Text("default".to_string()));
}

#[test]
fn matched_result_does_not_evaluate_later_branches() {
    // SWITCH(1, 1, 42, 2, 1/0) → 42 (second result 1/0 is never evaluated)
    let args = vec![
        Expr::Number(1.0, span()),
        Expr::Number(1.0, span()),
        Expr::Number(42.0, span()),
        Expr::Number(2.0, span()),
        Expr::BinaryOp {
            op: BinaryOp::Div,
            left: Box::new(Expr::Number(1.0, span())),
            right: Box::new(Expr::Number(0.0, span())),
            span: span(),
        },
    ];
    assert_eq!(run(args), Value::Number(42.0));
}

#[test]
fn unmatched_case_does_not_evaluate_skipped_result() {
    // SWITCH(2, 1, 1/0, 2, 99) → 99 (first result 1/0 is skipped, never evaluated)
    let args = vec![
        Expr::Number(2.0, span()),
        Expr::Number(1.0, span()),
        Expr::BinaryOp {
            op: BinaryOp::Div,
            left: Box::new(Expr::Number(1.0, span())),
            right: Box::new(Expr::Number(0.0, span())),
            span: span(),
        },
        Expr::Number(2.0, span()),
        Expr::Number(99.0, span()),
    ];
    assert_eq!(run(args), Value::Number(99.0));
}

