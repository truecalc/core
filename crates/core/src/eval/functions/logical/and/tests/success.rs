use super::super::and_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    and_fn(&args, &mut ctx)
}

#[test]
fn all_true_booleans() {
    let args = vec![Expr::Bool(true, span()), Expr::Bool(true, span())];
    assert_eq!(run(args), Value::Bool(true));
}

#[test]
fn single_true() {
    let args = vec![Expr::Bool(true, span())];
    assert_eq!(run(args), Value::Bool(true));
}

#[test]
fn nonzero_numbers_are_truthy() {
    let args = vec![Expr::Number(1.0, span()), Expr::Number(42.0, span())];
    assert_eq!(run(args), Value::Bool(true));
}
