use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Sheets,
    Excel,
}

impl Platform {
    pub fn dir_name(&self) -> &'static str {
        match self {
            Platform::Sheets => "google_sheets",
            Platform::Excel => "excel",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub description: String,
    pub formula: String,
    pub expected_value: String,
    pub test_category: String,
    pub expected_type: String,
}

impl TestCase {
    #[allow(dead_code)]
    pub fn new(
        description: impl Into<String>,
        formula: impl Into<String>,
        expected_value: impl Into<String>,
        test_category: impl Into<String>,
        expected_type: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            formula: formula.into(),
            expected_value: expected_value.into(),
            test_category: test_category.into(),
            expected_type: expected_type.into(),
        }
    }
}

#[allow(dead_code)]
pub const VOLATILE_FUNCTIONS: &[&str] = &["RAND", "RANDARRAY", "NOW", "TODAY", "RANDBETWEEN"];

#[allow(dead_code)]
pub const CONTEXT_LIMITED_FUNCTIONS: &[&str] = &[
    "INDIRECT",
    "OFFSET",
    "FORMULATEXT",
    "GETPIVOTDATA",
    "ISFORMULA",
    "CELL",
];

#[allow(dead_code)]
pub fn wrap_array(formula: &str, platform: Platform) -> String {
    match platform {
        Platform::Sheets => format!("ARRAYTOTEXT({}, 1)", formula),
        Platform::Excel => format!("TEXTJOIN(\",\", TRUE, {})", formula),
    }
}
