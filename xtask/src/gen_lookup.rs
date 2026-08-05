use crate::types::{Platform, TestCase, wrap_array};

pub fn generate(platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();
    let cat = "lookup";

    // ── VLOOKUP ──────────────────────────────────────────────────────────────
    // Present (>=50%)
    cases.push(TestCase::new(
        "VLOOKUP exact match found",
        r#"VLOOKUP("b",{"a","x";"b","y";"c","z"},2,FALSE)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "VLOOKUP exact match first row",
        r#"VLOOKUP("a",{"a","x";"b","y";"c","z"},2,FALSE)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "VLOOKUP exact match last row",
        r#"VLOOKUP("c",{"a","x";"b","y";"c","z"},2,FALSE)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "VLOOKUP exact match returns number",
        r#"VLOOKUP("b",{"a",1;"b",2;"c",3},2,FALSE)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "VLOOKUP approximate match sorted",
        r#"VLOOKUP(2.5,{1,10;2,20;3,30},2,TRUE)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "VLOOKUP approximate match exact boundary",
        r#"VLOOKUP(2,{1,10;2,20;3,30},2,TRUE)"#,
        "", cat, "number",
    ));
    // Absent (>=25%)
    cases.push(TestCase::new(
        "VLOOKUP exact match not found",
        r#"VLOOKUP("z",{"a","x";"b","y";"c","z"},2,FALSE)"#,
        "", cat, "error",
    ));
    cases.push(TestCase::new(
        "VLOOKUP col index out of range",
        r#"VLOOKUP("a",{"a","x"},5,FALSE)"#,
        "", cat, "error",
    ));

    // ── HLOOKUP ──────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "HLOOKUP exact match found",
        r#"HLOOKUP("b",{"a","b","c";1,2,3},2,FALSE)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "HLOOKUP exact match first column",
        r#"HLOOKUP("a",{"a","b","c";10,20,30},2,FALSE)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "HLOOKUP exact match last column",
        r#"HLOOKUP("c",{"a","b","c";10,20,30},2,FALSE)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "HLOOKUP approximate match",
        r#"HLOOKUP(2.5,{1,2,3;10,20,30},2,TRUE)"#,
        "", cat, "number",
    ));
    // Absent
    cases.push(TestCase::new(
        "HLOOKUP exact match not found",
        r#"HLOOKUP("z",{"a","b","c";1,2,3},2,FALSE)"#,
        "", cat, "error",
    ));

    // ── LOOKUP ───────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "LOOKUP vector form exact",
        r#"LOOKUP(2,{1,2,3},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "LOOKUP vector form approximate",
        r#"LOOKUP(2.5,{1,2,3},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "LOOKUP vector form first element",
        r#"LOOKUP(1,{1,2,3},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "LOOKUP vector form last element",
        r#"LOOKUP(3,{1,2,3},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "LOOKUP single range form",
        r#"LOOKUP(2,{1,2,3})"#,
        "", cat, "number",
    ));
    // Absent
    cases.push(TestCase::new(
        "LOOKUP value below range",
        r#"LOOKUP(0,{1,2,3},{10,20,30})"#,
        "", cat, "error",
    ));

    // ── XLOOKUP ──────────────────────────────────────────────────────────────
    // Present
    cases.push(TestCase::new(
        "XLOOKUP exact match found",
        r#"XLOOKUP("b",{"a","b","c"},{1,2,3})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP exact match first element",
        r#"XLOOKUP("a",{"a","b","c"},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP exact match last element",
        r#"XLOOKUP("c",{"a","b","c"},{10,20,30})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP exact match with if_not_found",
        r#"XLOOKUP("b",{"a","b","c"},{1,2,3},"none")"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP approx match_mode=1 next larger",
        r#"XLOOKUP(2.5,{1,2,3,4},{10,20,30,40},"none",1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP approx match_mode=-1 next smaller",
        r#"XLOOKUP(2.5,{1,2,3,4},{10,20,30,40},"none",-1)"#,
        "", cat, "number",
    ));
    // BUG-12: wildcard match_mode=2
    cases.push(TestCase::new(
        "XLOOKUP wildcard match_mode=2 BUG-12",
        r#"XLOOKUP("b*",{"a","ba","bc"},{1,2,3},,2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP wildcard match_mode=2 matches second",
        r#"XLOOKUP("b*",{"x","ba","bc"},{1,2,3},,2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP wildcard match_mode=2 prefix a",
        r#"XLOOKUP("a*",{"a","ba","bc"},{1,2,3},,2)"#,
        "", cat, "number",
    ));
    // Absent
    cases.push(TestCase::new(
        "XLOOKUP exact match not found no fallback",
        r#"XLOOKUP("z",{"a","b","c"},{1,2,3})"#,
        "", cat, "error",
    ));
    cases.push(TestCase::new(
        "XLOOKUP exact match not found with fallback",
        r#"XLOOKUP("z",{"a","b","c"},{1,2,3},"none")"#,
        "", cat, "text",
    ));
    // Wildcard no match
    cases.push(TestCase::new(
        "XLOOKUP wildcard no match with fallback",
        r#"XLOOKUP("z*",{"a","ba","bc"},{1,2,3},"none",2)"#,
        "", cat, "text",
    ));

    // ── XMATCH ───────────────────────────────────────────────────────────────
    // Present
    cases.push(TestCase::new(
        "XMATCH exact match position 1",
        r#"XMATCH("a",{"a","b","c"})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH exact match position 2",
        r#"XMATCH("b",{"a","b","c"})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH exact match position 3",
        r#"XMATCH("c",{"a","b","c"})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH exact match number",
        r#"XMATCH(2,{1,2,3,4,5})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH approx match_mode=1 sorted asc",
        r#"XMATCH(2.5,{1,2,3,4,5},1)"#,
        "", cat, "number",
    ));
    // BUG-13: wildcard match_mode=2
    cases.push(TestCase::new(
        "XMATCH wildcard match_mode=2 BUG-13",
        r#"XMATCH("b*",{"a","ba","bc"},2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH wildcard match_mode=2 prefix x",
        r#"XMATCH("x*",{"x1","x2","y"},2)"#,
        "", cat, "number",
    ));
    // BUG-13: reverse approx search_mode=-1
    cases.push(TestCase::new(
        "XMATCH reverse approx search_mode=-1 BUG-13",
        r#"XMATCH(3.5,{1,2,3,4,5},-1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XMATCH sorted desc match_mode=-1",
        r#"XMATCH(3,{5,4,3,2,1},-1)"#,
        "", cat, "number",
    ));
    // Absent
    cases.push(TestCase::new(
        "XMATCH exact not found",
        r#"XMATCH("z",{"a","b","c"})"#,
        "", cat, "error",
    ));

    // ── MATCH ────────────────────────────────────────────────────────────────
    // Present
    cases.push(TestCase::new(
        "MATCH exact match_type=0 found",
        r#"MATCH("b",{"a","b","c"},0)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH exact match_type=0 first",
        r#"MATCH("a",{"a","b","c"},0)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH exact match_type=0 last",
        r#"MATCH("c",{"a","b","c"},0)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH approx match_type=1 sorted asc",
        r#"MATCH(2.5,{1,2,3,4,5},1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH approx match_type=1 exact boundary",
        r#"MATCH(3,{1,2,3,4,5},1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH approx match_type=-1 sorted desc",
        r#"MATCH(3.5,{5,4,3,2,1},-1)"#,
        "", cat, "number",
    ));
    // BUG-14: wildcard match_type=0
    cases.push(TestCase::new(
        "MATCH wildcard match_type=0 BUG-14",
        r#"MATCH("b*",{"a","ba","bc"},0)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH wildcard question mark match_type=0",
        r#"MATCH("b?",{"a","ba","bc"},0)"#,
        "", cat, "number",
    ));
    // Absent
    cases.push(TestCase::new(
        "MATCH exact not found",
        r#"MATCH("z",{"a","b","c"},0)"#,
        "", cat, "error",
    ));

    // ── INDEX ────────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "INDEX 1D array row 1",
        r#"INDEX({10,20,30},1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 1D array row 2",
        r#"INDEX({10,20,30},2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 1D array row 3",
        r#"INDEX({10,20,30},3)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 2D array row 1 col 2",
        r#"INDEX({1,2;3,4},1,2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 2D array row 2 col 1",
        r#"INDEX({1,2;3,4},2,1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 2D array row 2 col 2",
        r#"INDEX({1,2;3,4},2,2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX text array",
        r#"INDEX({"a","b","c"},2)"#,
        "", cat, "text",
    ));
    // Out of bounds
    cases.push(TestCase::new(
        "INDEX out of bounds",
        r#"INDEX({10,20,30},5)"#,
        "", cat, "error",
    ));

    // ── ROWS ─────────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "ROWS 1D array",
        r#"ROWS({1,2,3})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROWS 2D array 2 rows",
        r#"ROWS({1,2;3,4})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROWS 2D array 3 rows",
        r#"ROWS({1,2;3,4;5,6})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROWS single value",
        r#"ROWS(42)"#,
        "", cat, "number",
    ));

    // ── COLUMNS ──────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "COLUMNS 1D array 3 cols",
        r#"COLUMNS({1,2,3})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMNS 2D array 2 cols",
        r#"COLUMNS({1,2;3,4})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMNS 2D array 3 cols",
        r#"COLUMNS({1,2,3;4,5,6})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMNS single value",
        r#"COLUMNS(42)"#,
        "", cat, "number",
    ));

    // ── CHOOSE ───────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "CHOOSE index 1",
        r#"CHOOSE(1,"alpha","beta","gamma")"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "CHOOSE index 2",
        r#"CHOOSE(2,"alpha","beta","gamma")"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "CHOOSE index 3",
        r#"CHOOSE(3,"alpha","beta","gamma")"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "CHOOSE number values index 2",
        r#"CHOOSE(2,10,20,30)"#,
        "", cat, "number",
    ));
    // Out of bounds
    cases.push(TestCase::new(
        "CHOOSE index out of range",
        r#"CHOOSE(5,"a","b","c")"#,
        "", cat, "error",
    ));
    cases.push(TestCase::new(
        "CHOOSE index zero",
        r#"CHOOSE(0,"a","b","c")"#,
        "", cat, "error",
    ));

    // ── ADDRESS ──────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "ADDRESS abs mode 1 A1 style",
        r#"ADDRESS(1,1)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "ADDRESS abs mode 1 row 3 col 4",
        r#"ADDRESS(3,4,1)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "ADDRESS abs mode 4 relative",
        r#"ADDRESS(3,4,4)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "ADDRESS abs mode 2 col relative",
        r#"ADDRESS(3,4,2)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "ADDRESS abs mode 3 row relative",
        r#"ADDRESS(3,4,3)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "ADDRESS R1C1 style",
        r#"ADDRESS(3,4,1,FALSE)"#,
        "", cat, "text",
    ));

    // ── ROW ──────────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "ROW no argument returns 1",
        r#"ROW()"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROW cell reference A1",
        r#"ROW(A1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROW cell reference C5",
        r#"ROW(C5)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "ROW cell reference Z10",
        r#"ROW(Z10)"#,
        "", cat, "number",
    ));
    // BUG-20 style: array arg
    cases.push(TestCase::new(
        "ROW array arg BUG-20",
        wrap_array("ROW({1;2;3})", platform),
        "", cat, "text",
    ));

    // ── COLUMN ───────────────────────────────────────────────────────────────
    cases.push(TestCase::new(
        "COLUMN no argument returns 1",
        r#"COLUMN()"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMN cell reference A1",
        r#"COLUMN(A1)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMN cell reference D5",
        r#"COLUMN(D5)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMN cell reference Z1",
        r#"COLUMN(Z1)"#,
        "", cat, "number",
    ));
    // BUG-20 style: array arg
    cases.push(TestCase::new(
        "COLUMN array arg BUG-20",
        wrap_array("COLUMN({1,2,3})", platform),
        "", cat, "text",
    ));

    // ── Additional VLOOKUP edge cases ─────────────────────────────────────────
    cases.push(TestCase::new(
        "VLOOKUP returns text value",
        r#"VLOOKUP("b",{"a","apple";"b","banana";"c","cherry"},2,FALSE)"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "VLOOKUP col index 1 returns key",
        r#"VLOOKUP("b",{"a","x";"b","y";"c","z"},1,FALSE)"#,
        "", cat, "text",
    ));

    // ── Additional XLOOKUP edge cases ─────────────────────────────────────────
    cases.push(TestCase::new(
        "XLOOKUP numeric lookup",
        r#"XLOOKUP(2,{1,2,3},{100,200,300})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "XLOOKUP text return value",
        r#"XLOOKUP("b",{"a","b","c"},{"alpha","beta","gamma"})"#,
        "", cat, "text",
    ));

    // ── Additional MATCH edge cases ───────────────────────────────────────────
    cases.push(TestCase::new(
        "MATCH default match_type (1)",
        r#"MATCH(3,{1,2,3,4,5})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "MATCH exact number array position 5",
        r#"MATCH(5,{1,2,3,4,5},0)"#,
        "", cat, "number",
    ));

    // ── Additional INDEX edge cases ───────────────────────────────────────────
    cases.push(TestCase::new(
        "INDEX 1D vertical array row 2",
        r#"INDEX({10;20;30},2)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "INDEX 2D array row 1 col 1",
        r#"INDEX({1,2;3,4},1,1)"#,
        "", cat, "number",
    ));

    // ── Additional CHOOSE edge cases ──────────────────────────────────────────
    cases.push(TestCase::new(
        "CHOOSE mixed types index 1",
        r#"CHOOSE(1,TRUE,2,"three")"#,
        "", cat, "bool",
    ));
    cases.push(TestCase::new(
        "CHOOSE two options index 2",
        r#"CHOOSE(2,"no","yes")"#,
        "", cat, "text",
    ));

    // ── Additional LOOKUP edge cases ──────────────────────────────────────────
    cases.push(TestCase::new(
        "LOOKUP text vector",
        r#"LOOKUP("b",{"a","b","c"},{"x","y","z"})"#,
        "", cat, "text",
    ));
    cases.push(TestCase::new(
        "LOOKUP text approximate last",
        r#"LOOKUP("d",{"a","b","c"},{"x","y","z"})"#,
        "", cat, "text",
    ));

    // ── Additional ADDRESS edge cases ─────────────────────────────────────────
    cases.push(TestCase::new(
        "ADDRESS large row and col",
        r#"ADDRESS(100,26)"#,
        "", cat, "text",
    ));

    // ── Additional ROWS/COLUMNS edge cases ────────────────────────────────────
    cases.push(TestCase::new(
        "ROWS vertical array 3 rows",
        r#"ROWS({1;2;3})"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMNS vertical array 1 col",
        r#"COLUMNS({1;2;3})"#,
        "", cat, "number",
    ));

    // ── Additional ROW/COLUMN with ranges ────────────────────────────────────
    cases.push(TestCase::new(
        "ROW range reference B3:D5",
        r#"ROW(B3:D5)"#,
        "", cat, "number",
    ));
    cases.push(TestCase::new(
        "COLUMN range reference B3:D5",
        r#"COLUMN(B3:D5)"#,
        "", cat, "number",
    ));

    cases
}
