//! Parser acceptance of the `#REF!` error literal (issue #716).
//!
//! `translate_formula` can shift a reference out of the Sheets grid and
//! render it back as literal `#REF!` text (e.g. `=A1` shifted up one row at
//! the top of the sheet becomes `=#REF!`); until the parser accepted that
//! literal, a host could never `set()` the resulting text back into a
//! workbook. These tests cover the full round trip: the literal parses, its
//! cell resolves to the `#REF!` error value after recalc, it propagates
//! through a binary operation, and a `translate_formula`-produced `#REF!`
//! formula can be set without a parse error.

use truecalc_core::Engine;
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn sheets_wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

#[test]
fn set_ref_error_literal_resolves_to_ref_error_after_recalc() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=#REF!".into()))
        .unwrap();
    wb.recalc(&ctx());

    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error("#REF!".into())
    );
}

#[test]
fn set_ref_error_literal_propagates_through_binary_op() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=#REF!+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error("#REF!".into())
    );
}

#[test]
fn translate_formula_out_of_bounds_ref_literal_round_trips_through_set() {
    // Shifting `=A1` up one row past the top of the sheet renders the
    // reference as literal `#REF!` text (Google Sheets/Excel fill-adjustment
    // behavior).
    let translated = Engine::sheets()
        .translate_formula("=A1", -1, 0)
        .expect("translate_formula succeeds");
    assert_eq!(translated, "=#REF!");

    // Before issue #716, `set` rejected this text with a parse error.
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula(translated))
        .expect("translated #REF! text must be settable without a parse error");
    wb.recalc(&ctx());

    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error("#REF!".into())
    );
}
