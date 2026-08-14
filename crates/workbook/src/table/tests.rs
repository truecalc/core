use super::*;

#[test]
fn table_serializes_like_named_range_shape() {
    let t = Table { name: "Recipe".to_string(), r#ref: "Sheet1!A1:D12".to_string() };
    let json = serde_json::to_value(&t).unwrap();
    assert_eq!(json, serde_json::json!({ "name": "Recipe", "ref": "Sheet1!A1:D12" }));
}

#[test]
fn table_rejects_unknown_fields() {
    let bad = serde_json::json!({ "name": "Recipe", "ref": "Sheet1!A1:D12", "headerRow": 1 });
    assert!(serde_json::from_value::<Table>(bad).is_err());
}
