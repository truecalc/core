use crate::types::{Platform, TestCase};

const DB: &str =
    r#"{"Name","Dept","Sales";"Alice","A",100;"Bob","B",200;"Carol","A",150;"Dave","B",80}"#;
const CRITERIA_A: &str = r#"{"Dept";"A"}"#;
const CRITERIA_GT100: &str = r#"{"Sales";">100"}"#;

pub fn generate(_platform: Platform) -> Vec<TestCase> {
    vec![
        TestCase::new(
            "DSUM inline array dept A",
            format!("DSUM({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DSUM inline array sales >100",
            format!("DSUM({},\"Sales\",{})", DB, CRITERIA_GT100),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DAVERAGE inline array dept A",
            format!("DAVERAGE({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DCOUNT inline array dept A",
            format!("DCOUNT({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DCOUNTA inline array dept A",
            format!("DCOUNTA({},\"Name\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DGET inline array single match",
            r#"DGET({"Name","Score";"Alice",90},"Name",{"Score";90})"#,
            "",
            "database",
            "string",
        ),
        TestCase::new(
            "DMAX inline array dept A",
            format!("DMAX({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DMIN inline array dept A",
            format!("DMIN({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DPRODUCT inline array sales >100",
            format!("DPRODUCT({},\"Sales\",{})", DB, CRITERIA_GT100),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DSTDEV inline array dept A",
            format!("DSTDEV({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DSTDEVP inline array dept A",
            format!("DSTDEVP({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DVAR inline array dept A",
            format!("DVAR({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
        TestCase::new(
            "DVARP inline array dept A",
            format!("DVARP({},\"Sales\",{})", DB, CRITERIA_A),
            "",
            "database",
            "number",
        ),
    ]
}
