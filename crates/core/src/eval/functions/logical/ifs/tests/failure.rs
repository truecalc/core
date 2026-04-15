use super::super::ifs_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    ifs_fn(&args, &mut ctx)
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(run(vec![]), Value::Error(ErrorKind::Value));
}

#[test]
fn odd_arg_count_returns_value_error() {
    let args = vec![
        Expr::Bool(true, span()),
        Expr::Number(1.0, span()),
        Expr::Bool(false, span()),
    ];
    assert_eq!(run(args), Value::Error(ErrorKind::Value));
}

#[test]
fn all_false_returns_na() {
    let args = vec![
        Expr::Bool(false, span()),
        Expr::Number(1.0, span()),
        Expr::Bool(false, span()),
        Expr::Number(2.0, span()),
    ];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}
