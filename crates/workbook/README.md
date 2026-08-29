# truecalc-workbook

[![crates.io](https://img.shields.io/crates/v/truecalc-workbook)](https://crates.io/crates/truecalc-workbook)
[![crates.io downloads](https://img.shields.io/crates/d/truecalc-workbook)](https://crates.io/crates/truecalc-workbook)
[![docs.rs](https://img.shields.io/docsrs/truecalc-workbook)](https://docs.rs/truecalc-workbook)
[![license](https://img.shields.io/badge/license-Elastic--2.0-blue)](https://github.com/truecalc/core/blob/main/crates/workbook/LICENSE)

Workbook layer for the [truecalc](https://github.com/truecalc/core) spreadsheet engine: engine-locked workbook, worksheet, and cell value types with a canonical JSON serialization contract.

## Install

```toml
[dependencies]
truecalc-workbook = "0.9"
```

Or via cargo:

```sh
cargo add truecalc-workbook
```

## Quick start

```rust
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

// Create a workbook locked to Google Sheets semantics.
let mut wb = Workbook::new(EngineFlavor::Sheets);

// Add a sheet and write some cells.
wb.add_sheet(Worksheet::new("Budget")).unwrap();
let a1 = Address::from_a1("A1").unwrap();
let a2 = Address::from_a1("A2").unwrap();
let a3 = Address::from_a1("A3").unwrap();
wb.set("Budget", a1, CellInput::Literal(Value::Number(1000.0))).unwrap();
wb.set("Budget", a2, CellInput::Literal(Value::Number(500.0))).unwrap();
wb.set("Budget", a3, CellInput::Formula("=SUM(A1:A2)".to_string())).unwrap();

// Recalculate to evaluate all formulas.
// RecalcContext::new(unix_ms, iana_tz, rng_seed)
let ctx = RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).unwrap();
let _changes = wb.recalc(&ctx);

// Read back the computed result.
assert_eq!(wb.get("Budget", a3).unwrap().value(), &Value::Number(1500.0));
```

## Design

- A `Workbook` is a **value object** — `Clone + PartialEq + Hash`, no hidden state, no
  callbacks. Mutate via [`Workbook::set`] / [`Workbook::clear`], then drive recalc.

- The **engine flavor** (`sheets` | `excel`) is required at creation and immutable for the
  workbook's lifetime. It controls formula semantics and the date serial system.

- **[`RecalcContext`]** pins volatile functions (`NOW`, `TODAY`) to a fixed UTC instant +
  IANA timezone via the vendored `chrono-tz` database (not the host OS tz tables).
  Same workbook + same context ⇒ byte-identical recomputed grid.

- **[`CellInput`]** distinguishes `Literal(value)` from `Formula("=...".to_string())`.
  Formula syntax is validated against the locked engine at `set` time.

## Recalc modes

- [`Workbook::recalc`] — full recalc, evaluates every formula cell in topological order.
- [`Workbook::recalc_incremental`] — incremental recalc, recomputes only the transitive
  dependents of the edited cells (plus all volatile cells). Produces the same result as
  full recalc.

Both return an ordered list of [`Change`] values describing every cell that changed.

## Performance

The dependency graph, incremental recalc, and full recalc are benchmarked with
[Criterion](https://docs.rs/criterion) and gated in CI against committed
baselines. A representative sample, single-sheet and many-sheet side by side —
how recalc cost scales across sheets in one workbook is exactly what these
benchmarks are meant to answer:

| shape | full recalc | one edit |
|---|---|---|
| 1,000 independent cells, one sheet | 4.38 ms | 1.13 ms |
| 10,000 cells in subtotal blocks, one sheet | 26.39 ms | 3.82 ms |
| 20,000-row sparse column, one sheet | 130.80 ms | — |
| 200 sheets × 50 cells | 45.61 ms | 10.75 ms |
| 200 sheets × 50 cells, cross-sheet refs | 44.44 ms | — |

("—" = no incremental-edit benchmark exists for that shape.)

**Method:** recorded on "Apple M1 Max (10 core), macOS 14.4, rustc 1.94.1,
release profile; best of 5 full bench runs" (verbatim from `recorded_on` in
`baselines.json`, below). These are best-of-5 minima on one machine on one
day — not a guarantee of what any other machine, workload, or day will show.

**What CI actually gates on:** not the milliseconds above. Each benchmark is
normalized as `ref_units = min(benchmark time) / min(reference time)` against
a `calibration/hash_alloc` reference — a fixed allocate-and-hash workload with
no dependency on truecalc code — measured in the same run. Dividing by that
reference is what lets a baseline recorded on one machine still mean something
on another; `best_ns_recorded` (what the table above is derived from) is
informational only, and nothing is checked against it directly. The
millisecond figures above are illustrative; the `ref_units` ratios in
`baselines.json` are the actual contract, checked both for regressions and for
unexpected improvements against two-sided bands by
[`.github/scripts/check_perf_regression.py`](../../.github/scripts/check_perf_regression.py).

**Source of truth:** [`benches/baselines.json`](benches/baselines.json) is the
authoritative, always-current record of every gated benchmark — it is
regenerated and re-gated in CI on every change, so it can drift from this
table over time. Most entries were recorded 2026-08-28; five `multi_sheet`
entries and four `incremental_recalc`/`incremental_recalc_cold` entries were
re-recorded 2026-08-29 on the same machine after a dependency-graph fix
changed their measured cost (see the JSON's `note` field for the full
explanation).

**Run locally** (from the repo root):

```sh
cargo bench -p truecalc-workbook --bench workbook_perf -- --output-format bencher
```

To reproduce the CI regression gate against the committed baselines:

```sh
cargo bench -p truecalc-workbook --bench workbook_perf -- --output-format bencher \
  | python3 .github/scripts/check_perf_regression.py
```

## JSON serialization

[`Workbook::to_json`] / [`Workbook::from_json`] implement the canonical RFC 8785 / JCS
serialization boundary — byte-identical output across Rust, WASM, MCP, and REST surfaces.
The JSON schema is the cross-surface contract; see `schema/` for the JSON Schema spec.

### Schema summary

```json
{
  "version": "1",
  "engine": "sheets",
  "names": [],
  "sheets": [
    {
      "name": "Budget",
      "cells": {
        "A1": { "value": 1000.0 },
        "A3": { "formula": "=SUM(A1:A2)", "value": 1500.0 }
      }
    }
  ]
}
```

Key schema invariants:
- `engine` is `"sheets"` or `"excel"` — required, immutable.
- `version` is the string `"1"` — compared by exact match.
- Cells without a formula have only `value`; formula cells store `formula` + last `value`.
- Named ranges are validated against existing sheets at deserialize time.

## Cookbook example

See [`examples/workbook-budget/`](../../examples/workbook-budget/) for a worked example
that creates a budget workbook, sets income and expense formulas, recalcs, and prints the
results.

## Related crates

- [`truecalc-core`](https://crates.io/crates/truecalc-core): the formula parser and evaluator.

## License

[Elastic License 2.0](https://github.com/truecalc/core/blob/main/crates/workbook/LICENSE) (`Elastic-2.0`) — source-available, not MIT.
You may use, copy, modify and redistribute it; you may not offer it to third
parties as a hosted or managed service that provides access to a substantial
set of its functionality.

This crate and `truecalc-wasm-workbook` (published to npm and JSR as
`@truecalc/workbook`) are the only parts of the workspace that are not MIT.
[`truecalc-core`](https://crates.io/crates/truecalc-core), the formula parser
and evaluator this crate depends on, remains MIT.

Every version of `truecalc-workbook` published before 9.0.0 — every 8.x release
and everything before it — was released under MIT and stays MIT permanently.
`9.0.0` is the first version under the new terms; nothing already published
is relicensed or withdrawn.

Full detail: [LICENSING.md](https://github.com/truecalc/core/blob/main/LICENSING.md).
