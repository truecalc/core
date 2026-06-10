use super::*;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    // xtask/ -> workspace root -> crates/core/tests/fixtures/google_sheets
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("crates/core/tests/fixtures/google_sheets")
}

#[test]
fn registry_has_expected_function_count() {
    let entries = build_registry();
    assert!(
        entries.len() >= 480,
        "expected >=480 functions, got {}",
        entries.len()
    );
}

#[test]
fn entries_are_sorted_and_complete() {
    let entries = build_registry();
    for w in entries.windows(2) {
        assert!(
            w[0].name <= w[1].name,
            "not sorted: {} > {}",
            w[0].name,
            w[1].name
        );
    }
    for e in &entries {
        assert!(!e.name.is_empty());
        assert!(!e.category.is_empty(), "{} has empty category", e.name);
        assert!(!e.syntax.is_empty(), "{} has empty syntax", e.name);
        assert!(
            !e.description.is_empty(),
            "{} has empty description",
            e.name
        );
    }
}

#[test]
fn known_function_present_with_category() {
    let entries = build_registry();
    let pmt = entries
        .iter()
        .find(|e| e.name == "PMT")
        .expect("PMT present");
    assert_eq!(pmt.category, "financial");
}

#[test]
fn outermost_function_name_cases() {
    assert_eq!(outermost_function_name("=SUM(1,2)").as_deref(), Some("SUM"));
    assert_eq!(
        outermost_function_name(
            "=ACCRINT(DATE(2010,1,1),DATE(2010,2,1),DATE(2012,12,31),0.05,100,4)"
        )
        .as_deref(),
        Some("ACCRINT")
    );
    assert_eq!(outermost_function_name("=abs(-3)").as_deref(), Some("ABS"));
    assert_eq!(outermost_function_name("=1+1"), None);
    assert_eq!(outermost_function_name("=A1+B1"), None);
    assert_eq!(outermost_function_name("=-SUM(1)"), None);
}

#[test]
fn examples_attached_from_fixtures() {
    let mut entries = build_registry();
    attach_examples(&mut entries, &fixtures_dir()).expect("attach examples");
    let sum = entries
        .iter()
        .find(|e| e.name == "SUM")
        .expect("SUM present");
    assert!(
        !sum.examples.is_empty(),
        "SUM should have >=1 fixture example"
    );
    assert!(sum.examples.len() <= MAX_EXAMPLES);
    for ex in &sum.examples {
        assert!(ex.formula.starts_with('='));
        assert!(!ex.result.is_empty());
    }
}
