use crate::types::{Platform, TestCase};

fn tc(description: &str, formula: &str, test_category: &str, expected_type: &str) -> TestCase {
    TestCase::new(description, formula, "", test_category, expected_type)
}

pub fn generate_parser(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── CONVERT ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("CONVERT 1 km -> 1000 m", r#"CONVERT(1,"km","m")"#, "basic", "number"),
        tc("CONVERT 1 m -> 0.001 km", r#"CONVERT(1,"m","km")"#, "basic", "number"),
        tc("CONVERT 100 C -> 212 F", r#"CONVERT(100,"C","F")"#, "basic", "number"),
        tc("CONVERT 0 C -> 32 F", r#"CONVERT(0,"C","F")"#, "basic", "number"),
        tc("CONVERT 32 F -> 0 C", r#"CONVERT(32,"F","C")"#, "basic", "number"),
        tc("CONVERT 1 hr -> 60 mn", r#"CONVERT(1,"hr","mn")"#, "basic", "number"),
        tc("CONVERT 1 day -> 24 hr", r#"CONVERT(1,"day","hr")"#, "basic", "number"),
        tc("CONVERT 1 kg -> lbm approx", r#"ROUND(CONVERT(1,"kg","lbm"),4)"#, "basic", "number"),
        tc("CONVERT 1 mi -> km approx", r#"ROUND(CONVERT(1,"mi","km"),4)"#, "edge", "number"),
        tc("CONVERT incompatible units J->W -> #N/A", r#"CONVERT(1,"J","W")"#, "error", "error"),
        tc("CONVERT unknown unit -> #N/A", r#"CONVERT(1,"xyz","m")"#, "error", "error"),
        tc("CONVERT no args -> #N/A", "CONVERT()", "error", "error"),
    ]);

    // ── TO_DATE ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("TO_DATE serial 1", "TO_DATE(1)", "basic", "number"),
        tc("TO_DATE serial 45292", "TO_DATE(45292)", "basic", "number"),
        tc("TO_DATE serial 0", "TO_DATE(0)", "basic", "number"),
        tc("TO_DATE serial 44927 (2023-01-01)", "TO_DATE(44927)", "basic", "number"),
        tc("TO_DATE negative serial -1", "TO_DATE(-1)", "edge", "number"),
        tc("TO_DATE DATE serial as input", "TO_DATE(DATE(2024,1,15))", "edge", "number"),
        tc("TO_DATE large serial 50000", "TO_DATE(50000)", "edge", "number"),
        tc("TO_DATE text -> error", r#"TO_DATE("text")"#, "error", "string"),
        tc("TO_DATE no args -> #N/A", "TO_DATE()", "error", "error"),
        tc("TO_DATE nested in ISNUMBER", "ISNUMBER(TO_DATE(45292))", "nested", "boolean"),
    ]);

    // ── TO_DOLLARS ───────────────────────────────────────────────────────────
    cases.extend([
        tc("TO_DOLLARS integer 42", "TO_DOLLARS(42)", "basic", "number"),
        tc("TO_DOLLARS decimal 3.14159", "TO_DOLLARS(3.14159)", "basic", "number"),
        tc("TO_DOLLARS zero", "TO_DOLLARS(0)", "basic", "number"),
        tc("TO_DOLLARS negative -5", "TO_DOLLARS(-5)", "basic", "number"),
        tc("TO_DOLLARS large number 1000000", "TO_DOLLARS(1000000)", "basic", "number"),
        tc("TO_DOLLARS small fraction 0.01", "TO_DOLLARS(0.01)", "basic", "number"),
        tc("TO_DOLLARS negative fraction -0.99", "TO_DOLLARS(-0.99)", "edge", "number"),
        tc("TO_DOLLARS text -> error", r#"TO_DOLLARS("text")"#, "error", "string"),
        tc("TO_DOLLARS no args -> #N/A", "TO_DOLLARS()", "error", "error"),
        tc("TO_PURE_NUMBER strips TO_DOLLARS", "TO_PURE_NUMBER(TO_DOLLARS(42))", "nested", "number"),
    ]);

    // ── TO_PERCENT ───────────────────────────────────────────────────────────
    cases.extend([
        tc("TO_PERCENT 0.5 -> 0.5 (shown 50%)", "TO_PERCENT(0.5)", "basic", "number"),
        tc("TO_PERCENT 1 -> 1 (shown 100%)", "TO_PERCENT(1)", "basic", "number"),
        tc("TO_PERCENT 0 -> 0", "TO_PERCENT(0)", "basic", "number"),
        tc("TO_PERCENT -0.25", "TO_PERCENT(-0.25)", "basic", "number"),
        tc("TO_PERCENT 0.001 small fraction", "TO_PERCENT(0.001)", "basic", "number"),
        tc("TO_PERCENT 2.5 over 100%", "TO_PERCENT(2.5)", "edge", "number"),
        tc("TO_PERCENT -1 negative 100%", "TO_PERCENT(-1)", "edge", "number"),
        tc("TO_PERCENT text -> error", r#"TO_PERCENT("text")"#, "error", "string"),
        tc("TO_PERCENT no args -> #N/A", "TO_PERCENT()", "error", "error"),
        tc("TO_PURE_NUMBER strips TO_PERCENT", "TO_PURE_NUMBER(TO_PERCENT(0.5))", "nested", "number"),
    ]);

    // ── TO_PURE_NUMBER ───────────────────────────────────────────────────────
    cases.extend([
        tc("TO_PURE_NUMBER integer 42", "TO_PURE_NUMBER(42)", "basic", "number"),
        tc("TO_PURE_NUMBER decimal 3.14", "TO_PURE_NUMBER(3.14)", "basic", "number"),
        tc("TO_PURE_NUMBER zero", "TO_PURE_NUMBER(0)", "basic", "number"),
        tc("TO_PURE_NUMBER negative -5.5", "TO_PURE_NUMBER(-5.5)", "basic", "number"),
        tc("TO_PURE_NUMBER large 1000000", "TO_PURE_NUMBER(1000000)", "basic", "number"),
        tc("TO_PURE_NUMBER strip TO_DOLLARS", "TO_PURE_NUMBER(TO_DOLLARS(42))", "nested", "number"),
        tc("TO_PURE_NUMBER strip TO_PERCENT", "TO_PURE_NUMBER(TO_PERCENT(0.5))", "nested", "number"),
        tc("TO_PURE_NUMBER text -> error", r#"TO_PURE_NUMBER("text")"#, "error", "string"),
        tc("TO_PURE_NUMBER no args -> #N/A", "TO_PURE_NUMBER()", "error", "error"),
        tc("TO_PURE_NUMBER ISNUMBER result", "ISNUMBER(TO_PURE_NUMBER(42))", "nested", "boolean"),
    ]);

    // ── TO_TEXT ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("TO_TEXT integer 42 -> string", "TO_TEXT(42)", "basic", "number"),
        tc("TO_TEXT decimal 3.14", "TO_TEXT(3.14)", "basic", "number"),
        tc("TO_TEXT TRUE -> \"TRUE\"", "TO_TEXT(TRUE)", "basic", "string"),
        tc("TO_TEXT FALSE -> \"FALSE\"", "TO_TEXT(FALSE)", "basic", "string"),
        tc("TO_TEXT zero -> \"0\"", "TO_TEXT(0)", "basic", "number"),
        tc("TO_TEXT negative -5", "TO_TEXT(-5)", "basic", "number"),
        tc("TO_TEXT large number 100000", "TO_TEXT(100000)", "edge", "number"),
        tc("TO_TEXT small decimal 0.001", "TO_TEXT(0.001)", "edge", "number"),
        tc("TO_TEXT no args -> #N/A", "TO_TEXT()", "error", "error"),
        tc("LEN(TO_TEXT(12345)) -> 5", "LEN(TO_TEXT(12345))", "nested", "number"),
    ]);

    cases
}

pub fn generate_web(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── ENCODEURL ────────────────────────────────────────────────────────────
    cases.extend([
        tc("ENCODEURL space -> %20", r#"ENCODEURL("hello world")"#, "basic", "string"),
        tc("ENCODEURL special chars encoded", r#"ENCODEURL("a+b=c&d")"#, "basic", "string"),
        tc("ENCODEURL no special chars -> unchanged", r#"ENCODEURL("hello")"#, "basic", "string"),
        tc("ENCODEURL empty string -> empty", r#"ENCODEURL("")"#, "basic", "string"),
        tc("ENCODEURL percent sign -> %25", r#"ENCODEURL("100%")"#, "basic", "string"),
        tc("ENCODEURL slash encoded", r#"ENCODEURL("a/b")"#, "edge", "string"),
        tc("ENCODEURL LEN of encoded space = 13", r#"LEN(ENCODEURL("hello world"))"#, "nested", "number"),
        tc("ENCODEURL no args -> #N/A", "ENCODEURL()", "error", "error"),
        tc("ENCODEURL number arg coercion", "ENCODEURL(42)", "coercion", "string"),
        tc("ENCODEURL unicode chars", r#"ENCODEURL("caf\u00e9")"#, "edge", "string"),
    ]);

    // ── HYPERLINK ────────────────────────────────────────────────────────────
    cases.extend([
        tc("HYPERLINK with label -> returns label", r#"HYPERLINK("https://example.com","Click here")"#, "basic", "string"),
        tc("HYPERLINK no label -> returns url", r#"HYPERLINK("https://example.com")"#, "basic", "string"),
        tc("HYPERLINK LEN of label = 5", r#"LEN(HYPERLINK("https://example.com","Click"))"#, "nested", "number"),
        tc("HYPERLINK empty label -> empty string", r#"HYPERLINK("https://example.com","")"#, "basic", "string"),
        tc("HYPERLINK numeric label coerced to text", r#"HYPERLINK("https://example.com",42)"#, "coercion", "number"),
        tc("HYPERLINK label with spaces -> returns label", r#"HYPERLINK("https://example.com","Go here now")"#, "basic", "string"),
        tc("HYPERLINK no args -> #N/A", "HYPERLINK()", "error", "error"),
        tc("HYPERLINK ISTEXT of label result", r#"ISTEXT(HYPERLINK("https://example.com","label"))"#, "nested", "boolean"),
        tc("HYPERLINK http url", r#"HYPERLINK("http://example.com","link")"#, "basic", "string"),
        tc("HYPERLINK label is boolean", r#"HYPERLINK("https://example.com",TRUE)"#, "coercion", "string"),
    ]);

    // ── ISURL ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISURL https url -> TRUE", r#"ISURL("https://example.com")"#, "basic", "boolean"),
        tc("ISURL http url -> TRUE", r#"ISURL("http://example.com")"#, "basic", "boolean"),
        tc("ISURL ftp url -> TRUE", r#"ISURL("ftp://example.com")"#, "basic", "boolean"),
        tc("ISURL plain text -> FALSE", r#"ISURL("not a url")"#, "basic", "boolean"),
        tc("ISURL empty string -> FALSE", r#"ISURL("")"#, "basic", "boolean"),
        tc("ISURL number -> FALSE", "ISURL(42)", "coercion", "boolean"),
        tc("ISURL no args -> #N/A", "ISURL()", "error", "error"),
        tc("ISURL domain only", r#"ISURL("example.com")"#, "basic", "boolean"),
        tc("IF(ISURL url, 1, 0)", r#"IF(ISURL("https://example.com"),1,0)"#, "nested", "number"),
        tc("AND(ISURL https, ISURL http)", r#"AND(ISURL("https://a.com"),ISURL("http://b.com"))"#, "nested", "boolean"),
    ]);

    cases
}
