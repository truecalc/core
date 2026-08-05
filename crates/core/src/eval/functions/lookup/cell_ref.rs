// Utilities for parsing cell references like "A1", "B5", "AA1", "A1:D4".

/// Parse a column label (e.g. "A" → 1, "Z" → 26, "AA" → 27) — 1-based.
pub fn col_label_to_index(label: &str) -> Option<usize> {
    if label.is_empty() {
        return None;
    }
    let mut result: usize = 0;
    for c in label.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        result = result * 26 + (c.to_ascii_uppercase() as usize - b'A' as usize + 1);
    }
    Some(result)
}

/// Convert a 1-based column index to a column label (1 → "A", 27 → "AA").
pub fn col_index_to_label(mut col: usize) -> String {
    let mut result = String::new();
    while col > 0 {
        let rem = (col - 1) % 26;
        result.push((b'A' + rem as u8) as char);
        col = (col - 1) / 26;
    }
    result.chars().rev().collect()
}

/// Parse a single cell reference like "A1" or "BC42".
/// Returns `(col_1based, row_1based)` or `None` if not a valid cell ref.
pub fn parse_cell_ref(name: &str) -> Option<(usize, usize)> {
    let bytes = name.as_bytes();
    let col_end = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    if col_end == 0 || col_end == bytes.len() {
        return None;
    }
    let row_str = &name[col_end..];
    if !row_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let row: usize = row_str.parse().ok()?;
    if row == 0 {
        return None;
    }
    let col = col_label_to_index(&name[..col_end])?;
    Some((col, row))
}

/// Parse a range reference like "A1:D4".
/// Returns `(start_col, start_row, end_col, end_row)` (all 1-based), or `None`.
pub fn parse_range_ref(name: &str) -> Option<(usize, usize, usize, usize)> {
    let colon = name.find(':')?;
    let (sc, sr) = parse_cell_ref(&name[..colon])?;
    let (ec, er) = parse_cell_ref(&name[colon + 1..])?;
    Some((sc, sr, ec, er))
}
