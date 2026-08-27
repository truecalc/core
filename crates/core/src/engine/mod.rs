use std::collections::HashMap;

use crate::eval::functions::Registry;
use crate::eval::{evaluate_expr, Context, EvalCtx, EvalHook, Resolver};
use crate::parser::{parse_formula, Expr};
use crate::types::{ErrorKind, ParseError, Value};

mod rename;
mod translate;

/// Which spreadsheet product's semantics the engine targets.
///
/// The engine flavor also locks the **date serial system** (P1.4, issue #526):
///
/// - `Sheets`: day 0 = 1899-12-30; no leap-year bug (1900-02-28 = serial 60,
///   1900-03-01 = serial 61, no serial for the nonexistent 1900-02-29).
/// - `Excel`: 1900 date system — serial 1 = 1900-01-01, **including** the
///   historical Lotus 1-2-3 leap-year bug (serial 60 = the fictitious
///   1900-02-29). Conversion helpers live in
///   `eval::functions::date::serial`; Excel evaluation itself is still
///   stubbed (`evaluate` returns `#UNSUPPORTED!`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum EngineFlavor {
    Sheets,
    Excel,
}

pub struct Engine {
    flavor: EngineFlavor,
    registry: Registry,
}

impl Engine {
    /// Engine targeting Google Sheets conformance.
    pub fn sheets() -> Self {
        Self { flavor: EngineFlavor::Sheets, registry: Registry::new() }
    }

    /// Engine targeting Excel conformance.
    ///
    /// Excel evaluation semantics are not implemented yet (they land in a
    /// later phase): [`Engine::evaluate`] returns
    /// `Value::Error(ErrorKind::Unsupported)` for every formula. [`Engine::parse`]
    /// and [`Engine::validate`] work.
    pub fn excel() -> Self {
        Self { flavor: EngineFlavor::Excel, registry: Registry::new() }
    }

    /// Deprecated alias for [`Engine::sheets`].
    #[deprecated(since = "0.7.0", note = "use Engine::sheets() — engine flavor is required; see ADR 2026-04-27; removal target: 0.7.0 coordinated release")]
    pub fn google_sheets() -> Self {
        Self::sheets()
    }

    /// The engine flavor this instance targets.
    ///
    /// Flavor is fixed at construction (`Engine::sheets()` / `Engine::excel()`);
    /// there is no way to change it on an existing engine (engine-flavor ADR
    /// 2026-04-27). The workbook layer uses this to assert a workbook's locked
    /// [`EngineFlavor`] matches the engine driving its recalc.
    pub fn flavor(&self) -> EngineFlavor {
        self.flavor
    }

    /// Parse a formula string into an expression tree.
    ///
    /// The formula may start with `=`. Returns a [`ParseError`] if the input
    /// is not a valid formula.
    ///
    /// Parsing is flavor-independent and never reads the function registry, so
    /// a caller that only needs the AST can call [`crate::parse_formula`]
    /// directly instead of constructing an engine (issue #900).
    pub fn parse(&self, formula: &str) -> Result<Expr, ParseError> {
        parse_formula(formula)
    }

    /// Validate that a formula string is syntactically correct without
    /// returning the AST.
    ///
    /// A syntax check is exactly a parse: see [`Engine::parse`] for why a
    /// caller that only validates need not build an engine.
    pub fn validate(&self, formula: &str) -> Result<(), ParseError> {
        self.parse(formula).map(|_| ())
    }

    /// Shift every relative axis of every cell/range reference in `formula`
    /// by `(d_row, d_col)` — the fill / copy-paste reference-adjustment
    /// transform. `$`-absolute axes are left unchanged. An axis that shifts
    /// out of the Sheets grid becomes a literal `#REF!` for that corner.
    ///
    /// Sheets flavor only: `Engine::excel().translate_formula(...)` returns
    /// `Err` until Excel grid bounds are established.
    pub fn translate_formula(&self, formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError> {
        if self.flavor == EngineFlavor::Excel {
            return Err(ParseError {
                message: "translate_formula: Excel flavor not yet supported".into(),
                position: 0,
            });
        }
        translate::translate_text(formula, d_row, d_col)
    }

    /// Rewrite the sheet qualifier of every cell/range reference in `formula`
    /// that points at `old` to point at `new` instead — the sheet-rename
    /// reference-rewrite transform. Sheet-name matching is case-insensitive
    /// (mirrors the workbook crate's own sheet-identity rule: sheet names are
    /// unique case-insensitively, and a pure case-change rename is allowed).
    /// Requoting is handled automatically. Unqualified refs, refs to other
    /// sheets, string literals, function names, and defined names are left
    /// untouched. No-op if `formula` has no `old`-qualified refs.
    pub fn rename_sheet_refs(&self, formula: &str, old: &str, new: &str) -> Result<String, ParseError> {
        rename::rename_sheet_refs_text(formula, old, new)
    }

    /// Evaluate a formula string with named variables.
    ///
    /// Array results flow through **unspilled**: a formula producing an array
    /// returns the full [`Value::Array`] — spilling it across cells (or
    /// collapsing it for a single-cell view) is the workbook/surface layer's
    /// job, not the evaluator's (P1.4, issue #526).
    ///
    /// Volatile date functions (`NOW`, `TODAY`) read the ambient local clock.
    /// Use [`Engine::evaluate_at`] to pin them for deterministic evaluation.
    pub fn evaluate(&self, formula: &str, variables: &HashMap<String, Value>) -> Value {
        self.evaluate_inner(formula, variables, None)
    }

    /// Evaluate a formula with the volatile date functions (`NOW`, `TODAY`)
    /// pinned to `now_serial`, a local-time spreadsheet serial datetime
    /// (integer part = day serial in this engine's date system, fractional
    /// part = time of day).
    ///
    /// Same formula + same variables + same `now_serial` ⇒ identical result.
    /// This is the core-level hook the workbook layer's `RecalcContext`
    /// (timestamp + IANA timezone, scope ADR 2026-06-07 Decision 3) builds on:
    /// the caller converts its UTC instant + timezone to a local serial and
    /// passes it here. Conformance fixture rows for volatile formulas are
    /// verified by pinning `now_serial` to the fixture's recorded
    /// `meta.evaluatedAt`.
    ///
    /// Returns `Value::Error(ErrorKind::Num)` if `now_serial` is not finite.
    pub fn evaluate_at(
        &self,
        formula: &str,
        variables: &HashMap<String, Value>,
        now_serial: f64,
    ) -> Value {
        if !now_serial.is_finite() {
            return Value::Error(ErrorKind::Num);
        }
        self.evaluate_inner(formula, variables, Some(now_serial))
    }

    /// Evaluate a formula string, resolving references through `resolver`.
    ///
    /// This is the workbook-facing entry point: unlike [`Engine::evaluate`]
    /// (which reads references from a variable map and treats anything unbound
    /// as [`Value::Empty`]), every cell, range, and name reference that is not
    /// shadowed by a LAMBDA parameter is read through `resolver`. The resolver
    /// owns workbook semantics -- `#REF!` for a missing sheet, `#NAME?` for an
    /// undefined name, ranges materialized to [`Value::Array`]. See
    /// [`Resolver`].
    ///
    /// The engine flavor stays explicit: `Engine::excel().evaluate_with_resolver`
    /// returns `#UNSUPPORTED!` until Excel evaluation lands, exactly like
    /// [`Engine::evaluate`].
    ///
    /// ```
    /// use truecalc_core::{Engine, ErrorKind, Ref, Resolver, Value};
    ///
    /// struct OneSheet;
    /// impl Resolver for OneSheet {
    ///     fn resolve(&mut self, r: &Ref) -> Value {
    ///         match r {
    ///             Ref::Cell { sheet: Some(s), .. } if s == "Data" => Value::Number(10.0),
    ///             Ref::Cell { sheet: Some(_), .. } => Value::Error(ErrorKind::Ref),
    ///             _ => Value::Empty,
    ///         }
    ///     }
    /// }
    ///
    /// let engine = Engine::sheets();
    /// assert_eq!(engine.evaluate_with_resolver("=Data!A1", &mut OneSheet), Value::Number(10.0));
    /// assert_eq!(
    ///     engine.evaluate_with_resolver("=Gone!A1", &mut OneSheet),
    ///     Value::Error(ErrorKind::Ref),
    /// );
    /// ```
    pub fn evaluate_with_resolver(&self, formula: &str, resolver: &mut impl Resolver) -> Value {
        self.evaluate_with_resolver_at(formula, resolver, None)
    }

    /// Like [`Engine::evaluate_with_resolver`], but with the volatile date
    /// functions (`NOW`, `TODAY`) pinned to `now_serial` (see
    /// [`Engine::evaluate_at`]). Returns `Value::Error(ErrorKind::Num)` if
    /// `now_serial` is not finite.
    pub fn evaluate_with_resolver_at(
        &self,
        formula: &str,
        resolver: &mut impl Resolver,
        now_serial: Option<f64>,
    ) -> Value {
        if let Some(n) = now_serial {
            if !n.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
        }
        if self.flavor == EngineFlavor::Excel {
            return Value::Error(ErrorKind::Unsupported);
        }
        match parse_formula(formula) {
            Err(_) => Value::Error(ErrorKind::Value),
            Ok(expr) => {
                let mut ctx = Context::empty();
                ctx.now_serial = now_serial;
                let mut eval_ctx = EvalCtx::with_resolver(ctx, &self.registry, resolver);
                evaluate_expr(&expr, &mut eval_ctx)
            }
        }
    }

    /// Like [`Engine::evaluate_with_resolver_at`] but also injects a per-cell
    /// RNG key. `rng_cell` is `(seed, sheet_index, row, col)`; when `None`
    /// this degrades to the non-deterministic SystemTime fallback in RAND.
    pub fn evaluate_with_resolver_at_keyed(
        &self,
        formula: &str,
        resolver: &mut dyn Resolver,
        now_serial: Option<f64>,
        now_utc_nanos: Option<i64>,
        rng_cell: Option<(u64, u32, u32, u32)>,
    ) -> Value {
        self.evaluate_with_resolver_at_keyed_hooked(
            formula,
            resolver,
            now_serial,
            now_utc_nanos,
            rng_cell,
            None,
        )
    }

    /// Like [`Engine::evaluate_with_resolver_at_keyed`], but additionally
    /// wires an opt-in per-node [`EvalHook`] (issue #743) onto the
    /// [`EvalCtx`] built for this evaluation. `hook: None` is exactly
    /// [`Engine::evaluate_with_resolver_at_keyed`] — same code path, same
    /// value, no observation overhead beyond the `Option` check already paid
    /// by [`evaluate_expr`]'s per-node hook branch. This is the seam the
    /// workbook layer's single-cell tracer (`Workbook::trace_cell`) uses to
    /// reach a real cell's evaluation with the same resolver-backed
    /// semantics `recalc` uses, rather than re-deriving its own `EvalCtx`.
    pub fn evaluate_with_resolver_at_keyed_hooked<'r>(
        &'r self,
        formula: &str,
        resolver: &'r mut dyn Resolver,
        now_serial: Option<f64>,
        now_utc_nanos: Option<i64>,
        rng_cell: Option<(u64, u32, u32, u32)>,
        hook: Option<&'r mut dyn EvalHook>,
    ) -> Value {
        if let Some(n) = now_serial {
            if !n.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
        }
        if self.flavor == EngineFlavor::Excel {
            return Value::Error(ErrorKind::NA);
        }
        match parse_formula(formula) {
            Err(_) => Value::Error(ErrorKind::Value),
            Ok(expr) => {
                let mut ctx = Context::empty();
                ctx.now_serial = now_serial;
                ctx.now_utc_nanos = now_utc_nanos;
                ctx.rng_cell = rng_cell;
                let mut eval_ctx = EvalCtx::with_resolver(ctx, &self.registry, resolver);
                eval_ctx.hook = hook;
                evaluate_expr(&expr, &mut eval_ctx)
            }
        }
    }

    fn evaluate_inner(
        &self,
        formula: &str,
        variables: &HashMap<String, Value>,
        now_serial: Option<f64>,
    ) -> Value {
        if self.flavor == EngineFlavor::Excel {
            // Excel evaluation semantics are not implemented yet.
            return Value::Error(ErrorKind::Unsupported);
        }
        match parse_formula(formula) {
            Err(_) => Value::Error(ErrorKind::Value),
            Ok(expr) => {
                let mut ctx = Context::new(variables.clone());
                ctx.now_serial = now_serial;
                let mut eval_ctx = EvalCtx::new(ctx, &self.registry);
                evaluate_expr(&expr, &mut eval_ctx)
            }
        }
    }
}

#[cfg(test)]
mod tests;
