use super::super::or_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    or_fn(&args, &mut ctx)
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(run(vec![]), Value::Error(ErrorKind::Value));
}

#[test]
fn text_arg_returns_value_error() {
    let args = vec![Expr::Text("hello".to_string(), span())];
    assert_eq!(run(args), Value::Error(ErrorKind::Value));
}

#[test]
fn all_zeros_returns_false() {
    let args = vec![Expr::Number(0.0, span()), Expr::Number(0.0, span())];
    assert_eq!(run(args), Value::Bool(false));
}
