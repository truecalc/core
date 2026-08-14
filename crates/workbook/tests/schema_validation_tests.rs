//! Schema-validation tests (issue #531): the committed machine-readable
//! `schema/workbook.v1.schema.json` validates the golden documents and the
//! canonical output of arbitrary workbooks. JSON Schema cannot express the
//! canonical-ordering / number-formatting / spill / duplicate-key / case-fold /
//! ref-canonicality / limit rules — those are the prose spec's job and are
//! covered elsewhere — but it pins the structural shape.

use proptest::prelude::*;
use truecalc_workbook::{Cell, EngineFlavor, NamedRange, Value, Workbook, Worksheet};

fn schema() -> serde_json::Value {
    let text = std::fs::read_to_string("schema/workbook.v1.schema.json").unwrap();
    serde_json::from_str(&text).unwrap()
}

fn validator() -> jsonschema::Validator {
    jsonschema::validator_for(&schema()).expect("schema compiles")
}

fn golden(name: &str) -> serde_json::Value {
    let text = std::fs::read_to_string(format!("tests/golden/{name}")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn schema_file_is_a_valid_json_schema() {
    // validator_for already compiles + meta-validates the schema.
    let _ = validator();
}

#[test]
fn worked_example_golden_validates() {
    let v = validator();
    let inst = golden("worked_example.json");
    assert!(v.is_valid(&inst), "worked_example.json must validate");
}

#[test]
fn empty_golden_validates() {
    assert!(validator().is_valid(&golden("empty_sheets.json")));
}

#[test]
fn number_boundaries_golden_validates() {
    assert!(validator().is_valid(&golden("number_boundaries.json")));
}

#[test]
fn unicode_golden_validates() {
    assert!(validator().is_valid(&golden("unicode.json")));
}

#[test]
fn a_hand_built_workbook_validates() {
    let mut wb = Workbook::new(EngineFlavor::Excel);
    wb.names_mut().push(NamedRange {
        name: "Rate".into(),
        r#ref: "S!A1".into(),
    });
    let mut s = Worksheet::new("S");
    s.cells_mut()
        .insert("A1".into(), Cell::literal(Value::Number(1.5)).unwrap());
    s.cells_mut().insert(
        "B2".into(),
        Cell::with_formula("=A1*Rate", Value::Date(46180.5)),
    );
    wb.sheets_mut().push(s);
    let json: serde_json::Value = serde_json::from_str(&wb.to_json().unwrap()).unwrap();
    assert!(validator().is_valid(&json), "got: {json}");
}

#[test]
fn a_document_missing_a_required_field_fails_schema() {
    let bad: serde_json::Value =
        serde_json::from_str(r#"{"engine":"sheets","names":[],"sheets":[]}"#).unwrap();
    assert!(
        !validator().is_valid(&bad),
        "missing version must fail schema"
    );
}

#[test]
fn a_document_with_a_bad_address_key_fails_schema() {
    // truecalc/core#861 final review, Finding 7: this fixture used to say
    // `"version":"1"` with no `tables` key while the schema's `version`
    // was pinned to `"2"` with `tables` unconditionally required — so this
    // document was rejected on that version/tables mismatch *before* the
    // validator ever reached the bad "a1" address key, passing for the
    // wrong reason. Now that the schema accepts both versions (Finding 6),
    // pin this fixture to the canonical v2 shape the library actually
    // writes, so the only remaining reason to fail is the bad address key.
    let bad: serde_json::Value = serde_json::from_str(
        r#"{"engine":"sheets","names":[],"sheets":[{"name":"S","cells":{"a1":{"value":{"type":"number","value":1}}}}],"tables":[],"version":"2"}"#,
    )
    .unwrap();
    assert!(
        !validator().is_valid(&bad),
        "lowercase address key must fail schema"
    );
}

fn finite_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        any::<f64>().prop_filter("finite", |x| x.is_finite()),
        (-1000i64..1000).prop_map(|n| n as f64),
    ]
}

fn small_workbook() -> impl Strategy<Value = Workbook> {
    (0usize..3, finite_f64()).prop_map(|(n, x)| {
        let mut wb = Workbook::new(EngineFlavor::Sheets);
        let cols = ["A", "D", "G"];
        let mut s = Worksheet::new("Sheet0");
        for (i, c) in cols.iter().enumerate().take(n) {
            s.cells_mut().insert(
                format!("{c}1"),
                Cell::literal(Value::Number(x + i as f64)).unwrap(),
            );
        }
        wb.sheets_mut().push(s);
        wb
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Canonical output of arbitrary workbooks always validates against the
    /// committed schema.
    #[test]
    fn canonical_output_validates_against_schema(wb in small_workbook()) {
        let json: serde_json::Value = serde_json::from_str(&wb.to_json().unwrap()).unwrap();
        let v = validator();
        prop_assert!(v.is_valid(&json));
    }
}
