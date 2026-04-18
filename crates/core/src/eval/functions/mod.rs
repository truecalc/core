pub mod array;
pub mod database;
pub mod date;
pub mod engineering;
pub mod filter;
pub mod financial;
pub mod logical;
pub mod lookup;
pub mod math;
pub mod operator;
pub mod parser;
pub mod statistical;
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

// ── FunctionMeta ──────────────────────────────────────────────────────────

/// Metadata for a user-facing spreadsheet function.
/// Co-located with the registration call so it can never drift.
#[derive(Debug)]
pub struct FunctionMeta {
    pub category: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
}

// ── Registry ──────────────────────────────────────────────────────────────

/// The runtime registry of built-in and user-registered spreadsheet functions.
pub struct Registry {
    functions: HashMap<String, FunctionKind>,
    metadata: HashMap<String, FunctionMeta>,
}

impl Registry {
    pub fn new() -> Self {
        let mut r = Self { functions: HashMap::new(), metadata: HashMap::new() };
        math::register_math(&mut r);
        logical::register_logical(&mut r);
        text::register_text(&mut r);
        financial::register_financial(&mut r);
        statistical::register_statistical(&mut r);
        operator::register_operator(&mut r);
        date::register_date(&mut r);
        parser::register_parser(&mut r);
        engineering::register_engineering(&mut r);
        filter::register_filter(&mut r);
        array::register_array(&mut r);
        database::register_database(&mut r);
        lookup::register_lookup(&mut r);
        r
    }

    /// Register a user-facing eager function with metadata.
    /// Appears in `list_functions()`.
    pub fn register_eager(&mut self, name: &str, f: EagerFn, meta: FunctionMeta) {
        let key = name.to_uppercase();
        self.functions.insert(key.clone(), FunctionKind::Eager(f));
        self.metadata.insert(key, meta);
    }

    /// Register a user-facing lazy function with metadata.
    /// Appears in `list_functions()`.
    pub fn register_lazy(&mut self, name: &str, f: LazyFn, meta: FunctionMeta) {
        let key = name.to_uppercase();
        self.functions.insert(key.clone(), FunctionKind::Lazy(f));
        self.metadata.insert(key, meta);
    }

    /// Register an internal/compiler-only eager function without metadata.
    /// Never appears in `list_functions()`.
    pub fn register_internal(&mut self, name: &str, f: EagerFn) {
        self.functions.insert(name.to_uppercase(), FunctionKind::Eager(f));
    }

    /// Register an internal/compiler-only lazy function without metadata.
    /// Never appears in `list_functions()`.
    pub fn register_internal_lazy(&mut self, name: &str, f: LazyFn) {
        self.functions.insert(name.to_uppercase(), FunctionKind::Lazy(f));
    }

    pub fn get(&self, name: &str) -> Option<&FunctionKind> {
        self.functions.get(&name.to_uppercase())
    }

    /// Iterate all user-facing functions with their metadata.
    /// The registry is the single source of truth — this can never drift.
    pub fn list_functions(&self) -> impl Iterator<Item = (&str, &FunctionMeta)> {
        self.metadata.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate argument count for eager functions (args already evaluated to `&[Value]`).
/// Returns `Some(Value::Error(ErrorKind::NA))` if the count is out of range
/// (matches Google Sheets / Excel behaviour for wrong argument count).
pub fn check_arity(args: &[Value], min: usize, max: usize) -> Option<Value> {
    if args.len() < min || args.len() > max {
        Some(Value::Error(ErrorKind::NA))
    } else {
        None
    }
}

/// Validate argument count for lazy functions (args are `&[Expr]`).
/// Returns `Some(Value::Error(ErrorKind::NA))` if the count is out of range.
pub fn check_arity_len(count: usize, min: usize, max: usize) -> Option<Value> {
    if count < min || count > max {
        Some(Value::Error(ErrorKind::NA))
    } else {
        None
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_functions_matches_registry() {
        let registry = Registry::new();
        let listed: Vec<(&str, &FunctionMeta)> = registry.list_functions().collect();
        assert!(!listed.is_empty(), "registry should expose at least one function");
        // Every listed name must be resolvable — catches metadata/functions map skew
        for (name, _meta) in &listed {
            assert!(
                registry.get(name).is_some(),
                "listed function {name} not found via get()"
            );
        }
        // metadata count == listed count (no orphaned metadata entries)
        assert_eq!(listed.len(), registry.metadata.len());
    }
}
