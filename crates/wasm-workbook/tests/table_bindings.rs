//! Coverage for issue #868: `JsWorkbook.defineTable`/`.redefineTable` wrap
//! `truecalc_workbook::Workbook::define_table`/`redefine_table` 1:1, the same
//! pattern `defineName`/`redefineName` already use.
//!
//! These exercise `truecalc_workbook::Workbook` directly — the same table
//! declaration + recalc path `JsWorkbook`'s wrappers call through to — so
//! they run natively under `cargo test` without a wasm runtime, matching
//! this crate's existing convention (see round_trip.rs).

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn sheets_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1".to_string())).unwrap();
    wb
}

fn set_formula(wb: &mut Workbook, a1: &str, formula: &str) -> Result<(), String> {
    let addr = Address::from_a1(a1).unwrap();
    wb.set("Sheet1", addr, CellInput::Formula(formula.to_string()))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn set_literal(wb: &mut Workbook, a1: &str, text: &str) {
    let addr = Address::from_a1(a1).unwrap();
    wb.set(
        "Sheet1",
        addr,
        CellInput::Literal(Value::Text(text.to_string())),
    )
    .unwrap();
}

fn recalc(wb: &mut Workbook) {
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    wb.recalc(&ctx);
}

fn resolved(wb: &Workbook, a1: &str) -> Option<Value> {
    let addr = Address::from_a1(a1).unwrap();
    wb.resolved("Sheet1", addr).map(|r| r.value)
}

/// A declared table's whole-column reference resolves through the same
/// `Workbook::define_table` + `set`/`recalc` path `JsWorkbook`'s wrapper
/// exercises — end-to-end proof the binding this issue adds actually
/// unblocks structured references, not just that the wrapper compiles.
#[test]
fn defined_table_resolves_whole_column_reference() {
    let mut wb = sheets_workbook();
    set_literal(&mut wb, "A1", "qty");
    set_formula(&mut wb, "A2", "=10").unwrap();
    set_formula(&mut wb, "A3", "=20").unwrap();
    wb.define_table("T", "Sheet1!A1:A3").unwrap();
    set_formula(&mut wb, "B1", "=SUM(T[qty])").unwrap();

    recalc(&mut wb);
    assert_eq!(resolved(&wb, "B1"), Some(Value::Number(30.0)));
}

/// `redefine_table` retargets an existing table's range — the wrapper's
/// second method — and a dependent formula picks up the new target on
/// recalc.
#[test]
fn redefined_table_resolves_against_new_range() {
    let mut wb = sheets_workbook();
    set_literal(&mut wb, "A1", "qty");
    set_formula(&mut wb, "A2", "=10").unwrap();
    set_literal(&mut wb, "B1", "qty");
    set_formula(&mut wb, "B2", "=99").unwrap();
    wb.define_table("T", "Sheet1!A1:A2").unwrap();
    wb.redefine_table("T", "Sheet1!B1:B2").unwrap();
    set_formula(&mut wb, "C1", "=SUM(T[qty])").unwrap();

    recalc(&mut wb);
    assert_eq!(resolved(&wb, "C1"), Some(Value::Number(99.0)));
}
