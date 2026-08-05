use std::path::Path;

use anyhow::Result;

use crate::types::TestCase;

pub fn write_tsv(cases: &[TestCase], path: &Path) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().delimiter(b'\t').from_path(path)?;

    wtr.write_record([
        "description",
        "formula_text",
        "expected_value",
        "test_category",
        "expected_type",
    ])?;

    for case in cases {
        wtr.write_record([
            &case.description,
            &format!("={}", case.formula),
            &case.expected_value,
            &case.test_category,
            &case.expected_type,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
