use crate::eval::functions::EvalCtx;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

/// Bare LAMBDA call without immediate invocation (e.g. `=LAMBDA()` or
/// `=LAMBDA(x,x*2)` stored in a cell but not called). Returns #N/A per
/// Google Sheets / Excel semantics.
///
/// Immediately-invoked LAMBDA (`=LAMBDA(x,x*2)(5)`) is handled by
/// `eval_apply` in `eval/mod.rs` via the `Expr::Apply` AST node.
pub fn lambda_fn(args: &[Expr], _ctx: &mut EvalCtx<'_>) -> Value {
    let _ = args;
    Value::Error(ErrorKind::NA)
}

#[cfg(test)]
mod tests;
