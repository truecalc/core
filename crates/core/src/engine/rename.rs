//! `rename_sheet_refs`: rewrite the sheet qualifier of every cell/range
//! reference in a formula that points at `old` to point at `new` instead —
//! the sheet-rename reference-rewrite transform. Unqualified refs, refs to
//! other sheets, string literals, function names, and defined names are left
//! untouched. Mirrors `translate.rs`'s fill/paste transform: parse, collect
//! the matching reference spans, splice replacement text back in.

use crate::parser::Ref;
use crate::types::ParseError;

use super::translate::collect_shiftable_refs;

/// Case-insensitive sheet-name comparison, in the same spirit as the
/// workbook crate's own sheet-identity rule (sheet names are unique
/// case-insensitively, and a pure case-change rename is allowed). Uses
/// `str::to_uppercase()` rather than the `icu_casemap`-based Unicode *simple*
/// case folding `workbook::casefold` uses — `core` doesn't depend on
/// `icu_casemap`, and the two diverge only for characters with multi-char
/// uppercase expansions (e.g. German `ß` → `SS`), which is not expected to
/// matter for realistic sheet names.
fn same_sheet(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
}

/// The sheet qualifier of `r`, if it has one.
fn ref_sheet(r: &Ref) -> Option<&str> {
    match r {
        Ref::Cell { sheet: Some(s), .. } | Ref::Range { sheet: Some(s), .. } => Some(s.as_str()),
        _ => None,
    }
}

/// Render `r` with its sheet qualifier replaced by `new`, requoting as
/// needed (handled by `Ref`'s own `Display` impl).
fn renamed_ref_text(r: &Ref, new: &str) -> String {
    match r {
        Ref::Cell { addr, .. } => Ref::Cell { sheet: Some(new.to_string()), addr: *addr }.to_string(),
        Ref::Range { start, end, .. } => {
            Ref::Range { sheet: Some(new.to_string()), start: *start, end: *end }.to_string()
        }
        Ref::Name(_) => unreachable!("collect_shiftable_refs never returns Ref::Name"),
    }
}

/// Parse `formula`, rewrite every reference whose sheet qualifier matches
/// `old` (case-insensitive) to `new`, and splice the result back into the
/// original text. No-op if `formula` has no `old`-qualified refs.
pub(crate) fn rename_sheet_refs_text(
    formula: &str,
    old: &str,
    new: &str,
) -> Result<String, ParseError> {
    let expr = crate::parser::parse_formula(formula)?;
    let mut spans: Vec<_> = collect_shiftable_refs(&expr)
        .into_iter()
        .filter(|(_, r)| ref_sheet(r).is_some_and(|s| same_sheet(s, old)))
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s.0.offset)); // right to left
    let mut out = formula.to_string();
    for (span, r) in spans {
        let replacement = renamed_ref_text(&r, new);
        let start = span.offset;
        let end = span.offset + span.length;
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
