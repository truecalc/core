use std::collections::HashMap;

use crate::eval::functions::Registry;
use crate::eval::{evaluate_expr, Context, EvalCtx, Resolver};
use crate::parser::{parse_formula, Expr};
use crate::types::{ErrorKind, ParseError, Value};

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
///   stubbed (`evaluate` returns `#N/A`).
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
    /// `Value::Error(ErrorKind::NA)` for every formula. [`Engine::parse`]
    /// and [`Engine::validate`] work.
    pub fn excel() -> Self {
        Self { flavor: EngineFlavor::Excel, registry: Registry::new() }
    }

    /// Deprecated alias for [`Engine::sheets`].
    #[deprecated(note = "use Engine::sheets() — engine flavor is required; see ADR 2026-04-27")]
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
    pub fn parse(&self, formula: &str) -> Result<Expr, ParseError> {
        parse_formula(formula)
    }

    /// Validate that a formula string is syntactically correct without
    /// returning the AST.
    pub fn validate(&self, formula: &str) -> Result<(), ParseError> {
        self.parse(formula).map(|_| ())
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
    /// returns `#N/A` until Excel evaluation lands, exactly like
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
            return Value::Error(ErrorKind::NA);
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

    fn evaluate_inner(
        &self,
        formula: &str,
        variables: &HashMap<String, Value>,
        now_serial: Option<f64>,
    ) -> Value {
        if self.flavor == EngineFlavor::Excel {
            // Excel evaluation semantics are not implemented yet.
            return Value::Error(ErrorKind::NA);
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
