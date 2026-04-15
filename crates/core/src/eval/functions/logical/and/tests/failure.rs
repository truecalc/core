use super::super::and_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    and_fn(&args, &mut ctx)
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(run(vec![]), Value::Error(ErrorKind::Value));
}

#[test]
fn one_false_returns_false() {
    let args = vec![Expr::Bool(true, span()), Expr::Bool(false, span())];
    assert_eq!(run(args), Value::Bool(false));
}

#[test]
fn text_arg_returns_value_error() {
    let args = vec![Expr::Text("hello".to_string(), span())];
    assert_eq!(run(args), Value::Error(ErrorKind::Value));
}
