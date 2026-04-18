use super::super::or_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    or_fn(&args, &mut ctx)
}

#[test]
fn one_true_returns_true() {
    let args = vec![Expr::Bool(false, span()), Expr::Bool(true, span())];
    assert_eq!(run(args), Value::Bool(true));
}

#[test]
fn all_false_returns_false() {
    let args = vec![Expr::Bool(false, span()), Expr::Bool(false, span())];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn nonzero_number_is_truthy() {
    let args = vec![Expr::Number(5.0, span())];
    assert_eq!(run(args), Value::Bool(true));
}
