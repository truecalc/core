use crate::types::{Platform, TestCase};

/// Helper: produce a TestCase with empty expected_value (evaluator fills it).
fn tc(description: &str, formula: &str, test_category: &str, expected_type: &str) -> TestCase {
    TestCase::new(description, formula, "", test_category, expected_type)
}

pub fn generate(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── ABS ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ABS positive", "ABS(5)", "basic", "number"),
        tc("ABS negative", "ABS(-3)", "basic", "number"),
        tc("ABS zero", "ABS(0)", "basic", "number"),
        tc("ABS large negative", "ABS(-1E15)", "edge", "number"),
        tc(
            "ABS boolean TRUE coercion",
            "ABS(TRUE)",
            "coercion",
            "number",
        ),
        tc(
            "ABS numeric string coercion",
            r#"ABS("7")"#,
            "coercion",
            "number",
        ),
        tc("ABS non-numeric text", r#"ABS("hello")"#, "error", "error"),
    ]);

    // ── CEILING ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("CEILING positive", "CEILING(4.3,1)", "basic", "number"),
        tc(
            "CEILING negative significance",
            "CEILING(-4.3,-1)",
            "basic",
            "number",
        ),
        tc("CEILING zero value", "CEILING(0,1)", "basic", "number"),
        tc(
            "CEILING fractional significance",
            "CEILING(4.3,0.5)",
            "basic",
            "number",
        ),
        tc("CEILING large value", "CEILING(1E10,100)", "edge", "number"),
        tc(
            "CEILING coercion string num",
            r#"CEILING("4.3",1)"#,
            "coercion",
            "number",
        ),
        tc(
            "CEILING non-numeric text",
            r#"CEILING("x",1)"#,
            "error",
            "error",
        ),
    ]);

    // ── CEILING.MATH ──────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "CEILING.MATH positive",
            "CEILING.MATH(4.3)",
            "basic",
            "number",
        ),
        tc(
            "CEILING.MATH negative default",
            "CEILING.MATH(-4.3)",
            "basic",
            "number",
        ),
        tc(
            "CEILING.MATH with significance",
            "CEILING.MATH(4.3,2)",
            "basic",
            "number",
        ),
        tc(
            "CEILING.MATH negative mode=1",
            "CEILING.MATH(-4.3,1,1)",
            "edge",
            "number",
        ),
        tc("CEILING.MATH zero", "CEILING.MATH(0)", "basic", "number"),
    ]);

    // ── CEILING.PRECISE ───────────────────────────────────────────────────────
    cases.extend([
        tc(
            "CEILING.PRECISE positive",
            "CEILING.PRECISE(4.3,1)",
            "basic",
            "number",
        ),
        tc(
            "CEILING.PRECISE negative",
            "CEILING.PRECISE(-4.3,1)",
            "basic",
            "number",
        ),
        tc(
            "CEILING.PRECISE no sig",
            "CEILING.PRECISE(4.3)",
            "basic",
            "number",
        ),
    ]);

    // ── FLOOR ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("FLOOR positive", "FLOOR(4.7,1)", "basic", "number"),
        tc("FLOOR negative", "FLOOR(-4.7,-1)", "basic", "number"),
        tc("FLOOR fractional sig", "FLOOR(4.7,0.5)", "basic", "number"),
        tc("FLOOR zero", "FLOOR(0,1)", "basic", "number"),
        tc("FLOOR large", "FLOOR(1E10,1000)", "edge", "number"),
        tc("FLOOR coercion", r#"FLOOR("4.7",1)"#, "coercion", "number"),
        tc("FLOOR non-numeric", r#"FLOOR("x",1)"#, "error", "error"),
    ]);

    // ── FLOOR.MATH ────────────────────────────────────────────────────────────
    cases.extend([
        tc("FLOOR.MATH positive", "FLOOR.MATH(4.7)", "basic", "number"),
        tc(
            "FLOOR.MATH negative default",
            "FLOOR.MATH(-4.7)",
            "basic",
            "number",
        ),
        tc(
            "FLOOR.MATH with significance",
            "FLOOR.MATH(4.7,2)",
            "basic",
            "number",
        ),
        tc(
            "FLOOR.MATH negative mode=1",
            "FLOOR.MATH(-4.7,1,1)",
            "edge",
            "number",
        ),
    ]);

    // ── FLOOR.PRECISE ─────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "FLOOR.PRECISE positive",
            "FLOOR.PRECISE(4.7,1)",
            "basic",
            "number",
        ),
        tc(
            "FLOOR.PRECISE negative",
            "FLOOR.PRECISE(-4.7,1)",
            "basic",
            "number",
        ),
        tc(
            "FLOOR.PRECISE no sig",
            "FLOOR.PRECISE(4.7)",
            "basic",
            "number",
        ),
    ]);

    // ── EVEN ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("EVEN positive odd", "EVEN(3)", "basic", "number"),
        tc("EVEN positive even", "EVEN(4)", "basic", "number"),
        tc("EVEN negative", "EVEN(-3)", "basic", "number"),
        tc("EVEN zero", "EVEN(0)", "basic", "number"),
        tc("EVEN fractional", "EVEN(1.5)", "edge", "number"),
        tc("EVEN boolean coercion", "EVEN(TRUE)", "coercion", "number"),
    ]);

    // ── ODD ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ODD positive even", "ODD(4)", "basic", "number"),
        tc("ODD positive odd", "ODD(3)", "basic", "number"),
        tc("ODD negative", "ODD(-4)", "basic", "number"),
        tc("ODD zero", "ODD(0)", "basic", "number"),
        tc("ODD fractional", "ODD(1.5)", "edge", "number"),
    ]);

    // ── INT ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("INT positive", "INT(4.9)", "basic", "number"),
        tc("INT negative", "INT(-4.1)", "basic", "number"),
        tc("INT zero", "INT(0)", "basic", "number"),
        tc("INT boolean coercion", "INT(TRUE)", "coercion", "number"),
        tc("INT numeric string", r#"INT("3.7")"#, "coercion", "number"),
        tc("INT non-numeric", r#"INT("x")"#, "error", "error"),
    ]);

    // ── TRUNC ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("TRUNC positive no digits", "TRUNC(4.9)", "basic", "number"),
        tc("TRUNC negative no digits", "TRUNC(-4.9)", "basic", "number"),
        tc("TRUNC with digits", "TRUNC(3.14159,2)", "basic", "number"),
        tc(
            "TRUNC zero digits explicit",
            "TRUNC(3.7,0)",
            "basic",
            "number",
        ),
        tc("TRUNC large", "TRUNC(1.23456789E10,2)", "edge", "number"),
    ]);

    // ── ROUND ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ROUND positive half up", "ROUND(4.5,0)", "basic", "number"),
        tc(
            "ROUND positive round down",
            "ROUND(4.4,0)",
            "basic",
            "number",
        ),
        tc("ROUND negative", "ROUND(-4.5,0)", "basic", "number"),
        tc("ROUND two decimals", "ROUND(3.14159,2)", "basic", "number"),
        tc(
            "ROUND negative digits",
            "ROUND(123.456,-1)",
            "basic",
            "number",
        ),
        tc("ROUND zero", "ROUND(0,2)", "edge", "number"),
    ]);

    // ── ROUNDUP ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("ROUNDUP positive", "ROUNDUP(4.1,0)", "basic", "number"),
        tc("ROUNDUP negative", "ROUNDUP(-4.1,0)", "basic", "number"),
        tc(
            "ROUNDUP with decimals",
            "ROUNDUP(3.14159,3)",
            "basic",
            "number",
        ),
        tc(
            "ROUNDUP negative digits",
            "ROUNDUP(123.456,-2)",
            "basic",
            "number",
        ),
    ]);

    // ── ROUNDDOWN ─────────────────────────────────────────────────────────────
    cases.extend([
        tc("ROUNDDOWN positive", "ROUNDDOWN(4.9,0)", "basic", "number"),
        tc("ROUNDDOWN negative", "ROUNDDOWN(-4.9,0)", "basic", "number"),
        tc(
            "ROUNDDOWN with decimals",
            "ROUNDDOWN(3.14159,3)",
            "basic",
            "number",
        ),
        tc(
            "ROUNDDOWN negative digits",
            "ROUNDDOWN(123.456,-2)",
            "basic",
            "number",
        ),
    ]);

    // ── MROUND ────────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MROUND round to nearest 5",
            "MROUND(7,5)",
            "basic",
            "number",
        ),
        tc(
            "MROUND round to nearest 0.5",
            "MROUND(1.3,0.5)",
            "basic",
            "number",
        ),
        tc("MROUND negative", "MROUND(-7,-5)", "basic", "number"),
        tc("MROUND exact multiple", "MROUND(10,5)", "edge", "number"),
        tc("MROUND sign mismatch", "MROUND(7,-5)", "error", "error"),
    ]);

    // ── MOD ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("MOD positive", "MOD(10,3)", "basic", "number"),
        tc("MOD negative dividend", "MOD(-10,3)", "basic", "number"),
        tc("MOD negative divisor", "MOD(10,-3)", "basic", "number"),
        tc("MOD both negative", "MOD(-10,-3)", "basic", "number"),
        tc("MOD zero dividend", "MOD(0,3)", "edge", "number"),
        tc("MOD div by zero", "MOD(10,0)", "error", "error"),
        tc("MOD fractional", "MOD(5.5,2)", "basic", "number"),
    ]);

    // ── QUOTIENT ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("QUOTIENT positive", "QUOTIENT(10,3)", "basic", "number"),
        tc("QUOTIENT negative", "QUOTIENT(-10,3)", "basic", "number"),
        tc(
            "QUOTIENT both negative",
            "QUOTIENT(-10,-3)",
            "basic",
            "number",
        ),
        tc("QUOTIENT exact", "QUOTIENT(10,2)", "basic", "number"),
        tc("QUOTIENT div by zero", "QUOTIENT(10,0)", "error", "error"),
    ]);

    // ── SIGN ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SIGN positive", "SIGN(5)", "basic", "number"),
        tc("SIGN negative", "SIGN(-5)", "basic", "number"),
        tc("SIGN zero", "SIGN(0)", "basic", "number"),
        tc("SIGN boolean TRUE", "SIGN(TRUE)", "coercion", "number"),
        tc("SIGN boolean FALSE", "SIGN(FALSE)", "coercion", "number"),
    ]);

    // ── SQRT ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SQRT perfect square", "SQRT(9)", "basic", "number"),
        tc("SQRT non-perfect", "SQRT(2)", "basic", "number"),
        tc("SQRT zero", "SQRT(0)", "edge", "number"),
        tc("SQRT large", "SQRT(1E20)", "edge", "number"),
        tc("SQRT negative", "SQRT(-1)", "error", "error"),
        tc("SQRT boolean coercion", "SQRT(TRUE)", "coercion", "number"),
    ]);

    // ── SQRTPI ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SQRTPI one", "SQRTPI(1)", "basic", "number"),
        tc("SQRTPI four", "SQRTPI(4)", "basic", "number"),
        tc("SQRTPI zero", "SQRTPI(0)", "edge", "number"),
        tc("SQRTPI negative", "SQRTPI(-1)", "error", "error"),
    ]);

    // ── POWER ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("POWER integer exponent", "POWER(2,10)", "basic", "number"),
        tc(
            "POWER fractional exponent",
            "POWER(4,0.5)",
            "basic",
            "number",
        ),
        tc(
            "POWER negative base integer exp",
            "POWER(-2,3)",
            "basic",
            "number",
        ),
        tc("POWER zero base", "POWER(0,5)", "edge", "number"),
        tc("POWER zero exponent", "POWER(5,0)", "edge", "number"),
        tc(
            "POWER negative base fractional exp",
            "POWER(-2,0.5)",
            "error",
            "error",
        ),
        tc("POWER boolean base", "POWER(TRUE,3)", "coercion", "number"),
    ]);

    // ── EXP ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("EXP one", "EXP(1)", "basic", "number"),
        tc("EXP zero", "EXP(0)", "basic", "number"),
        tc("EXP negative", "EXP(-1)", "basic", "number"),
        tc("EXP large", "EXP(10)", "edge", "number"),
        tc("EXP boolean coercion", "EXP(TRUE)", "coercion", "number"),
    ]);

    // ── LOG ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LOG base 10", "LOG(100,10)", "basic", "number"),
        tc("LOG base 2", "LOG(8,2)", "basic", "number"),
        tc("LOG default base", "LOG(1000)", "basic", "number"),
        tc("LOG one", "LOG(1,10)", "edge", "number"),
        tc("LOG zero error", "LOG(0,10)", "error", "error"),
        tc("LOG negative error", "LOG(-1,10)", "error", "error"),
    ]);

    // ── LOG10 ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LOG10 100", "LOG10(100)", "basic", "number"),
        tc("LOG10 one", "LOG10(1)", "edge", "number"),
        tc("LOG10 fractional", "LOG10(0.01)", "basic", "number"),
        tc("LOG10 zero error", "LOG10(0)", "error", "error"),
    ]);

    // ── LN ────────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LN e", "LN(EXP(1))", "basic", "number"),
        tc("LN one", "LN(1)", "edge", "number"),
        tc("LN positive", "LN(10)", "basic", "number"),
        tc("LN zero error", "LN(0)", "error", "error"),
        tc("LN negative error", "LN(-1)", "error", "error"),
    ]);

    // ── SUM ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SUM scalars", "SUM(1,2,3)", "basic", "number"),
        tc("SUM array", "SUM({1,2,3,4,5})", "basic", "number"),
        tc("SUM negative", "SUM(-1,-2,-3)", "basic", "number"),
        tc("SUM mixed sign", "SUM(5,-3,2)", "basic", "number"),
        tc("SUM zero only", "SUM(0)", "edge", "number"),
        tc("SUM single element", "SUM(42)", "basic", "number"),
        tc(
            "SUM boolean coercion",
            "SUM(TRUE,FALSE,TRUE)",
            "coercion",
            "number",
        ),
        tc(
            "SUM numeric string coercion",
            r#"SUM("3","4")"#,
            "coercion",
            "number",
        ),
        tc(
            "SUM non-numeric text ignored",
            r#"SUM(1,"x",2)"#,
            "basic",
            "number",
        ),
        tc("SUM large values", "SUM(1E14,2E14)", "edge", "number"),
    ]);

    // ── SUMIF ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "SUMIF numeric criterion",
            r#"SUMIF({1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "SUMIF with sum range",
            r#"SUMIF({1,2,3},">1",{10,20,30})"#,
            "basic",
            "number",
        ),
        tc(
            "SUMIF text criterion",
            r#"SUMIF({"a","b","a"},"a",{1,2,3})"#,
            "basic",
            "number",
        ),
        tc(
            "SUMIF no match",
            r#"SUMIF({1,2,3},">100")"#,
            "edge",
            "number",
        ),
        tc(
            "SUMIF wildcard",
            r#"SUMIF({"ab","ac","bc"},"a*",{1,2,3})"#,
            "basic",
            "number",
        ),
        // additional battle-test cases
        tc("SUMIF larger range with sum range", r#"SUMIF({1,2,3,4,5},">3",{10,20,30,40,50})"#, "basic", "number"),
        tc("SUMIF equals criterion", r#"SUMIF({1,2,2,3},"=2",{10,20,30,40})"#, "basic", "number"),
        tc("SUMIF less than criterion", r#"SUMIF({1,2,3,4},"<3")"#, "basic", "number"),
        tc("SUMIF gte criterion", r#"SUMIF({1,2,3,4},">=3",{10,20,30,40})"#, "basic", "number"),
        tc("SUMIF single match", r#"SUMIF({5,10,15},"=10")"#, "edge", "number"),
        tc("SUMIF all match", r#"SUMIF({1,2,3},">0")"#, "edge", "number"),
        tc("SUMIF negative values", r#"SUMIF({-1,-2,3},"<0")"#, "edge", "number"),
        tc("SUMIF mismatched range sizes", r#"SUMIF({1,2,3},">1",{10,20})"#, "edge", "number"),
        tc("SUMIF exact text match", r#"SUMIF({"a","b","a"},"a")"#, "basic", "number"),
        tc("SUMIF question mark wildcard", r#"SUMIF({"ab","ac","bc"},"a?",{1,2,3})"#, "basic", "number"),
    ]);

    // ── SUMIFS (BUG-02) ───────────────────────────────────────────────────────
    cases.extend([
        tc(
            "SUMIFS inline array BUG-02",
            r#"SUMIFS({10,20,30},{1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "SUMIFS two conditions",
            r#"SUMIFS({10,20,30},{1,2,3},">1",{4,5,6},"<6")"#,
            "basic",
            "number",
        ),
        tc(
            "SUMIFS all match",
            r#"SUMIFS({1,2,3},{1,2,3},">=1")"#,
            "edge",
            "number",
        ),
        tc(
            "SUMIFS no match",
            r#"SUMIFS({1,2,3},{1,2,3},">100")"#,
            "edge",
            "number",
        ),
    ]);

    // ── SUMPRODUCT ────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "SUMPRODUCT two arrays",
            "SUMPRODUCT({1,2,3},{4,5,6})",
            "basic",
            "number",
        ),
        tc(
            "SUMPRODUCT single array",
            "SUMPRODUCT({1,2,3,4})",
            "basic",
            "number",
        ),
        tc(
            "SUMPRODUCT three arrays",
            "SUMPRODUCT({1,2},{3,4},{5,6})",
            "basic",
            "number",
        ),
        tc(
            "SUMPRODUCT with condition",
            "SUMPRODUCT(({1,2,3}>1)*{10,20,30})",
            "nested",
            "number",
        ),
        tc(
            "SUMPRODUCT mismatched sizes",
            "SUMPRODUCT({1,2},{3,4,5})",
            "error",
            "error",
        ),
    ]);

    // ── SUMSQ ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SUMSQ scalars", "SUMSQ(3,4)", "basic", "number"),
        tc("SUMSQ array", "SUMSQ({1,2,3})", "basic", "number"),
        tc("SUMSQ negative", "SUMSQ(-3,-4)", "basic", "number"),
        tc("SUMSQ single", "SUMSQ(5)", "basic", "number"),
    ]);

    // ── SUMX2MY2 ─────────────────────────────────────────────────────────────
    cases.extend([
        tc("SUMX2MY2 basic", "SUMX2MY2({2,3},{1,2})", "basic", "number"),
        tc(
            "SUMX2MY2 equal arrays",
            "SUMX2MY2({1,2,3},{1,2,3})",
            "edge",
            "number",
        ),
    ]);

    // ── SUMX2PY2 ─────────────────────────────────────────────────────────────
    cases.extend([
        tc("SUMX2PY2 basic", "SUMX2PY2({1,2},{3,4})", "basic", "number"),
        tc("SUMX2PY2 zeros", "SUMX2PY2({0,0},{0,0})", "edge", "number"),
    ]);

    // ── SUMXMY2 ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("SUMXMY2 basic", "SUMXMY2({2,4},{1,3})", "basic", "number"),
        tc("SUMXMY2 equal", "SUMXMY2({1,2},{1,2})", "edge", "number"),
    ]);

    // ── PRODUCT ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("PRODUCT scalars", "PRODUCT(2,3,4)", "basic", "number"),
        tc("PRODUCT array", "PRODUCT({2,3,4})", "basic", "number"),
        tc("PRODUCT with zero", "PRODUCT(2,0,4)", "edge", "number"),
        tc("PRODUCT negative", "PRODUCT(-2,3)", "basic", "number"),
        tc("PRODUCT single", "PRODUCT(7)", "basic", "number"),
        tc(
            "PRODUCT boolean coercion",
            "PRODUCT(TRUE,2,3)",
            "coercion",
            "number",
        ),
    ]);

    // ── AVERAGE ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("AVERAGE scalars", "AVERAGE(1,2,3)", "basic", "number"),
        tc("AVERAGE array", "AVERAGE({1,2,3,4,5})", "basic", "number"),
        tc("AVERAGE single", "AVERAGE(5)", "edge", "number"),
        tc("AVERAGE negative", "AVERAGE(-1,-2,-3)", "basic", "number"),
        tc("AVERAGE mixed", "AVERAGE(10,-5,3)", "basic", "number"),
        tc(
            "AVERAGE boolean coercion",
            "AVERAGE(TRUE,FALSE,TRUE)",
            "coercion",
            "number",
        ),
    ]);

    // ── AVERAGEIF ────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "AVERAGEIF numeric criterion",
            r#"AVERAGEIF({1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "AVERAGEIF with avg range",
            r#"AVERAGEIF({1,2,3},">1",{10,20,30})"#,
            "basic",
            "number",
        ),
        tc(
            "AVERAGEIF text criterion",
            r#"AVERAGEIF({"a","b","a"},"a",{1,3,5})"#,
            "basic",
            "number",
        ),
        tc(
            "AVERAGEIF no match error",
            r#"AVERAGEIF({1,2,3},">100")"#,
            "error",
            "error",
        ),
        // additional battle-test cases
        tc("AVERAGEIF larger range with avg range", r#"AVERAGEIF({10,20,30},">15",{10,20,30})"#, "basic", "number"),
        tc("AVERAGEIF equals criterion", r#"AVERAGEIF({1,2,2,3},"=2",{10,20,30,40})"#, "basic", "number"),
        tc("AVERAGEIF less than", r#"AVERAGEIF({1,2,3,4},"<3")"#, "basic", "number"),
        tc("AVERAGEIF gte criterion", r#"AVERAGEIF({1,2,3,4},">=3",{10,20,30,40})"#, "basic", "number"),
        tc("AVERAGEIF single match", r#"AVERAGEIF({5,10,15},"=10")"#, "edge", "number"),
        tc("AVERAGEIF all match", r#"AVERAGEIF({2,4,6},">0")"#, "edge", "number"),
        tc("AVERAGEIF negative values", r#"AVERAGEIF({-1,-2,3},"<0")"#, "edge", "number"),
        tc("AVERAGEIF question mark wildcard", r#"AVERAGEIF({"ab","ac","bc"},"a?",{1,2,3})"#, "basic", "number"),
        tc("AVERAGEIF single element match", r#"AVERAGEIF({42},"=42")"#, "edge", "number"),
    ]);

    // ── AVERAGEIFS (BUG-02) ───────────────────────────────────────────────────
    cases.extend([
        tc(
            "AVERAGEIFS inline array BUG-02",
            r#"AVERAGEIFS({10,20,30},{1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "AVERAGEIFS two conditions",
            r#"AVERAGEIFS({10,20,30},{1,2,3},">1",{4,5,6},"<6")"#,
            "basic",
            "number",
        ),
        tc(
            "AVERAGEIFS all match",
            r#"AVERAGEIFS({1,2,3},{1,2,3},">=1")"#,
            "edge",
            "number",
        ),
        tc(
            "AVERAGEIFS no match error",
            r#"AVERAGEIFS({1,2,3},{1,2,3},">100")"#,
            "error",
            "error",
        ),
    ]);

    // ── MAXIFS (BUG-02) ───────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MAXIFS inline array BUG-02",
            r#"MAXIFS({10,20,30},{1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "MAXIFS two conditions",
            r#"MAXIFS({10,20,30},{1,2,3},">1",{4,5,6},"<6")"#,
            "basic",
            "number",
        ),
        tc(
            "MAXIFS all match",
            r#"MAXIFS({5,10,15},{1,2,3},">=1")"#,
            "edge",
            "number",
        ),
    ]);

    // ── MINIFS (BUG-02) ───────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MINIFS inline array BUG-02",
            r#"MINIFS({10,20,30},{1,2,3},">1")"#,
            "basic",
            "number",
        ),
        tc(
            "MINIFS two conditions",
            r#"MINIFS({10,20,30},{1,2,3},">1",{4,5,6},"<6")"#,
            "basic",
            "number",
        ),
        tc(
            "MINIFS all match",
            r#"MINIFS({5,10,15},{1,2,3},">=1")"#,
            "edge",
            "number",
        ),
    ]);

    // ── PI ────────────────────────────────────────────────────────────────────
    cases.extend([
        tc("PI value", "PI()", "basic", "number"),
        tc("PI in expression", "PI()*2", "nested", "number"),
    ]);

    // ── TAU ───────────────────────────────────────────────────────────────────
    cases.extend([tc("TAU value", "TAU()", "basic", "number")]);

    // ── DEGREES ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("DEGREES pi radians", "DEGREES(PI())", "basic", "number"),
        tc("DEGREES zero", "DEGREES(0)", "edge", "number"),
        tc("DEGREES pi/2", "DEGREES(PI()/2)", "basic", "number"),
        tc("DEGREES negative", "DEGREES(-PI())", "basic", "number"),
    ]);

    // ── RADIANS ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("RADIANS 180", "RADIANS(180)", "basic", "number"),
        tc("RADIANS 0", "RADIANS(0)", "edge", "number"),
        tc("RADIANS 90", "RADIANS(90)", "basic", "number"),
        tc("RADIANS negative", "RADIANS(-180)", "basic", "number"),
    ]);

    // ── SIN ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SIN zero", "SIN(0)", "basic", "number"),
        tc("SIN pi/2", "SIN(PI()/2)", "basic", "number"),
        tc("SIN pi", "SIN(PI())", "edge", "number"),
        tc("SIN negative", "SIN(-PI()/2)", "basic", "number"),
    ]);

    // ── COS ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("COS zero", "COS(0)", "basic", "number"),
        tc("COS pi", "COS(PI())", "basic", "number"),
        tc("COS pi/2", "COS(PI()/2)", "edge", "number"),
        tc("COS negative", "COS(-PI())", "basic", "number"),
    ]);

    // ── TAN ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("TAN zero", "TAN(0)", "basic", "number"),
        tc("TAN pi/4", "TAN(PI()/4)", "basic", "number"),
        tc("TAN pi", "TAN(PI())", "edge", "number"),
        tc("TAN negative", "TAN(-PI()/4)", "basic", "number"),
    ]);

    // ── ASIN ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ASIN zero", "ASIN(0)", "basic", "number"),
        tc("ASIN one", "ASIN(1)", "basic", "number"),
        tc("ASIN negative one", "ASIN(-1)", "basic", "number"),
        tc("ASIN half", "ASIN(0.5)", "basic", "number"),
        tc("ASIN out of range", "ASIN(2)", "error", "error"),
    ]);

    // ── ACOS ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ACOS one", "ACOS(1)", "basic", "number"),
        tc("ACOS zero", "ACOS(0)", "basic", "number"),
        tc("ACOS negative one", "ACOS(-1)", "basic", "number"),
        tc("ACOS out of range", "ACOS(2)", "error", "error"),
    ]);

    // ── ATAN ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ATAN zero", "ATAN(0)", "basic", "number"),
        tc("ATAN one", "ATAN(1)", "basic", "number"),
        tc("ATAN negative", "ATAN(-1)", "basic", "number"),
        tc("ATAN large", "ATAN(1E10)", "edge", "number"),
    ]);

    // ── ATAN2 ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ATAN2 first quadrant", "ATAN2(1,1)", "basic", "number"),
        tc("ATAN2 second quadrant", "ATAN2(-1,1)", "basic", "number"),
        tc("ATAN2 positive x-axis", "ATAN2(1,0)", "basic", "number"),
        tc("ATAN2 origin error", "ATAN2(0,0)", "error", "error"),
        tc("ATAN2 negative x", "ATAN2(-1,0)", "basic", "number"),
    ]);

    // ── SINH ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("SINH zero", "SINH(0)", "basic", "number"),
        tc("SINH one", "SINH(1)", "basic", "number"),
        tc("SINH negative", "SINH(-1)", "basic", "number"),
    ]);

    // ── COSH ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("COSH zero", "COSH(0)", "basic", "number"),
        tc("COSH one", "COSH(1)", "basic", "number"),
        tc("COSH negative", "COSH(-1)", "basic", "number"),
    ]);

    // ── TANH ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("TANH zero", "TANH(0)", "basic", "number"),
        tc("TANH one", "TANH(1)", "basic", "number"),
        tc("TANH large", "TANH(100)", "edge", "number"),
    ]);

    // ── ASINH ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ASINH zero", "ASINH(0)", "basic", "number"),
        tc("ASINH one", "ASINH(1)", "basic", "number"),
        tc("ASINH negative", "ASINH(-1)", "basic", "number"),
    ]);

    // ── ACOSH ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ACOSH one", "ACOSH(1)", "basic", "number"),
        tc("ACOSH two", "ACOSH(2)", "basic", "number"),
        tc("ACOSH below range error", "ACOSH(0)", "error", "error"),
    ]);

    // ── ATANH ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ATANH zero", "ATANH(0)", "basic", "number"),
        tc("ATANH half", "ATANH(0.5)", "basic", "number"),
        tc("ATANH one error", "ATANH(1)", "error", "error"),
        tc("ATANH negative half", "ATANH(-0.5)", "basic", "number"),
    ]);

    // ── MMULT ────────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MMULT row vector by col vector",
            "MMULT({1,2,3},{1;2;3})",
            "basic",
            "number",
        ),
        tc(
            "MMULT incompatible dimensions",
            "MMULT({1,2},{3,4})",
            "error",
            "error",
        ),
    ]);

    // ── MDETERM ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("MDETERM 2x2", "MDETERM({1,2;3,4})", "basic", "number"),
        tc(
            "MDETERM 3x3 identity",
            "MDETERM({1,0,0;0,1,0;0,0,1})",
            "basic",
            "number",
        ),
        tc("MDETERM singular", "MDETERM({1,2;2,4})", "edge", "number"),
        tc(
            "MDETERM non-square error",
            "MDETERM({1,2,3;4,5,6})",
            "error",
            "error",
        ),
    ]);

    // ── MINVERSE ─────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MINVERSE singular error",
            "MINVERSE({1,2;2,4})",
            "error",
            "error",
        ),
    ]);

    // ── SEQUENCE ─────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "SEQUENCE single cell",
            "SEQUENCE(1,1,5,1)",
            "edge",
            "number",
        ),
    ]);

    // ── AGGREGATE ────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "AGGREGATE sum (9)",
            "AGGREGATE(9,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE average (1)",
            "AGGREGATE(1,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE max (4)",
            "AGGREGATE(4,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE min (5)",
            "AGGREGATE(5,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE count (2)",
            "AGGREGATE(2,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE stdev (7)",
            "AGGREGATE(7,0,{1,2,3,4,5})",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE large (14)",
            "AGGREGATE(14,0,{5,3,1,4,2},2)",
            "basic",
            "number",
        ),
        tc(
            "AGGREGATE small (15)",
            "AGGREGATE(15,0,{5,3,1,4,2},2)",
            "basic",
            "number",
        ),
    ]);

    // ── SUBTOTAL (BUG-01) ─────────────────────────────────────────────────────
    cases.extend([
        tc(
            "SUBTOTAL SUM (9) BUG-01",
            "SUBTOTAL(9,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL AVERAGE (1) BUG-01",
            "SUBTOTAL(1,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL COUNT (2) BUG-01",
            "SUBTOTAL(2,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL STDEV (4) BUG-01",
            "SUBTOTAL(4,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL VAR (5) BUG-01",
            "SUBTOTAL(5,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL SUM hidden (109) BUG-01",
            "SUBTOTAL(109,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL MAX (4)",
            "SUBTOTAL(4,{5,3,1,4,2})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL MIN (5)",
            "SUBTOTAL(5,{5,3,1,4,2})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL invalid func_num error",
            "SUBTOTAL(0,{1,2,3})",
            "error",
            "error",
        ),
    ]);

    // ── ROMAN ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ROMAN 4", "ROMAN(4)", "basic", "string"),
        tc("ROMAN 14", "ROMAN(14)", "basic", "string"),
        tc("ROMAN 1999", "ROMAN(1999)", "basic", "string"),
        tc("ROMAN classic style", "ROMAN(1,0)", "basic", "string"),
        tc("ROMAN 3999", "ROMAN(3999)", "edge", "string"),
        tc("ROMAN zero", "ROMAN(0)", "edge", "string"),
        tc("ROMAN negative error", "ROMAN(-1)", "error", "error"),
        tc("ROMAN too large error", "ROMAN(4000)", "error", "error"),
    ]);

    // ── ARABIC ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("ARABIC IV", r#"ARABIC("IV")"#, "basic", "number"),
        tc("ARABIC XIV", r#"ARABIC("XIV")"#, "basic", "number"),
        tc("ARABIC MCMXCIX", r#"ARABIC("MCMXCIX")"#, "basic", "number"),
        tc("ARABIC empty string", r#"ARABIC("")"#, "edge", "number"),
        tc("ARABIC lowercase", r#"ARABIC("iv")"#, "basic", "number"),
    ]);

    // ── BASE ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("BASE decimal to binary", "BASE(10,2)", "basic", "string"),
        tc("BASE decimal to hex", "BASE(255,16)", "basic", "string"),
        tc("BASE with min length", "BASE(10,2,8)", "basic", "string"),
        tc("BASE zero", "BASE(0,2)", "edge", "string"),
        tc("BASE radix 2 boundary", "BASE(1,2)", "basic", "string"),
        tc("BASE invalid radix error", "BASE(10,1)", "error", "error"),
    ]);

    // ── DECIMAL ──────────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "DECIMAL binary to decimal",
            r#"DECIMAL("1010",2)"#,
            "basic",
            "number",
        ),
        tc(
            "DECIMAL hex to decimal",
            r#"DECIMAL("FF",16)"#,
            "basic",
            "number",
        ),
        tc("DECIMAL octal", r#"DECIMAL("17",8)"#, "basic", "number"),
        tc("DECIMAL zero", r#"DECIMAL("0",2)"#, "edge", "number"),
        tc(
            "DECIMAL invalid digit error",
            r#"DECIMAL("2",2)"#,
            "error",
            "error",
        ),
    ]);

    // ── COMBIN ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("COMBIN basic", "COMBIN(5,2)", "basic", "number"),
        tc("COMBIN n equal k", "COMBIN(5,5)", "edge", "number"),
        tc("COMBIN k zero", "COMBIN(5,0)", "edge", "number"),
        tc("COMBIN large", "COMBIN(20,10)", "basic", "number"),
        tc("COMBIN k gt n error", "COMBIN(3,5)", "error", "error"),
    ]);

    // ── COMBINA ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("COMBINA basic", "COMBINA(5,2)", "basic", "number"),
        tc("COMBINA k zero", "COMBINA(5,0)", "edge", "number"),
        tc("COMBINA both zero", "COMBINA(0,0)", "edge", "number"),
    ]);

    // ── PERMUT ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("PERMUT basic", "PERMUT(5,2)", "basic", "number"),
        tc("PERMUT k equals n", "PERMUT(5,5)", "edge", "number"),
        tc("PERMUT k zero", "PERMUT(5,0)", "edge", "number"),
        tc("PERMUT k gt n error", "PERMUT(3,5)", "error", "error"),
    ]);

    // ── PERMUTA ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("PERMUTA basic", "PERMUTA(5,2)", "basic", "number"),
        tc("PERMUTA k zero", "PERMUTA(5,0)", "edge", "number"),
        tc("PERMUTA both zero", "PERMUTA(0,0)", "edge", "number"),
    ]);

    // ── FACT ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("FACT zero", "FACT(0)", "edge", "number"),
        tc("FACT one", "FACT(1)", "basic", "number"),
        tc("FACT five", "FACT(5)", "basic", "number"),
        tc("FACT ten", "FACT(10)", "basic", "number"),
        tc("FACT negative error", "FACT(-1)", "error", "error"),
        tc("FACT fractional truncates", "FACT(4.9)", "edge", "number"),
    ]);

    // ── FACTDOUBLE ───────────────────────────────────────────────────────────
    cases.extend([
        tc("FACTDOUBLE zero", "FACTDOUBLE(0)", "edge", "number"),
        tc("FACTDOUBLE one", "FACTDOUBLE(1)", "basic", "number"),
        tc("FACTDOUBLE six", "FACTDOUBLE(6)", "basic", "number"),
        tc("FACTDOUBLE seven", "FACTDOUBLE(7)", "basic", "number"),
        tc(
            "FACTDOUBLE negative one",
            "FACTDOUBLE(-1)",
            "edge",
            "number",
        ),
        tc(
            "FACTDOUBLE negative two error",
            "FACTDOUBLE(-2)",
            "error",
            "error",
        ),
    ]);

    // ── MULTINOMIAL ──────────────────────────────────────────────────────────
    cases.extend([
        tc(
            "MULTINOMIAL two args",
            "MULTINOMIAL(2,3)",
            "basic",
            "number",
        ),
        tc(
            "MULTINOMIAL three args",
            "MULTINOMIAL(1,2,3)",
            "basic",
            "number",
        ),
        tc("MULTINOMIAL single arg", "MULTINOMIAL(5)", "edge", "number"),
        tc(
            "MULTINOMIAL with zero",
            "MULTINOMIAL(0,5)",
            "edge",
            "number",
        ),
    ]);

    // ── GCD ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("GCD two numbers", "GCD(12,8)", "basic", "number"),
        tc("GCD coprime", "GCD(7,13)", "basic", "number"),
        tc("GCD with zero", "GCD(12,0)", "edge", "number"),
        tc("GCD multiple args", "GCD(12,8,6)", "basic", "number"),
        tc("GCD equal", "GCD(5,5)", "edge", "number"),
        tc("GCD negative error", "GCD(-12,8)", "error", "error"),
    ]);

    // ── LCM ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LCM two numbers", "LCM(4,6)", "basic", "number"),
        tc("LCM coprime", "LCM(7,13)", "basic", "number"),
        tc("LCM with one", "LCM(5,1)", "edge", "number"),
        tc("LCM multiple args", "LCM(4,6,10)", "basic", "number"),
        tc("LCM negative error", "LCM(-4,6)", "error", "error"),
    ]);

    // ── ISBETWEEN (operator) ──────────────────────────────────────────────────
    cases.extend([
        tc("ISBETWEEN in range", "ISBETWEEN(3,1,5)", "basic", "boolean"),
        tc(
            "ISBETWEEN at lower bound",
            "ISBETWEEN(1,1,5)",
            "edge",
            "boolean",
        ),
        tc(
            "ISBETWEEN at upper bound",
            "ISBETWEEN(5,1,5)",
            "edge",
            "boolean",
        ),
        tc(
            "ISBETWEEN below range",
            "ISBETWEEN(0,1,5)",
            "basic",
            "boolean",
        ),
        tc(
            "ISBETWEEN above range",
            "ISBETWEEN(6,1,5)",
            "basic",
            "boolean",
        ),
        tc(
            "ISBETWEEN exclusive lower",
            "ISBETWEEN(1,1,5,FALSE,TRUE)",
            "edge",
            "boolean",
        ),
        tc(
            "ISBETWEEN exclusive upper",
            "ISBETWEEN(5,1,5,TRUE,FALSE)",
            "edge",
            "boolean",
        ),
        tc(
            "ISBETWEEN text values",
            r#"ISBETWEEN("b","a","c")"#,
            "basic",
            "boolean",
        ),
    ]);

    // ── Nested / composition cases ────────────────────────────────────────────
    cases.extend([
        tc("ROUND of SQRT", "ROUND(SQRT(2),4)", "nested", "number"),
        tc("ABS of MOD", "ABS(MOD(-7,3))", "nested", "number"),
        tc("POWER of SQRT", "POWER(SQRT(9),2)", "nested", "number"),
        tc("LOG10 of POWER", "LOG10(POWER(10,3))", "nested", "number"),
        tc(
            "SUM of SUMSQ",
            "SUM(SUMSQ(3,4),SUMSQ(5,12))",
            "nested",
            "number",
        ),
        tc(
            "PRODUCT of FACT",
            "PRODUCT(FACT(3),FACT(4))",
            "nested",
            "number",
        ),
        tc("INT of SQRT", "INT(SQRT(10))", "nested", "number"),
        tc("MOD of ROUND", "MOD(ROUND(7.6,0),3)", "nested", "number"),
        tc(
            "GCD nested SUM",
            "GCD(SUM(6,6),SUM(4,4))",
            "nested",
            "number",
        ),
        tc(
            "LCM nested PRODUCT",
            "LCM(PRODUCT(2,3),PRODUCT(2,5))",
            "nested",
            "number",
        ),
    ]);

    // ── Additional ABS/SIGN/ROUND edge cases ──────────────────────────────────
    cases.extend([
        tc("ABS very small", "ABS(-0.000001)", "edge", "number"),
        tc(
            "SIGN very small positive",
            "SIGN(0.000001)",
            "edge",
            "number",
        ),
        tc(
            "SIGN very small negative",
            "SIGN(-0.000001)",
            "edge",
            "number",
        ),
        tc(
            "ROUND tie-break at 2 decimal",
            "ROUND(2.555,2)",
            "edge",
            "number",
        ),
        tc(
            "ROUND negative tie-break",
            "ROUND(-2.5,0)",
            "edge",
            "number",
        ),
        tc("ROUNDUP already integer", "ROUNDUP(5,0)", "edge", "number"),
        tc(
            "ROUNDDOWN already integer",
            "ROUNDDOWN(5,0)",
            "edge",
            "number",
        ),
        tc("TRUNC negative digits", "TRUNC(1234,-2)", "edge", "number"),
    ]);

    // ── Additional SUM / SUMPRODUCT coercion and edge cases ───────────────────
    cases.extend([
        tc("SUM empty array elements", "SUM({0,0,0})", "edge", "number"),
        tc(
            "SUMPRODUCT boolean array",
            "SUMPRODUCT({TRUE,FALSE,TRUE}*{1,2,3})",
            "coercion",
            "number",
        ),
        tc("SUMSQ single zero", "SUMSQ(0)", "edge", "number"),
        tc("PRODUCT single one", "PRODUCT(1)", "edge", "number"),
        tc("AVERAGE two elements", "AVERAGE(2,8)", "basic", "number"),
    ]);

    // ── Additional trig edge cases ────────────────────────────────────────────
    cases.extend([
        tc("SIN two pi", "SIN(2*PI())", "edge", "number"),
        tc("COS two pi", "COS(2*PI())", "edge", "number"),
        tc("TAN pi/3", "TAN(PI()/3)", "basic", "number"),
        tc("ASIN inverse of SIN", "ASIN(SIN(0.5))", "nested", "number"),
        tc("ACOS inverse of COS", "ACOS(COS(0.5))", "nested", "number"),
        tc("ATAN inverse of TAN", "ATAN(TAN(0.5))", "nested", "number"),
        tc("SINH negative large", "SINH(-5)", "edge", "number"),
        tc("COSH large", "COSH(10)", "edge", "number"),
        tc("TANH negative one", "TANH(-1)", "basic", "number"),
        tc("ASINH large", "ASINH(100)", "edge", "number"),
        tc("ACOSH large", "ACOSH(100)", "edge", "number"),
        tc("ATANH negative half", "ATANH(-0.5)", "basic", "number"),
    ]);

    // ── Additional LOG / EXP edge cases ──────────────────────────────────────
    cases.extend([
        tc("LN of EXP roundtrip", "LN(EXP(5))", "nested", "number"),
        tc("EXP of LN roundtrip", "EXP(LN(5))", "nested", "number"),
        tc("LOG base e", "LOG(EXP(1),EXP(1))", "nested", "number"),
        tc("LOG10 of 1E-3", "LOG10(0.001)", "basic", "number"),
        tc("LOG10 of 1E6", "LOG10(1000000)", "basic", "number"),
    ]);

    // ── Additional SEQUENCE and MMULT edge cases ──────────────────────────────
    cases.extend([
        tc("MDETERM 1x1", "MDETERM({5})", "edge", "number"),
        tc("MINVERSE 1x1", "MINVERSE({4})", "edge", "array"),
    ]);

    // ── Additional COMBIN / PERMUT large values ───────────────────────────────
    cases.extend([
        tc("COMBIN large n", "COMBIN(52,5)", "basic", "number"),
        tc("PERMUT large n", "PERMUT(10,4)", "basic", "number"),
        tc("FACT large", "FACT(15)", "basic", "number"),
        tc("FACTDOUBLE large even", "FACTDOUBLE(10)", "basic", "number"),
        tc("FACTDOUBLE large odd", "FACTDOUBLE(9)", "basic", "number"),
        tc(
            "MULTINOMIAL four args",
            "MULTINOMIAL(1,1,1,1)",
            "basic",
            "number",
        ),
    ]);

    // ── Additional GCD / LCM edge cases ──────────────────────────────────────
    cases.extend([
        tc("GCD single value", "GCD(12)", "edge", "number"),
        tc("LCM single value", "LCM(7)", "edge", "number"),
        tc("GCD large values", "GCD(1000000,999999)", "edge", "number"),
        tc("LCM large values", "LCM(12,18)", "basic", "number"),
    ]);

    // ── Additional ROMAN / ARABIC edge cases ─────────────────────────────────
    cases.extend([
        tc("ROMAN 499 simplified", "ROMAN(499,1)", "basic", "string"),
        tc(
            "ROMAN 499 very simplified",
            "ROMAN(499,4)",
            "basic",
            "string",
        ),
        tc("ARABIC III", r#"ARABIC("III")"#, "basic", "number"),
        tc("ARABIC MMXXIV", r#"ARABIC("MMXXIV")"#, "basic", "number"),
    ]);

    // ── Additional BASE / DECIMAL edge cases ─────────────────────────────────
    cases.extend([
        tc("BASE octal", "BASE(8,8)", "basic", "string"),
        tc("BASE base 36", "BASE(35,36)", "basic", "string"),
        tc("DECIMAL base 36", r#"DECIMAL("Z",36)"#, "basic", "number"),
        tc(
            "BASE large with padding",
            "BASE(255,2,12)",
            "basic",
            "string",
        ),
    ]);

    // ── Additional SUBTOTAL function codes ───────────────────────────────────
    cases.extend([
        tc(
            "SUBTOTAL COUNTA (3)",
            "SUBTOTAL(3,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL MAX (4) scalars",
            "SUBTOTAL(4,{10,20,30})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL MIN (5) scalars",
            "SUBTOTAL(5,{10,20,30})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL PRODUCT (6)",
            "SUBTOTAL(6,{1,2,3,4})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL STDEVP (7)",
            "SUBTOTAL(7,{2,4,6})",
            "basic",
            "number",
        ),
        tc("SUBTOTAL VAR (8)", "SUBTOTAL(8,{2,4,6})", "basic", "number"),
        tc(
            "SUBTOTAL VARP (9 hidden)",
            "SUBTOTAL(9,{2,4,6})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL COUNTA hidden (103)",
            "SUBTOTAL(103,{1,2,3})",
            "basic",
            "number",
        ),
        tc(
            "SUBTOTAL COUNT hidden (102)",
            "SUBTOTAL(102,{1,2,3})",
            "basic",
            "number",
        ),
    ]);

    cases
}
