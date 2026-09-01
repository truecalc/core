//! End-to-end WASM-surface tests (issue #973), exercising the real
//! wasm-bindgen ABI. Run with `wasm-pack test --node crates/wasm-workbook`
//! (or `--headless --chrome`).
//!
//! Gated to `wasm32` so the native `cargo nextest` run (CI's test job) skips
//! it; the native shape coverage for `JsWorkbook`'s other methods lives in
//! `round_trip.rs`, `table_bindings.rs` and `dependency_graph.rs`, which test
//! through `truecalc_workbook::Workbook` directly because a `JsValue`-touching
//! `JsWorkbook` method aborts when called outside a real wasm runtime. CI
//! builds the wasm package via `wasm-pack build` but does not currently run
//! `wasm-pack test`, so these are developer-facing checks of the live ABI —
//! see `crates/wasm/tests/wasm_surface.rs` for the same pattern on the
//! calc-only binding.
#![cfg(target_arch = "wasm32")]

use truecalc_wasm_workbook::JsWorkbook;
use wasm_bindgen_test::*;

/// Parses `resolved()`'s tagged-JSON string into a `serde_json::Value` for
/// field-level assertions — robust to the untested key order `serde_json`'s
/// default (non-`preserve_order`) map emits.
fn resolved_json(wb: &JsWorkbook, sheet: &str, a1: &str) -> serde_json::Value {
    let s = wb
        .resolved(sheet, a1)
        .unwrap()
        .as_string()
        .expect("resolved() returns a JSON string for a non-empty cell");
    serde_json::from_str(&s).expect("resolved() returns valid JSON")
}

/// `removeName` (issue #973) is missing from the JS surface entirely before
/// this change — this test does not compile against the pre-fix ABI. Once it
/// exists: defining a name, referencing it in a formula, then removing the
/// name through the binding and recalculating must turn the formula's stale
/// resolved value into `#NAME?`, the same way deleting the name and
/// rebuilding the workbook from scratch already does.
#[wasm_bindgen_test]
fn remove_name_through_binding_invalidates_dependent_formula() {
    let mut wb = JsWorkbook::new("sheets");
    wb.add_sheet("Sheet1").unwrap();
    wb.set("Sheet1", "B1", "10").unwrap();
    wb.set("Sheet1", "B2", "20").unwrap();
    wb.define_name("Total", "Sheet1!B1:B2").unwrap();
    wb.set("Sheet1", "A1", "=SUM(Total)").unwrap();

    wb.recalc(r#"{"timestamp_ms":0,"timezone":"UTC","rng_seed":0}"#)
        .unwrap();
    let before = resolved_json(&wb, "Sheet1", "A1");
    assert_eq!(before["type"], "number", "sanity check: {before}");
    assert_eq!(before["value"], 30.0, "sanity check: {before}");

    wb.remove_name("Total");
    wb.recalc(r#"{"timestamp_ms":0,"timezone":"UTC","rng_seed":0}"#)
        .unwrap();

    let after = resolved_json(&wb, "Sheet1", "A1");
    assert_eq!(
        after["type"], "error",
        "the removed name must no longer resolve — a stale `30` here is \
         exactly the silent-wrong-value bug issue #973 reports: {after}"
    );
    assert_eq!(after["error"], "#NAME?", "got {after}");
}
