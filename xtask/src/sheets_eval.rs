use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::types::TestCase;

pub struct SheetsEvaluator {
    web_app_url: String,
    client: reqwest::blocking::Client,
}

impl SheetsEvaluator {
    pub fn new(web_app_url: String) -> Self {
        Self {
            web_app_url,
            client: reqwest::blocking::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    /// Evaluate a batch of TestCases; returns them with expected_value/expected_type filled in.
    pub fn evaluate(&self, cases: &[TestCase]) -> Result<Vec<TestCase>> {
        const BATCH_SIZE: usize = 200;
        let mut results = Vec::with_capacity(cases.len());

        for chunk in cases.chunks(BATCH_SIZE) {
            let evaluated = self.evaluate_batch(chunk)?;
            results.extend(evaluated);
        }

        Ok(results)
    }

    fn evaluate_batch(&self, cases: &[TestCase]) -> Result<Vec<TestCase>> {
        let formulas: Vec<&str> = cases.iter().map(|c| c.formula.as_str()).collect();
        let body = json!({ "formulas": formulas });

        // Step 1: POST to the web app — it responds with a 302 to the actual result URL
        let resp = self
            .client
            .post(&self.web_app_url)
            .json(&body)
            .send()
            .context("web app request failed")?;

        let status = resp.status();
        if status.as_u16() != 302 {
            bail!("expected 302 redirect from web app, got {}", status);
        }

        let location = resp
            .headers()
            .get("location")
            .context("no Location header in 302 response")?
            .to_str()
            .context("Location header is not valid UTF-8")?
            .to_string();

        // Step 2: Follow the redirect with a GET to retrieve the JSON result
        let result_resp = self
            .client
            .get(&location)
            .send()
            .context("redirect GET request failed")?;

        let result_status = result_resp.status();
        if !result_status.is_success() {
            bail!("redirect GET failed with status {}", result_status);
        }

        let json: Value = result_resp.json().context("web app result: invalid JSON")?;

        let results_arr = json["results"]
            .as_array()
            .context("web app result: missing 'results' array")?;

        if results_arr.len() != cases.len() {
            bail!(
                "result count mismatch: expected {}, got {}",
                cases.len(),
                results_arr.len()
            );
        }

        let mut out = Vec::with_capacity(cases.len());
        for (case, result) in cases.iter().zip(results_arr.iter()) {
            let value = result["value"].as_str().unwrap_or("").to_string();
            let typ = result["type"].as_str().unwrap_or("string").to_string();
            out.push(TestCase {
                description: case.description.clone(),
                formula: case.formula.clone(),
                expected_value: value,
                test_category: case.test_category.clone(),
                expected_type: typ,
            });
        }

        Ok(out)
    }
}
