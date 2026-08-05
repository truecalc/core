use super::super::sheets_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    sheets_fn(&args, &mut ctx)
}

#[test]
fn number_literal_returns_na() {
    let args = vec![Expr::Number(42.0, span())];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}

#[test]
fn text_literal_returns_na() {
    let args = vec![Expr::Text("hello".to_string(), span())];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = vec![
        Expr::Variable("A1".to_string(), span()),
        Expr::Variable("B1".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Error(ErrorKind::NA));
}
