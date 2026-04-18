use super::super::switch_fn;
use crate::eval::{Context, EvalCtx, Registry};
use crate::parser::ast::{Expr, Span};
use crate::types::Value;

fn span() -> Span { Span::new(0, 1) }

fn run(args: Vec<Expr>) -> Value {
    let reg = Registry::new();
    let mut ctx = EvalCtx::new(Context::empty(), &reg);
    switch_fn(&args, &mut ctx)
}

#[test]
fn matches_first_case() {
    let args = vec![
        Expr::Number(1.0, span()),
        Expr::Number(1.0, span()),
        Expr::Text("one".to_string(), span()),
        Expr::Number(2.0, span()),
        Expr::Text("two".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Text("one".to_string()));
}

#[test]
fn matches_second_case() {
    let args = vec![
        Expr::Number(2.0, span()),
        Expr::Number(1.0, span()),
        Expr::Text("one".to_string(), span()),
        Expr::Number(2.0, span()),
        Expr::Text("two".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Text("two".to_string()));
}

#[test]
fn uses_default_when_no_match() {
    // SWITCH(3, 1, "one", 2, "two", "other") — odd remaining count = has default
    let args = vec![
        Expr::Number(3.0, span()),
        Expr::Number(1.0, span()),
        Expr::Text("one".to_string(), span()),
        Expr::Number(2.0, span()),
        Expr::Text("two".to_string(), span()),
        Expr::Text("other".to_string(), span()),
    ];
    assert_eq!(run(args), Value::Text("other".to_string()));
}
