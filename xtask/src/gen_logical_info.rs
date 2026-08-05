use crate::types::{Platform, TestCase};

fn tc(description: &str, formula: &str, test_category: &str, expected_type: &str) -> TestCase {
    TestCase::new(description, formula, "", test_category, expected_type)
}

pub fn generate_logical(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── AND ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("AND two TRUE", "AND(TRUE,TRUE)", "basic", "boolean"),
        tc("AND one FALSE", "AND(TRUE,FALSE)", "basic", "boolean"),
        tc("AND 1 coerces TRUE", "AND(1,1)", "coercion", "boolean"),
        tc("AND 0 coerces FALSE", "AND(1,0)", "coercion", "boolean"),
        tc("AND text -> #VALUE!", r#"AND("1",TRUE)"#, "error", "error"),
        tc("AND no args -> error", "AND()", "error", "error"),
        tc("AND nested comparisons", "AND(1>0,2>1)", "nested", "boolean"),
        tc("AND #DIV/0! propagates", "AND(1/0,TRUE)", "error", "error"),
    ]);

    // ── OR ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("OR one TRUE", "OR(TRUE,FALSE)", "basic", "boolean"),
        tc("OR all FALSE", "OR(FALSE,FALSE)", "basic", "boolean"),
        tc("OR 1,0 coercion", "OR(1,0)", "coercion", "boolean"),
        tc("OR text -> #VALUE!", r#"OR("TRUE")"#, "error", "error"),
        tc("OR no args -> error", "OR()", "error", "error"),
        tc("OR nested formula args", r#"OR(ISNUMBER("a"),ISTEXT("a"))"#, "nested", "boolean"),
        tc("OR #N/A propagates", "OR(NA(),FALSE)", "error", "error"),
        tc("OR three some true", "OR(FALSE,FALSE,TRUE)", "basic", "boolean"),
    ]);

    // ── XOR ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("XOR TRUE,FALSE -> TRUE", "XOR(TRUE,FALSE)", "basic", "boolean"),
        tc("XOR TRUE,TRUE -> FALSE", "XOR(TRUE,TRUE)", "basic", "boolean"),
        tc("XOR FALSE,FALSE -> FALSE", "XOR(FALSE,FALSE)", "basic", "boolean"),
        tc("XOR 1,0 coercion", "XOR(1,0)", "coercion", "boolean"),
        tc("XOR 1,1 -> FALSE", "XOR(1,1)", "coercion", "boolean"),
        tc("XOR three odd -> TRUE", "XOR(TRUE,TRUE,TRUE)", "basic", "boolean"),
        tc("XOR no args -> error", "XOR()", "error", "error"),
        tc("XOR text -> error", r#"XOR("TRUE",FALSE)"#, "error", "error"),
    ]);

    // ── NOT ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("NOT(TRUE) -> FALSE", "NOT(TRUE)", "basic", "boolean"),
        tc("NOT(FALSE) -> TRUE", "NOT(FALSE)", "basic", "boolean"),
        tc("NOT 1 -> FALSE", "NOT(1)", "coercion", "boolean"),
        tc("NOT 0 -> TRUE", "NOT(0)", "coercion", "boolean"),
        tc("NOT text -> error", r#"NOT("1")"#, "error", "error"),
        tc("NOT no args -> error", "NOT()", "error", "error"),
        tc("NOT #DIV/0! propagates", "NOT(1/0)", "error", "error"),
        tc("NOT nested double NOT", "NOT(NOT(TRUE))", "nested", "boolean"),
    ]);

    // ── IF ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("IF true branch", "IF(TRUE,1,2)", "basic", "number"),
        tc("IF false branch", "IF(FALSE,1,2)", "basic", "number"),
        tc("IF string branches", r#"IF(TRUE,"yes","no")"#, "basic", "string"),
        tc("IF 1 is truthy", r#"IF(1,"yes","no")"#, "coercion", "string"),
        tc("IF 0 is falsy", r#"IF(0,"yes","no")"#, "coercion", "string"),
        tc("IF missing false branch", "IF(FALSE,1)", "edge", "boolean"),
        tc("IF error in condition propagates", "IF(1/0,1,2)", "error", "error"),
        tc("IF nested", r#"IF(1>2,"big",IF(1<2,"small","equal"))"#, "nested", "string"),
    ]);

    // ── IFERROR ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("IFERROR non-error passes through", r#"IFERROR(1,"err")"#, "basic", "number"),
        tc("IFERROR catches #DIV/0!", r#"IFERROR(1/0,"caught")"#, "basic", "string"),
        tc("IFERROR catches #NUM!", r#"IFERROR(SQRT(-1),"caught")"#, "basic", "string"),
        tc("IFERROR catches #N/A", r#"IFERROR(NA(),"caught")"#, "basic", "string"),
        tc("IFERROR returns 0 on error", "IFERROR(1/0,0)", "basic", "number"),
        tc("IFERROR missing 2nd arg on error -> empty", "IFERROR(1/0)", "edge", "string"),
        tc("IFERROR no args -> error", "IFERROR()", "error", "error"),
        tc("IFERROR nested chain", r#"IFERROR(1/0,IFERROR(SQRT(-1),42))"#, "nested", "number"),
    ]);

    // ── IFNA ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("IFNA non-NA passes through", r#"IFNA(1,"x")"#, "basic", "number"),
        tc("IFNA catches #N/A", r#"IFNA(NA(),"caught")"#, "basic", "string"),
        tc("IFNA #DIV/0! NOT caught", r#"IFNA(1/0,"caught")"#, "error", "error"),
        tc("IFNA text passes through", r#"IFNA("hello","x")"#, "basic", "string"),
        tc("IFNA no args -> error", "IFNA()", "error", "error"),
        tc("IFNA nested double", r#"IFNA(IFNA(NA(),NA()),"both")"#, "nested", "string"),
        tc("IFNA zero passes through", r#"IFNA(0,"x")"#, "basic", "number"),
        tc("IFNA FALSE passes through", r#"IFNA(FALSE,"x")"#, "basic", "boolean"),
    ]);

    // ── IFS ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("IFS first condition true", r#"IFS(TRUE,"a","b")"#, "basic", "string"),
        tc("IFS second condition", r#"IFS(FALSE,"a",TRUE,"b")"#, "basic", "string"),
        tc("IFS numeric condition coercion", r#"IFS(1,"one",0,"zero")"#, "coercion", "string"),
        tc("IFS no condition true -> #N/A", r#"IFS(FALSE,"a",FALSE,"b")"#, "error", "error"),
        tc("IFS no args -> error", "IFS()", "error", "error"),
        tc("IFS nested formula condition", r#"IFS(1>2,"big",1<2,"small")"#, "nested", "string"),
        tc("IFS first match wins", r#"IFS(TRUE,"first",TRUE,"second")"#, "basic", "string"),
        tc("IFS #N/A propagates in condition", r#"IFS(NA(),"x")"#, "error", "error"),
    ]);

    // ── SWITCH ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("SWITCH first match", r#"SWITCH(1,1,"one",2,"two")"#, "basic", "string"),
        tc("SWITCH second match", r#"SWITCH(2,1,"one",2,"two")"#, "basic", "string"),
        tc("SWITCH default used", r#"SWITCH(3,1,"one",2,"two","other")"#, "basic", "string"),
        tc("SWITCH no match no default -> #N/A", r#"SWITCH(3,1,"one",2,"two")"#, "error", "error"),
        tc("SWITCH string match", r#"SWITCH("b","a",1,"b",2)"#, "basic", "number"),
        tc("SWITCH no args -> error", "SWITCH()", "error", "error"),
        tc("SWITCH nested formula expr", r#"SWITCH(ABS(-1),1,"one",2,"two")"#, "nested", "string"),
        tc("SWITCH boolean match", r#"SWITCH(TRUE,TRUE,"yes",FALSE,"no")"#, "basic", "string"),
    ]);

    // ── LET ──────────────────────────────────────────────────────────────────
    cases.extend([
        tc("LET basic binding", "LET(x,5,x*2)", "basic", "number"),
        tc("LET two bindings", "LET(x,3,y,4,x+y)", "basic", "number"),
        tc("LET string value", r#"LET(s,"hello",LEN(s))"#, "nested", "number"),
        tc("LET reuse binding in body", "LET(n,10,n*n)", "basic", "number"),
        tc("LET nested arithmetic", "LET(a,2,b,3,a*b+1)", "basic", "number"),
        tc("LET error in binding propagates", "LET(x,1/0,x+1)", "error", "error"),
        tc("LET no args -> error", "LET()", "error", "error"),
        tc("LET binding used in nested fn", "LET(x,4,SQRT(x))", "nested", "number"),
    ]);

    // ── LAMBDA ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("LAMBDA immediately invoked double", "LAMBDA(x,x*2)(5)", "basic", "number"),
        tc("LAMBDA add two args", "LAMBDA(x,y,x+y)(3,4)", "basic", "number"),
        tc("LAMBDA string result", r#"LAMBDA(s,LEN(s))("hello")"#, "nested", "number"),
        tc("LAMBDA returns boolean", "LAMBDA(x,x>0)(5)", "basic", "boolean"),
        tc("LAMBDA nested call", "LAMBDA(x,LAMBDA(y,x+y)(10))(5)", "nested", "number"),
        tc("LAMBDA no args -> error", "LAMBDA()", "error", "error"),
        tc("LAMBDA zero returns zero", "LAMBDA(x,x*0)(99)", "basic", "number"),
        tc("LAMBDA uses ABS", "LAMBDA(x,ABS(x))(-7)", "basic", "number"),
    ]);

    // ── NA ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("NA returns #N/A", "NA()", "basic", "error"),
        tc("NA with arg -> error", "NA(1)", "error", "error"),
        tc("ISNA(NA()) -> TRUE", "ISNA(NA())", "nested", "boolean"),
        tc("IFERROR catches NA()", r#"IFERROR(NA(),"caught")"#, "nested", "string"),
        tc("NA in arithmetic propagates", "NA()+1", "error", "error"),
        tc("IFNA catches NA()", r#"IFNA(NA(),"ok")"#, "nested", "string"),
        tc("ISERROR(NA()) -> TRUE", "ISERROR(NA())", "nested", "boolean"),
        tc("ERROR.TYPE(NA()) -> 7", "ERROR.TYPE(NA())", "nested", "number"),
    ]);

    // ── N ────────────────────────────────────────────────────────────────────
    cases.extend([
        tc("N number returns number", "N(5)", "basic", "number"),
        tc("N zero", "N(0)", "basic", "number"),
        tc("N negative", "N(-3.14)", "basic", "number"),
        tc("N TRUE -> 1", "N(TRUE)", "coercion", "number"),
        tc("N FALSE -> 0", "N(FALSE)", "coercion", "number"),
        tc("N text -> 0", r#"N("hello")"#, "coercion", "number"),
        tc("N error propagates", "N(1/0)", "error", "error"),
        tc("N no args -> error", "N()", "error", "error"),
    ]);

    // ── TRUE / FALSE ─────────────────────────────────────────────────────────
    cases.extend([
        tc("TRUE() returns TRUE", "TRUE()", "basic", "boolean"),
        tc("TRUE no arg required", "TRUE()", "basic", "boolean"),
        tc("FALSE() returns FALSE", "FALSE()", "basic", "boolean"),
        tc("NOT(TRUE()) -> FALSE", "NOT(TRUE())", "nested", "boolean"),
        tc("AND(TRUE(),FALSE()) -> FALSE", "AND(TRUE(),FALSE())", "nested", "boolean"),
        tc("IF(TRUE(),1,2) -> 1", "IF(TRUE(),1,2)", "nested", "number"),
        tc("TRUE() with arg -> error", "TRUE(1)", "error", "error"),
        tc("FALSE() with arg -> error", "FALSE(1)", "error", "error"),
    ]);

    cases
}

pub fn generate_info(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── ISBLANK ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISBLANK empty string -> FALSE", r#"ISBLANK("")"#, "basic", "boolean"),
        tc("ISBLANK number -> FALSE", "ISBLANK(0)", "basic", "boolean"),
        tc("ISBLANK text -> FALSE", r#"ISBLANK("text")"#, "basic", "boolean"),
        tc("ISBLANK TRUE -> FALSE", "ISBLANK(TRUE)", "basic", "boolean"),
        tc("ISBLANK error -> FALSE", "ISBLANK(NA())", "edge", "boolean"),
        tc("ISBLANK formula result -> FALSE", "ISBLANK(1+0)", "edge", "boolean"),
        tc("ISBLANK no args -> error", "ISBLANK()", "error", "error"),
        tc("NOT(ISBLANK text)", r#"NOT(ISBLANK("x"))"#, "nested", "boolean"),
    ]);

    // ── ISDATE ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISDATE date serial -> TRUE", "ISDATE(DATE(2024,1,15))", "basic", "boolean"),
        tc("ISDATE plain number -> FALSE", "ISDATE(42)", "basic", "boolean"),
        tc("ISDATE text date string -> TRUE", r#"ISDATE("2024-01-15")"#, "basic", "boolean"),
        tc("ISDATE text non-date -> FALSE", r#"ISDATE("hello")"#, "basic", "boolean"),
        tc("ISDATE TRUE -> FALSE", "ISDATE(TRUE)", "basic", "boolean"),
        tc("ISDATE no args -> error", "ISDATE()", "error", "error"),
        tc("ISDATE error -> FALSE", "ISDATE(NA())", "edge", "boolean"),
        tc("IF(ISDATE(DATE(2024,1,1)),1,0)", "IF(ISDATE(DATE(2024,1,1)),1,0)", "nested", "number"),
    ]);

    // ── ISEMAIL ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISEMAIL valid email -> TRUE", r#"ISEMAIL("user@example.com")"#, "basic", "boolean"),
        tc("ISEMAIL no @ -> FALSE", r#"ISEMAIL("notanemail")"#, "basic", "boolean"),
        tc("ISEMAIL empty string -> FALSE", r#"ISEMAIL("")"#, "basic", "boolean"),
        tc("ISEMAIL number -> FALSE", "ISEMAIL(42)", "coercion", "boolean"),
        tc("ISEMAIL TRUE -> FALSE", "ISEMAIL(TRUE)", "coercion", "boolean"),
        tc("ISEMAIL no args -> error", "ISEMAIL()", "error", "error"),
        tc("ISEMAIL domain only -> FALSE", r#"ISEMAIL("@example.com")"#, "edge", "boolean"),
        tc("IF(ISEMAIL email,1,0)", r#"IF(ISEMAIL("a@b.com"),1,0)"#, "nested", "number"),
    ]);

    // ── ISERR ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISERR #DIV/0! -> TRUE", "ISERR(1/0)", "basic", "boolean"),
        tc("ISERR #N/A -> FALSE (excludes N/A)", "ISERR(NA())", "basic", "boolean"),
        tc("ISERR #NUM! -> TRUE", "ISERR(SQRT(-1))", "basic", "boolean"),
        tc("ISERR number -> FALSE", "ISERR(1)", "basic", "boolean"),
        tc("ISERR text -> FALSE", r#"ISERR("hello")"#, "basic", "boolean"),
        tc("ISERR no args -> error", "ISERR()", "error", "error"),
        tc("ISERR nested", r#"ISERR(ABS("x"))"#, "nested", "boolean"),
        tc("IF(ISERR(1/0),err,ok)", r#"IF(ISERR(1/0),"err","ok")"#, "nested", "string"),
    ]);

    // ── ISERROR ──────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISERROR #DIV/0! -> TRUE", "ISERROR(1/0)", "basic", "boolean"),
        tc("ISERROR #N/A -> TRUE (includes N/A)", "ISERROR(NA())", "basic", "boolean"),
        tc("ISERROR #NUM! -> TRUE", "ISERROR(SQRT(-1))", "basic", "boolean"),
        tc("ISERROR number -> FALSE", "ISERROR(1)", "basic", "boolean"),
        tc("ISERROR text -> FALSE", r#"ISERROR("hello")"#, "basic", "boolean"),
        tc("ISERROR no args -> error", "ISERROR()", "error", "error"),
        tc("ISERROR vs ISERR on #N/A", "ISERROR(NA())-ISERR(NA())", "nested", "number"),
        tc("IF(ISERROR(SQRT(-1)),bad,ok)", r#"IF(ISERROR(SQRT(-1)),"bad","ok")"#, "nested", "string"),
    ]);

    // ── ISLOGICAL ────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISLOGICAL TRUE -> TRUE", "ISLOGICAL(TRUE)", "basic", "boolean"),
        tc("ISLOGICAL FALSE -> TRUE", "ISLOGICAL(FALSE)", "basic", "boolean"),
        tc("ISLOGICAL number -> FALSE", "ISLOGICAL(1)", "basic", "boolean"),
        tc("ISLOGICAL text -> FALSE", r#"ISLOGICAL("TRUE")"#, "basic", "boolean"),
        tc("ISLOGICAL error -> FALSE", "ISLOGICAL(NA())", "edge", "boolean"),
        tc("ISLOGICAL no args -> error", "ISLOGICAL()", "error", "error"),
        tc("ISLOGICAL AND result -> TRUE", "ISLOGICAL(AND(TRUE,FALSE))", "nested", "boolean"),
        tc("ISLOGICAL comparison -> TRUE", "ISLOGICAL(1>0)", "nested", "boolean"),
    ]);

    // ── ISNA ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISNA #N/A -> TRUE", "ISNA(NA())", "basic", "boolean"),
        tc("ISNA #DIV/0! -> FALSE", "ISNA(1/0)", "basic", "boolean"),
        tc("ISNA number -> FALSE", "ISNA(1)", "basic", "boolean"),
        tc("ISNA text -> FALSE", r#"ISNA("hello")"#, "basic", "boolean"),
        tc("ISNA empty string -> FALSE", r#"ISNA("")"#, "edge", "boolean"),
        tc("ISNA no args -> error", "ISNA()", "error", "error"),
        tc("ISNA(IFERROR(NA(),NA()))", "ISNA(IFERROR(NA(),NA()))", "nested", "boolean"),
        tc("IF(ISNA(NA()),na,ok)", r#"IF(ISNA(NA()),"na","ok")"#, "nested", "string"),
    ]);

    // ── ISNONTEXT ────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISNONTEXT number -> TRUE", "ISNONTEXT(1)", "basic", "boolean"),
        tc("ISNONTEXT TRUE -> TRUE", "ISNONTEXT(TRUE)", "basic", "boolean"),
        tc("ISNONTEXT text -> FALSE", r#"ISNONTEXT("hello")"#, "basic", "boolean"),
        tc("ISNONTEXT empty string -> FALSE", r#"ISNONTEXT("")"#, "basic", "boolean"),
        tc("ISNONTEXT error -> TRUE", "ISNONTEXT(NA())", "edge", "boolean"),
        tc("ISNONTEXT no args -> error", "ISNONTEXT()", "error", "error"),
        tc("NOT(ISNONTEXT text)", r#"NOT(ISNONTEXT("hello"))"#, "nested", "boolean"),
        tc("ISNONTEXT zero -> TRUE", "ISNONTEXT(0)", "basic", "boolean"),
    ]);

    // ── ISNUMBER ─────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISNUMBER integer -> TRUE", "ISNUMBER(42)", "basic", "boolean"),
        tc("ISNUMBER decimal -> TRUE", "ISNUMBER(3.14)", "basic", "boolean"),
        tc("ISNUMBER zero -> TRUE", "ISNUMBER(0)", "basic", "boolean"),
        tc("ISNUMBER text -> FALSE", r#"ISNUMBER("42")"#, "basic", "boolean"),
        tc("ISNUMBER TRUE -> FALSE", "ISNUMBER(TRUE)", "basic", "boolean"),
        tc("ISNUMBER error -> FALSE", "ISNUMBER(NA())", "edge", "boolean"),
        tc("ISNUMBER no args -> error", "ISNUMBER()", "error", "error"),
        tc("AND(ISNUMBER(1),ISNUMBER(2))", "AND(ISNUMBER(1),ISNUMBER(2))", "nested", "boolean"),
    ]);

    // ── ISREF ────────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISREF number -> FALSE", "ISREF(1)", "basic", "boolean"),
        tc("ISREF text -> FALSE", r#"ISREF("A1")"#, "basic", "boolean"),
        tc("ISREF TRUE -> FALSE", "ISREF(TRUE)", "basic", "boolean"),
        tc("ISREF error -> FALSE", "ISREF(NA())", "edge", "boolean"),
        tc("ISREF no args -> error", "ISREF()", "error", "error"),
        tc("ISREF empty string -> FALSE", r#"ISREF("")"#, "basic", "boolean"),
        tc("NOT(ISREF(1))", "NOT(ISREF(1))", "nested", "boolean"),
        tc("IF(ISREF(1),yes,no)", r#"IF(ISREF(1),"yes","no")"#, "nested", "string"),
    ]);

    // ── ISTEXT ───────────────────────────────────────────────────────────────
    cases.extend([
        tc("ISTEXT text -> TRUE", r#"ISTEXT("hello")"#, "basic", "boolean"),
        tc("ISTEXT empty string -> TRUE", r#"ISTEXT("")"#, "basic", "boolean"),
        tc("ISTEXT number -> FALSE", "ISTEXT(42)", "basic", "boolean"),
        tc("ISTEXT TRUE -> FALSE", "ISTEXT(TRUE)", "basic", "boolean"),
        tc("ISTEXT error -> FALSE", "ISTEXT(NA())", "edge", "boolean"),
        tc("ISTEXT no args -> error", "ISTEXT()", "error", "error"),
        tc("AND(ISTEXT text, ISTEXT num)", r#"AND(ISTEXT("a"),ISTEXT(1))"#, "nested", "boolean"),
        tc("ISTEXT numeric string -> TRUE", r#"ISTEXT("42")"#, "basic", "boolean"),
    ]);

    // ── ERROR.TYPE ───────────────────────────────────────────────────────────
    cases.extend([
        tc("ERROR.TYPE #DIV/0! -> 2", "ERROR.TYPE(1/0)", "basic", "number"),
        tc("ERROR.TYPE #N/A -> 7", "ERROR.TYPE(NA())", "basic", "number"),
        tc("ERROR.TYPE #NUM! -> 6", "ERROR.TYPE(SQRT(-1))", "basic", "number"),
        tc("ERROR.TYPE non-error -> #N/A", "ERROR.TYPE(1)", "edge", "error"),
        tc("ERROR.TYPE no args -> error", "ERROR.TYPE()", "error", "error"),
        tc("ERROR.TYPE nested result +1", "ERROR.TYPE(NA())+1", "nested", "number"),
        tc("ERROR.TYPE #VALUE! -> 3", r#"ERROR.TYPE("a"+1)"#, "basic", "number"),
        tc("ERROR.TYPE nested error", r#"ERROR.TYPE(ABS("x"))"#, "nested", "number"),
    ]);

    // ── N ────────────────────────────────────────────────────────────────────
    cases.extend([
        tc("N number returns number", "N(5)", "basic", "number"),
        tc("N TRUE -> 1", "N(TRUE)", "coercion", "number"),
        tc("N FALSE -> 0", "N(FALSE)", "coercion", "number"),
        tc("N text -> 0", r#"N("hello")"#, "coercion", "number"),
        tc("N zero", "N(0)", "basic", "number"),
        tc("N error propagates", "N(1/0)", "error", "error"),
        tc("N no args -> error", "N()", "error", "error"),
        tc("N nested in arithmetic", "N(TRUE)+N(FALSE)", "nested", "number"),
    ]);

    // ── NA ───────────────────────────────────────────────────────────────────
    cases.extend([
        tc("NA() returns #N/A", "NA()", "basic", "error"),
        tc("NA with arg -> error", "NA(1)", "error", "error"),
        tc("ISNA(NA()) -> TRUE", "ISNA(NA())", "nested", "boolean"),
        tc("ISERROR(NA()) -> TRUE", "ISERROR(NA())", "nested", "boolean"),
        tc("ISERR(NA()) -> FALSE", "ISERR(NA())", "nested", "boolean"),
        tc("IFNA(NA(),0) -> 0", "IFNA(NA(),0)", "nested", "number"),
        tc("NA propagates in sum", "NA()+1", "error", "error"),
        tc("ERROR.TYPE(NA()) -> 7", "ERROR.TYPE(NA())", "nested", "number"),
    ]);

    // ── TYPE ─────────────────────────────────────────────────────────────────
    cases.extend([
        tc("TYPE number -> 1", "TYPE(5)", "basic", "number"),
        tc("TYPE text -> 2", r#"TYPE("hello")"#, "basic", "number"),
        tc("TYPE boolean -> 4", "TYPE(TRUE)", "basic", "number"),
        tc("TYPE error -> 16", "TYPE(NA())", "basic", "number"),
        tc("TYPE array -> 64", "TYPE({1,2,3})", "basic", "number"),
        tc("TYPE zero -> 1", "TYPE(0)", "basic", "number"),
        tc("TYPE no args -> error", "TYPE()", "error", "error"),
        tc("TYPE nested formula result", "TYPE(1+1)", "nested", "number"),
    ]);

    cases
}
