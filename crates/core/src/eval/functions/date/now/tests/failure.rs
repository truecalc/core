use super::super::*;
use crate::eval::functions::{EvalCtx, Registry};
use crate::eval::Context;
use crate::parser::ast::{Expr, Span};
use crate::types::{ErrorKind, Value};

#[test]
fn too_many_args() {
    let registry = Registry::new();
    let mut eval_ctx = EvalCtx::new(Context::empty(), &registry);
    let arg = Expr::Number(0.0, Span::new(0, 0));
    assert_eq!(now_fn(&[arg], &mut eval_ctx), Value::Error(ErrorKind::NA));
}
