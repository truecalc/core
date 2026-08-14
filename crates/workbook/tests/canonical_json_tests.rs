//! Canonical (RFC 8785 / JCS) serialization (schema spec §8) and the
//! golden-byte suite required by issue #530: golden files are byte-identical
//! across Linux + macOS CI, and the number-boundary cases of the schema spec's
//! normative implementation warning are covered.

use truecalc_workbook::{Cell, EngineFlavor, NamedRange, Table, Value, Workbook, Worksheet};

/// Builds the worked example of schema spec §11.
fn worked_example() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.names_mut().push(NamedRange {
        name: "TaxRate".to_owned(),
        r#ref: "Sheet2!B5".to_owned(),
    });
    let mut s1 = Worksheet::new("Sheet1");
    s1.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(100.0)).unwrap());
    s1.cells_mut().insert(
        "A2".into(),
        Cell::with_formula("=1/0", Value::Error("#DIV/0!".into())),
    );
    s1.cells_mut().insert(
        "C1".into(),
        Cell::with_formula(
            "={1,2;3,4}",
            Value::Array(vec![
                vec![Value::Number(1.0), Value::Number(2.0)],
                vec![Value::Number(3.0), Value::Number(4.0)],
            ]),
        ),
    );
    let mut s2 = Worksheet::new("Sheet2");
    s2.cells_mut().insert(
        "A1".into(),
        Cell::with_formula("=Sheet1!A1*TaxRate", Value::Number(8.0)),
    );
    s2.cells_mut()
        .insert("B5".into(), Cell::literal(Value::Number(0.08)).unwrap());
    wb.sheets_mut().push(s1);
    wb.sheets_mut().push(s2);
    wb
}

fn golden(name: &str) -> String {
    std::fs::read_to_string(format!("tests/golden/{name}")).unwrap()
}

#[test]
fn worked_example_is_byte_identical_to_golden() {
    assert_eq!(
        worked_example().to_json().unwrap(),
        golden("worked_example.json")
    );
}

#[test]
fn empty_workbook_is_byte_identical_to_golden() {
    let wb = Workbook::new(EngineFlavor::Sheets);
    assert_eq!(wb.to_json().unwrap(), golden("empty_sheets.json"));
}

#[test]
fn number_boundaries_are_byte_identical_to_golden() {
    let mut wb = Workbook::new(EngineFlavor::Excel);
    let mut sh = Worksheet::new("N");
    let cases = [
        ("A1", 1e21f64),
        ("A2", 1e-7),
        ("A3", 1e-6),
        ("A4", 0.1 + 0.2),
        ("A5", -0.0),
        ("A6", 8.0),
        ("A7", 9.999999999999999e20),
        ("A8", 1.0000000000000002e21),
        ("A9", 5e-324),
        ("A10", 1.7976931348623157e308),
    ];
    for (k, v) in cases {
        sh.cells_mut()
            .insert(k.into(), Cell::literal(Value::Number(v)).unwrap());
    }
    wb.sheets_mut().push(sh);
    assert_eq!(wb.to_json().unwrap(), golden("number_boundaries.json"));
}

#[test]
fn unicode_is_byte_identical_to_golden() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.names_mut().push(NamedRange {
        name: "Data".into(),
        r#ref: "'Q2 Données'!A1:A3".into(),
    });
    let mut us = Worksheet::new("Q2 Données");
    us.cells_mut().insert(
        "A1".into(),
        Cell::literal(Value::Text("café".into())).unwrap(),
    );
    wb.sheets_mut().push(us);
    assert_eq!(wb.to_json().unwrap(), golden("unicode.json"));
}

#[test]
fn canonical_form_has_no_trailing_newline_or_whitespace() {
    let json = worked_example().to_json().unwrap();
    assert!(!json.ends_with('\n'));
    assert!(!json.contains('\n'));
    assert!(!json.contains(": "));
}

#[test]
fn the_1e21_boundary_uses_exponent_form_not_serde_default() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(1e21)).unwrap());
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    assert!(json.contains(r#""value":1e+21"#), "got: {json}");
}

#[test]
fn just_below_1e21_stays_decimal() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut().insert(
        "A1".into(),
        Cell::literal(Value::Number(9.999999999999999e20)).unwrap(),
    );
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    assert!(
        json.contains(r#""value":999999999999999900000"#),
        "got: {json}"
    );
}

#[test]
fn the_1e_minus_7_boundary_uses_exponent_but_1e_minus_6_stays_decimal() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(1e-7)).unwrap());
    sh.cells_mut()
        .insert("A2".into(), Cell::literal(Value::Number(1e-6)).unwrap());
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    assert!(
        json.contains(r#""A1":{"value":{"type":"number","value":1e-7}}"#),
        "got: {json}"
    );
    assert!(
        json.contains(r#""A2":{"value":{"type":"number","value":0.000001}}"#),
        "got: {json}"
    );
}

#[test]
fn negative_zero_serializes_as_zero() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(-0.0)).unwrap());
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    assert!(json.contains(r#""value":0}"#), "got: {json}");
    assert!(!json.contains("-0"), "got: {json}");
}

#[test]
fn integral_numbers_print_without_a_fraction() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(8.0)).unwrap());
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    assert!(json.contains(r#""value":8}"#), "got: {json}");
}

#[test]
fn cells_keys_sort_in_jcs_utf16_order() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("S");
    sh.cells_mut()
        .insert("A2".into(), Cell::literal(Value::Number(2.0)).unwrap());
    sh.cells_mut()
        .insert("A10".into(), Cell::literal(Value::Number(10.0)).unwrap());
    wb.sheets_mut().push(sh);
    let json = wb.to_json().unwrap();
    let a10 = json.find("\"A10\"").unwrap();
    let a2 = json.find("\"A2\"").unwrap();
    assert!(a10 < a2, "A10 must precede A2: {json}");
}

#[test]
fn names_are_sorted_by_name_in_canonical_output() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("Sheet1");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(1.0)).unwrap());
    wb.sheets_mut().push(sh);
    wb.names_mut().push(NamedRange {
        name: "Zeta".into(),
        r#ref: "Sheet1!A1".into(),
    });
    wb.names_mut().push(NamedRange {
        name: "Alpha".into(),
        r#ref: "Sheet1!A1".into(),
    });
    let json = wb.to_json().unwrap();
    assert!(
        json.find("Alpha").unwrap() < json.find("Zeta").unwrap(),
        "got: {json}"
    );
}

#[test]
fn sheets_keep_authored_tab_order_not_sorted() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.sheets_mut().push(Worksheet::new("Zebra"));
    wb.sheets_mut().push(Worksheet::new("Alpha"));
    let json = wb.to_json().unwrap();
    assert!(
        json.find("Zebra").unwrap() < json.find("Alpha").unwrap(),
        "got: {json}"
    );
}

#[test]
fn canonical_form_sorts_tables_by_name() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let mut sh = Worksheet::new("Sheet1");
    sh.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(1.0)).unwrap());
    wb.sheets_mut().push(sh);
    wb.tables_mut().push(Table {
        name: "Zebra".to_owned(),
        r#ref: "Sheet1!A1:B2".to_owned(),
    });
    wb.tables_mut().push(Table {
        name: "Apple".to_owned(),
        r#ref: "Sheet1!D1:E2".to_owned(),
    });
    let json = wb.to_json().unwrap();
    let apple_pos = json.find("\"name\":\"Apple\"").expect("Apple not found");
    let zebra_pos = json.find("\"name\":\"Zebra\"").expect("Zebra not found");
    assert!(
        apple_pos < zebra_pos,
        "Apple must precede Zebra in canonical output: {json}"
    );
}

#[test]
fn non_canonical_input_is_accepted_and_output_is_canonical() {
    let pretty = br#"{
        "version": "1",
        "engine": "sheets",
        "sheets": [ { "name": "S", "cells": { "A2": {"value":{"type":"number","value":2}}, "A10": {"value":{"type":"number","value":10}} } } ],
        "names": []
    }"#;
    let wb = Workbook::from_json(pretty).unwrap();
    let canonical = wb.to_json().unwrap();
    assert_eq!(
        canonical,
        r#"{"engine":"sheets","names":[],"sheets":[{"cells":{"A10":{"value":{"type":"number","value":10}},"A2":{"value":{"type":"number","value":2}}},"name":"S"}],"tables":[],"version":"2"}"#
    );
}

#[test]
fn extreme_exponent_floats_round_trip_byte_identically() {
    // Regression: serde_json's default float parser is off by up to one ULP for
    // some extreme exponents; without the `float_roundtrip` feature this value
    // re-serializes to a different last digit, breaking to_json ∘ from_json = id.
    let mut wb = Workbook::new(EngineFlavor::Excel);
    let mut sh = Worksheet::new("S");
    for (i, v) in [
        1.5926045055164742e-152f64,
        5e-324,
        2.2250738585072014e-308,
        1.7976931348623157e308,
    ]
    .into_iter()
    .enumerate()
    {
        sh.cells_mut().insert(
            format!("A{}", i + 1),
            Cell::literal(Value::Number(v)).unwrap(),
        );
    }
    wb.sheets_mut().push(sh);
    let once = wb.to_json().unwrap();
    let back = Workbook::from_json(once.as_bytes()).unwrap();
    let twice = back.to_json().unwrap();
    assert_eq!(
        once, twice,
        "extreme-exponent floats must round-trip byte-identically"
    );
}
