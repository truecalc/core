//! Invariant: workbooks are engine-locked at creation (engine-flavor ADR).

use truecalc_workbook::{EngineFlavor, Workbook};

#[test]
fn workbook_is_locked_to_engine_at_creation() {
    assert_eq!(
        Workbook::new(EngineFlavor::Sheets).engine(),
        EngineFlavor::Sheets
    );
    assert_eq!(
        Workbook::new(EngineFlavor::Excel).engine(),
        EngineFlavor::Excel
    );
}

#[test]
fn engine_survives_round_trip() {
    let wb = Workbook::new(EngineFlavor::Excel);
    let json = serde_json::to_string(&wb).unwrap();
    let back: Workbook = serde_json::from_str(&json).unwrap();
    assert_eq!(back.engine(), EngineFlavor::Excel);
    assert_eq!(back, wb);
}

#[test]
fn engine_is_required_no_default() {
    let missing = serde_json::from_str::<Workbook>(r#"{"names":[],"sheets":[],"version":"1"}"#);
    assert!(missing.is_err());
}

#[test]
fn unknown_engine_value_is_rejected() {
    let bad = serde_json::from_str::<Workbook>(
        r#"{"engine":"lotus123","names":[],"sheets":[],"version":"1"}"#,
    );
    assert!(bad.is_err());
}

#[test]
fn engine_serializes_as_lowercase_literal() {
    let sheets = serde_json::to_string(&Workbook::new(EngineFlavor::Sheets)).unwrap();
    assert!(sheets.contains(r#""engine":"sheets""#));
    let excel = serde_json::to_string(&Workbook::new(EngineFlavor::Excel)).unwrap();
    assert!(excel.contains(r#""engine":"excel""#));
}
