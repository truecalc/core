# `$`-absolute reference support — design

Issue: [truecalc/core#708](https://github.com/truecalc/core/issues/708)

## Problem

The engine parse-errors on any `$` in a cell reference. `=$A$1+1`, `=$A1`,
`=A$1` all fail (Google Sheets accepts all three — `$` marks a column and/or
row as absolute; for a static read the value is identical to the relative
form). `$` support was never implemented: there is no absolute/relative
concept anywhere in the reference model, and `$` does not appear in the
grammar at all today.

Repro (confirmed against `Engine::sheets().parse()`):

| Formula | Result |
|---|---|
| `=$A$1+1` | `Parse error @ 1` |
| `=$A1` | `Parse error @ 1` |
| `=A$1` | `Unexpected input '$1' @ 2` |
| `=A1` | OK — `Variable("A1", ...)` |

## Goals

- Parse `$` on either or both axes of a cell reference, bare or
  sheet-qualified, including range endpoints (`$A$1:D4`, `A1:$D$4`).
- Retain the per-axis absolute/relative marker through parse → `Display`,
  so a future formula-translation feature can preserve it.
- Static evaluation of `$A$1` and `A1` must produce identical values — `$`
  is a display/anchoring marker only, never a resolution difference.
- No behavior change for any formula that doesn't use `$` (it appears
  nowhere in the grammar today, so every input this change newly accepts
  was previously a hard parse error).

## Non-goals

- A `translateFormula` / fill-across API that actually *uses* the
  absolute/relative markers to rewrite formulas on copy — that's a
  separate, follow-on feature this work is a prerequisite for.
- R1C1-style references — not used anywhere in this codebase.

## Public API impact

`CellAddr` (public struct, public `col`/`row` fields, not
`#[non_exhaustive]`) gets two new public fields. This breaks any external
`CellAddr { col, row }` struct literal or exhaustive match. **Accepted**,
per maintainer decision. Ship as a breaking change — conventional-commit
`!`/`BREAKING CHANGE:` footer so release-please bumps the major version,
plus a CHANGELOG entry.

Confirmed via direct inspection: `crates/wasm` and `crates/mcp` never
reference `CellAddr`/`Ref` directly, so the in-repo blast radius is
`crates/core` (parser/eval) and `crates/workbook` (which reads `Ref`'s
`Display` output as a dedup key in one place — see below). Downstream
consumers outside this repo (e.g. the private `@truecalc/workbook` npm
package the issue was filed from) are out of scope for this repo's PR;
they adapt to the major-version bump on their own schedule.

## Design

### 1. Data model — `crates/core/src/parser/refs.rs`

`CellAddr` gains `pub col_abs: bool`, `pub row_abs: bool`.

```rust
impl CellAddr {
    /// Relative address (no `$` on either axis) — the common case.
    pub const fn new(col: u32, row: u32) -> Self {
        Self { col, row, col_abs: false, row_abs: false }
    }
    pub const fn with_col_abs(mut self, abs: bool) -> Self { self.col_abs = abs; self }
    pub const fn with_row_abs(mut self, abs: bool) -> Self { self.row_abs = abs; self }
}
```

`CellAddr::new` + chainable builders instead of raw struct literals or a
positional constructor, to avoid a `col_abs`/`row_abs` transposition
footgun at call sites.

`CellAddr::parse(text)`: accepts an optional `$` immediately before the
column letters and an optional `$` immediately before the row digits.
Column/row numeric parsing (including overflow and `row == 0` rejection)
is otherwise unchanged.

`Display for CellAddr`: emits `$` before the column letters and/or before
the row digits per the flags.

`Ref` gains one new method:

```rust
impl Ref {
    /// Canonical text with all `$` anchors stripped — used as an
    /// identity/lookup key where `$` must not affect equality (e.g.
    /// resolver-override lookups, dependency-graph dedup). Built
    /// structurally (zeroing col_abs/row_abs before Display), not by
    /// string-replacing `$` out of the rendered text, because a quoted
    /// sheet name may legitimately contain a literal `$` character.
    pub fn relative_display(&self) -> String { ... }
}
```

`Ref::classify` needs **no changes** — bare refs stay raw source text
until classified, and `CellAddr::parse` now understands `$`, so
`"$A$1"` / `"$A$1:$D$4"` classify into `Ref::Cell`/`Ref::Range`
automatically.

### 2. Parsing — `tokens.rs` + `parser/mod.rs`

New tokenizer in `tokens.rs`:

```rust
/// `$?letters$?digits` — e.g. `A1`, `$A1`, `A$1`, `$A$1`. Requires at
/// least one literal `$` to match, so it never competes with the plain
/// `identifier()` grammar (every string this *can* match was previously
/// always a parse error, since `$` appears nowhere else in the grammar).
/// On failure, returns the *original* input untouched.
pub fn dollar_cell_ref(i: &str) -> IResult<&str, &str> { ... }
```

Unit test: `dollar_cell_ref("A1")` fails with the input unchanged (so the
`identifier()` fallback in callers parses from the correct offset).

`parser/mod.rs` changes:

- Replace the `is_cell_ref(name)` shape-gate (used to decide whether a
  parsed identifier may take a `:end` range tail) with
  `CellAddr::parse(name).is_some()` **everywhere it's used as a range-end
  gate** — this one change makes both the existing bare-range-tail path
  and the sheet-qualified range-tail path (`parse_ref_body`) `$`-aware
  without a second, parallel "dollar-flavored" range implementation.
  (`is_cell_ref` was a looser shape check than full `CellAddr::parse`
  validation — e.g. it doesn't reject `row == 0` or column overflow — so
  this is also a minor correctness tightening, not just a `$` accommodation.)
- `parse_primary`: try `dollar_cell_ref(i)` as an early alternative
  *before* the `identifier()` branch. A `$`-bearing token can only ever be
  a bare cell/range reference (never a sheet name, function call, or
  plain variable), so on match: check for a `:` tail (end parsed via
  `identifier(...).or(dollar_cell_ref(...))`, gated by
  `CellAddr::parse(...).is_some()`, consistent with the bullet above), else
  emit `Expr::Variable(span, ...)` directly — mirroring exactly how plain
  `A1` becomes `Expr::Variable("A1", ...)` today. If the tail doesn't gate
  through, the colon is not consumed (matches today's behavior for
  `A1:FOO`, which is a parse-time error, not a runtime `#NAME?`).
- `parse_ref_body` (the post-`Sheet1!` cell text): swap
  `identifier(i)` for `identifier(i).or(dollar_cell_ref(i))` for both the
  cell text and its range-tail end, so `Sheet1!$A$1`,
  `Sheet1!$A$1:$D$4`, and mixed forms (`Sheet1!$A$1:D4`) all parse.

### 3. Evaluation-time name normalization — `eval/context/mod.rs`

`Context` centralizes key normalization: replace the bare `.to_uppercase()`
in `get`/`lookup`/`set`/`remove` with

```rust
fn normalize(name: &str) -> String {
    name.to_uppercase().replace('$', "")
}
```

Safe unconditionally here because `$` can only appear in `Expr::Variable`
text when it's a cell/range reference (produced by `dollar_cell_ref`) —
never a legitimate parameter/variable/named-range name, since
`identifier()` still disallows `$`. This closes a real gap the naive
version of this design had: without centralizing in `Context`,
`=LET($A$1, 5, $A$1+1)` would `set` its binding under key `"$A$1"` but
`get` it back under a separately-stripped `"A1"` — a silent miss that
falls through to resolving the real cell A1 instead of returning 6.

`evaluate_expr`'s `Expr::Reference` arm changes its override-lookup call
from `ctx.ctx.lookup(&r.to_string())` to
`ctx.ctx.lookup(&r.relative_display())` — structural, not a string
replace, so a sheet legitimately named `Q1$Data` isn't corrupted.

`ctx.resolve_ref(r)` itself (the actual value resolution, as opposed to
the override-binding lookup) needs **no changes** — it reads
`addr.col`/`addr.row` only, which are unaffected by the new flags. `$A$1`
and `A1` resolve to the same value automatically.

### 4. `crates/workbook/src/depgraph.rs`

`Precedent::Unresolved(r.to_string())` (four call sites, used as a
`HashSet` dedup key) has the same identity-key issue as the eval
override-lookup: once `Display` embeds `$`, a missing-sheet/missing-name
reference reached via `$A$1` and one reached via `A1` would no longer
dedupe as "the same unresolved precedent." Swap to
`Precedent::Unresolved(r.relative_display())`, reusing the same helper.
Small, mechanical, same root cause as (3).

### 5. Migrating existing `CellAddr { col, row }` construction

~15 struct-literal sites across `crates/core/tests/{parser_refs,resolver,
extract_refs_registry,workbook_inputs_conformance}.rs`, plus 2 doc-tests
(`refs.rs` module doc-comment, `eval/resolver.rs` `extract_refs` doc
example) — all switch to `CellAddr::new(col, row)`. Acceptance bar:
`cargo build --workspace --tests --doc` clean.

## Testing plan (TDD)

- `tokens.rs`: `dollar_cell_ref` — all four shapes, no-match-no-consume on
  plain `A1`/generic identifiers, malformed shapes (`$1A`, `A$$1`, `$$A1`,
  `A$`, `$A`).
- `refs.rs`: `CellAddr::parse`/`Display` round-trip for all four
  col_abs×row_abs combinations; invalid shapes still rejected; `row == 0`
  and column-overflow still rejected with `$` present.
- `parser_refs.rs` (or a new test module): `=$A$1+1`, `=$A1`, `=A$1`,
  `=Sheet1!$A$1`, `='Sheet 1'!$A1:B$4`, mixed-corner ranges
  (`=$A$1:D4`, `=A1:$D$4`), malformed `$A$1:FOO` (parse error, matching
  today's `A1:FOO` behavior).
- `resolver.rs` / an eval test: `$A$1` and `A1` resolve to the same value
  via a `MapResolver`; a caller-supplied override binding keyed without
  `$` is found when the formula uses `$` (`Expr::Reference` path) and
  vice versa is not required (only the no-`$` canonical key needs to
  match).
- New test: `=LET($A$1, 5, $A$1+1)` evaluates to `6` (closes the
  Context-normalization gap directly).
- `crates/workbook`: a depgraph test confirming an unresolved reference
  reached via `$`- and non-`$`-forms of the same missing target dedupes
  to one `Precedent::Unresolved`.
- No new fixture-TSV rows — this is parser-level, not a Google-Sheets
  value-conformance question covered by the pipeline fixtures.

## Open risks / explicitly accepted

- Breaking public API change, by design (see above).
- `LAMBDA`/`LET` can (accidentally) take a `$`-shaped token as a parameter
  name, e.g. `LAMBDA($A$1, $A$1+1)`. This is not rejected — it's treated
  like any other identifier, normalized the same as every other
  parameter. Semantically odd but harmless once (3) is in place; adding a
  dedicated rejection would be extra special-casing for no correctness
  benefit.
