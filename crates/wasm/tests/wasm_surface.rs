//! End-to-end WASM-surface tests (issue #569), exercising the real wasm-bindgen
//! ABI. Run with `wasm-pack test --node crates/wasm` (or `--headless --chrome`).
//!
//! Gated to `wasm32` so the native `cargo nextest` run (CI's test job) skips it;
//! the native shape coverage lives in `eval_result.rs`. CI builds the wasm
//! package via `wasm-pack build` but does not currently run `wasm-pack test`, so
//! these are developer-facing checks of the live ABI.
#![cfg(target_arch = "wasm32")]

use truecalc_wasm::{evaluate, EvalResult};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn array_formula_returns_nested_array_result() {
    // SEQUENCE(2,2) -> 2x2 array; previously errored with "array not supported".
    match evaluate("SEQUENCE(2,2)", JsValue::UNDEFINED) {
        EvalResult::Array { value } => {
            assert_eq!(value.len(), 2, "two rows");
            for row in value {
                match row {
                    EvalResult::Array { value: cells } => assert_eq!(cells.len(), 2),
                    other => panic!("expected row array, got {other:?}"),
                }
            }
        }
        other => panic!("expected array result, got {other:?}"),
    }
}

#[wasm_bindgen_test]
fn one_dimensional_array_formula_returns_flat_array() {
    match evaluate("{1,2,3}", JsValue::UNDEFINED) {
        EvalResult::Array { value } => assert_eq!(value.len(), 3),
        other => panic!("expected array result, got {other:?}"),
    }
}

#[wasm_bindgen_test]
fn date_formula_returns_distinct_date_result() {
    // DATE(...) is date-typed; previously collapsed to `number`.
    match evaluate("DATE(2026,6,9)", JsValue::UNDEFINED) {
        EvalResult::Date { value } => assert_eq!(value, 46180.0),
        other => panic!("expected date result, got {other:?}"),
    }
}

#[wasm_bindgen_test]
fn scalar_number_formula_still_returns_number() {
    match evaluate("1+1", JsValue::UNDEFINED) {
        EvalResult::Number { value } => assert_eq!(value, 2.0),
        other => panic!("expected number result, got {other:?}"),
    }
}
