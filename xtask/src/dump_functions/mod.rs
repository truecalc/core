//! `dump-functions` xtask: emit `functions.json` from the engine registry,
//! enriched with verified examples extracted from the Google Sheets fixtures.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;
use truecalc_core::Registry;

/// One verified example: a formula and its Google-Sheets-validated result string.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Example {
    pub formula: String,
    pub result: String,
}

/// A function reference entry destined for `functions.json`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FunctionEntry {
    pub name: String,
    pub category: String,
    pub syntax: String,
    pub description: String,
    pub examples: Vec<Example>,
}

/// Maximum headline examples attached per function.
const MAX_EXAMPLES: usize = 3;

/// Build the base entries from the registry (single source of truth), sorted by name.
pub fn build_registry() -> Vec<FunctionEntry> {
    let reg = Registry::new();
    let mut entries: Vec<FunctionEntry> = reg
        .list_functions()
        .map(|(name, meta)| FunctionEntry {
            name: name.to_string(),
            category: meta.category.to_string(),
            syntax: meta.signature.to_string(),
            description: meta.description.to_string(),
            examples: Vec::new(),
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Extract the outermost function name from a formula, e.g.
/// `=ACCRINT(DATE(2010,1,1),...)` → `Some("ACCRINT")`. Returns `None` for
/// formulas that do not begin with a function call (e.g. `=1+1`, `=A1`).
pub fn outermost_function_name(formula: &str) -> Option<String> {
    let s = formula.trim();
    let s = s.strip_prefix('=').unwrap_or(s);
    let s = s.trim_start();
    let mut ident = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            ident.push(ch);
        } else if ch == '(' {
            // identifier is a function call only if there is a name and it
            // starts with a letter
            if ident
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
            {
                return Some(ident.to_uppercase());
            }
            return None;
        } else {
            return None;
        }
    }
    None
}

/// A formula is "self-contained" if it has no cell or sheet references — it can
/// be evaluated by the stateless engine and run in the docs Try-It widget.
/// Heuristic: rejects sheet refs (`!`), the INDIRECT family, and A1-style cell
/// references / ranges (one or more letters immediately followed by a digit,
/// e.g. `A1`, `BC12`). Inline array literals like `{1,2;3,4}` stay self-contained.
pub fn is_self_contained(formula: &str) -> bool {
    if formula.contains('!') || formula.to_uppercase().contains("INDIRECT") {
        return false;
    }
    // reject an A1-style cell ref: a run of ASCII letters directly followed by a digit
    let bytes = formula.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i].is_ascii_digit() && i > 0 && bytes[i - 1].is_ascii_alphabetic() {
            // walk back over the letter-run preceding this digit
            let mut j = i - 1;
            while j > 0 && bytes[j - 1].is_ascii_alphabetic() {
                j -= 1;
            }
            // the char before the letter-run must not be alphanumeric, otherwise
            // this is part of a larger token (e.g. an identifier or named range)
            let prev_ok = j == 0 || !bytes[j - 1].is_ascii_alphanumeric();
            // a cell ref's column is 1..=3 letters; longer runs are not cell refs
            let letter_run = i - j;
            // scan forward over the digit-run; if it is immediately followed by `(`
            // or another letter, then letters+digits form an identifier (a function
            // name like LOG10, ATAN2, or BIN2DEC), not a cell ref — skip it.
            let mut k = i;
            while k < bytes.len() && bytes[k].is_ascii_digit() {
                k += 1;
            }
            let is_identifier =
                k < bytes.len() && (bytes[k] == b'(' || bytes[k].is_ascii_alphabetic());
            if prev_ok && !is_identifier && (1..=3).contains(&letter_run) {
                return false;
            }
        }
    }
    true
}

/// Whether a fixture row is a good headline example: validated, scalar-typed,
/// non-empty, and from a clean category (not an error/edge case).
fn is_headline_row(test_category: &str, expected_type: &str, expected_value: &str) -> bool {
    matches!(test_category, "official" | "basic")
        && matches!(expected_type, "number" | "text" | "bool" | "date")
        && !expected_value.trim().is_empty()
}

/// Read every `*.tsv` (except `bugs.tsv`) in `fixtures_dir` and attach up to
/// `MAX_EXAMPLES` verified examples to each matching entry.
pub fn attach_examples(entries: &mut [FunctionEntry], fixtures_dir: &Path) -> Result<()> {
    use std::collections::HashMap;

    // function name (uppercase) -> collected examples
    let mut by_fn: HashMap<String, Vec<Example>> = HashMap::new();

    for dirent in std::fs::read_dir(fixtures_dir)
        .with_context(|| format!("read fixtures dir: {}", fixtures_dir.display()))?
    {
        let path = dirent?.path();
        let is_tsv = path.extension().is_some_and(|e| e == "tsv");
        let is_bugs = path.file_name().is_some_and(|n| n == "bugs.tsv");
        if !is_tsv || is_bugs {
            continue;
        }
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(&path)
            .with_context(|| format!("open TSV: {}", path.display()))?;
        for record in rdr.records() {
            let record = record?;
            let formula = record.get(1).unwrap_or("").trim().to_string();
            let expected_value = record.get(2).unwrap_or("").to_string();
            let test_category = record.get(3).unwrap_or("");
            let expected_type = record.get(4).unwrap_or("");
            if !is_headline_row(test_category, expected_type, &expected_value) {
                continue;
            }
            let Some(fname) = outermost_function_name(&formula) else {
                continue;
            };
            let bucket = by_fn.entry(fname).or_default();
            // Collect ALL matching headline rows (deduped by formula); we rank and
            // truncate below so self-contained examples can win the headline slot.
            if !bucket.iter().any(|e| e.formula == formula) {
                bucket.push(Example {
                    formula,
                    result: expected_value,
                });
            }
        }
    }

    for entry in entries.iter_mut() {
        if let Some(mut examples) = by_fn.remove(&entry.name) {
            // Prefer self-contained (widget-runnable) examples for the headline,
            // preserving original file order within each group (stable sort).
            examples.sort_by_key(|e| !is_self_contained(&e.formula));
            examples.truncate(MAX_EXAMPLES);
            entry.examples = examples;
        }
    }
    Ok(())
}

/// Build the full registry, attach examples, and write pretty JSON to `out`.
pub fn run(out: &Path, fixtures_dir: &Path) -> Result<()> {
    let mut entries = build_registry();
    attach_examples(&mut entries, fixtures_dir)?;
    let json = serde_json::to_string_pretty(&entries)?;
    std::fs::write(out, format!("{json}\n")).with_context(|| format!("write {}", out.display()))?;
    eprintln!(
        "dump-functions: wrote {} functions to {}",
        entries.len(),
        out.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests;
