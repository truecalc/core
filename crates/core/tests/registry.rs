use truecalc_core::eval::functions::{Registry, FunctionMeta};
use truecalc_core::Value;

#[test]
#[should_panic(expected = "duplicate function registration: 'ABS'")]
fn duplicate_eager_registration_panics() {
    let mut r = Registry { functions: Default::default(), metadata: Default::default() };
    let meta = FunctionMeta {
        category: "test",
        signature: "ABS(x)",
        description: "test",
    };
    r.register_eager("ABS", |args: &[Value]| args[0].clone(), meta.clone());
    r.register_eager("ABS", |args: &[Value]| args[0].clone(), meta); // should panic
}

#[test]
#[should_panic(expected = "duplicate function registration: 'ABS'")]
fn duplicate_lazy_registration_panics() {
    use truecalc_core::eval::functions::EvalCtx;
    use truecalc_core::parser::Expr;
    let mut r = Registry { functions: Default::default(), metadata: Default::default() };
    let meta = FunctionMeta {
        category: "test",
        signature: "ABS(x)",
        description: "test",
    };
    r.register_lazy("ABS", |_args: &[Expr], _ctx: &mut EvalCtx<'_>| Value::Number(0.0), meta.clone());
    r.register_lazy("ABS", |_args: &[Expr], _ctx: &mut EvalCtx<'_>| Value::Number(0.0), meta);
}

#[test]
fn registry_new_has_no_duplicates() {
    // This will panic at startup if any duplicate slips through.
    // The fact that this test runs at all proves no duplicates exist.
    let _r = Registry::new();
}

#[test]
fn alias_resolves_to_same_result() {
    use std::collections::HashMap;
    use truecalc_core::evaluate;
    // TTEST and T.TEST are aliases — same result
    let r1 = evaluate("TTEST({1,2,3},{4,5,6},2,1)", &HashMap::new());
    let r2 = evaluate("T.TEST({1,2,3},{4,5,6},2,1)", &HashMap::new());
    assert_eq!(r1, r2, "TTEST and T.TEST should return identical results");
}

#[test]
fn alias_does_not_appear_in_metadata() {
    let r = Registry::new();
    // Aliases should not be listed as independent functions in metadata
    let meta = r.get_metadata();
    let ttest_entries: Vec<_> = meta.iter().filter(|m| m.name == "TTEST").collect();
    assert!(ttest_entries.is_empty(), "TTEST is an alias and should not appear in metadata");
    let t_test_entries: Vec<_> = meta.iter().filter(|m| m.name == "T.TEST").collect();
    assert_eq!(t_test_entries.len(), 1, "T.TEST (canonical) should appear exactly once");
}

#[test]
fn context_limited_functions_return_name_error() {
    use std::collections::HashMap;
    use truecalc_core::{evaluate, Value, ErrorKind};
    // These functions require a cell grid — they should not be registered
    // in the standalone evaluator. Callers should get #NAME? not #N/A.
    let cases = [
        "INDIRECT(\"A1\")",
        "OFFSET({1,2,3},0,1)",
        "FORMULATEXT(SUM(1,2))",
        "GETPIVOTDATA(\"Sales\",{1})",
        "ISFORMULA(SUM(1,2))",
    ];
    for formula in cases {
        let result = evaluate(formula, &HashMap::new());
        assert_eq!(
            result,
            Value::Error(ErrorKind::Name),
            "expected #NAME? for context-limited function in: {}",
            formula
        );
    }
}

#[test]
fn rows_and_columns_work_with_array_input() {
    use std::collections::HashMap;
    use truecalc_core::{evaluate, Value};
    assert_eq!(evaluate("ROWS({1,2,3;4,5,6})", &HashMap::new()), Value::Number(2.0));
    assert_eq!(evaluate("ROWS({1,2,3})", &HashMap::new()),        Value::Number(1.0));
    assert_eq!(evaluate("COLUMNS({1,2,3;4,5,6})", &HashMap::new()), Value::Number(3.0));
    assert_eq!(evaluate("COLUMNS({1})", &HashMap::new()),             Value::Number(1.0));
}
