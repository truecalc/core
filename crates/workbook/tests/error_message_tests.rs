//! Diagnostic error messages surfaced through recalc (issue #728).
//!
//! A wrong-arity formula in a cell computes to an error whose CODE is unchanged
//! but which now carries an additive diagnostic message. The message is
//! in-memory metadata only: it must not affect value equality, hashing, or the
//! canonical JSON serialization (the immutable `to_json ∘ from_json = id`
//! contract).

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
fn wrong_arity_cell_carries_message() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=DATE()".into()))
        .unwrap();
    wb.recalc(&ctx());

    let value = wb.get("Sheet1", a1("A1")).unwrap().value().clone();
    assert_eq!(
        value.error_message(),
        Some("Wrong number of arguments to DATE. Expected 3 arguments, but got 0.")
    );
    // The code is unchanged, and equality/identity are by code only.
    assert_eq!(value, Value::Error("#N/A".into()));
}

#[test]
fn message_is_dropped_from_canonical_json() {
    let mut wb = sheets_wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=DATE()".into()))
        .unwrap();
    wb.recalc(&ctx());

    let json = wb.to_json().expect("workbook serializes");
    // The error code persists; the diagnostic message does NOT (schema §8).
    assert!(json.contains("#N/A"));
    assert!(
        !json.contains("Wrong number of arguments"),
        "canonical JSON must not carry the in-memory diagnostic message"
    );
}
