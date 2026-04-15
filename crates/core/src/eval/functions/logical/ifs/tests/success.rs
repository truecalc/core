use super::super::ifs_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    ifs_fn(&args, &mut ctx)
}

#[test]
fn first_condition_true_returns_first_value() {
    let args = vec![
        Expr::Bool(true, span()),
        Expr::Number(1.0, span()),
        Expr::Bool(false, span()),
        Expr::Number(2.0, span()),
    ];
    assert_eq!(run(args), Value::Number(1.0));
}

#[test]
fn second_condition_true_returns_second_value() {
    let args = vec![
        Expr::Bool(false, span()),
        Expr::Number(1.0, span()),
        Expr::Bool(true, span()),
        Expr::Number(2.0, span()),
    ];
    assert_eq!(run(args), Value::Number(2.0));
}

#[test]
fn single_pair_true() {
    let args = vec![Expr::Bool(true, span()), Expr::Text("ok".to_string(), span())];
    assert_eq!(run(args), Value::Text("ok".to_string()));
}
