# translate_formula Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `Engine::translate_formula(formula, d_row, d_col) -> Result<String, ParseError>` to `truecalc-core` — a pure text-in/text-out API that shifts a formula's relative cell/range references by a row/column offset (the fill/copy-paste transform), preserving `$`-absolute axes — and expose it through `crates/wasm` as `translate_formula(formula, d_row, d_col) -> TranslateResult`.

**Architecture:** Span-splice, not full AST reserialization. Parse the formula into the existing `Expr` AST, walk it with a scope-aware traversal (tracking `LET`/`LAMBDA` local bindings so they're never mistaken for cell references) to collect `(Span, Ref)` pairs for every reference that should shift, render each replacement via the existing `CellAddr`/`Ref` `Display` impls (which already round-trip `$`), and splice the replacement text into the original formula string right-to-left by byte offset. Everything outside a reference span (operators, function names, string literals, whitespace) is copied through untouched.

**Tech Stack:** Rust (`truecalc-core`, `truecalc-wasm`), existing `nom`-based parser, `wasm-bindgen`/`tsify-next` for the WASM surface. No new dependencies.

## Global Constraints

- v1 is **Sheets flavor only** — `Engine::excel().translate_formula(...)` returns `Err`. (Design spec §Scope)
- Out-of-bounds is **per-corner**, not per-span: only the failing endpoint of a range becomes literal `#REF!`; the other endpoint keeps its shifted address. A single-cell reference has one corner, so it becomes a plain `#REF!` when out of bounds. (Design spec §Algorithm step 4)
- Reuse the existing Sheets grid bounds (`MAX_COL = 18_278`, `MAX_ROW = 10_000_000`, currently private consts in `crates/core/src/eval/functions/lookup/indirect/mod.rs:5-6`) — do not duplicate the magic numbers.
- `LET`/`LAMBDA` binding names and any body reference shadowed by them must never be shifted. A `LET` pair's own value expression is evaluated **before** its name enters scope (matches `let_fn.rs`'s actual bind order), so a self-referencing name in its own value expression is a real cell reference, not the local binding.
- Test files are separate from production source per this repo's CLAUDE.md: `#[cfg(test)] mod tests;` in the module file, tests live in a sibling `tests.rs` (or `crates/core/tests/*.rs` for integration-level, `crates/wasm/tests/*.rs` for the wasm crate).
- No conformance-fixture-TSV additions — `translate_formula` isn't a Sheets-evaluated formula.
- Design spec: `docs/superpowers/specs/2026-07-14-translate-formula-design.md` (already committed on this branch).

---

## File Structure

- **Create** `crates/core/src/engine/translate.rs` — the whole algorithm: `shift_addr`, `shift_ref_text`, `collect_shiftable_refs`, `translate_text`. All `pub(crate)`/private; only `translate_text` is called from outside the module.
- **Create** `crates/core/src/engine/translate/tests.rs` — unit tests for the four functions above, via `#[cfg(test)] mod tests;` in `translate.rs`.
- **Modify** `crates/core/src/eval/functions/lookup/indirect/mod.rs` — widen `MAX_COL`/`MAX_ROW` from private to `pub(crate)`.
- **Modify** `crates/core/src/parser/refs.rs` — widen `write_sheet` from private/`&mut fmt::Formatter` to `pub(crate)`/`&mut dyn fmt::Write`, so `translate.rs` can reuse the exact same sheet-qualifier quoting logic the `Display` impls already use.
- **Modify** `crates/core/src/engine/mod.rs` — add `mod translate;` and the public `Engine::translate_formula` method.
- **Modify** `crates/core/src/engine/tests.rs` — add integration-level tests for the public `Engine::translate_formula` API (flavor gating, parse-error propagation).
- **Modify** `crates/wasm/src/lib.rs` — add `TranslateResult` and the `#[wasm_bindgen] pub fn translate_formula`.
- **Create** `crates/wasm/tests/translate_formula.rs` — native surface-shape tests (no `JsValue` involved, so no wasm runtime needed), mirroring `crates/wasm/tests/eval_result.rs`'s existing style.

---

### Task 1: `shift_addr` — shift a single `CellAddr`, respecting `$`-absolute axes and grid bounds

**Files:**
- Modify: `crates/core/src/eval/functions/lookup/indirect/mod.rs:5-6`
- Create: `crates/core/src/engine/translate.rs`
- Create: `crates/core/src/engine/translate/tests.rs`
- Modify: `crates/core/src/engine/mod.rs` (add `mod translate;`)

**Interfaces:**
- Produces: `pub(crate) fn shift_addr(addr: CellAddr, d_row: i64, d_col: i64) -> Option<CellAddr>` in `crates/core/src/engine/translate.rs` — `None` means the shifted address fell outside the Sheets grid on a relative axis. Later tasks in this module call this directly (same file, no `crate::` prefix needed).

- [ ] **Step 1: Widen the grid-bounds constants to `pub(crate)`**

In `crates/core/src/eval/functions/lookup/indirect/mod.rs`, change:

```rust
/// Google Sheets limits: 18,278 columns (≈ column ZZZ) and 10,000,000 rows.
const MAX_COL: usize = 18_278;
const MAX_ROW: usize = 10_000_000;
```

to:

```rust
/// Google Sheets limits: 18,278 columns (≈ column ZZZ) and 10,000,000 rows.
pub(crate) const MAX_COL: usize = 18_278;
pub(crate) const MAX_ROW: usize = 10_000_000;
```

- [ ] **Step 2: Create the module file with a failing test**

Create `crates/core/src/engine/translate.rs`:

```rust
//! `translate_formula`: shift a formula's relative cell/range references by
//! (d_row, d_col) — the fill/copy-paste reference-adjustment transform.
//! `$`-absolute axes are left unchanged. See
//! `docs/superpowers/specs/2026-07-14-translate-formula-design.md`.

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::CellAddr;

/// Shift `addr` by `(d_row, d_col)`, skipping any axis marked `$`-absolute.
/// Returns `None` if a *relative* axis lands outside the Sheets grid
/// (`1..=MAX_COL` / `1..=MAX_ROW`) — the caller renders that as `#REF!`.
pub(crate) fn shift_addr(addr: CellAddr, d_row: i64, d_col: i64) -> Option<CellAddr> {
    let col = if addr.col_abs { addr.col as i64 } else { addr.col as i64 + d_col };
    let row = if addr.row_abs { addr.row as i64 } else { addr.row as i64 + d_row };
    if (1..=MAX_COL as i64).contains(&col) && (1..=MAX_ROW as i64).contains(&row) {
        Some(CellAddr::new(col as u32, row as u32).with_col_abs(addr.col_abs).with_row_abs(addr.row_abs))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
```

Create `crates/core/src/engine/translate/tests.rs`:

```rust
use super::*;

#[test]
fn shifts_relative_both_axes() {
    let a = CellAddr::new(1, 1); // A1
    assert_eq!(shift_addr(a, 1, 1), Some(CellAddr::new(2, 2)));
}

#[test]
fn skips_absolute_column() {
    let a = CellAddr::new(1, 1).with_col_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(CellAddr::new(1, 6).with_col_abs(true)));
}

#[test]
fn skips_absolute_row() {
    let a = CellAddr::new(1, 1).with_row_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(CellAddr::new(6, 1).with_row_abs(true)));
}

#[test]
fn skips_both_absolute_axes() {
    let a = CellAddr::new(1, 1).with_col_abs(true).with_row_abs(true);
    assert_eq!(shift_addr(a, 5, 5), Some(a));
}

#[test]
fn out_of_bounds_negative_row_is_none() {
    let a = CellAddr::new(1, 1);
    assert_eq!(shift_addr(a, -5, 0), None);
}

#[test]
fn out_of_bounds_negative_col_is_none() {
    let a = CellAddr::new(1, 1);
    assert_eq!(shift_addr(a, 0, -5), None);
}

#[test]
fn in_bounds_at_grid_edge() {
    let a = CellAddr::new(MAX_COL as u32, 1);
    assert_eq!(shift_addr(a, 0, 0), Some(a));
}

#[test]
fn out_of_bounds_past_grid_edge() {
    let a = CellAddr::new(MAX_COL as u32, 1);
    assert_eq!(shift_addr(a, 0, 1), None);
}
```

In `crates/core/src/engine/mod.rs`, add a module declaration near the top, after the existing `use` block (do not remove or reorder the existing imports):

```rust
mod translate;
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: 8 tests pass (`shifts_relative_both_axes`, `skips_absolute_column`, `skips_absolute_row`, `skips_both_absolute_axes`, `out_of_bounds_negative_row_is_none`, `out_of_bounds_negative_col_is_none`, `in_bounds_at_grid_edge`, `out_of_bounds_past_grid_edge`).

If it instead fails to compile, re-check the `use` paths above against the actual module tree (`crate::eval::functions::lookup::indirect`, `crate::parser::CellAddr`) before proceeding.

- [ ] **Step 4: Commit**

```bash
git add crates/core/src/eval/functions/lookup/indirect/mod.rs crates/core/src/engine/translate.rs crates/core/src/engine/translate/tests.rs crates/core/src/engine/mod.rs
git commit -m "feat(core): add shift_addr for translate_formula (#709)"
```

---

### Task 2: `shift_ref_text` — render a shifted `Ref` back to formula text, with per-corner `#REF!`

**Files:**
- Modify: `crates/core/src/parser/refs.rs`
- Modify: `crates/core/src/engine/translate.rs`
- Modify: `crates/core/src/engine/translate/tests.rs`

**Interfaces:**
- Consumes: `shift_addr(addr: CellAddr, d_row: i64, d_col: i64) -> Option<CellAddr>` (Task 1, same file).
- Produces: `pub(crate) fn shift_ref_text(r: &Ref, d_row: i64, d_col: i64) -> String` in `crates/core/src/engine/translate.rs`. Task 4 calls this.

- [ ] **Step 1: Widen `write_sheet` for reuse outside `Display::fmt`**

In `crates/core/src/parser/refs.rs`, change:

```rust
fn write_sheet(f: &mut fmt::Formatter<'_>, sheet: &Option<String>) -> fmt::Result {
```

to:

```rust
pub(crate) fn write_sheet(f: &mut dyn fmt::Write, sheet: &Option<String>) -> fmt::Result {
```

Leave the function body and every call site (`write_sheet(f, sheet)?` inside `impl fmt::Display for Ref`) unchanged — `fmt::Formatter` implements `fmt::Write`, so `f: &mut fmt::Formatter<'_>` coerces to `&mut dyn fmt::Write` automatically at the call site.

- [ ] **Step 2: Write the failing tests**

Append to `crates/core/src/engine/translate/tests.rs`:

```rust
#[test]
fn cell_within_bounds_renders_shifted_text() {
    let r = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 1), "B2");
}

#[test]
fn cell_out_of_bounds_renders_ref_error() {
    let r = Ref::Cell { sheet: None, addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, -5, 0), "#REF!");
}

#[test]
fn sheet_qualified_cell_preserves_sheet_prefix() {
    let r = Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 0), "Data!A2");
}

#[test]
fn quoted_sheet_name_preserved() {
    let r = Ref::Cell { sheet: Some("Q2 Data".to_string()), addr: CellAddr::new(1, 1) };
    assert_eq!(shift_ref_text(&r, 1, 0), "'Q2 Data'!A2");
}

#[test]
fn range_shifts_both_corners_independently() {
    let r = Ref::Range {
        sheet: None,
        start: CellAddr::new(1, 1),
        end: CellAddr::new(4, 4).with_col_abs(true),
    };
    // start A1 -> B2; end $D4 -> $D5 (column absolute, row shifts 4 -> 5)
    assert_eq!(shift_ref_text(&r, 1, 1), "B2:$D5");
}

#[test]
fn range_one_corner_out_of_bounds_only_that_corner_becomes_ref_error() {
    let r = Ref::Range { sheet: None, start: CellAddr::new(1, 1), end: CellAddr::new(2, 10) };
    // start row 1-5=-4 (OOB); end row 10-5=5 (OK)
    assert_eq!(shift_ref_text(&r, -5, 0), "#REF!:B5");
}
```

These reference `Ref` and `CellAddr` — add to the top of `crates/core/src/engine/translate.rs`'s existing `use` block (see Step 3) so `use super::*;` in `tests.rs` picks them up.

- [ ] **Step 3: Run the tests to verify they fail to compile**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: compile error, `cannot find function 'shift_ref_text' in this scope`.

- [ ] **Step 4: Implement `shift_ref_text`**

In `crates/core/src/engine/translate.rs`, update the `use` block and add the function:

```rust
use std::fmt::Write as _;

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::refs::write_sheet;
use crate::parser::{CellAddr, Ref};
use crate::types::ErrorKind;
```

```rust
fn addr_text(addr: CellAddr, d_row: i64, d_col: i64) -> String {
    match shift_addr(addr, d_row, d_col) {
        Some(shifted) => shifted.to_string(),
        None => ErrorKind::Ref.to_string(),
    }
}

/// Render `r` shifted by `(d_row, d_col)` back to formula text. A corner
/// that shifts out of the Sheets grid becomes literal `#REF!`; the other
/// corner of a range (if in bounds) keeps its shifted address.
pub(crate) fn shift_ref_text(r: &Ref, d_row: i64, d_col: i64) -> String {
    match r {
        Ref::Cell { sheet, addr } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*addr, d_row, d_col));
            out
        }
        Ref::Range { sheet, start, end } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*start, d_row, d_col));
            out.push(':');
            out.push_str(&addr_text(*end, d_row, d_col));
            out
        }
        Ref::Name(name) => name.clone(), // never reached by the Task 3 traversal
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: all 14 tests in this module pass (8 from Task 1 + 6 new).

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/parser/refs.rs crates/core/src/engine/translate.rs crates/core/src/engine/translate/tests.rs
git commit -m "feat(core): add shift_ref_text with per-corner #REF! (#709)"
```

---

### Task 3: `collect_shiftable_refs` — scope-aware AST walk, skipping `LET`/`LAMBDA` local bindings

**Files:**
- Modify: `crates/core/src/engine/translate.rs`
- Modify: `crates/core/src/engine/translate/tests.rs`

**Interfaces:**
- Produces: `fn collect_shiftable_refs(expr: &Expr) -> Vec<(Span, Ref)>` in `crates/core/src/engine/translate.rs`. Task 4 calls this. Every returned `Ref` is `Ref::Cell` or `Ref::Range` (never `Ref::Name`) and every returned `Span` is disjoint from every other (leaf AST nodes only).

- [ ] **Step 1: Write the failing tests**

Append to `crates/core/src/engine/translate/tests.rs`:

```rust
fn spans_text<'a>(formula: &'a str) -> Vec<&'a str> {
    let expr = crate::parser::parse_formula(formula).unwrap();
    collect_shiftable_refs(&expr)
        .into_iter()
        .map(|(span, _)| &formula[span.offset..span.offset + span.length])
        .collect()
}

#[test]
fn bare_cell_reference_is_collected() {
    assert_eq!(spans_text("=A1"), vec!["A1"]);
}

#[test]
fn sheet_qualified_reference_is_collected() {
    assert_eq!(spans_text("=Sheet1!A1"), vec!["Sheet1!A1"]);
}

#[test]
fn defined_name_is_not_collected() {
    assert_eq!(spans_text("=TAX_RATE"), Vec::<&str>::new());
}

#[test]
fn function_name_is_not_collected() {
    assert_eq!(spans_text("=SUM(A1,B1)"), vec!["A1", "B1"]);
}

#[test]
fn string_literal_is_not_collected() {
    assert_eq!(spans_text("=CONCAT(\"A1\", B1)"), vec!["B1"]);
}

#[test]
fn range_is_collected_as_single_span() {
    assert_eq!(spans_text("=SUM(A1:B2)"), vec!["A1:B2"]);
}

#[test]
fn let_binding_name_and_shadowed_body_use_are_skipped() {
    assert_eq!(spans_text("=LET(A1, 5, A1*2)"), Vec::<&str>::new());
}

#[test]
fn let_value_expr_self_reference_is_a_real_cell_ref() {
    // A1 inside the value expr is not yet bound (LET binds only after its
    // own value expr evaluates), so it's the real cell, not the local name.
    assert_eq!(spans_text("=LET(A1, A1+1, A1*2)"), vec!["A1"]);
}

#[test]
fn let_second_pair_value_expr_sees_first_binding_as_local() {
    assert_eq!(spans_text("=LET(A1, 5, B1, A1+1, B1)"), Vec::<&str>::new());
}

#[test]
fn lambda_param_and_body_use_are_skipped() {
    assert_eq!(spans_text("=LAMBDA(A1, A1+1)(5)"), Vec::<&str>::new());
}

#[test]
fn lambda_call_args_are_evaluated_in_outer_scope() {
    // the invocation argument is a real cell ref, not shadowed by the param
    assert_eq!(spans_text("=LAMBDA(A1, A1+1)(B1)"), vec!["B1"]);
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: compile error, `cannot find function 'collect_shiftable_refs' in this scope`.

- [ ] **Step 3: Implement `collect_shiftable_refs`**

Add to `crates/core/src/engine/translate.rs` (add `use crate::parser::ast::{Expr, Span};` to the `use` block):

```rust
fn normalize_name(name: &str) -> String {
    name.to_uppercase().replace('$', "")
}

/// Collect every reference in `expr` that should be shifted: sheet-qualified
/// `Expr::Reference` nodes, and bare `Expr::Variable` nodes that classify as
/// `Ref::Cell`/`Ref::Range` and are not shadowed by an enclosing `LET`/
/// `LAMBDA` binding of the same (normalized) name.
fn collect_shiftable_refs(expr: &Expr) -> Vec<(Span, Ref)> {
    let mut out = Vec::new();
    let mut scope: Vec<String> = Vec::new();
    walk(expr, &mut scope, &mut out);
    out
}

fn walk(expr: &Expr, scope: &mut Vec<String>, out: &mut Vec<(Span, Ref)>) {
    match expr {
        Expr::Number(..) | Expr::Text(..) | Expr::Bool(..) => {}
        Expr::Reference(r, span) => out.push((span.clone(), r.clone())),
        Expr::Variable(name, span) => {
            if !scope.contains(&normalize_name(name)) {
                match Ref::classify(name) {
                    Ref::Name(_) => {}
                    r => out.push((span.clone(), r)),
                }
            }
        }
        Expr::UnaryOp { operand, .. } => walk(operand, scope, out),
        Expr::BinaryOp { left, right, .. } => {
            walk(left, scope, out);
            walk(right, scope, out);
        }
        Expr::FunctionCall { name, args, .. }
            if name == "LET" && args.len() >= 3 && args.len() % 2 == 1 =>
        {
            let pair_count = (args.len() - 1) / 2;
            let mut bound = 0;
            for i in 0..pair_count {
                // Value expr sees only bindings 0..i-1 (matches let_fn.rs:
                // evaluate_expr runs before ctx.set for this pair).
                walk(&args[i * 2 + 1], scope, out);
                match &args[i * 2] {
                    Expr::Variable(n, _) => {
                        scope.push(normalize_name(n));
                        bound += 1;
                    }
                    other => walk(other, scope, out), // malformed name slot
                }
            }
            walk(&args[args.len() - 1], scope, out); // body sees all bindings
            for _ in 0..bound {
                scope.pop();
            }
        }
        Expr::FunctionCall { name, args, .. } if name == "LAMBDA" && !args.is_empty() => {
            let param_count = args.len() - 1;
            let mut bound = 0;
            for param_expr in &args[..param_count] {
                match param_expr {
                    Expr::Variable(n, _) => {
                        scope.push(normalize_name(n));
                        bound += 1;
                    }
                    other => walk(other, scope, out), // malformed param slot
                }
            }
            walk(&args[args.len() - 1], scope, out); // body
            for _ in 0..bound {
                scope.pop();
            }
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                walk(arg, scope, out);
            }
        }
        Expr::Array(elems, _) => {
            for elem in elems {
                walk(elem, scope, out);
            }
        }
        Expr::Apply { func, call_args, .. } => {
            // call_args are evaluated in the caller's (outer) scope, before
            // func's own LAMBDA params are pushed.
            for arg in call_args {
                walk(arg, scope, out);
            }
            walk(func, scope, out);
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: all 25 tests in this module pass (14 from Tasks 1-2 + 11 new).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/engine/translate.rs crates/core/src/engine/translate/tests.rs
git commit -m "feat(core): scope-aware ref collection skips LET/LAMBDA bindings (#709)"
```

---

### Task 4: `translate_text` — parse, collect, splice right-to-left

**Files:**
- Modify: `crates/core/src/engine/translate.rs`
- Modify: `crates/core/src/engine/translate/tests.rs`

**Interfaces:**
- Consumes: `collect_shiftable_refs` (Task 3), `shift_ref_text` (Task 2), `crate::parser::parse_formula(formula: &str) -> Result<Expr, ParseError>` (existing, `crates/core/src/parser/mod.rs:486`).
- Produces: `pub(crate) fn translate_text(formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError>` in `crates/core/src/engine/translate.rs`. Task 5 (`Engine::translate_formula`) calls this directly.

- [ ] **Step 1: Write the failing tests**

Append to `crates/core/src/engine/translate/tests.rs`:

```rust
#[test]
fn shifts_simple_relative_reference() {
    assert_eq!(translate_text("=A1", 1, 1).unwrap(), "=B2");
}

#[test]
fn preserves_absolute_axis() {
    assert_eq!(translate_text("=$A$1+B1", 1, 1).unwrap(), "=$A$1+C2");
}

#[test]
fn shifts_range_both_corners() {
    assert_eq!(translate_text("=SUM(A1:B2)", 1, 1).unwrap(), "=SUM(B2:C3)");
}

#[test]
fn out_of_bounds_becomes_ref_error() {
    assert_eq!(translate_text("=A1", -1, 0).unwrap(), "=#REF!");
}

#[test]
fn leaves_defined_names_and_function_names_untouched() {
    assert_eq!(translate_text("=SUM(A1,TAX_RATE)", 1, 0).unwrap(), "=SUM(A2,TAX_RATE)");
}

#[test]
fn leaves_string_literals_untouched() {
    assert_eq!(translate_text("=CONCAT(\"A1\",B1)", 1, 0).unwrap(), "=CONCAT(\"A1\",B2)");
}

#[test]
fn propagates_parse_errors() {
    assert!(translate_text("=SUM(", 0, 0).is_err());
}

#[test]
fn splice_survives_length_changing_replacement_at_higher_offset() {
    // A9 -> A10 grows by one byte; must not corrupt the earlier A1 -> A2 splice.
    assert_eq!(translate_text("=A1+A9", 1, 0).unwrap(), "=A2+A10");
}

#[test]
fn cross_sheet_reference_shifts_correctly() {
    assert_eq!(translate_text("=Sheet1!A1", 1, 1).unwrap(), "=Sheet1!B2");
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: compile error, `cannot find function 'translate_text' in this scope`.

- [ ] **Step 3: Implement `translate_text`**

Add to `crates/core/src/engine/translate.rs` (add `use crate::types::ParseError;` to the `use` block):

```rust
/// Parse `formula`, shift every relative reference axis by `(d_row, d_col)`,
/// and splice the result back into the original text. See the module-level
/// doc comment and `docs/superpowers/specs/2026-07-14-translate-formula-design.md`.
pub(crate) fn translate_text(formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError> {
    let expr = crate::parser::parse_formula(formula)?;
    let mut spans = collect_shiftable_refs(&expr);
    spans.sort_by(|a, b| b.0.offset.cmp(&a.0.offset)); // right to left
    let mut out = formula.to_string();
    for (span, r) in spans {
        let replacement = shift_ref_text(&r, d_row, d_col);
        let start = span.offset;
        let end = span.offset + span.length;
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p truecalc-core engine::translate:: -- --nocapture`
Expected: all 34 tests in this module pass (25 from Tasks 1-3 + 9 new).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/engine/translate.rs crates/core/src/engine/translate/tests.rs
git commit -m "feat(core): add translate_text splice entry point (#709)"
```

---

### Task 5: `Engine::translate_formula` — public API with flavor gating

**Files:**
- Modify: `crates/core/src/engine/mod.rs`
- Modify: `crates/core/src/engine/tests.rs`

**Interfaces:**
- Consumes: `translate::translate_text(formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError>` (Task 4).
- Produces: `pub fn translate_formula(&self, formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError>` on `impl Engine`. This is the public, documented, stable API surface for #709; Task 6's WASM wrapper calls this (via `Engine::sheets()`).

- [ ] **Step 1: Write the failing tests**

Append to `crates/core/src/engine/tests.rs` (which already starts with `use super::*; use std::collections::HashMap;`):

```rust
#[test]
fn translate_formula_shifts_relative_reference() {
    let engine = Engine::sheets();
    assert_eq!(engine.translate_formula("=A1", 1, 1), Ok("=B2".to_string()));
}

#[test]
fn translate_formula_preserves_absolute_reference() {
    let engine = Engine::sheets();
    assert_eq!(engine.translate_formula("=$A$1", 5, 5), Ok("=$A$1".to_string()));
}

#[test]
fn translate_formula_excel_flavor_is_unsupported() {
    let engine = Engine::excel();
    assert!(engine.translate_formula("=A1", 1, 1).is_err());
}

#[test]
fn translate_formula_propagates_parse_error() {
    let engine = Engine::sheets();
    assert!(engine.translate_formula("=SUM(", 0, 0).is_err());
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cargo test -p truecalc-core engine:: -- --nocapture`
Expected: compile error, `no method named 'translate_formula' found for struct 'Engine'`.

- [ ] **Step 3: Implement `Engine::translate_formula`**

In `crates/core/src/engine/mod.rs`, add this method inside `impl Engine` (a natural spot is right after `pub fn validate`, before `pub fn evaluate`):

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p truecalc-core engine:: -- --nocapture`
Expected: all tests in `crates/core/src/engine/tests.rs` pass, including the 4 new ones.

- [ ] **Step 5: Run the full core test suite and clippy**

Run: `cargo test -p truecalc-core`
Expected: all tests pass, zero failures.

Run: `cargo clippy -p truecalc-core -- -D warnings`
Expected: clean, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/engine/mod.rs crates/core/src/engine/tests.rs
git commit -m "feat(core): expose Engine::translate_formula (closes #709)"
```

---

### Task 6: WASM wrapper — `TranslateResult` + `translate_formula`

**Files:**
- Modify: `crates/wasm/src/lib.rs`
- Create: `crates/wasm/tests/translate_formula.rs`

**Interfaces:**
- Consumes: `truecalc_core::Engine::sheets().translate_formula(formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError>` (Task 5).
- Produces: `pub struct TranslateResult { pub formula: Option<String>, pub error: Option<String> }` and `#[wasm_bindgen] pub fn translate_formula(formula: &str, d_row: i32, d_col: i32) -> TranslateResult`, both at the `truecalc_wasm` crate root.

- [ ] **Step 1: Write the failing tests**

Create `crates/wasm/tests/translate_formula.rs`:

```rust
//! Surface-shape tests for `translate_formula`/`TranslateResult` (issue #709).
//!
//! These run natively under `cargo nextest`/`cargo test`; no wasm runtime is
//! needed since neither the input nor the output touches `JsValue` (unlike
//! `evaluate`'s `variables` parameter — see `wasm_surface.rs` for that case).

use truecalc_wasm::translate_formula;

#[test]
fn shifts_relative_reference() {
    let result = translate_formula("=A1", 1, 1);
    assert_eq!(result.formula.as_deref(), Some("=B2"));
    assert_eq!(result.error, None);
}

#[test]
fn preserves_absolute_reference() {
    let result = translate_formula("=$A$1", 5, 5);
    assert_eq!(result.formula.as_deref(), Some("=$A$1"));
}

#[test]
fn parse_error_surfaces_in_error_field() {
    let result = translate_formula("=SUM(", 0, 0);
    assert_eq!(result.formula, None);
    assert!(result.error.is_some());
}

#[test]
fn out_of_bounds_becomes_ref_error_text() {
    let result = translate_formula("=A1", -1, 0);
    assert_eq!(result.formula.as_deref(), Some("=#REF!"));
    assert_eq!(result.error, None);
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cargo test -p truecalc-wasm --test translate_formula`
Expected: compile error, `unresolved import 'truecalc_wasm::translate_formula'`.

- [ ] **Step 3: Implement `TranslateResult` and `translate_formula`**

In `crates/wasm/src/lib.rs`, add immediately after the existing `validate` function (after the closing brace that follows `ValidateResult { valid: false, error: Some(e.to_string()) }`):

```rust
#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct TranslateResult {
    #[tsify(optional)]
    pub formula: Option<String>,
    #[tsify(optional)]
    pub error: Option<String>,
}

/// Shift every relative cell/range reference in `formula` by `(d_row, d_col)`
/// — the fill / copy-paste reference-adjustment transform. `$`-absolute axes
/// are left unchanged; an out-of-bounds corner becomes literal `#REF!`.
///
/// Sheets flavor only (issue #709 v1); Excel support is a follow-up.
#[wasm_bindgen]
pub fn translate_formula(formula: &str, d_row: i32, d_col: i32) -> TranslateResult {
    match truecalc_core::Engine::sheets().translate_formula(formula, d_row as i64, d_col as i64) {
        Ok(f) => TranslateResult { formula: Some(f), error: None },
        Err(e) => TranslateResult { formula: None, error: Some(e.to_string()) },
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p truecalc-wasm --test translate_formula`
Expected: all 4 tests pass.

- [ ] **Step 5: Run the full wasm crate test suite and clippy**

Run: `cargo test -p truecalc-wasm`
Expected: all tests pass, zero failures.

Run: `cargo clippy -p truecalc-wasm -- -D warnings`
Expected: clean, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/wasm/src/lib.rs crates/wasm/tests/translate_formula.rs
git commit -m "feat(wasm): expose translate_formula (#709)"
```

---

### Task 7: Full workspace verification

**Files:** none (verification only)

**Interfaces:** none — this task only runs checks across everything Tasks 1-6 produced.

- [ ] **Step 1: Run the full workspace test suite**

Run: `cargo test --workspace`
Expected: zero failures across every crate (`core`, `workbook`, `mcp`, `wasm`, `wasm-workbook`, `xtask`).

- [ ] **Step 2: Run clippy across the full workspace**

Run: `cargo clippy --workspace -- -D warnings`
Expected: clean, no warnings. This is CI's exact invocation per this repo's CLAUDE.md.

- [ ] **Step 3: Run cargo fmt check**

Run: `cargo fmt --check`
Expected: no diff. If there is one, run `cargo fmt` and re-run Step 1-2, then amend the affected task's commit is not appropriate this late — instead commit the formatting fix separately:

```bash
git add -u
git commit -m "chore: cargo fmt"
```

- [ ] **Step 4: Confirm the branch is ready for a PR**

Run: `git log --oneline main..HEAD` and `git status`
Expected: a clean sequence of the commits from Tasks 1-6 (plus an optional fmt fix), no uncommitted changes. Do not open the PR as part of this plan — that's a separate, explicit step per this repo's CLAUDE.md PR Lifecycle section (open PR, then monitor CI with `gh run list`/`gh run view --log-failed` until green, then assign `@hhimanshu`).
