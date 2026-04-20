//! One-time migration: convert xlsx fixtures in m1-m4/ to 5-column TSV in google_sheets/.
//!
//! Run once with: cargo nextest run -p truecalc-core --test migrate_fixtures -- --run-ignored all
//! Then delete this file.

use calamine::{open_workbook, CellErrorType, Data, Reader, Xlsx};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[test]
#[ignore = "run once to migrate fixtures"]
fn migrate_xlsx_to_tsv() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest.join("tests/fixtures");
    let out_dir = fixtures_dir.join("google_sheets");
    fs::create_dir_all(&out_dir).expect("failed to create google_sheets dir");

    // Map from milestone/file → category name in TSV
    let file_map: &[(&str, &str, &str)] = &[
        ("m1", "Math.xlsx",        "math"),
        ("m1", "Logical.xlsx",     "logical"),
        ("m1", "Info.xlsx",        "info"),
        ("m1", "Statistical.xlsx", "statistical"),
        ("m1", "Operator.xlsx",    "operator"),
        ("m1", "Text.xlsx",        "text"),
        ("m2", "Date.xlsx",        "date"),
        ("m2", "Engineering.xlsx", "engineering"),
        ("m2", "Info.xlsx",        "info"),
        ("m2", "Logical.xlsx",     "logical"),
        ("m2", "Lookup.xlsx",      "lookup"),
        ("m2", "Math.xlsx",        "math"),
        ("m2", "Parser.xlsx",      "parser"),
        ("m2", "Statistical.xlsx", "statistical"),
        ("m2", "Text.xlsx",        "text"),
        ("m3", "Database.xlsx",    "database"),
        ("m3", "Engineering.xlsx", "engineering"),
        ("m3", "Financial.xlsx",   "financial"),
        ("m3", "Info.xlsx",        "info"),
        ("m3", "Lookup.xlsx",      "lookup"),
        ("m3", "Math.xlsx",        "math"),
        ("m3", "Statistical.xlsx", "statistical"),
        ("m4", "Array.xlsx",       "array"),
        ("m4", "Filter.xlsx",      "filter"),
        ("m4", "Info.xlsx",        "info"),
        ("m4", "Logical.xlsx",     "logical"),
        ("m4", "Lookup.xlsx",      "lookup"),
        ("m4", "Math.xlsx",        "math"),
        ("m4", "Operator.xlsx",    "operator"),
        ("m4", "Web.xlsx",         "web"),
    ];

    // Accumulate rows per category
    let mut category_rows: HashMap<String, Vec<[String; 5]>> = HashMap::new();

    for (milestone, filename, category) in file_map {
        let path = fixtures_dir.join(milestone).join(filename);
        if !path.exists() {
            println!("SKIP missing: {}", path.display());
            continue;
        }

        let mut workbook: Xlsx<_> = open_workbook(&path)
            .unwrap_or_else(|e| panic!("failed to open {:?}: {}", path, e));
        let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

        for sheet_name in &sheet_names {
            let range = workbook
                .worksheet_range(sheet_name)
                .unwrap_or_else(|e| panic!("failed to read sheet {sheet_name}: {e}"));

            for row in range.rows().skip(1) {
                if row.len() < 3 {
                    continue;
                }

                // Col A: description
                let desc = match &row[0] {
                    Data::String(s) => s.clone(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    _ => continue,
                };

                // Col B: formula text (must be a string)
                let formula = match &row[1] {
                    Data::String(s) => s.clone(),
                    _ => continue,
                };

                // Skip empty formulas
                if formula.trim().is_empty() {
                    continue;
                }

                // Col C: expected oracle value
                let (expected_value, expected_type) = match cell_to_tsv_value(&row[2]) {
                    Some(v) => v,
                    None => continue, // skip empty/datetime cells
                };

                // Col D: test category (use from xlsx if present, otherwise use file-level category)
                let test_category = if row.len() >= 4 {
                    match &row[3] {
                        Data::String(s) if !s.trim().is_empty() => s.trim().to_lowercase(),
                        _ => category.to_string(),
                    }
                } else {
                    category.to_string()
                };

                let rows = category_rows.entry(category.to_string()).or_default();
                rows.push([desc, formula, expected_value, test_category, expected_type]);
            }
        }
    }

    // Write one TSV per category
    let mut total_rows = 0usize;
    for (category, rows) in &category_rows {
        let tsv_path = out_dir.join(format!("{category}.tsv"));
        let mut file = fs::File::create(&tsv_path)
            .unwrap_or_else(|e| panic!("failed to create {:?}: {}", tsv_path, e));

        writeln!(file, "description\tformula_text\texpected_value\ttest_category\texpected_type")
            .unwrap();
        for row in rows {
            // Sanitize: replace tabs/newlines within cells
            let sanitized: Vec<String> = row.iter().map(|s| {
                s.replace('\t', " ").replace('\n', " ").replace('\r', "")
            }).collect();
            writeln!(file, "{}", sanitized.join("\t")).unwrap();
        }
        println!("Wrote {} rows to {}", rows.len(), tsv_path.display());
        total_rows += rows.len();
    }

    println!("Migration complete: {} rows across {} categories", total_rows, category_rows.len());
}

fn cell_to_tsv_value(cell: &Data) -> Option<(String, String)> {
    match cell {
        Data::Float(f) => Some((format_float(*f), "number".to_string())),
        Data::Int(i) => Some((i.to_string(), "number".to_string())),
        Data::Bool(b) => Some((
            if *b { "TRUE".to_string() } else { "FALSE".to_string() },
            "boolean".to_string(),
        )),
        Data::String(s) => {
            let trimmed = s.trim();
            // Check if it looks like an error string
            if is_error_str(trimmed) {
                Some((trimmed.to_string(), "error".to_string()))
            } else {
                Some((s.clone(), "string".to_string()))
            }
        }
        Data::Error(e) => {
            let err_str = match e {
                CellErrorType::Div0  => "#DIV/0!",
                CellErrorType::Value => "#VALUE!",
                CellErrorType::Ref   => "#REF!",
                CellErrorType::Name  => "#NAME?",
                CellErrorType::Num   => "#NUM!",
                CellErrorType::NA    => "#N/A",
                CellErrorType::Null  => "#NULL!",
                _                    => return None,
            };
            Some((err_str.to_string(), "error".to_string()))
        }
        Data::Empty | Data::DateTimeIso(_) | Data::DurationIso(_) | Data::DateTime(_) => None,
    }
}

fn is_error_str(s: &str) -> bool {
    matches!(s, "#DIV/0!" | "#VALUE!" | "#REF!" | "#NAME?" | "#NUM!" | "#N/A" | "#NULL!" | "#ERROR!")
}

fn format_float(f: f64) -> String {
    // Avoid scientific notation for typical spreadsheet values
    if f == f.trunc() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        // Use enough precision to round-trip accurately
        format!("{:.10}", f).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
