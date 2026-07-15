//! End-to-end tests for diagnostic error messages (issue #728).
//!
//! Eval errors carry an *optional*, additive message (Google Sheets parity)
//! without changing the error **code**. The high-ROI slice is the shared arity
//! check: any function routing through `check_arity`/`check_arity_len` now emits
//! "Wrong number of arguments to <FN>. Expected N arguments, but got M." The
//! function name is filled in at the dispatch site.

use std::collections::HashMap;

use truecalc_core::{Engine, ErrorKind, Value};

fn eval(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
}

#[test]
fn date_wrong_arity_carries_google_sheets_message() {
    let v = eval("=DATE()");
    // The error CODE is byte-identical to before (conformance is unaffected).
    assert_eq!(v.error_kind(), Some(&ErrorKind::NA));
    // The rendered code string is exactly `#N/A`.
    assert_eq!(v.error_kind().unwrap().to_string(), "#N/A");
    // ...and now it also carries the diagnostic.
    assert_eq!(
        v.error_message(),
        Some("Wrong number of arguments to DATE. Expected 3 arguments, but got 0.")
    );
    assert!(matches!(v, Value::ErrorMsg(ErrorKind::NA, _)));
}

#[test]
fn function_name_is_upper_cased_for_parity() {
    // The name is taken from the call site and upper-cased (Sheets parity).
    let v = eval("=date()");
    assert_eq!(
        v.error_message(),
        Some("Wrong number of arguments to DATE. Expected 3 arguments, but got 0.")
    );
}

#[test]
fn bounded_range_arity_uses_between_phrasing() {
    // SUM accepts 1..=255 arguments, so an out-of-range call reports the range.
    let v = eval("=SUM()");
    assert_eq!(v.error_kind(), Some(&ErrorKind::NA));
    assert_eq!(
        v.error_message(),
        Some("Wrong number of arguments to SUM. Expected between 1 and 255 arguments, but got 0.")
    );
}

#[test]
fn messageless_error_has_no_message() {
    // A #DIV/0! is produced without the arity helper, so it stays a bare error.
    let v = eval("=1/0");
    assert_eq!(v.error_kind(), Some(&ErrorKind::DivByZero));
    assert_eq!(v.error_message(), None);
    assert!(matches!(v, Value::Error(ErrorKind::DivByZero)));
    assert!(v.is_error());
}

#[test]
fn arity_error_still_equals_bare_error_of_same_code() {
    // The message is additive metadata: it must never change value equality,
    // so conformance comparisons (which compare by code) are unaffected.
    assert_eq!(eval("=DATE()"), Value::Error(ErrorKind::NA));
}

#[test]
fn arity_error_propagates_through_arithmetic() {
    // A propagated arity error keeps its code (and its message).
    let v = eval("=DATE()+1");
    assert_eq!(v.error_kind(), Some(&ErrorKind::NA));
    assert_eq!(
        v.error_message(),
        Some("Wrong number of arguments to DATE. Expected 3 arguments, but got 0.")
    );
}

#[test]
fn iferror_catches_arity_error() {
    // IFERROR must treat a message-carrying arity error as an error.
    assert_eq!(eval("=IFERROR(DATE(), \"caught\")"), Value::Text("caught".into()));
}

#[test]
fn iserror_and_isna_see_arity_error() {
    assert_eq!(eval("=ISERROR(DATE())"), Value::Bool(true));
    assert_eq!(eval("=ISNA(DATE())"), Value::Bool(true));
}
