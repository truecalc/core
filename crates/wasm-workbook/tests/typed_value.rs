//! Shape coverage for `JsWorkbook::get_typed`/`resolved_typed` ("getTyped" /
//! "resolvedTyped" on the JS surface): the additive, non-breaking typed
//! replacements for `get`/`resolved`'s JSON-string return.
//!
//! Neither new method touches `JsValue` — the WASM-ABI marshaling happens only
//! in the `#[wasm_bindgen]`-generated wrapper the JS host calls through, not
//! in the inherent Rust method these tests call directly — so, like
//! `round_trip.rs` and `sheet_and_value_bindings.rs`, this runs natively under
//! `cargo test`/`cargo nextest` without a wasm runtime. `wasm_surface.rs`
//! additionally exercises the real ABI end-to-end for one representative case.
//!
//! Literal `Zoned`/`Sparkline`/`Array` values have no `JsWorkbook.set(...)`
//! string form, so this builds the workbook natively via `truecalc_workbook`
//! and loads it through `JsWorkbook::fromJSON` — the same JSON a real host
//! would have produced by calling `toJSON()` on a workbook built through the
//! ordinary JS `set`/`setDate` calls.

use truecalc_core::types::zoned::parse_rfc9557;
use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_wasm_value::EvalResult;
use truecalc_wasm_workbook::JsWorkbook;
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// Builds a `JsWorkbook` whose `Sheet1` carries one literal of every `Value`
/// variant, addressed `A1`..`A9`, plus a spilling array formula at `B1` (so
/// `B2` is a *spilled*, non-anchor cell — the case `resolvedTyped`'s `anchor`
/// field exists for).
fn fixture() -> JsWorkbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1".to_string())).unwrap();

    let mut set = |a1: &str, input: CellInput| {
        let addr = Address::from_a1(a1).unwrap();
        wb.set("Sheet1", addr, input).unwrap();
    };

    set("A1", CellInput::Literal(Value::Number(1.5)));
    set("A2", CellInput::Literal(Value::Text("yes".into())));
    set("A3", CellInput::Literal(Value::Boolean(true)));
    set("A4", CellInput::Literal(Value::Date(46180.0)));
    let zoned = parse_rfc9557("2026-07-14T11:00:00+02:00[Europe/Berlin]").unwrap();
    set("A5", CellInput::Literal(Value::Zoned(Box::new(zoned))));
    set("A6", CellInput::Literal(Value::Error("#REF!".into())));
    set("A7", CellInput::Formula("=1/0".to_string())); // ErrorMsg via a diagnostic-carrying error, if any
                                                       // `Value::Empty` cannot be authored as a literal (it means "no formula has
                                                       // run here yet" / "an unauthored reference"), so a formula pointing at a
                                                       // never-authored cell is the way to get one.
    set("A8", CellInput::Formula("=A100".to_string()));
    let sparkline = SparklineSpec {
        chart_type: SparklineChartType::Bar,
        data: vec![SparklineValue::number(1.0), SparklineValue::number(2.0)],
        options: vec![],
    };
    set(
        "A9",
        CellInput::Literal(Value::Sparkline(Box::new(sparkline))),
    );
    // A 2x1 (one column, two rows) spill anchor at B1: B1 is the authored
    // formula, B2 is the spilled (non-anchor) cell `resolvedTyped`'s `anchor`
    // field describes. `;` separates rows (`,` would spill sideways into C1).
    set("B1", CellInput::Formula("={10;20}".to_string()));

    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    wb.recalc(&ctx);

    let json = wb.to_json().unwrap();
    JsWorkbook::from_json(&json).unwrap()
}

fn resolved_value(wb: &JsWorkbook, a1: &str) -> EvalResult {
    wb.resolved_typed("Sheet1", a1).unwrap().unwrap().value
}

#[test]
fn resolved_typed_number() {
    assert!(
        matches!(resolved_value(&fixture(), "A1"), EvalResult::Number { value } if value == 1.5)
    );
}

#[test]
fn resolved_typed_text() {
    assert!(
        matches!(resolved_value(&fixture(), "A2"), EvalResult::Text { value } if value == "yes")
    );
}

#[test]
fn resolved_typed_bool() {
    assert!(matches!(resolved_value(&fixture(), "A3"), EvalResult::Bool { value } if value));
}

#[test]
fn resolved_typed_date() {
    assert!(
        matches!(resolved_value(&fixture(), "A4"), EvalResult::Date { value } if value == 46180.0)
    );
}

#[test]
fn resolved_typed_zoned() {
    match resolved_value(&fixture(), "A5") {
        EvalResult::Zoned { value } => {
            assert_eq!(value, "2026-07-14T11:00:00+02:00[Europe/Berlin]");
        }
        other => panic!("expected Zoned, got {other:?}"),
    }
}

#[test]
fn resolved_typed_error() {
    match resolved_value(&fixture(), "A6") {
        EvalResult::Error { error, message } => {
            assert_eq!(error, "#REF!");
            assert_eq!(message, None);
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn resolved_typed_error_with_message() {
    // `=1/0` is a bare `#DIV/0!` in this engine (no diagnostic message), so
    // this only asserts the shape stays a plain error, not the `message`
    // field's presence — there is no easy fixture for a diagnostic-carrying
    // error through a literal/formula at this layer (see `eval_result.rs`'s
    // `error_with_message_maps_to_error_with_message_key` for that coverage
    // against `truecalc_core::Value::ErrorMsg` directly).
    match resolved_value(&fixture(), "A7") {
        EvalResult::Error { error, .. } => assert_eq!(error, "#DIV/0!"),
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn resolved_typed_empty() {
    assert!(matches!(
        resolved_value(&fixture(), "A8"),
        EvalResult::Empty
    ));
}

#[test]
fn resolved_typed_sparkline() {
    match resolved_value(&fixture(), "A9") {
        EvalResult::Sparkline { value } => {
            assert_eq!(value.charttype, "bar");
            assert_eq!(value.data.len(), 2);
            assert!(matches!(&value.data[0], EvalResult::Number { value } if *value == 1.0));
            assert!(matches!(&value.data[1], EvalResult::Number { value } if *value == 2.0));
        }
        other => panic!("expected Sparkline, got {other:?}"),
    }
}

/// The anchor's own resolved value is the *whole* array: an outer `array` of
/// single-cell `array` rows (`{10;20}` is 2 rows x 1 column) — the recursive
/// row-array shape `EvalResult`'s doc comment describes for a 2-D result.
#[test]
fn resolved_typed_array_anchor_is_array_of_array_rows() {
    match resolved_value(&fixture(), "B1") {
        EvalResult::Array { value } => {
            assert_eq!(value.len(), 2);
            for (row, expected) in value.iter().zip([10.0, 20.0]) {
                match row {
                    EvalResult::Array { value: cells } => {
                        assert!(
                            matches!(cells.as_slice(), [EvalResult::Number { value }] if *value == expected)
                        );
                    }
                    other => panic!("expected an array row, got {other:?}"),
                }
            }
        }
        other => panic!("expected Array, got {other:?}"),
    }
}

/// The spilled (non-anchor) cell carries the reconstructed scalar element and
/// the anchor's A1 address — the one thing `getTyped` never carries, since
/// `get` only ever sees what is physically authored.
#[test]
fn resolved_typed_spilled_cell_carries_anchor() {
    let wb = fixture();
    let spilled = wb.resolved_typed("Sheet1", "B2").unwrap().unwrap();
    assert!(matches!(spilled.value, EvalResult::Number { value } if value == 20.0));
    assert_eq!(spilled.anchor.as_deref(), Some("B1"));
}

#[test]
fn resolved_typed_missing_cell_is_none() {
    let wb = fixture();
    assert!(wb.resolved_typed("Sheet1", "Z99").unwrap().is_none());
}

#[test]
fn get_typed_missing_cell_is_none() {
    let wb = fixture();
    assert!(wb.get_typed("Sheet1", "Z99").unwrap().is_none());
}

/// `getTyped` returns the authored formula alongside the typed value — the
/// one thing `resolvedTyped` never carries (it is a spill-resolved *anchor*
/// cell here, not a plain literal, to also confirm `getTyped` returns the
/// anchor's own array value, not the spilled element `resolvedTyped` would
/// reconstruct at a non-anchor address).
#[test]
fn get_typed_carries_formula_and_value() {
    let wb = fixture();
    let got = wb.get_typed("Sheet1", "B1").unwrap().unwrap();
    assert_eq!(got.formula.as_deref(), Some("={10;20}"));
    assert!(matches!(got.value, EvalResult::Array { .. }));
}

/// A literal cell (no formula) reports `formula: None` — mirrors `get()`'s
/// `"formula": null`.
#[test]
fn get_typed_literal_cell_has_no_formula() {
    let wb = fixture();
    let got = wb.get_typed("Sheet1", "A1").unwrap().unwrap();
    assert_eq!(got.formula, None);
    assert!(matches!(got.value, EvalResult::Number { value } if value == 1.5));
}
