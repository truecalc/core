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
