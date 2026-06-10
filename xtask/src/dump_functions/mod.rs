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
            if bucket.len() < MAX_EXAMPLES && !bucket.iter().any(|e| e.formula == formula) {
                bucket.push(Example {
                    formula,
                    result: expected_value,
                });
            }
        }
    }

    for entry in entries.iter_mut() {
        if let Some(examples) = by_fn.remove(&entry.name) {
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
