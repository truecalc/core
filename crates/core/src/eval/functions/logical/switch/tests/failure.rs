use super::super::switch_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    switch_fn(&args, &mut ctx)
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(run(vec![]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_few_args_returns_value_error() {
    // Only expr + one more arg — not enough for a case/value pair
    let args = vec![Expr::Number(1.0, span()), Expr::Number(1.0, span())];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}

#[test]
fn no_match_without_default_returns_na() {
    // SWITCH(99, 1, "one") — even remaining = no default
    let args = vec![
        Expr::Number(99.0, span()),
        Expr::Number(1.0, span()),
        Expr::Text("one".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}
