use crate::types::{Platform, TestCase};

fn tc(description: &str, formula: &str, test_category: &str, expected_type: &str) -> TestCase {
    TestCase::new(description, formula, "", test_category, expected_type)
}

pub fn generate_operator(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── ADD ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ADD integers", "ADD(3,4)", "basic", "number"),
        tc("ADD decimals", "ADD(1.5,2.5)", "basic", "number"),
        tc("ADD negative", "ADD(-5,3)", "basic", "number"),
        tc("ADD both zero", "ADD(0,0)", "basic", "number"),
        tc("ADD large numbers", "ADD(1E+15,1E+15)", "edge", "number"),
        tc("ADD TRUE coerces to 1", "ADD(TRUE,1)", "coercion", "number"),
        tc("ADD text number coercion", r#"ADD("5",3)"#, "coercion", "number"),
        tc("ADD text abc -> #VALUE!", r#"ADD("abc",1)"#, "error", "error"),
        tc("ADD #DIV/0! propagates", "ADD(1/0,1)", "error", "error"),
        tc("ADD no args -> error", "ADD()", "error", "error"),
    ]);

    // ── MINUS ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("MINUS integers", "MINUS(10,3)", "basic", "number"),
        tc("MINUS decimals", "MINUS(5.5,2.5)", "basic", "number"),
        tc("MINUS negative result", "MINUS(3,10)", "basic", "number"),
        tc("MINUS both zero", "MINUS(0,0)", "basic", "number"),
        tc("MINUS large numbers", "MINUS(1E+15,1E+14)", "edge", "number"),
        tc("MINUS TRUE coerces to 1", "MINUS(TRUE,1)", "coercion", "number"),
        tc("MINUS text number", r#"MINUS("10",3)"#, "coercion", "number"),
        tc("MINUS text abc -> #VALUE!", r#"MINUS("abc",1)"#, "error", "error"),
        tc("MINUS #N/A propagates", "MINUS(NA(),1)", "error", "error"),
        tc("MINUS no args -> error", "MINUS()", "error", "error"),
    ]);

    // ── MULTIPLY ─────────────────────────────────────────────────────────────
    cases.extend([
        tc("MULTIPLY integers", "MULTIPLY(3,4)", "basic", "number"),
        tc("MULTIPLY decimals", "MULTIPLY(1.5,2.0)", "basic", "number"),
        tc("MULTIPLY by zero", "MULTIPLY(5,0)", "basic", "number"),
        tc("MULTIPLY negatives", "MULTIPLY(-3,-4)", "basic", "number"),
        tc("MULTIPLY large numbers", "MULTIPLY(1E+7,1E+7)", "edge", "number"),
        tc("MULTIPLY TRUE coercion", "MULTIPLY(TRUE,5)", "coercion", "number"),
        tc("MULTIPLY text number", r#"MULTIPLY("3",4)"#, "coercion", "number"),
        tc("MULTIPLY text -> #VALUE!", r#"MULTIPLY("abc",1)"#, "error", "error"),
        tc("MULTIPLY #DIV/0! propagates", "MULTIPLY(1/0,2)", "error", "error"),
        tc("MULTIPLY no args -> error", "MULTIPLY()", "error", "error"),
    ]);

    // ── DIVIDE ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("DIVIDE integers", "DIVIDE(10,2)", "basic", "number"),
        tc("DIVIDE fractional result", "DIVIDE(1,3)", "basic", "number"),
        tc("DIVIDE zero dividend", "DIVIDE(0,5)", "basic", "number"),
        tc("DIVIDE negative dividend", "DIVIDE(-10,2)", "basic", "number"),
        tc("DIVIDE both negative", "DIVIDE(-10,-2)", "basic", "number"),
        tc("DIVIDE by zero -> #DIV/0!", "DIVIDE(5,0)", "error", "error"),
        tc("DIVIDE TRUE coercion", "DIVIDE(TRUE,2)", "coercion", "number"),
        tc("DIVIDE text number", r#"DIVIDE("10",2)"#, "coercion", "number"),
        tc("DIVIDE text -> #VALUE!", r#"DIVIDE("abc",2)"#, "error", "error"),
        tc("DIVIDE no args -> error", "DIVIDE()", "error", "error"),
    ]);

    // ── EQ ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("EQ equal integers -> TRUE", "EQ(1,1)", "basic", "boolean"),
        tc("EQ unequal integers -> FALSE", "EQ(1,2)", "basic", "boolean"),
        tc("EQ equal strings -> TRUE", r#"EQ("abc","abc")"#, "basic", "boolean"),
        tc("EQ case-insensitive match", r#"EQ("abc","ABC")"#, "basic", "boolean"),
        tc("EQ equal decimals", "EQ(3.14,3.14)", "basic", "boolean"),
        tc("EQ zero equals zero", "EQ(0,0)", "basic", "boolean"),
        tc("EQ number vs text -> FALSE", r#"EQ(1,"1")"#, "coercion", "boolean"),
        tc("EQ #N/A propagates", "EQ(NA(),1)", "error", "error"),
        tc("EQ no args -> error", "EQ()", "error", "error"),
        tc("EQ nested formula args", "EQ(ABS(-5),5)", "nested", "boolean"),
    ]);

    // ── GT ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("GT greater -> TRUE", "GT(5,3)", "basic", "boolean"),
        tc("GT equal -> FALSE", "GT(3,3)", "basic", "boolean"),
        tc("GT less -> FALSE", "GT(2,5)", "basic", "boolean"),
        tc("GT negative comparison", "GT(-1,-5)", "basic", "boolean"),
        tc("GT decimals", "GT(3.14,3.13)", "basic", "boolean"),
        tc("GT TRUE coerces to 1", "GT(TRUE,0)", "coercion", "boolean"),
        tc("GT text comparison", r#"GT("b","a")"#, "basic", "boolean"),
        tc("GT #N/A propagates", "GT(NA(),1)", "error", "error"),
        tc("GT no args -> error", "GT()", "error", "error"),
        tc("GT nested formula args", "GT(SUM(2,3),4)", "nested", "boolean"),
    ]);

    // ── GTE ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("GTE greater -> TRUE", "GTE(5,3)", "basic", "boolean"),
        tc("GTE equal -> TRUE", "GTE(3,3)", "basic", "boolean"),
        tc("GTE less -> FALSE", "GTE(2,5)", "basic", "boolean"),
        tc("GTE negative equal", "GTE(-3,-3)", "basic", "boolean"),
        tc("GTE decimals equal", "GTE(1.5,1.5)", "basic", "boolean"),
        tc("GTE TRUE coerces to 1", "GTE(TRUE,1)", "coercion", "boolean"),
        tc("GTE text comparison", r#"GTE("b","a")"#, "basic", "boolean"),
        tc("GTE #N/A propagates", "GTE(NA(),1)", "error", "error"),
        tc("GTE no args -> error", "GTE()", "error", "error"),
        tc("GTE nested formula args", "GTE(ABS(-4),4)", "nested", "boolean"),
    ]);

    // ── LT ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LT less -> TRUE", "LT(3,5)", "basic", "boolean"),
        tc("LT equal -> FALSE", "LT(3,3)", "basic", "boolean"),
        tc("LT greater -> FALSE", "LT(5,2)", "basic", "boolean"),
        tc("LT negative comparison", "LT(-5,-1)", "basic", "boolean"),
        tc("LT decimals", "LT(3.13,3.14)", "basic", "boolean"),
        tc("LT FALSE coerces to 0", "LT(FALSE,1)", "coercion", "boolean"),
        tc("LT text comparison", r#"LT("a","b")"#, "basic", "boolean"),
        tc("LT #N/A propagates", "LT(NA(),1)", "error", "error"),
        tc("LT no args -> error", "LT()", "error", "error"),
        tc("LT nested formula args", "LT(MINUS(10,8),3)", "nested", "boolean"),
    ]);

    // ── LTE ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LTE less -> TRUE", "LTE(3,5)", "basic", "boolean"),
        tc("LTE equal -> TRUE", "LTE(3,3)", "basic", "boolean"),
        tc("LTE greater -> FALSE", "LTE(5,2)", "basic", "boolean"),
        tc("LTE negative equal", "LTE(-3,-3)", "basic", "boolean"),
        tc("LTE decimals equal", "LTE(1.5,1.5)", "basic", "boolean"),
        tc("LTE FALSE coerces to 0", "LTE(FALSE,0)", "coercion", "boolean"),
        tc("LTE text comparison", r#"LTE("a","b")"#, "basic", "boolean"),
        tc("LTE #N/A propagates", "LTE(NA(),1)", "error", "error"),
        tc("LTE no args -> error", "LTE()", "error", "error"),
        tc("LTE nested formula args", "LTE(ADD(1,2),3)", "nested", "boolean"),
    ]);

    // ── NE ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("NE unequal -> TRUE", "NE(1,2)", "basic", "boolean"),
        tc("NE equal -> FALSE", "NE(1,1)", "basic", "boolean"),
        tc("NE strings unequal -> TRUE", r#"NE("abc","xyz")"#, "basic", "boolean"),
        tc("NE strings equal -> FALSE", r#"NE("abc","abc")"#, "basic", "boolean"),
        tc("NE case-insensitive equal", r#"NE("abc","ABC")"#, "basic", "boolean"),
        tc("NE number vs text -> TRUE", r#"NE(1,"1")"#, "coercion", "boolean"),
        tc("NE TRUE vs FALSE", "NE(TRUE,FALSE)", "basic", "boolean"),
        tc("NE #N/A propagates", "NE(NA(),1)", "error", "error"),
        tc("NE no args -> error", "NE()", "error", "error"),
        tc("NE nested formula args", "NE(ABS(-3),3)", "nested", "boolean"),
    ]);

    // ── POW ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("POW 2^3 -> 8", "POW(2,3)", "basic", "number"),
        tc("POW 5^0 -> 1", "POW(5,0)", "basic", "number"),
        tc("POW 0^5 -> 0", "POW(0,5)", "basic", "number"),
        tc("POW decimal exponent", "POW(4,0.5)", "basic", "number"),
        tc("POW negative base even exp", "POW(-2,2)", "basic", "number"),
        tc("POW negative base odd exp", "POW(-2,3)", "basic", "number"),
        tc("POW large", "POW(10,10)", "edge", "number"),
        tc("POW 0^0 -> 1", "POW(0,0)", "edge", "number"),
        tc("POW #N/A propagates", "POW(NA(),2)", "error", "error"),
        tc("POW no args -> error", "POW()", "error", "error"),
    ]);

    // ── UMINUS ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("UMINUS positive -> negative", "UMINUS(5)", "basic", "number"),
        tc("UMINUS negative -> positive", "UMINUS(-5)", "basic", "number"),
        tc("UMINUS zero -> zero", "UMINUS(0)", "basic", "number"),
        tc("UMINUS decimal", "UMINUS(3.14)", "basic", "number"),
        tc("UMINUS large number", "UMINUS(1E+15)", "edge", "number"),
        tc("UMINUS TRUE coerces to 1", "UMINUS(TRUE)", "coercion", "number"),
        tc("UMINUS FALSE coerces to 0", "UMINUS(FALSE)", "coercion", "number"),
        tc("UMINUS text -> #VALUE!", r#"UMINUS("abc")"#, "error", "error"),
        tc("UMINUS #N/A propagates", "UMINUS(NA())", "error", "error"),
        tc("UMINUS no args -> error", "UMINUS()", "error", "error"),
    ]);

    // ── UPLUS ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("UPLUS positive", "UPLUS(5)", "basic", "number"),
        tc("UPLUS negative", "UPLUS(-5)", "basic", "number"),
        tc("UPLUS zero", "UPLUS(0)", "basic", "number"),
        tc("UPLUS decimal", "UPLUS(3.14)", "basic", "number"),
        tc("UPLUS large number", "UPLUS(1E+15)", "edge", "number"),
        tc("UPLUS TRUE coerces to 1", "UPLUS(TRUE)", "coercion", "number"),
        tc("UPLUS FALSE coerces to 0", "UPLUS(FALSE)", "coercion", "number"),
        tc("UPLUS text -> #VALUE!", r#"UPLUS("abc")"#, "error", "error"),
        tc("UPLUS #N/A propagates", "UPLUS(NA())", "error", "error"),
        tc("UPLUS no args -> error", "UPLUS()", "error", "error"),
    ]);

    // ── UNARY_PERCENT ─────────────────────────────────────────────────────────
    cases.extend([
        tc("UNARY_PERCENT 100 -> 1", "UNARY_PERCENT(100)", "basic", "number"),
        tc("UNARY_PERCENT 50 -> 0.5", "UNARY_PERCENT(50)", "basic", "number"),
        tc("UNARY_PERCENT 0 -> 0", "UNARY_PERCENT(0)", "basic", "number"),
        tc("UNARY_PERCENT negative", "UNARY_PERCENT(-50)", "basic", "number"),
        tc("UNARY_PERCENT decimal 0.5 -> 0.005", "UNARY_PERCENT(0.5)", "basic", "number"),
        tc("UNARY_PERCENT TRUE coerces to 1", "UNARY_PERCENT(TRUE)", "coercion", "number"),
        tc("UNARY_PERCENT text -> #VALUE!", r#"UNARY_PERCENT("abc")"#, "error", "error"),
        tc("UNARY_PERCENT #N/A propagates", "UNARY_PERCENT(NA())", "error", "error"),
        tc("UNARY_PERCENT no args -> error", "UNARY_PERCENT()", "error", "error"),
        tc("UNARY_PERCENT nested 200 -> 2", "UNARY_PERCENT(ADD(100,100))", "nested", "number"),
    ]);

    // ── CONCAT ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("CONCAT two strings", r#"CONCAT("Hello"," World")"#, "basic", "string"),
        tc("CONCAT empty+text", r#"CONCAT("","hello")"#, "basic", "string"),
        tc("CONCAT both empty", r#"CONCAT("","")"#, "basic", "string"),
        tc("CONCAT number+number", "CONCAT(1,2)", "coercion", "number"),
        tc("CONCAT text+number", r#"CONCAT("x",5)"#, "coercion", "string"),
        tc("CONCAT TRUE coercion", r#"CONCAT(TRUE,"!")"#, "coercion", "string"),
        tc("CONCAT FALSE coercion", r#"CONCAT(FALSE,"?")"#, "coercion", "string"),
        tc("CONCAT #DIV/0! propagates", r#"CONCAT(1/0,"x")"#, "error", "error"),
        tc("CONCAT no args -> error", "CONCAT()", "error", "error"),
        tc("CONCAT nested CHAR args", "CONCAT(CHAR(65),CHAR(66))", "nested", "string"),
        // variadic (3+ args)
        tc("CONCAT three strings", r#"CONCAT("Hello"," ","World")"#, "basic", "string"),
        tc("CONCAT four strings", r#"CONCAT("a","b","c","d")"#, "basic", "string"),
        tc("CONCAT five args mixed", r#"CONCAT("x",1," ",TRUE,"!")"#, "coercion", "string"),
        tc("CONCAT one arg string", r#"CONCAT("solo")"#, "edge", "string"),
        tc("CONCAT three numbers", "CONCAT(1,2,3)", "coercion", "string"),
        tc("CONCAT empty middle arg", r#"CONCAT("a","","b")"#, "edge", "string"),
        tc("CONCAT error in third arg", r#"CONCAT("a","b",1/0)"#, "error", "error"),
    ]);

    // ── ISBETWEEN ────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISBETWEEN value in range -> TRUE", "ISBETWEEN(5,1,10)", "basic", "boolean"),
        tc("ISBETWEEN value at lower bound -> TRUE", "ISBETWEEN(1,1,10)", "basic", "boolean"),
        tc("ISBETWEEN value at upper bound -> TRUE", "ISBETWEEN(10,1,10)", "basic", "boolean"),
        tc("ISBETWEEN value below range -> FALSE", "ISBETWEEN(0,1,10)", "basic", "boolean"),
        tc("ISBETWEEN value above range -> FALSE", "ISBETWEEN(11,1,10)", "basic", "boolean"),
        tc("ISBETWEEN exclusive lower bound", "ISBETWEEN(1,1,10,FALSE,TRUE)", "edge", "boolean"),
        tc("ISBETWEEN exclusive upper bound", "ISBETWEEN(10,1,10,TRUE,FALSE)", "edge", "boolean"),
        tc("ISBETWEEN negative range", "ISBETWEEN(-5,-10,-1)", "basic", "boolean"),
        tc("ISBETWEEN no args -> error", "ISBETWEEN()", "error", "error"),
        tc("ISBETWEEN nested formula value", "ISBETWEEN(ABS(-5),1,10)", "nested", "boolean"),
    ]);

    cases
}
