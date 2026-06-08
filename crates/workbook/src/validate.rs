//! Document-level validation for [`Workbook::from_json`](crate::Workbook::from_json)
//! — the normative rules that the plain serde layer (`#[serde(deny_unknown_fields)]`,
//! value encodings, version match, empty-literal) cannot express.
//!
//! Covers, per the schema spec and review follow-up #568:
//! - §1 duplicate keys and UTF-8/BOM (handled by [`crate::strict_json`] before
//!   this runs);
//! - §2 sheet-name uniqueness under Unicode **simple** case folding, and the
//!   count limit;
//! - §3 sheet-name non-empty / ≤ 100 scalar values; cell-key A1 syntax + bounds;
//! - §5 spill-rectangle document validity (authored cell inside an anchor's
//!   reconstructed rectangle; overlapping rectangles; out-of-bounds rectangle);
//! - §7 named-range name/`ref` validity, case-insensitive uniqueness, no
//!   dangling sheet refs;
//! - §6/Decision 5 resource limits (cells, text length, array elements, sheets,
//!   formula length, named-range count).
//!
//! Input is the duplicate-checked [`serde_json::Value`] tree; this runs before
//! the typed `serde_json::from_value` so a single clear error is surfaced for
//! these rules. The two passes are intentionally independent: serde owns shape,
//! this owns cross-field/document invariants.

use std::collections::HashMap;

use icu_casemap::CaseMapperBorrowed;
use serde_json::Value;

use crate::address::{parse_a1, Address};
use crate::limits;
use crate::named_ref;

/// Validates the document-level rules. Returns a description of the first
/// violation, or `Ok(())`.
pub fn validate_document(root: &Value) -> Result<(), String> {
    let obj = root
        .as_object()
        .ok_or_else(|| "a workbook document must be a JSON object".to_string())?;

    let sheets = obj
        .get("sheets")
        .and_then(Value::as_array)
        .ok_or_else(|| "the workbook \"sheets\" field must be an array".to_string())?;

    if sheets.len() > limits::MAX_SHEETS {
        return Err(format!(
            "workbook has {} sheets, exceeding the limit of {} (scope ADR Decision 5)",
            sheets.len(),
            limits::MAX_SHEETS
        ));
    }

    let folder = CaseMapperBorrowed::new();
    let mut seen_sheet_names: HashMap<String, &str> = HashMap::new();
    let mut sheet_name_set: Vec<String> = Vec::new();
    let mut total_cells: usize = 0;

    for sheet in sheets {
        let sheet_obj = sheet
            .as_object()
            .ok_or_else(|| "each worksheet must be a JSON object".to_string())?;

        let name = sheet_obj
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "each worksheet requires a string \"name\"".to_string())?;

        // §3: non-empty, ≤ 100 Unicode scalar values.
        let name_len = name.chars().count();
        if name_len == 0 {
            return Err("a worksheet name must be non-empty (schema spec §3)".to_string());
        }
        if name_len > limits::MAX_SHEET_NAME_LEN {
            return Err(format!(
                "worksheet name {name:?} has {name_len} scalar values, exceeding the limit of \
                 {} (schema spec §3)",
                limits::MAX_SHEET_NAME_LEN
            ));
        }

        // §2: uniqueness under Unicode simple case folding (not to_lowercase()).
        let folded = simple_fold(&folder, name);
        if let Some(prev) = seen_sheet_names.insert(folded, name) {
            return Err(format!(
                "duplicate sheet name: {name:?} collides with {prev:?} under simple \
                 case folding (schema spec §2)"
            ));
        }
        sheet_name_set.push(name.to_owned());

        total_cells += validate_sheet_cells(sheet_obj, name)?;
    }

    if total_cells > limits::MAX_CELLS_PER_WORKBOOK {
        return Err(format!(
            "workbook has {total_cells} populated cells, exceeding the limit of {} \
             (scope ADR Decision 5)",
            limits::MAX_CELLS_PER_WORKBOOK
        ));
    }

    validate_named_ranges(obj, &sheet_name_set, &folder)?;

    Ok(())
}

/// Validates one sheet's `cells`: A1 key syntax + bounds (§3), per-value limits
/// (§6), formula length, and spill-rectangle document validity (§5). Returns
/// the populated-cell count for this sheet.
fn validate_sheet_cells(
    sheet_obj: &serde_json::Map<String, Value>,
    sheet_name: &str,
) -> Result<usize, String> {
    let cells = sheet_obj
        .get("cells")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("worksheet {sheet_name:?} \"cells\" must be an object"))?;

    // Authored addresses and any spill anchors (address -> array dims).
    let mut authored: Vec<Address> = Vec::with_capacity(cells.len());
    let mut anchors: Vec<(Address, usize, usize)> = Vec::new();

    for (key, cell) in cells {
        let addr = parse_a1(key).ok_or_else(|| {
            format!(
                "invalid cell address {key:?} in sheet {sheet_name:?}: keys must match \
                 ^[A-Z]{{1,3}}[1-9][0-9]{{0,7}}$ and lie within the address bounds (schema spec §3)"
            )
        })?;
        authored.push(addr);

        let cell_obj = cell
            .as_object()
            .ok_or_else(|| format!("cell {key:?} in sheet {sheet_name:?} must be an object"))?;

        // Formula length limit (Decision 5).
        if let Some(formula) = cell_obj.get("formula").and_then(Value::as_str) {
            if formula.len() > limits::MAX_FORMULA_LEN {
                return Err(format!(
                    "formula in cell {key:?} of sheet {sheet_name:?} is {} bytes, exceeding the \
                     limit of {} (scope ADR Decision 5)",
                    formula.len(),
                    limits::MAX_FORMULA_LEN
                ));
            }
        }

        if let Some(value) = cell_obj.get("value") {
            validate_value_limits(value, key, sheet_name)?;
            if let Some((rows, cols)) = array_dims(value) {
                anchors.push((addr, rows, cols));
            }
        }
    }

    // §5 document validity: build the set of authored addresses, then check
    // every anchor's reconstructed rectangle.
    validate_spill_rectangles(&authored, &anchors, sheet_name)?;

    // Populated-cell count includes spilled/materialized cells (Decision 5):
    // a non-anchor authored cell counts as 1; an anchor counts as its full
    // m×n rectangle (the spilled cells it materializes).
    let mut count = 0usize;
    let anchor_addrs: Vec<Address> = anchors.iter().map(|(a, _, _)| *a).collect();
    for addr in &authored {
        if let Some((_, rows, cols)) = anchors.iter().find(|(a, _, _)| a == addr) {
            count += rows * cols;
        } else if !anchor_addrs.contains(addr) {
            count += 1;
        }
    }

    Ok(count)
}

/// Per-value resource limits (Decision 5): text length and array element count.
fn validate_value_limits(value: &Value, key: &str, sheet: &str) -> Result<(), String> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return Ok(()), // shape errors are serde's job
    };
    match obj.get("type").and_then(Value::as_str) {
        Some("text") => {
            if let Some(s) = obj.get("value").and_then(Value::as_str) {
                let len = s.chars().count();
                if len > limits::MAX_TEXT_LEN {
                    return Err(format!(
                        "text value in cell {key:?} of sheet {sheet:?} has {len} scalar values, \
                         exceeding the limit of {} (scope ADR Decision 5)",
                        limits::MAX_TEXT_LEN
                    ));
                }
            }
        }
        Some("array") => {
            if let Some((rows, cols)) = array_dims(value) {
                let elems = rows * cols;
                if elems > limits::MAX_ARRAY_ELEMENTS {
                    return Err(format!(
                        "array value in cell {key:?} of sheet {sheet:?} has {elems} elements, \
                         exceeding the limit of {} (scope ADR Decision 5)",
                        limits::MAX_ARRAY_ELEMENTS
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Returns `(rows, cols)` if `value` is an `array` value, else `None`.
fn array_dims(value: &Value) -> Option<(usize, usize)> {
    let obj = value.as_object()?;
    if obj.get("type").and_then(Value::as_str) != Some("array") {
        return None;
    }
    let rows = obj.get("value")?.as_array()?;
    let r = rows.len();
    let c = rows.first().and_then(Value::as_array).map_or(0, Vec::len);
    Some((r, c))
}

/// §5 document-validity rules: an authored cell may not lie inside another
/// anchor's reconstructed rectangle; two anchors' rectangles may not overlap;
/// no rectangle may exceed the sheet's address bounds.
fn validate_spill_rectangles(
    authored: &[Address],
    anchors: &[(Address, usize, usize)],
    sheet: &str,
) -> Result<(), String> {
    for (anchor, rows, cols) in anchors {
        // Out-of-bounds rectangle.
        let last_row = anchor.row as u64 + *rows as u64 - 1;
        let last_col = anchor.column as u64 + *cols as u64 - 1;
        if last_row > limits::MAX_ROW as u64 || last_col > limits::MAX_COLUMN as u64 {
            return Err(format!(
                "spill anchor at row {} col {} in sheet {sheet:?} reconstructs a {rows}×{cols} \
                 rectangle that exceeds the sheet's address bounds (schema spec §5)",
                anchor.row, anchor.column
            ));
        }

        // Authored cell inside this rectangle (other than the anchor itself).
        for a in authored {
            if a == anchor {
                continue;
            }
            if in_rect(a, anchor, *rows, *cols) {
                return Err(format!(
                    "authored cell at row {} col {} in sheet {sheet:?} lies inside the spill \
                     rectangle of the anchor at row {} col {} (schema spec §5)",
                    a.row, a.column, anchor.row, anchor.column
                ));
            }
        }
    }

    // Overlapping rectangles between distinct anchors.
    for i in 0..anchors.len() {
        for j in (i + 1)..anchors.len() {
            let (a, ar, ac) = &anchors[i];
            let (b, br, bc) = &anchors[j];
            if rects_overlap(a, *ar, *ac, b, *br, *bc) {
                return Err(format!(
                    "spill rectangles of the anchors at row {} col {} and row {} col {} in \
                     sheet {sheet:?} overlap (schema spec §5)",
                    a.row, a.column, b.row, b.column
                ));
            }
        }
    }

    Ok(())
}

fn in_rect(p: &Address, anchor: &Address, rows: usize, cols: usize) -> bool {
    p.row >= anchor.row
        && p.row < anchor.row + rows as u32
        && p.column >= anchor.column
        && p.column < anchor.column + cols as u32
}

fn rects_overlap(a: &Address, ar: usize, ac: usize, b: &Address, br: usize, bc: usize) -> bool {
    let a_r0 = a.row;
    let a_r1 = a.row + ar as u32 - 1;
    let a_c0 = a.column;
    let a_c1 = a.column + ac as u32 - 1;
    let b_r0 = b.row;
    let b_r1 = b.row + br as u32 - 1;
    let b_c0 = b.column;
    let b_c1 = b.column + bc as u32 - 1;
    a_r0 <= b_r1 && b_r0 <= a_r1 && a_c0 <= b_c1 && b_c0 <= a_c1
}

/// §7 named-range validity: name regex / exclusions, `ref` canonical form,
/// case-insensitive name uniqueness, no dangling sheet refs; and the
/// named-range count limit (Decision 5).
fn validate_named_ranges(
    obj: &serde_json::Map<String, Value>,
    sheet_names: &[String],
    folder: &CaseMapperBorrowed<'static>,
) -> Result<(), String> {
    let names = obj
        .get("names")
        .and_then(Value::as_array)
        .ok_or_else(|| "the workbook \"names\" field must be an array".to_string())?;

    if names.len() > limits::MAX_NAMED_RANGES {
        return Err(format!(
            "workbook has {} named ranges, exceeding the limit of {} (scope ADR Decision 5)",
            names.len(),
            limits::MAX_NAMED_RANGES
        ));
    }

    // Folded sheet names for dangling-ref resolution (refs target sheets
    // case-insensitively; resolution semantics are fixture-verified, but a ref
    // to a sheet that does not exist under simple folding is dangling).
    let folded_sheets: Vec<String> = sheet_names.iter().map(|s| simple_fold(folder, s)).collect();

    let mut seen: HashMap<String, &str> = HashMap::new();
    for nr in names {
        let nr_obj = nr
            .as_object()
            .ok_or_else(|| "each named range must be a JSON object".to_string())?;
        let name = nr_obj
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "each named range requires a string \"name\"".to_string())?;
        let r = nr_obj
            .get("ref")
            .and_then(Value::as_str)
            .ok_or_else(|| "each named range requires a string \"ref\"".to_string())?;

        if !named_ref::is_valid_name(name) {
            return Err(format!(
                "named-range name {name:?} is invalid: it must match ^[A-Za-z_][A-Za-z0-9_]*$ and \
                 must not be an A1 address, an R1C1-style reference, or a boolean literal \
                 (schema spec §7)"
            ));
        }

        let parsed = named_ref::parse_canonical_ref(r)?;

        // Dangling sheet ref: the targeted sheet must exist (case-insensitive).
        let folded_target = simple_fold(folder, &parsed.sheet);
        if !folded_sheets.contains(&folded_target) {
            return Err(format!(
                "named range {name:?} refers to sheet {:?}, which does not exist (schema spec §7)",
                parsed.sheet
            ));
        }

        // Case-insensitive uniqueness across the workbook.
        let folded = simple_fold(folder, name);
        if let Some(prev) = seen.insert(folded, name) {
            return Err(format!(
                "duplicate named range: {name:?} collides with {prev:?} under simple case \
                 folding (schema spec §7)"
            ));
        }
    }

    Ok(())
}

/// Unicode **simple** case folding applied per character (schema spec §2): not
/// `to_lowercase()`/full folding, which disagree on some case pairs and would
/// break cross-surface determinism.
fn simple_fold(folder: &CaseMapperBorrowed<'static>, s: &str) -> String {
    s.chars().map(|c| folder.simple_fold(c)).collect()
}
