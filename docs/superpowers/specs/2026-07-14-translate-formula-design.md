# `Engine::translate_formula` design

Issue: [truecalc/core#709](https://github.com/truecalc/core/issues/709)
Prerequisite: [truecalc/core#708](https://github.com/truecalc/core/issues/708) (merged via #710) — `CellAddr` now
carries per-axis `col_abs`/`row_abs` and `Display` round-trips `$`.

## Summary

Expose `Engine::translate_formula(formula, d_row, d_col) -> Result<String, ParseError>`: given formula text,
return the same formula with every relative cell/range reference shifted by `(d_row, d_col)` — the transform
behind spreadsheet fill / copy-paste reference adjustment. `$`-absolute axes stay fixed. This is a pure
formula-text → formula-text operation; it never evaluates anything.

## Scope (v1)

- **Sheets flavor only.** Excel evaluation is stubbed engine-wide (`Engine::excel().evaluate()` always returns
  `#UNSUPPORTED!`), and the only existing grid-bounds constants in the codebase
  (`crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW}`, `18_278` / `10_000_000`) are Sheets-only.
  Calling `translate_formula` on `Engine::excel()` returns
  `Err(ParseError { message: "translate_formula: Excel flavor not yet supported".into(), position: 0 })`.
  Excel support is a follow-up once Excel grid bounds are established.
- No whole-row/column syntax (`A:A`, `1:1`) and no R1C1 AST-level syntax exist in this parser today — nothing
  to handle; not a gap this design introduces.
- `INDIRECT("A1")`/`ADDRESS(...)`-style string arguments are `Expr::Text` literals and are never touched, which
  matches real spreadsheet behavior (fill/copy never rewrites the contents of a string literal).

## API

```rust
impl Engine {
    /// Shift every relative axis of every cell/range reference in `formula` by
    /// `(d_row, d_col)`. `$`-absolute axes are left unchanged. An axis that
    /// shifts out of the Sheets grid (`1..=MAX_ROW` / `1..=MAX_COL`) becomes a
    /// literal `#REF!` for that corner. Sheets flavor only in v1.
    pub fn translate_formula(&self, formula: &str, d_row: i64, d_col: i64) -> Result<String, ParseError>
}
```

- `Err(ParseError)`: malformed input formula (existing parse-error path), or Excel flavor (see above).
- `Ok(String)`: always syntactically valid, even when a reference goes out of bounds.

## Algorithm — span-splice, not full AST reserialization

`Expr` has no `Display`/text-serializer today (only `CellAddr` and `Ref` do — `crates/core/src/parser/refs.rs:49,150`).
Rather than writing a new full-expression-to-text printer, every `Expr` node already carries a `Span` (byte
offset into the *original* formula string, via `Expr::span()` in `crates/core/src/parser/ast.rs`), and span
coverage is byte-accurate with no leading/trailing whitespace or `=`-sign inclusion (verified against
`crates/core/src/parser/mod.rs` and `crates/core/src/parser/tokens.rs::offset`).

1. Parse `formula` into `Expr` via the existing `parse_formula`.
2. Walk the AST with a scope-aware traversal (see below), collecting `(Span, Ref)` for every reference that
   should be shifted.
3. For each `(span, ref)`: shift every `CellAddr`'s `col`/`row` by `d_col`/`d_row` **only on axes where
   `col_abs`/`row_abs` is `false`**. Do the add/compare in `i64` (`col as i64 + d_col`), not `u32`, to avoid
   wraparound on large negative offsets, then range-check before casting back to `u32`. For `Ref::Range`,
   shift `start` and `end` independently (mixed-corner ranges like `A1:$D$4` behave correctly per corner).
4. **Bounds check per corner, not per span.** If a shifted `CellAddr`'s relative axis falls outside
   `1..=MAX_COL` / `1..=MAX_ROW` (constants widened from `private` to `pub(crate)` in
   `crates/core/src/eval/functions/lookup/indirect/mod.rs` and reused here rather than duplicated), that
   *corner's* text becomes the literal `ErrorKind::Ref.to_string()` (`"#REF!"`, reusing the existing `Display`
   impl instead of hardcoding the string twice) while the other corner (if any) keeps its shifted address. A
   single-cell reference has one corner, so it becomes a plain `#REF!` when out of bounds. This matches real
   Excel/Sheets #REF!-propagation behavior more closely than an earlier whole-span-replacement draft.
5. Otherwise, re-render via the existing `Ref` / `CellAddr` `Display` impls (already round-trip `$` since #710)
   and splice the new text into the original formula string at `span`. Splice **right to left** (descending
   start-offset order) so earlier spans' byte offsets stay valid as the string length changes. `Expr::Reference`
   / `Expr::Variable` are leaf AST nodes, so collected spans are always disjoint — no nesting/overlap risk.
6. Everything outside a reference span — operators, function names, numbers, string literals, whitespace,
   defined names — is copied through verbatim from the original source text.

### Scope-aware traversal (LET/LAMBDA local bindings)

**This is the one correctness-critical addition surfaced by independent review.** `LET`/`LAMBDA` binding names
and their in-body uses are plain `Expr::Variable` nodes — structurally indistinguishable from a genuine cell
reference. The evaluator resolves them via local-scope lookup *before* falling back to a real reference
(`Expr::Variable(name, _) => match ctx.ctx.lookup(&name.replace('$', ""))` at `crates/core/src/eval/mod.rs:42`;
`eval_apply` at `crates/core/src/eval/mod.rs:120-136` does the same for LAMBDA params). Without scope tracking,
`=LET(A1, 5, A1*2)` would have its binding name and body reference both shifted, silently corrupting the
formula.

The traversal (structurally similar to `extract_refs`/`collect_refs` in `crates/core/src/eval/resolver.rs`,
but building a splice list instead of a flat `Vec<Ref>`, and scope-aware) must special-case two `Expr` shapes,
using the **same name normalization the evaluator already uses** (`name.to_uppercase().replace('$', "")`,
per `crates/core/src/eval/functions/logical/let_fn.rs:38` and `crates/core/src/eval/mod.rs:133`) so the scope
set matches exactly what the evaluator would bind:

- `Expr::FunctionCall { name, args, .. }` where `name` case-insensitively equals `"LET"`: for the `(name, value)`
  pairs, the name-slot (`args[i*2]`) is never a shift candidate — push its normalized text onto the active scope
  set *before* walking the corresponding value expr (`args[i*2+1]`) and all later pairs/the body (later
  bindings can reference earlier ones, per the existing comment in `let_fn.rs`). Pop all of this `LET` call's
  bindings after finishing its subtree.
- `Expr::FunctionCall { name, args, .. }` where `name` case-insensitively equals `"LAMBDA"`: all args except the
  last are parameter name-slots — never shift candidates, pushed onto the active scope before walking the body
  (last arg). Popped after. (This covers both `Expr::Apply { func: <LAMBDA FunctionCall>, .. }` — since `func`
  is walked through the same generic `FunctionCall` branch — and a bare, uninvoked `=LAMBDA(x, x+1)`.)
- `Expr::Apply { func, call_args, .. }`: `call_args` are evaluated in the **caller's** scope (the arguments
  passed *into* the lambda), so they're walked with the scope as it stood *before* `func`'s own LAMBDA-param
  push. Only `func`'s walk (per the bullet above) sees the new params.
- Any other `Expr::Variable(name, _)`: only a shift candidate if its normalized name is **not** currently in the
  active scope set (i.e., not shadowed by an enclosing `LET`/`LAMBDA` binding of the same name).

## WASM wrapper (`crates/wasm/src/lib.rs`)

Mirrors the existing `validate(formula: &str) -> ValidateResult` pattern (`crates/wasm/src/lib.rs:145-151`):

```rust
#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct TranslateResult {
    #[tsify(optional)]
    pub formula: Option<String>,
    #[tsify(optional)]
    pub error: Option<String>,
}

#[wasm_bindgen]
pub fn translate_formula(formula: &str, d_row: i32, d_col: i32) -> TranslateResult {
    match truecalc_core::Engine::sheets().translate_formula(formula, d_row as i64, d_col as i64) {
        Ok(f) => TranslateResult { formula: Some(f), error: None },
        Err(e) => TranslateResult { formula: None, error: Some(e.to_string()) },
    }
}
```

Note: like `evaluate`/`validate`, this free function hardcodes `Engine::sheets()`. The `Excel flavor -> Err`
path exists in the core API but isn't reachable from the WASM surface in v1 (`createEngine` only supports
`"google-sheets"` today) — acceptable asymmetry, not a defect, since v1 is Sheets-only by design.

## Testing

Rust unit tests (per this repo's CLAUDE.md: test files separate from production source), covering:

- plain relative shift, all four directions, including negative `d_row`/`d_col`
- per-axis `$` — all four combinations (`A1`, `$A1`, `A$1`, `$A$1`), bare and sheet-qualified
- range shift — both endpoints, mixed corners (e.g. `A1:$D$4`)
- cross-sheet references (`Sheet1!A1`, quoted sheet names)
- out-of-bounds: negative/zero row or column, and > `MAX_ROW`/`MAX_COL`, per-corner `#REF!` (including a range
  with exactly one corner out of bounds keeping the other corner's shifted address)
- `LET`/`LAMBDA` binding names and shadowed body references left untouched; a body reference to a name *not*
  shadowed by the local binding still shifts normally
- defined names, string literals, and function names left untouched
- Excel flavor → `Err`

No conformance-fixture-TSV additions — `translateFormula` isn't a Sheets-evaluated formula, so it doesn't
belong in `crates/core/tests/fixtures/google_sheets/`. The out-of-bounds `#REF!` behavior is treated as
well-established, product-agnostic spreadsheet convention rather than something requiring live-Sheets fixture
verification.

## Non-goals

- Excel flavor support (follow-up issue once Excel grid bounds are established).
- Whole-row/column (`A:A`) and R1C1 syntax — not modeled by this parser at all today.
- Rewriting string-literal formula text (e.g. inside `INDIRECT("A1")`).
