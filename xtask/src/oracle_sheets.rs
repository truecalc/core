use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::types::TestCase;

pub struct SheetsOracle {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl SheetsOracle {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Evaluate a batch of TestCases; returns them with expected_value/expected_type filled in.
    pub fn evaluate(&self, cases: &[TestCase]) -> Result<Vec<TestCase>> {
        const BATCH_SIZE: usize = 500;
        let mut results = Vec::with_capacity(cases.len());

        for chunk in cases.chunks(BATCH_SIZE) {
            let evaluated = self.evaluate_batch(chunk)?;
            results.extend(evaluated);
        }

        Ok(results)
    }

    fn evaluate_batch(&self, cases: &[TestCase]) -> Result<Vec<TestCase>> {
        let n = cases.len();

        // Step 1: Create a new spreadsheet
        let spreadsheet_id = self.create_spreadsheet()?;

        // Ensure cleanup happens even on error
        let result = self.evaluate_batch_inner(cases, &spreadsheet_id, n);
        self.delete_spreadsheet(&spreadsheet_id).ok();
        result
    }

    fn evaluate_batch_inner(
        &self,
        cases: &[TestCase],
        spreadsheet_id: &str,
        n: usize,
    ) -> Result<Vec<TestCase>> {
        // Step 2: Write formulas to column A
        // formula_text in the TSV already has the "=" prefix; use it directly.
        let rows: Vec<Value> = cases
            .iter()
            .map(|c| json!([c.formula]))
            .collect();

        self.write_values(spreadsheet_id, n, &rows)?;

        // Step 3: Read evaluated values from column A
        let values = self.read_values(spreadsheet_id, n)?;

        // Step 4: Build result TestCases
        let mut results = Vec::with_capacity(n);
        for (i, case) in cases.iter().enumerate() {
            let cell_value = values.get(i).and_then(|row| row.first());
            let (expected_value, expected_type) = infer_value_and_type(cell_value);
            results.push(TestCase {
                description: case.description.clone(),
                formula: case.formula.clone(),
                expected_value,
                test_category: case.test_category.clone(),
                expected_type,
            });
        }

        Ok(results)
    }

    fn create_spreadsheet(&self) -> Result<String> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets?key={}",
            self.api_key
        );
        let body = json!({
            "properties": { "title": "truecalc-oracle-tmp" }
        });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .context("create spreadsheet request failed")?;

        let status = resp.status();
        let json: Value = resp.json().context("create spreadsheet: invalid JSON")?;

        if !status.is_success() {
            bail!("create spreadsheet failed ({}): {}", status, json);
        }

        json["spreadsheetId"]
            .as_str()
            .map(|s| s.to_string())
            .context("create spreadsheet: missing spreadsheetId")
    }

    fn write_values(&self, id: &str, n: usize, rows: &[Value]) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/A1:A{}?valueInputOption=USER_ENTERED&key={}",
            id, n, self.api_key
        );
        let body = json!({ "values": rows });
        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .context("write values request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let json: Value = resp.json().unwrap_or(Value::Null);
            bail!("write values failed ({}): {}", status, json);
        }

        Ok(())
    }

    fn read_values(&self, id: &str, n: usize) -> Result<Vec<Vec<Value>>> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/A1:A{}?valueRenderOption=UNFORMATTED_VALUE&key={}",
            id, n, self.api_key
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .context("read values request failed")?;

        let status = resp.status();
        let json: Value = resp.json().context("read values: invalid JSON")?;

        if !status.is_success() {
            bail!("read values failed ({}): {}", status, json);
        }

        // "values" may be absent if the sheet is empty
        let values = match json.get("values").and_then(|v| v.as_array()) {
            Some(rows) => rows
                .iter()
                .map(|row| {
                    row.as_array()
                        .cloned()
                        .unwrap_or_default()
                })
                .collect(),
            None => vec![],
        };

        Ok(values)
    }

    fn delete_spreadsheet(&self, id: &str) -> Result<()> {
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?key={}",
            id, self.api_key
        );
        self.client
            .delete(&url)
            .send()
            .context("delete spreadsheet request failed")?;
        Ok(())
    }
}

/// Infer (expected_value, expected_type) from a raw JSON cell value.
fn infer_value_and_type(cell: Option<&Value>) -> (String, String) {
    match cell {
        None => (String::new(), "error".to_string()),
        Some(Value::Number(n)) => (n.to_string(), "number".to_string()),
        Some(Value::Bool(b)) => (b.to_string(), "boolean".to_string()),
        Some(Value::String(s)) => {
            // Error values start with "#" followed by letters (e.g., "#REF!", "#VALUE!")
            let is_error = s.starts_with('#')
                && s.len() > 1
                && s.chars().nth(1).is_some_and(|c| c.is_ascii_alphabetic());
            if is_error {
                (s.clone(), "error".to_string())
            } else {
                (s.clone(), "string".to_string())
            }
        }
        Some(other) => (other.to_string(), "string".to_string()),
    }
}
