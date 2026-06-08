//! Workbook/Worksheet serde mapping against the schema spec, including the
//! worked example of §11.

use truecalc_workbook::{
    Cell, EngineFlavor, NamedRange, Value, Workbook, Worksheet, SCHEMA_VERSION,
};

/// The worked example from schema spec §11 (canonical key order, pretty
/// whitespace — non-canonical whitespace must be accepted).
const WORKED_EXAMPLE: &str = r##"{
  "engine": "sheets",
  "names": [
    { "name": "TaxRate", "ref": "Sheet2!B5" }
  ],
  "sheets": [
    {
      "cells": {
        "A1": { "value": { "type": "number", "value": 100 } },
        "A2": {
          "formula": "=1/0",
          "value": { "error": "#DIV/0!", "type": "error" }
        },
        "C1": {
          "formula": "={1,2;3,4}",
          "value": { "type": "array", "value": [
            [ { "type": "number", "value": 1 }, { "type": "number", "value": 2 } ],
            [ { "type": "number", "value": 3 }, { "type": "number", "value": 4 } ]
          ] }
        }
      },
      "name": "Sheet1"
    },
    {
      "cells": {
        "A1": {
          "formula": "=Sheet1!A1*TaxRate",
          "value": { "type": "number", "value": 8 }
        },
        "B5": { "value": { "type": "number", "value": 0.08 } }
      },
      "name": "Sheet2"
    }
  ],
  "version": "1"
}"##;

fn worked_example_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.names_mut().push(NamedRange {
        name: "TaxRate".to_owned(),
        r#ref: "Sheet2!B5".to_owned(),
    });

    let mut sheet1 = Worksheet::new("Sheet1");
    sheet1.cells_mut().insert(
        "A1".to_owned(),
        Cell::literal(Value::Number(100.0)).unwrap(),
    );
    sheet1.cells_mut().insert(
        "A2".to_owned(),
        Cell::with_formula("=1/0", Value::Error("#DIV/0!".to_owned())),
    );
    sheet1.cells_mut().insert(
        "C1".to_owned(),
        Cell::with_formula(
            "={1,2;3,4}",
            Value::Array(vec![
                vec![Value::Number(1.0), Value::Number(2.0)],
                vec![Value::Number(3.0), Value::Number(4.0)],
            ]),
        ),
    );

    let mut sheet2 = Worksheet::new("Sheet2");
    sheet2.cells_mut().insert(
        "A1".to_owned(),
        Cell::with_formula("=Sheet1!A1*TaxRate", Value::Number(8.0)),
    );
    sheet2
        .cells_mut()
        .insert("B5".to_owned(), Cell::literal(Value::Number(0.08)).unwrap());

    wb.sheets_mut().push(sheet1);
    wb.sheets_mut().push(sheet2);
    wb
}

#[test]
fn worked_example_deserializes_to_expected_workbook() {
    let parsed: Workbook = serde_json::from_str(WORKED_EXAMPLE).unwrap();
    assert_eq!(parsed, worked_example_workbook());
}

#[test]
fn worked_example_round_trips() {
    let wb = worked_example_workbook();
    let json = serde_json::to_string(&wb).unwrap();
    let back: Workbook = serde_json::from_str(&json).unwrap();
    assert_eq!(back, wb);
}

#[test]
fn new_workbook_serializes_all_four_fields() {
    // §2: all four top-level fields are always present, even when empty.
    assert_eq!(
        serde_json::to_string(&Workbook::new(EngineFlavor::Sheets)).unwrap(),
        r#"{"engine":"sheets","names":[],"sheets":[],"version":"1"}"#
    );
}

#[test]
fn new_workbook_writes_current_schema_version() {
    assert_eq!(SCHEMA_VERSION, "1");
    assert_eq!(Workbook::new(EngineFlavor::Excel).version(), "1");
}

#[test]
fn version_field_is_required() {
    let missing = serde_json::from_str::<Workbook>(r#"{"engine":"sheets","names":[],"sheets":[]}"#);
    assert!(missing.is_err());
}

#[test]
fn unknown_top_level_fields_are_rejected() {
    let extra = serde_json::from_str::<Workbook>(
        r#"{"engine":"sheets","names":[],"sheets":[],"version":"1","theme":"dark"}"#,
    );
    assert!(extra.is_err());
}

#[test]
fn unknown_worksheet_fields_are_rejected() {
    let extra = serde_json::from_str::<Worksheet>(r#"{"cells":{},"name":"Sheet1","hidden":true}"#);
    assert!(extra.is_err());
}

#[test]
fn unknown_named_range_fields_are_rejected() {
    let extra = serde_json::from_str::<NamedRange>(
        r#"{"name":"TaxRate","ref":"Sheet2!B5","scope":"sheet"}"#,
    );
    assert!(extra.is_err());
}

#[test]
fn named_range_serializes_ref_key() {
    let nr = NamedRange {
        name: "TaxRate".to_owned(),
        r#ref: "Sheet2!B5".to_owned(),
    };
    assert_eq!(
        serde_json::to_string(&nr).unwrap(),
        r#"{"name":"TaxRate","ref":"Sheet2!B5"}"#
    );
}

#[test]
fn cell_keys_serialize_in_jcs_byte_order() {
    // JCS sorts by UTF-16 code units: "A10" sorts before "A2".
    let mut sheet = Worksheet::new("S");
    sheet
        .cells_mut()
        .insert("A2".to_owned(), Cell::literal(Value::Number(2.0)).unwrap());
    sheet.cells_mut().insert(
        "A10".to_owned(),
        Cell::literal(Value::Number(10.0)).unwrap(),
    );
    let json = serde_json::to_string(&sheet).unwrap();
    let a10 = json.find("\"A10\"").unwrap();
    let a2 = json.find("\"A2\"").unwrap();
    assert!(a10 < a2, "A10 must serialize before A2: {json}");
}

#[test]
fn sheet_tab_order_is_preserved() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.sheets_mut().push(Worksheet::new("Zebra"));
    wb.sheets_mut().push(Worksheet::new("Alpha"));
    let json = serde_json::to_string(&wb).unwrap();
    let back: Workbook = serde_json::from_str(&json).unwrap();
    let names: Vec<&str> = back.sheets().iter().map(Worksheet::name).collect();
    assert_eq!(names, ["Zebra", "Alpha"]);
}
