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
pub mod timezone;
pub mod web;

use std::collections::HashMap;
use crate::eval::context::Context;
use crate::eval::resolver::Resolver;
use crate::parser::ast::{BinaryOp, Expr, Span, UnaryOp};
use crate::types::{ErrorKind, Value};

// ── EvalOp / EvalHook (per-node observation seam, issue #732; span-carrying
// enhancement per distributions ADR D10) ────────────────────────────────────

/// A lightweight, borrowed description of the operation an evaluated node
/// performs. Handed to an [`EvalHook`] alongside the node's resulting
/// [`Value`] so an observer can profile or trace evaluation without touching
/// the AST directly or any function. Constructing it borrows from the
/// expression and allocates nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvalOp<'a> {
    /// Numeric literal.
    Number,
    /// Text literal.
    Text,
    /// Boolean literal.
    Bool,
    /// Bare-identifier read (local binding or reference), carrying its name.
    Variable(&'a str),
    /// Cell / range / name reference read.
    Reference,
    /// Unary operator (negation, percent).
    UnaryOp(&'a UnaryOp),
    /// Binary operator (arithmetic, comparison, concatenation).
    BinaryOp(&'a BinaryOp),
    /// Array literal.
    Array,
    /// Immediately-invoked lambda application.
    Apply,
    /// Function call, carrying the (uppercased) function name.
    FunctionCall(&'a str),
}

impl<'a> EvalOp<'a> {
    /// Derive the operation descriptor for an expression node. Pure and
    /// allocation-free — every variant only borrows from `expr`.
    pub fn of(expr: &'a Expr) -> Self {
        match expr {
            Expr::Number(..) => EvalOp::Number,
            Expr::Text(..) => EvalOp::Text,
            Expr::Bool(..) => EvalOp::Bool,
            Expr::Variable(name, _) => EvalOp::Variable(name),
            Expr::Reference(..) => EvalOp::Reference,
            Expr::UnaryOp { op, .. } => EvalOp::UnaryOp(op),
            Expr::BinaryOp { op, .. } => EvalOp::BinaryOp(op),
            Expr::Array(..) => EvalOp::Array,
            Expr::Apply { .. } => EvalOp::Apply,
            Expr::FunctionCall { name, .. } => EvalOp::FunctionCall(name),
        }
    }
}

/// An observer invoked once per evaluated node, in post-order (children before
/// parents), with the node's [`EvalOp`], its source [`Span`], and resulting
/// [`Value`]. Purely observational: it is handed shared/by-value data and
/// cannot alter evaluation.
///
/// # Why `Span`
///
/// A single post-order stream is ambiguous for variable/dynamic-arity nodes
/// (`FunctionCall`, `Array`, `Apply`; lazy `IF`/`AND`/`OR` that skip un-taken
/// branches): a consumer cannot tell, from operation + value alone, which
/// events belong to which parent, or how many children a node had. Carrying
/// each node's byte-range `Span` — every `Expr` already has one — lets a
/// consumer reconstruct the full tree from the flat stream by *span
/// containment* (a child's span always falls inside its parent's), which is
/// robust to short-circuiting by construction: an unfired branch simply has
/// no event, and containment among the events that *do* fire is unaffected.
/// The span doubles as the byte range a UI highlights to explain a node (see
/// distributions ADR D10).
///
/// # Apply / LAMBDA callee (see [`EvalOp::Variable`] parameter-binding note)
///
/// The `LAMBDA(...)` callee of an `Apply` is pattern-destructured, not
/// evaluated, so it never reduces to a `Value` and its own `FunctionCall`
/// node never fires — there is no honest `Value` to give a lambda (no such
/// [`Value`] variant exists). Each parameter *binding* does have an honest
/// value, though (the argument bound to it), so it fires as an ordinary
/// [`EvalOp::Variable`] event at bind time, carrying the parameter's own
/// span and bound value — see `eval_apply`. A consumer can still recover the
/// callee's source extent (it is a sub-span of the `Apply` node's span) but
/// gets no discrete event, and no value, for the callee as a whole.
///
/// Blanket-implemented for every `FnMut(EvalOp<'_>, Span, &Value)`, so a
/// closure can be wired directly. Wiring is opt-in via [`EvalCtx::hook`]:
/// when it is `None` no descriptor is built and the only per-node cost is a
/// single branch; when present, each node costs one `EvalOp::of`, one
/// `Span` copy (two `usize`s), and one dynamic (vtable) call through the
/// `&mut dyn EvalHook` trait object.
pub trait EvalHook {
    fn on_node(&mut self, op: EvalOp<'_>, span: Span, value: &Value);
}

impl<F: FnMut(EvalOp<'_>, Span, &Value)> EvalHook for F {
    fn on_node(&mut self, op: EvalOp<'_>, span: Span, value: &Value) {
        self(op, span, value)
    }
}

// ── EvalCtx ───────────────────────────────────────────────────────────────

/// Bundles the variable context, function registry, and reference resolver
/// for use during evaluation. Passed to lazy functions so they can recursively
/// evaluate sub-expressions.
///
/// References that are not bound as local variables (e.g. a LAMBDA parameter)
/// are read through `resolver`; see [`crate::Resolver`]. When `resolver` is
/// `None` (the default, via [`EvalCtx::new`]) every such reference reads as
/// [`Value::Empty`], preserving the historical contract of
/// [`crate::Engine::evaluate`].
pub struct EvalCtx<'r> {
    pub ctx: Context,
    pub registry: &'r Registry,
    /// Resolver for references not bound as local variables. `None` ⇒ such
    /// references read as [`Value::Empty`] (the historical
    /// [`crate::Engine::evaluate`] contract).
    pub resolver: Option<&'r mut dyn Resolver>,
    /// Opt-in per-node observation hook (issue #732). `None` (the default) ⇒
    /// zero per-node work beyond a single branch; the callback only observes
    /// and can never alter evaluation. Set the field directly to wire one.
    pub hook: Option<&'r mut dyn EvalHook>,
}

impl<'r> EvalCtx<'r> {
    /// Build an `EvalCtx` with no resolver: unbound references read as
    /// [`Value::Empty`]. Use [`EvalCtx::with_resolver`] to supply real
    /// workbook semantics.
    pub fn new(ctx: Context, registry: &'r Registry) -> Self {
        Self { ctx, registry, resolver: None, hook: None }
    }

    /// Build an `EvalCtx` that resolves references through `resolver`.
    pub fn with_resolver(
        ctx: Context,
        registry: &'r Registry,
        resolver: &'r mut dyn Resolver,
    ) -> Self {
        Self { ctx, registry, resolver: Some(resolver), hook: None }
    }

    /// Resolve a reference that was not bound as a local variable, delegating
    /// to [`EvalCtx::resolver`] when present and falling back to
    /// [`Value::Empty`] otherwise.
    pub fn resolve_ref(&mut self, r: &crate::parser::refs::Ref) -> Value {
        match self.resolver {
            Some(ref mut res) => res.resolve(r),
            None => Value::Empty,
        }
    }
}

// ── Function kinds ─────────────────────────────────────────────────────────

/// A function that receives pre-evaluated arguments.
/// Argument errors are caught before dispatch — the slice never contains `Value::Error`.
pub type EagerFn = fn(&[Value]) -> Value;

/// A function that receives raw AST nodes and controls its own evaluation order.
/// Used for short-circuit operators like `IF`, `AND`, `OR`.
pub type LazyFn  = fn(&[Expr], &mut EvalCtx<'_>) -> Value;

#[derive(Clone)]
pub enum FunctionKind {
    Eager(EagerFn),
    Lazy(LazyFn),
}

// ── FunctionMeta ──────────────────────────────────────────────────────────

/// Metadata for a user-facing spreadsheet function.
/// Co-located with the registration call so it can never drift.
#[derive(Debug, Clone)]
pub struct FunctionMeta {
    pub category: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
}

/// A metadata entry returned by `Registry::get_metadata()`.
pub struct FunctionMetaEntry<'a> {
    pub name: &'a str,
    pub meta: &'a FunctionMeta,
}

// ── Registry ──────────────────────────────────────────────────────────────

/// The runtime registry of built-in and user-registered spreadsheet functions.
pub struct Registry {
    pub functions: HashMap<String, FunctionKind>,
    pub metadata: HashMap<String, FunctionMeta>,
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
        web::register_web(&mut r);
        timezone::register_timezone(&mut r);
        r
    }

    /// Register a user-facing eager function with metadata.
    /// Appears in `list_functions()`.
    /// Panics if `name` is already registered (duplicate registration).
    pub fn register_eager(&mut self, name: &str, f: EagerFn, meta: FunctionMeta) {
        let key = name.to_uppercase();
        assert!(
            !self.functions.contains_key(&key),
            "duplicate function registration: '{}'",
            key
        );
        self.functions.insert(key.clone(), FunctionKind::Eager(f));
        self.metadata.insert(key, meta);
    }

    /// Register a user-facing lazy function with metadata.
    /// Appears in `list_functions()`.
    /// Panics if `name` is already registered (duplicate registration).
    pub fn register_lazy(&mut self, name: &str, f: LazyFn, meta: FunctionMeta) {
        let key = name.to_uppercase();
        assert!(
            !self.functions.contains_key(&key),
            "duplicate function registration: '{}'",
            key
        );
        self.functions.insert(key.clone(), FunctionKind::Lazy(f));
        self.metadata.insert(key, meta);
    }

    /// Register `alias` as an alternate name for `canonical`.
    /// The alias shares the same handler but does NOT appear in function metadata
    /// (it will not show up in `list_functions()` or autocomplete).
    /// Panics if `alias` is already registered or `canonical` is not yet registered.
    pub fn register_alias(&mut self, alias: &str, canonical: &str) {
        let alias_key = alias.to_uppercase();
        let canonical_key = canonical.to_uppercase();
        assert!(
            !self.functions.contains_key(&alias_key),
            "duplicate function registration: '{}'",
            alias_key
        );
        let kind = self
            .functions
            .get(&canonical_key)
            .unwrap_or_else(|| {
                panic!(
                    "register_alias: canonical '{}' must be registered before alias '{}'",
                    canonical_key, alias_key
                )
            })
            .clone();
        self.functions.insert(alias_key, kind);
        // Intentionally no metadata entry — aliases are not user-facing
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

    /// Return all function metadata entries as a Vec of named structs.
    /// Used for inspection (e.g. counting functions, verifying aliases are absent).
    pub fn get_metadata(&self) -> Vec<FunctionMetaEntry<'_>> {
        self.metadata
            .iter()
            .map(|(k, v)| FunctionMetaEntry { name: k.as_str(), meta: v })
            .collect()
    }

    /// Return all user-facing function names (from metadata, not aliases).
    pub fn metadata_names(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
    }
}

impl Registry {
    /// Volatile functions — outputs change on every evaluation.
    /// Excluded from conformance fixtures; covered by property tests instead.
    pub const VOLATILE_FUNCTIONS: &'static [&'static str] = &[
        "RAND", "RANDARRAY", "NOW", "TODAY", "RANDBETWEEN", "TZNOW",
    ];
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder that stands in for the function name inside an arity diagnostic
/// message. `check_arity`/`check_arity_len` do not know the calling function's
/// name, so they emit this token; the evaluator substitutes the real name at
/// the dispatch site (see [`crate::eval::finalize_call_result`]). Chosen from
/// control characters so it can never collide with a real function name.
pub const FN_NAME_PLACEHOLDER: &str = "\u{1}FN\u{1}";

/// Build the Google-Sheets-style "wrong number of arguments" diagnostic. The
/// function name is left as [`FN_NAME_PLACEHOLDER`] for the dispatch site to
/// fill in. Example (min == max == 3, got == 0):
/// `"Wrong number of arguments to DATE. Expected 3 arguments, but got 0 arguments."`
fn arity_message(min: usize, max: usize, got: usize) -> String {
    fn plural(n: usize) -> &'static str {
        if n == 1 { "" } else { "s" }
    }
    let expected = if min == max {
        format!("{min} argument{}", plural(min))
    } else if max == usize::MAX {
        format!("at least {min} argument{}", plural(min))
    } else {
        format!("between {min} and {max} arguments")
    };
    format!(
        "Wrong number of arguments to {FN_NAME_PLACEHOLDER}. Expected {expected}, but got {got} argument{}.",
        plural(got)
    )
}

/// Validate argument count for eager functions (args already evaluated to `&[Value]`).
/// Returns `Some(Value::ErrorMsg(ErrorKind::NA, <message>))` if the count is out
/// of range (matches Google Sheets / Excel behaviour for wrong argument count).
/// The error *code* is unchanged (`#N/A`); only an additive diagnostic message
/// is attached.
pub fn check_arity(args: &[Value], min: usize, max: usize) -> Option<Value> {
    check_arity_len(args.len(), min, max)
}

/// Validate argument count for lazy functions (args are `&[Expr]`).
/// Returns `Some(Value::ErrorMsg(ErrorKind::NA, <message>))` if the count is out
/// of range.
pub fn check_arity_len(count: usize, min: usize, max: usize) -> Option<Value> {
    if count < min || count > max {
        Some(Value::ErrorMsg(ErrorKind::NA, arity_message(min, max, count)))
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
