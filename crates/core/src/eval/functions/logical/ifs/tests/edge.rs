use super::super::ifs_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{BinaryOp, Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    ifs_fn(&args, &mut ctx)
}

#[test]
fn single_pair_false_returns_na() {
    let args = vec![Expr::Bool(false, span()), Expr::Number(1.0, span())];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}

#[test]
fn condition_coercion_error_propagates() {
    let args = vec![
        Expr::Text("bad".to_string(), span()),
        Expr::Number(1.0, span()),
    ];
    assert_eq!(run(args), Value::Error(ErrorKind::Value));
}

#[test]
fn number_zero_condition_is_falsy() {
    let args = vec![
        Expr::Number(0.0, span()),
        Expr::Number(1.0, span()),
        Expr::Bool(true, span()),
        Expr::Number(2.0, span()),
    ];
    assert_eq!(run(args), Value::Number(2.0));
}

#[test]
fn matched_result_does_not_evaluate_later_branches() {
    // IFS(TRUE, 42, FALSE, 1/0) → 42 (second result 1/0 is never evaluated)
    let args = vec![
        Expr::Bool(true, span()),
        Expr::Number(42.0, span()),
        Expr::Bool(false, span()),
        Expr::BinaryOp {
            op: BinaryOp::Div,
            left: Box::new(Expr::Number(1.0, span())),
            right: Box::new(Expr::Number(0.0, span())),
            span: span(),
        },
    ];
    assert_eq!(run(args), Value::Number(42.0));
}
