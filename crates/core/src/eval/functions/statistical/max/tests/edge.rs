use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_no_args_returns_na() {
    assert_eq!(max_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn max_text_in_args_returns_value_error() {
    assert_eq!(
        max_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_empty_array_is_ref_error() {
    assert_eq!(
        max_fn(&[Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_non_empty_array_without_numbers_returns_zero() {
    // `=MAX({"a","b"})` and `=MAX({TRUE,FALSE})` are both 0: neither text nor
    // booleans contribute a number in array context, but both are enough to
    // lift the array out of the #REF! rule. These two variants are the whole
    // of the carve-out — nothing else sets `had_content`.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Bool(true), Value::Bool(false)])]),
        Value::Number(0.0)
    );
}

#[test]
fn max_array_of_only_blanks_is_zero() {
    // `=MAX(A1:A3)` over empty cells is 0 in Google Sheets — the same answer
    // MIN, MAXA and MINA give it. MAX used to be alone at #REF! here.
    //
    // That answer is captured across seven range shapes, each with a populated
    // control, but **none of those rows are in this repo yet**: they land in a
    // separate fixtures-only PR (see `stat_helpers::is_blank_only_array` for
    // the shapes and the branch). Read from this repo alone, this test pins
    // the behaviour, not the Sheets answer.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Empty, Value::Empty])]),
        Value::Number(0.0)
    );
    // Same through the nested-row shape a vertical range materializes as.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Array(vec![Value::Empty]),
            Value::Array(vec![Value::Empty]),
        ])]),
        Value::Number(0.0)
    );
}

#[test]
fn max_blank_beside_something_else_does_not_become_the_blank_only_zero() {
    // The blank-only rule is decided over the arguments as a whole, not by a
    // per-element flag, so a blank cannot on its own pull an array that holds
    // something else into the blank-only 0.
    //
    // This was pinned with a date element and `#REF!`, on the note that "if
    // that ever moves it has to move deliberately, not as fallout from this
    // rule". It moved deliberately: dates now participate, so this array
    // answers the date. The invariant the pin exists for is unchanged and
    // still discriminating — the answer is the date, *not* the blank-only 0.
    //
    // There is no longer any variant that reaches MAX's numberless `#REF!`
    // through a populated array: text and booleans set `array_had_content`,
    // dates contribute a number, a sparkline sets its own flag, and a zoned
    // instant is intercepted by `zoned_extreme` before the loop runs. The
    // empty-array assertion below is what still carries the `#REF!` side.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Date(43831.0)])]),
        Value::Date(43831.0)
    );
    // An empty array argument stays #REF! even alongside an all-blank one.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty]), Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_array_of_only_dates_returns_the_latest_date() {
    // Was #REF! until dates were captured: a date-only array now answers the
    // largest serial, date-typed. Replaces the pin that recorded the old
    // #REF! as unprobed.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Date(43831.0),
            Value::Date(44197.0)
        ])]),
        Value::Date(44197.0)
    );
    // Same for a date arriving through a nested-row range materialization.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Array(vec![Value::Date(43831.0)])])]),
        Value::Date(43831.0)
    );
}

#[test]
fn max_dates_compare_as_bare_serials_and_type_the_answer() {
    // A plain number and a date are compared on the serial with no special
    // casing, and the answer is date-typed because a date took part — even
    // when the plain number is the one that won (that is the MIN direction;
    // pinned in min's tests, mirrored here for the losing-date direction).
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Date(43831.0), Value::Number(5.0)])]),
        Value::Date(43831.0)
    );
    // Direct (non-array) arguments take the same rule.
    assert_eq!(
        max_fn(&[Value::Date(43831.0), Value::Date(44197.0)]),
        Value::Date(44197.0)
    );
    assert_eq!(
        max_fn(&[Value::Date(43831.0), Value::Number(5.0)]),
        Value::Date(43831.0)
    );
    // No date in scope: the answer stays a plain number.
    assert_eq!(
        max_fn(&[Value::Number(5.0), Value::Number(1.0)]),
        Value::Number(5.0)
    );
}

#[test]
fn max_date_beside_a_blank_no_longer_falls_into_the_ref_rule() {
    // A date leaves a number behind, so the "populated but numberless" #REF!
    // rule can no longer be reached with a date in scope.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Date(43831.0), Value::Empty])]),
        Value::Date(43831.0)
    );
    // An all-blank array is a separate, captured case and answers 0, not the
    // date rule and not #REF! — see `max_array_of_only_blanks_is_zero`. Kept
    // here as the contrast: a date in scope types the answer, an array with
    // nothing in it but blanks does not.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Empty])]),
        Value::Number(0.0)
    );
}

#[test]
fn max_one_non_blank_lifts_the_array_out_of_the_ref_rule() {
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Empty,
            Value::Text("z".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Number(4.0)])]),
        Value::Number(4.0)
    );
}

#[test]
fn max_negative_numbers() {
    assert_eq!(
        max_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-1.0)
    );
}
