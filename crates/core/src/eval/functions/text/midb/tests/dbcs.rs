//! `starting_at` is a character index; `num_bytes` is a DBCS byte budget.
//!
//! Every case here mirrors a recorded Google Sheets conformance row — none of
//! the expected values are inferred from this implementation.

use super::super::*;
use crate::types::Value;

fn midb(text: &str, start: f64, num_bytes: f64) -> Value {
    midb_fn(&[
        Value::Text(text.to_string()),
        Value::Number(start),
        Value::Number(num_bytes),
    ])
}

/// Character 2 of `aあb` is `あ`, and 2 bytes is exactly its DBCS width. The
/// decisive case: a byte-indexed start would land inside `あ`, and a
/// character-counted *length* would return `あb`.
#[test]
fn start_counts_characters_while_length_counts_bytes() {
    assert_eq!(midb("aあb", 2.0, 2.0), Value::Text("あ".to_string()));
}

/// `熊本` is 4 DBCS bytes but only 2 characters, so character 3 does not exist.
#[test]
fn start_past_last_character_of_dbcs_text_is_empty() {
    assert_eq!(midb("熊本", 3.0, 2.0), Value::Text(String::new()));
    assert_eq!(midb("熊本", 3.0, 4.0), Value::Text(String::new()));
}

/// `FINDB`/`SEARCHB` return byte offsets, so feeding one to `MIDB` can name a
/// character index past the end: byte 5 of `农历新年` is `新`, but the string is
/// only 4 characters long.
#[test]
fn byte_offset_fed_start_past_character_count_is_empty() {
    assert_eq!(midb("农历新年", 5.0, 2.0), Value::Text(String::new()));
}

/// The byte budget still truncates mid-string: 2 bytes from character 1 of
/// `熊本` is one double-byte character, not two.
#[test]
fn length_stays_in_dbcs_bytes() {
    assert_eq!(midb("熊本", 1.0, 2.0), Value::Text("熊".to_string()));
}

/// For single-byte text the character and byte models coincide.
#[test]
fn ascii_is_unaffected() {
    assert_eq!(midb("Hello", 2.0, 3.0), Value::Text("ell".to_string()));
    assert_eq!(
        midb("hello world", 7.0, 100.0),
        Value::Text("world".to_string())
    );
}
