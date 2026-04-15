pub mod financial;
pub mod logical;
pub mod math;
pub mod text;

use std::collections::HashMap;
use crate::eval::context::Context;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// ── EvalCtx ───────────────────────────────────────────────────────────────

/// Bundles the variable context and function registry for use during evaluation.
/// Passed to lazy functions so they can recursively evaluate sub-expressions.
pub struct EvalCtx<'r> {
    pub ctx: Context,
    pub registry: &'r Registry,
}

impl<'r> EvalCtx<'r> {
    pub fn new(ctx: Context, registry: &'r Registry) -> Self {
        Self { ctx, registry }
    }
}

// ── Function kinds ─────────────────────────────────────────────────────────

/// A function that receives pre-evaluated arguments.
/// Argument errors are caught before dispatch — the slice never contains `Value::Error`.
pub type EagerFn = fn(&[Value]) -> Value;

/// A function that receives raw AST nodes and controls its own evaluation order.
/// Used for short-circuit operators like `IF`, `AND`, `OR`.
pub type LazyFn  = fn(&[Expr], &mut EvalCtx<'_>) -> Value;

pub enum FunctionKind {
    Eager(EagerFn),
    Lazy(LazyFn),
}

// ── Registry ──────────────────────────────────────────────────────────────

/// The runtime registry of built-in and user-registered spreadsheet functions.
pub struct Registry {
    functions: HashMap<String, FunctionKind>,
}

impl Registry {
    pub fn new() -> Self {
        let mut r = Self { functions: HashMap::new() };
        math::register_math(&mut r);
        logical::register_logical(&mut r);
        text::register_text(&mut r);
        financial::register_financial(&mut r);
        r
    }

    pub fn register_eager(&mut self, name: &str, f: EagerFn) {
        self.functions.insert(name.to_uppercase(), FunctionKind::Eager(f));
    }

    pub fn register_lazy(&mut self, name: &str, f: LazyFn) {
        self.functions.insert(name.to_uppercase(), FunctionKind::Lazy(f));
    }

    pub fn get(&self, name: &str) -> Option<&FunctionKind> {
        self.functions.get(&name.to_uppercase())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate argument count for eager functions (args already evaluated to `&[Value]`).
/// Returns `Some(Value::Error(ErrorKind::Value))` if the count is out of range.
pub fn check_arity(args: &[Value], min: usize, max: usize) -> Option<Value> {
    if args.len() < min || args.len() > max {
        Some(Value::Error(ErrorKind::Value))
    } else {
        None
    }
}

/// Validate argument count for lazy functions (args are `&[Expr]`).
/// Returns `Some(Value::Error(ErrorKind::Value))` if the count is out of range.
pub fn check_arity_len(count: usize, min: usize, max: usize) -> Option<Value> {
    if count < min || count > max {
        Some(Value::Error(ErrorKind::Value))
    } else {
        None
    }
}
