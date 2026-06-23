//! Phase 5: MIN/MAX/SORT participation for zone-aware (`Value::Zoned`) values.
//! Exercised end-to-end through the engine.

use std::collections::HashMap;
use truecalc_core::{Engine, ErrorKind, Value};

fn eval(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
}

#[test]
fn min_and_max_over_zoned_return_zoned() {
    let earlier = "TZDATETIME(2026,7,14,9,0,0,\"Europe/Berlin\")";
    let later = "TZDATETIME(2026,7,14,11,0,0,\"Europe/Berlin\")";
    assert_eq!(
        eval(&format!("=TZSTRING(MIN({later},{earlier}))")),
        Value::Text("2026-07-14T09:00:00+02:00[Europe/Berlin]".to_string())
    );
    assert_eq!(
        eval(&format!("=TZSTRING(MAX({earlier},{later}))")),
        Value::Text("2026-07-14T11:00:00+02:00[Europe/Berlin]".to_string())
    );
}

#[test]
fn min_compares_on_the_instant_across_zones() {
    // 09:00 Berlin (07:00Z) is earlier than 09:00 New York (13:00Z).
    let berlin = "TZDATETIME(2026,7,14,9,0,0,\"Europe/Berlin\")";
    let ny = "TZDATETIME(2026,7,14,9,0,0,\"America/New_York\")";
    assert_eq!(
        eval(&format!("=TZOFFSET(MIN({ny},{berlin}))")),
        Value::Number(120.0) // the Berlin one wins (earlier instant)
    );
}

#[test]
fn mixing_naive_and_zoned_is_value_error() {
    assert_eq!(
        eval("=MIN(TZDATETIME(2026,1,1,0,0,0,\"UTC\"), 5)"),
        Value::Error(ErrorKind::Value)
    );
    assert_eq!(
        eval("=MAX(7, TZDATETIME(2026,1,1,0,0,0,\"UTC\"))"),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn sort_orders_zoned_by_instant() {
    // Inline column of two instants (later first); SORT ascending puts the
    // earlier one first -- proving instant ordering, not insertion order.
    let f = "=TZSTRING(INDEX(SORT({TZDATETIME(2026,7,14,11,0,0,\"UTC\");\
             TZDATETIME(2026,7,14,9,0,0,\"UTC\")}),1,1))";
    assert_eq!(eval(f), Value::Text("2026-07-14T09:00:00+00:00[UTC]".to_string()));
}
