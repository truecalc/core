use std::collections::HashMap;

use crate::eval::functions::Registry;
use crate::eval::{evaluate_expr, Context, EvalCtx, EvalHook, Resolver};
use crate::parser::{parse_formula, Expr};
use crate::types::{ErrorKind, ParseError, Value};

mod grid_edit;

pub use grid_edit::{Axis, AxisMove, GridEdit};
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

    /// Rewrite the cell/range references in `formula` for a row/column insert
    /// or delete — the structural-edit reference-rewrite transform.
    ///
    /// Unlike [`Engine::translate_formula`], which applies a uniform offset,
    /// a [`GridEdit`] moves references *conditionally*: those before the edit
    /// index stay put, those at or after it shift by `count`, a range
    /// straddling the edit grows (insert) or shrinks (delete), and a
    /// reference whose every row/column was deleted becomes `#REF!` — the
    /// whole reference, sheet qualifier included, since `Sheet1!#REF!` does
    /// not re-parse.
    ///
    /// `$` anchors do **not** exempt an axis here: `$` governs how a
    /// reference is *copied*, not which cell it points at, so `$A$5` tracks
    /// its cell through an insert exactly as `A5` does. The anchors are
    /// preserved in the output.
    ///
    /// `formula_sheet` is the sheet the formula lives on — what a bare `A1`
    /// resolves to; `edited_sheet` is the sheet the rows/columns were
    /// inserted into or deleted from. Only references resolving to
    /// `edited_sheet` are touched, so a formula's references to other sheets
    /// never move. Matching is case-insensitive, as in
    /// [`Engine::rename_sheet_refs`]. String literals, function names,
    /// defined names and `LET`/`LAMBDA` bindings are left untouched.
    ///
    /// Returns `Err` if `formula` does not parse, if the edit's `at` is `0`
    /// (rows and columns are 1-based), or for `EngineFlavor::Excel`, whose
    /// grid bounds are not established yet — the same guard
    /// [`Engine::translate_formula`] carries.
    ///
    /// ```
    /// use truecalc_core::{Engine, GridEdit};
    ///
    /// let engine = Engine::sheets();
    /// let edit = GridEdit::DeleteRows { at: 2, count: 2 };
    ///
    /// // A formula on Sheet1, and rows deleted from Sheet1: the range shrinks
    /// // and the cell inside the deleted band is gone.
    /// let out = engine.shift_refs_for_grid_edit("=SUM(A1:A5)+A3", "Sheet1", "Sheet1", edit).unwrap();
    /// assert_eq!(out, "=SUM(A1:A3)+#REF!");
    ///
    /// // The same formula living on Sheet2 instead: its bare refs mean
    /// // Sheet2, which the Sheet1 edit does not touch, so nothing moves. Note
    /// // the argument order — `formula_sheet` first, then `edited_sheet`.
    /// let out = engine.shift_refs_for_grid_edit("=SUM(A1:A5)+A3", "Sheet2", "Sheet1", edit).unwrap();
    /// assert_eq!(out, "=SUM(A1:A5)+A3");
    ///
    /// // ...but its explicitly Sheet1-qualified refs still move.
    /// let out = engine.shift_refs_for_grid_edit("=SUM(Sheet1!A1:A5)", "Sheet2", "Sheet1", edit).unwrap();
    /// assert_eq!(out, "=SUM(Sheet1!A1:A3)");
    /// ```
    pub fn shift_refs_for_grid_edit(
        &self,
        formula: &str,
        formula_sheet: &str,
        edited_sheet: &str,
        edit: GridEdit,
    ) -> Result<String, ParseError> {
        if self.flavor == EngineFlavor::Excel {
            return Err(ParseError {
                message: "shift_refs_for_grid_edit: Excel flavor not yet supported".into(),
                position: 0,
            });
        }
        grid_edit::shift_refs_text(formula, formula_sheet, edited_sheet, edit)
    }

    /// Rewrite the cell/range references in `formula` for a row/column
    /// **move** — relocating the contiguous band `mv.start..=mv.end` on
    /// `edited_sheet` so it starts at `mv.at`, without inserting or deleting
    /// anything.
    ///
    /// Unlike [`Engine::shift_refs_for_grid_edit`], which can shift a
    /// reference away or drop it as `#REF!`, a move is a *total* remap:
    /// nothing is created or destroyed, so every coordinate on the moved
    /// axis maps to exactly one output coordinate. A coordinate inside the
    /// moved band translates onto the band's new start; a coordinate
    /// between the band's old and new position slides by the band's width
    /// in the opposite direction, closing the gap the band left; everything
    /// else is unchanged.
    ///
    /// `mv.at` landing inside `mv.start..=mv.end` has no well-defined
    /// destination — there is no way to "move a band into the middle of
    /// itself" — so it is a silent no-op, the same way
    /// [`GridEdit`]'s own `count: 0` is. `mv.at == mv.end + 1` is *not* part
    /// of that no-op range: it is the smallest genuine forward move,
    /// swapping the band with the immediately following equal-width block.
    ///
    /// Mapping a range's two endpoints independently can flip their
    /// relative order even when they were written ascending — moving rows
    /// 5:7 to before row 2 sends row 3's content to row 6 and row 6's
    /// content to row 3, so `A4:A6` maps to `A3:A7`, not `A7:A3`. This
    /// normalization only ever corrects an inversion the move itself
    /// introduced; a range that was already written backwards (`A7:A5`)
    /// keeps its written orientation, exactly as
    /// [`Engine::shift_refs_for_grid_edit`] does for insert/delete.
    ///
    /// `$` anchors do **not** exempt an axis here either: `$` governs how a
    /// reference is *copied*, not which cell it points at, so `$A$6` moves
    /// exactly as `A6` does. The anchors are preserved in the output.
    ///
    /// `formula_sheet` is the sheet the formula lives on — what a bare `A1`
    /// resolves to; `edited_sheet` is the sheet the band moved on. Only
    /// references resolving to `edited_sheet` are touched. Matching is
    /// case-insensitive, as in [`Engine::rename_sheet_refs`].
    ///
    /// Returns `Err` if `formula` does not parse, if `mv.start` or `mv.at`
    /// is `0` (rows and columns are 1-based), if `mv.start > mv.end`, if
    /// `mv.at` would push the band off the grid, or for `EngineFlavor::Excel`,
    /// whose grid bounds are not established yet — the same guard
    /// [`Engine::shift_refs_for_grid_edit`] carries. A move never grows the
    /// sheet, so an off-grid *result* cannot happen from a well-formed
    /// [`AxisMove`]; only an off-grid *request* is rejected, once, here.
    ///
    /// ```
    /// use truecalc_core::{Axis, AxisMove, Engine};
    ///
    /// let engine = Engine::sheets();
    ///
    /// // Moving rows 5:7 to row 2: independently-mapped endpoints invert
    /// // (row 4's content ends up at row 7, row 6's at row 3), so the
    /// // range is normalized back to ascending order.
    /// let mv = AxisMove { axis: Axis::Row, start: 5, end: 7, at: 2 };
    /// let out = engine.shift_refs_for_move("=SUM(A4:A6)", "Sheet1", "Sheet1", mv).unwrap();
    /// assert_eq!(out, "=SUM(A3:A7)");
    ///
    /// // `$` governs how a reference copies, not what it points at, so it
    /// // does not exempt an axis from a move either.
    /// let out = engine.shift_refs_for_move("=$A$6", "Sheet1", "Sheet1", mv).unwrap();
    /// assert_eq!(out, "=$A$3");
    ///
    /// // `at` inside the band itself has no well-defined destination: a no-op.
    /// let noop = AxisMove { axis: Axis::Row, start: 5, end: 7, at: 6 };
    /// let out = engine.shift_refs_for_move("=SUM(A1:A10)", "Sheet1", "Sheet1", noop).unwrap();
    /// assert_eq!(out, "=SUM(A1:A10)");
    /// ```
    pub fn shift_refs_for_move(
        &self,
        formula: &str,
        formula_sheet: &str,
        edited_sheet: &str,
        mv: AxisMove,
    ) -> Result<String, ParseError> {
        if self.flavor == EngineFlavor::Excel {
            return Err(ParseError {
                message: "shift_refs_for_move: Excel flavor not yet supported".into(),
                position: 0,
            });
        }
        grid_edit::shift_refs_for_move(formula, formula_sheet, edited_sheet, mv)
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
