use crate::types::{wrap_array, Platform, TestCase};

// Helper: build a TestCase with empty expected_value (evaluator fills it in).
fn tc(description: &str, formula: &str, category: &str) -> TestCase {
    TestCase::new(description, formula, "", category, "array")
}

fn tc_scalar(description: &str, formula: &str, category: &str) -> TestCase {
    TestCase::new(description, formula, "", category, "number")
}

fn tc_text(description: &str, formula: &str, category: &str) -> TestCase {
    TestCase::new(description, formula, "", category, "text")
}

#[allow(clippy::vec_init_then_push)]
pub fn generate_array(platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── SORT ─────────────────────────────────────────────────────────────────
    // BUG-09: SORT 1D descending
    cases.push(tc(
        "SORT 1D array ascending (default)",
        &wrap_array("SORT({3,1,2},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D array descending (BUG-09)",
        &wrap_array("SORT({3,1,2},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D already sorted ascending",
        &wrap_array("SORT({1,2,3},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D already sorted descending",
        &wrap_array("SORT({3,2,1},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D with duplicates ascending",
        &wrap_array("SORT({3,1,2,1,3},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D with duplicates descending",
        &wrap_array("SORT({3,1,2,1,3},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 2D by first column ascending",
        &wrap_array("SORT({3,\"c\";1,\"a\";2,\"b\"},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 2D by first column descending",
        &wrap_array("SORT({3,\"c\";1,\"a\";2,\"b\"},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 2D by second column ascending",
        &wrap_array("SORT({3,\"c\";1,\"a\";2,\"b\"},2,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT single-element array",
        &wrap_array("SORT({42},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D strings ascending",
        &wrap_array("SORT({\"banana\",\"apple\",\"cherry\"},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 1D strings descending",
        &wrap_array("SORT({\"banana\",\"apple\",\"cherry\"},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 2D by col 2 descending",
        &wrap_array("SORT({3,30;1,10;2,20},2,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 4-element array ascending",
        &wrap_array("SORT({4,2,3,1},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 4-element array descending",
        &wrap_array("SORT({4,2,3,1},1,-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 5-element array ascending",
        &wrap_array("SORT({5,1,4,2,3},1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORT 5-element array descending",
        &wrap_array("SORT({5,1,4,2,3},1,-1)", platform),
        "array",
    ));

    // ── SORTBY ───────────────────────────────────────────────────────────────
    // BUG-10: SORTBY descending
    cases.push(tc(
        "SORTBY ascending (works)",
        &wrap_array("SORTBY({3,1,2},{2,1,3},1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY descending (BUG-10)",
        &wrap_array("SORTBY({3,1,2},{2,1,3},-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY with equal keys ascending",
        &wrap_array("SORTBY({\"a\",\"b\",\"c\"},{2,2,1},1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY with equal keys descending",
        &wrap_array("SORTBY({\"a\",\"b\",\"c\"},{2,2,1},-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY 2D data by key array ascending",
        &wrap_array("SORTBY({10,20,30},{3,1,2},1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY 2D data by key array descending",
        &wrap_array("SORTBY({10,20,30},{3,1,2},-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY with 5 elements ascending",
        &wrap_array("SORTBY({\"e\",\"d\",\"c\",\"b\",\"a\"},{5,4,3,2,1},1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY with 5 elements descending",
        &wrap_array("SORTBY({\"e\",\"d\",\"c\",\"b\",\"a\"},{5,4,3,2,1},-1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY 2D array by separate key ascending",
        &wrap_array("SORTBY({\"a\",1;\"b\",2;\"c\",3},{3,2,1},1)", platform),
        "array",
    ));
    cases.push(tc(
        "SORTBY 2D array by separate key descending",
        &wrap_array("SORTBY({\"a\",1;\"b\",2;\"c\",3},{3,2,1},-1)", platform),
        "array",
    ));

    // ── UNIQUE ───────────────────────────────────────────────────────────────
    // BUG-04: UNIQUE exactly_once=TRUE
    cases.push(tc(
        "UNIQUE basic dedup",
        &wrap_array("UNIQUE({1,2,2,3})", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE exactly_once=TRUE (BUG-04)",
        &wrap_array("UNIQUE({1,2,2,3},FALSE,TRUE)", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE no duplicates",
        &wrap_array("UNIQUE({1,2,3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE all duplicates exactly_once=TRUE returns empty",
        &wrap_array("UNIQUE({2,2,3,3},FALSE,TRUE)", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE single element",
        &wrap_array("UNIQUE({5})", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE three identical elements exactly_once returns empty",
        &wrap_array("UNIQUE({7,7,7},FALSE,TRUE)", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE mixed with one unique",
        &wrap_array("UNIQUE({1,2,2,3,3},FALSE,TRUE)", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE strings basic",
        &wrap_array("UNIQUE({\"a\",\"b\",\"a\",\"c\"})", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE strings exactly_once",
        &wrap_array("UNIQUE({\"a\",\"b\",\"a\",\"c\"},FALSE,TRUE)", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE numbers with many duplicates",
        &wrap_array("UNIQUE({1,1,2,2,3,3})", platform),
        "array",
    ));
    cases.push(tc(
        "UNIQUE five distinct values",
        &wrap_array("UNIQUE({5,4,3,2,1})", platform),
        "array",
    ));

    // ── TRANSPOSE ────────────────────────────────────────────────────────────
    cases.push(tc(
        "TRANSPOSE 1D row to column",
        &wrap_array("TRANSPOSE({1,2,3})", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE 2D matrix",
        &wrap_array("TRANSPOSE({1,2;3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE single element",
        &wrap_array("TRANSPOSE({42})", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE 3x1 column",
        &wrap_array("TRANSPOSE({1;2;3})", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE 2x3 matrix",
        &wrap_array("TRANSPOSE({1,2,3;4,5,6})", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE 3x2 matrix",
        &wrap_array("TRANSPOSE({1,2;3,4;5,6})", platform),
        "array",
    ));

    // ── SEQUENCE ─────────────────────────────────────────────────────────────
    cases.push(tc(
        "SEQUENCE 1 row 5 cols",
        &wrap_array("SEQUENCE(1,5)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 5 rows 1 col",
        &wrap_array("SEQUENCE(5,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 3x3",
        &wrap_array("SEQUENCE(3,3)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE with start and step",
        &wrap_array("SEQUENCE(1,5,0,2)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE with negative step",
        &wrap_array("SEQUENCE(1,5,10,-2)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE single element",
        &wrap_array("SEQUENCE(1,1,7,1)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 2x4 grid",
        &wrap_array("SEQUENCE(2,4)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 4x2 grid",
        &wrap_array("SEQUENCE(4,2)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 1x10 range",
        &wrap_array("SEQUENCE(1,10)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 1x3 start 5 step 5",
        &wrap_array("SEQUENCE(1,3,5,5)", platform),
        "array",
    ));
    cases.push(tc(
        "SEQUENCE 3x1 start 1 step 10",
        &wrap_array("SEQUENCE(3,1,1,10)", platform),
        "array",
    ));

    // ── ROWS / COLUMNS ───────────────────────────────────────────────────────
    // BUG-20: ROWS/COLUMNS on 1D vs 2D
    cases.push(tc_scalar(
        "ROWS of 1D array",
        "ROWS({1,2,3})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS of 2D array",
        "ROWS({1,2;3,4;5,6})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS of single element",
        "ROWS({42})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS of 1D array",
        "COLUMNS({1,2,3})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS of 2D array",
        "COLUMNS({1,2;3,4;5,6})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS of single element",
        "COLUMNS({42})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS of column vector (BUG-20)",
        "ROWS({1;2;3})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS of column vector (BUG-20)",
        "COLUMNS({1;2;3})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS 2x5 matrix",
        "ROWS({1,2,3,4,5;6,7,8,9,10})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS 2x5 matrix",
        "COLUMNS({1,2,3,4,5;6,7,8,9,10})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS 5x1 column",
        "ROWS({1;2;3;4;5})",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS 1x5 row",
        "COLUMNS({1,2,3,4,5})",
        "array",
    ));
    cases.push(tc_scalar(
        "ROWS of SEQUENCE result",
        "ROWS(SEQUENCE(3,4))",
        "array",
    ));
    cases.push(tc_scalar(
        "COLUMNS of SEQUENCE result",
        "COLUMNS(SEQUENCE(3,4))",
        "array",
    ));

    // ── INDEX ─────────────────────────────────────────────────────────────────
    cases.push(tc_scalar(
        "INDEX 1D array first element",
        "INDEX({10,20,30},1,1)",
        "array",
    ));
    cases.push(tc_scalar(
        "INDEX 1D array last element",
        "INDEX({10,20,30},1,3)",
        "array",
    ));
    cases.push(tc_scalar(
        "INDEX 2D array element",
        "INDEX({1,2;3,4},2,2)",
        "array",
    ));
    cases.push(tc_scalar(
        "INDEX 2D first row first col",
        "INDEX({10,20;30,40},1,1)",
        "array",
    ));
    cases.push(tc(
        "INDEX 2D entire row",
        &wrap_array("INDEX({1,2,3;4,5,6},2,0)", platform),
        "array",
    ));
    cases.push(tc(
        "INDEX return whole column 1",
        &wrap_array("INDEX({1,2;3,4;5,6},0,1)", platform),
        "array",
    ));
    cases.push(tc_scalar(
        "INDEX 3x3 center element",
        "INDEX({1,2,3;4,5,6;7,8,9},2,2)",
        "array",
    ));
    cases.push(tc_scalar(
        "INDEX 3x3 corner element",
        "INDEX({1,2,3;4,5,6;7,8,9},3,3)",
        "array",
    ));

    // ── ARRAY_CONSTRAIN ───────────────────────────────────────────────────────
    cases.push(tc(
        "ARRAY_CONSTRAIN 3x3 to 2x2",
        &wrap_array("ARRAY_CONSTRAIN({1,2,3;4,5,6;7,8,9},2,2)", platform),
        "array",
    ));
    cases.push(tc(
        "ARRAY_CONSTRAIN 1D to 1x2",
        &wrap_array("ARRAY_CONSTRAIN({1,2,3,4},1,2)", platform),
        "array",
    ));
    cases.push(tc(
        "ARRAY_CONSTRAIN larger than source",
        &wrap_array("ARRAY_CONSTRAIN({1,2;3,4},3,3)", platform),
        "array",
    ));
    cases.push(tc(
        "ARRAY_CONSTRAIN 4x4 to 3x3",
        &wrap_array("ARRAY_CONSTRAIN({1,2,3,4;5,6,7,8;9,10,11,12;13,14,15,16},3,3)", platform),
        "array",
    ));
    cases.push(tc(
        "ARRAY_CONSTRAIN to 1x1",
        &wrap_array("ARRAY_CONSTRAIN({1,2;3,4},1,1)", platform),
        "array",
    ));

    // ── FLATTEN ───────────────────────────────────────────────────────────────
    cases.push(tc(
        "FLATTEN 1D array",
        &wrap_array("FLATTEN({1,2,3})", platform),
        "array",
    ));
    cases.push(tc(
        "FLATTEN 2D array",
        &wrap_array("FLATTEN({1,2;3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "FLATTEN single value",
        &wrap_array("FLATTEN({42})", platform),
        "array",
    ));
    cases.push(tc(
        "FLATTEN 3x3 grid",
        &wrap_array("FLATTEN({1,2,3;4,5,6;7,8,9})", platform),
        "array",
    ));
    cases.push(tc(
        "FLATTEN already flat 1D",
        &wrap_array("FLATTEN({10,20,30,40})", platform),
        "array",
    ));

    // ── HSTACK / VSTACK ───────────────────────────────────────────────────────
    cases.push(tc(
        "HSTACK two 1D arrays",
        &wrap_array("HSTACK({1,2},{3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "HSTACK three arrays",
        &wrap_array("HSTACK({1},{2},{3})", platform),
        "array",
    ));
    cases.push(tc(
        "VSTACK two 1D arrays",
        &wrap_array("VSTACK({1,2},{3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "VSTACK two column vectors",
        &wrap_array("VSTACK({1;2},{3;4})", platform),
        "array",
    ));
    cases.push(tc(
        "HSTACK 2D matrices",
        &wrap_array("HSTACK({1,2;3,4},{5,6;7,8})", platform),
        "array",
    ));
    cases.push(tc(
        "VSTACK 2D matrices",
        &wrap_array("VSTACK({1,2;3,4},{5,6;7,8})", platform),
        "array",
    ));
    cases.push(tc(
        "HSTACK three 1-element arrays",
        &wrap_array("HSTACK({10},{20},{30})", platform),
        "array",
    ));
    cases.push(tc(
        "VSTACK three row arrays",
        &wrap_array("VSTACK({1,2,3},{4,5,6},{7,8,9})", platform),
        "array",
    ));
    cases.push(tc(
        "HSTACK four single values",
        &wrap_array("HSTACK({1},{2},{3},{4})", platform),
        "array",
    ));
    cases.push(tc(
        "VSTACK four single rows",
        &wrap_array("VSTACK({1,2},{3,4},{5,6},{7,8})", platform),
        "array",
    ));

    // ── TOCOL / TOROW ─────────────────────────────────────────────────────────
    cases.push(tc(
        "TOCOL 2D array to column",
        &wrap_array("TOCOL({1,2;3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "TOROW 2D array to row",
        &wrap_array("TOROW({1,2;3,4})", platform),
        "array",
    ));
    cases.push(tc(
        "TOCOL 1D row to column",
        &wrap_array("TOCOL({1,2,3})", platform),
        "array",
    ));
    cases.push(tc(
        "TOROW column vector to row",
        &wrap_array("TOROW({1;2;3})", platform),
        "array",
    ));
    cases.push(tc(
        "TOCOL ignore empty (mode 1)",
        &wrap_array("TOCOL({1,\"\",2,\"\",3},1)", platform),
        "array",
    ));
    cases.push(tc(
        "TOROW ignore empty (mode 1)",
        &wrap_array("TOROW({1;\"\";2;\"\";3},1)", platform),
        "array",
    ));

    // ── WRAPCOLS / WRAPROWS ───────────────────────────────────────────────────
    cases.push(tc(
        "WRAPROWS wrap 6 elements into rows of 2",
        &wrap_array("WRAPROWS({1,2,3,4,5,6},2)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPROWS wrap 6 elements into rows of 3",
        &wrap_array("WRAPROWS({1,2,3,4,5,6},3)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPCOLS wrap 6 elements into cols of 2",
        &wrap_array("WRAPCOLS({1,2,3,4,5,6},2)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPCOLS wrap 6 elements into cols of 3",
        &wrap_array("WRAPCOLS({1,2,3,4,5,6},3)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPROWS uneven wrap with pad",
        &wrap_array("WRAPROWS({1,2,3,4,5},3,0)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPCOLS uneven wrap with pad",
        &wrap_array("WRAPCOLS({1,2,3,4,5},3,0)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPROWS wrap 8 into 4",
        &wrap_array("WRAPROWS({1,2,3,4,5,6,7,8},4)", platform),
        "array",
    ));
    cases.push(tc(
        "WRAPCOLS wrap 8 into 4",
        &wrap_array("WRAPCOLS({1,2,3,4,5,6,7,8},4)", platform),
        "array",
    ));

    // ── CHOOSECOLS / CHOOSEROWS ────────────────────────────────────────────────
    cases.push(tc(
        "CHOOSECOLS select column 1 and 3",
        &wrap_array("CHOOSECOLS({1,2,3;4,5,6},1,3)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSECOLS select single column",
        &wrap_array("CHOOSECOLS({10,20,30},2)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSEROWS select row 1 and 3",
        &wrap_array("CHOOSEROWS({1,2;3,4;5,6},1,3)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSEROWS select single row",
        &wrap_array("CHOOSEROWS({10;20;30},2)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSECOLS reorder columns",
        &wrap_array("CHOOSECOLS({1,2,3;4,5,6},3,2,1)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSEROWS reorder rows",
        &wrap_array("CHOOSEROWS({1,2;3,4;5,6},3,2,1)", platform),
        "array",
    ));
    cases.push(tc(
        "CHOOSECOLS duplicate column",
        &wrap_array("CHOOSECOLS({1,2,3;4,5,6},1,1)", platform),
        "array",
    ));

    // ── MMULT ─────────────────────────────────────────────────────────────────
    cases.push(tc(
        "MMULT 2x2 matrices",
        &wrap_array("MMULT({1,2;3,4},{5,6;7,8})", platform),
        "array",
    ));
    cases.push(tc(
        "MMULT identity matrix",
        &wrap_array("MMULT({1,0;0,1},{5,6;7,8})", platform),
        "array",
    ));
    cases.push(tc(
        "MMULT 1x2 by 2x1",
        &wrap_array("MMULT({1,2},{3;4})", platform),
        "array",
    ));
    cases.push(tc(
        "MMULT 3x2 by 2x3",
        &wrap_array("MMULT({1,2;3,4;5,6},{7,8,9;10,11,12})", platform),
        "array",
    ));
    cases.push(tc(
        "MMULT zeros matrix",
        &wrap_array("MMULT({0,0;0,0},{1,2;3,4})", platform),
        "array",
    ));

    // ── BYROW / BYCOL ─────────────────────────────────────────────────────────
    cases.push(tc(
        "BYROW sum each row",
        &wrap_array("BYROW({1,2;3,4},LAMBDA(row,SUM(row)))", platform),
        "array",
    ));
    cases.push(tc(
        "BYROW max each row",
        &wrap_array("BYROW({1,5;3,2},LAMBDA(row,MAX(row)))", platform),
        "array",
    ));
    cases.push(tc(
        "BYCOL sum each column",
        &wrap_array("BYCOL({1,2;3,4},LAMBDA(col,SUM(col)))", platform),
        "array",
    ));
    cases.push(tc(
        "BYCOL max each column",
        &wrap_array("BYCOL({1,5;3,2},LAMBDA(col,MAX(col)))", platform),
        "array",
    ));
    cases.push(tc(
        "BYROW count elements per row",
        &wrap_array("BYROW({1,2,3;4,5,6},LAMBDA(row,COLUMNS(row)))", platform),
        "array",
    ));
    cases.push(tc(
        "BYCOL min each column",
        &wrap_array("BYCOL({3,1;4,2;1,5},LAMBDA(col,MIN(col)))", platform),
        "array",
    ));

    // ── MAP ───────────────────────────────────────────────────────────────────
    cases.push(tc(
        "MAP double each element",
        &wrap_array("MAP({1,2,3},LAMBDA(x,x*2))", platform),
        "array",
    ));
    cases.push(tc(
        "MAP add two arrays element-wise",
        &wrap_array("MAP({1,2,3},{10,20,30},LAMBDA(a,b,a+b))", platform),
        "array",
    ));
    cases.push(tc(
        "MAP square each element",
        &wrap_array("MAP({1,2,3,4},LAMBDA(x,x^2))", platform),
        "array",
    ));
    cases.push(tc(
        "MAP negate each element",
        &wrap_array("MAP({1,2,3,4},LAMBDA(x,-x))", platform),
        "array",
    ));
    cases.push(tc(
        "MAP concat strings",
        &wrap_array("MAP({\"a\",\"b\",\"c\"},LAMBDA(x,x&\"!\"))", platform),
        "array",
    ));

    // ── REDUCE ────────────────────────────────────────────────────────────────
    cases.push(tc_scalar(
        "REDUCE sum accumulator",
        "REDUCE(0,{1,2,3,4,5},LAMBDA(acc,x,acc+x))",
        "array",
    ));
    cases.push(tc_scalar(
        "REDUCE product accumulator",
        "REDUCE(1,{1,2,3,4},LAMBDA(acc,x,acc*x))",
        "array",
    ));
    cases.push(tc_scalar(
        "REDUCE max value",
        "REDUCE(0,{3,1,4,1,5,9},LAMBDA(acc,x,MAX(acc,x)))",
        "array",
    ));
    cases.push(tc_scalar(
        "REDUCE count positive",
        "REDUCE(0,{1,-2,3,-4,5},LAMBDA(acc,x,acc+IF(x>0,1,0)))",
        "array",
    ));

    // ── SCAN ──────────────────────────────────────────────────────────────────
    cases.push(tc(
        "SCAN running sum",
        &wrap_array("SCAN(0,{1,2,3,4},LAMBDA(acc,x,acc+x))", platform),
        "array",
    ));
    cases.push(tc(
        "SCAN running product",
        &wrap_array("SCAN(1,{1,2,3,4},LAMBDA(acc,x,acc*x))", platform),
        "array",
    ));
    cases.push(tc(
        "SCAN running max",
        &wrap_array("SCAN(0,{3,1,4,1,5},LAMBDA(acc,x,MAX(acc,x)))", platform),
        "array",
    ));

    // ── MAKEARRAY ─────────────────────────────────────────────────────────────
    cases.push(tc(
        "MAKEARRAY 2x3 multiplication table",
        &wrap_array("MAKEARRAY(2,3,LAMBDA(r,c,r*c))", platform),
        "array",
    ));
    cases.push(tc(
        "MAKEARRAY 3x3 with row+col",
        &wrap_array("MAKEARRAY(3,3,LAMBDA(r,c,r+c))", platform),
        "array",
    ));
    cases.push(tc(
        "MAKEARRAY 1x5 row indices",
        &wrap_array("MAKEARRAY(1,5,LAMBDA(r,c,c))", platform),
        "array",
    ));
    cases.push(tc(
        "MAKEARRAY 4x1 column indices",
        &wrap_array("MAKEARRAY(4,1,LAMBDA(r,c,r))", platform),
        "array",
    ));

    // ── FREQUENCY ─────────────────────────────────────────────────────────────
    cases.push(tc(
        "FREQUENCY basic histogram",
        &wrap_array("FREQUENCY({1,2,3,4,5},{2,4})", platform),
        "array",
    ));
    cases.push(tc(
        "FREQUENCY all in one bin",
        &wrap_array("FREQUENCY({1,1,1},{5})", platform),
        "array",
    ));

    // ── Combinations ──────────────────────────────────────────────────────────
    cases.push(tc(
        "UNIQUE after SORT",
        &wrap_array("UNIQUE(SORT({3,1,2,1,3},1,1))", platform),
        "array",
    ));
    cases.push(tc(
        "SORT after UNIQUE",
        &wrap_array("SORT(UNIQUE({3,1,2,1,3}),1,1)", platform),
        "array",
    ));
    cases.push(tc(
        "TRANSPOSE then BYROW sum rows",
        &wrap_array("BYROW(TRANSPOSE({1,2;3,4}),LAMBDA(r,SUM(r)))", platform),
        "array",
    ));

    cases
}

#[allow(clippy::vec_init_then_push)]
pub fn generate_filter(platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // ── FILTER ────────────────────────────────────────────────────────────────
    cases.push(tc(
        "FILTER values greater than 2",
        &wrap_array("FILTER({1,2,3,4,5},{1,2,3,4,5}>2)", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER all pass",
        &wrap_array("FILTER({10,20,30},{TRUE,TRUE,TRUE})", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER none pass returns if_empty",
        "FILTER({1,2,3},{FALSE,FALSE,FALSE},\"none\")",
        "filter",
    ));
    cases.push(tc(
        "FILTER first element only",
        &wrap_array("FILTER({10,20,30},{TRUE,FALSE,FALSE})", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER last element only",
        &wrap_array("FILTER({10,20,30},{FALSE,FALSE,TRUE})", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER alternating elements",
        &wrap_array("FILTER({1,2,3,4,5,6},{TRUE,FALSE,TRUE,FALSE,TRUE,FALSE})", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER 2D array rows",
        &wrap_array("FILTER({1,2;3,4;5,6},{TRUE,FALSE,TRUE})", platform),
        "filter",
    ));
    cases.push(tc(
        "FILTER with equality condition",
        &wrap_array("FILTER({1,2,3,2,1},{1,2,3,2,1}=2)", platform),
        "filter",
    ));
    cases.push(tc_text(
        "FILTER with if_empty string",
        "FILTER({1,2,3},{FALSE,FALSE,FALSE},\"no results\")",
        "filter",
    ));

    // ── SORT (filter category) ────────────────────────────────────────────────
    // BUG-09: SORT 1D descending
    cases.push(tc(
        "SORT 1D ascending (filter cat)",
        &wrap_array("SORT({3,1,2},1,1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORT 1D descending (BUG-09, filter cat)",
        &wrap_array("SORT({3,1,2},1,-1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORT 2D by col 1 ascending",
        &wrap_array("SORT({3,30;1,10;2,20},1,1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORT 2D by col 2 descending",
        &wrap_array("SORT({3,30;1,10;2,20},2,-1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORT strings ascending",
        &wrap_array("SORT({\"banana\",\"apple\",\"cherry\"},1,1)", platform),
        "filter",
    ));

    // ── SORTN ─────────────────────────────────────────────────────────────────
    // BUG-11: SORTN top-N
    cases.push(tc(
        "SORTN top 3 (BUG-11)",
        &wrap_array("SORTN({5,3,1,4,2},3)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTN top 2 (BUG-11)",
        &wrap_array("SORTN({5,3,1,4,2},2)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTN top 1",
        &wrap_array("SORTN({5,3,1,4,2},1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTN all elements (n=5)",
        &wrap_array("SORTN({5,3,1,4,2},5)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTN with ties mode 0",
        &wrap_array("SORTN({5,3,3,1,4,2},3,0)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTN 2D top 2 rows by first col",
        &wrap_array("SORTN({5,\"e\";3,\"c\";1,\"a\";4,\"d\";2,\"b\"},2)", platform),
        "filter",
    ));

    // ── UNIQUE (filter category) ──────────────────────────────────────────────
    // BUG-04: UNIQUE exactly_once=TRUE
    cases.push(tc(
        "UNIQUE basic dedup (filter cat)",
        &wrap_array("UNIQUE({1,2,2,3})", platform),
        "filter",
    ));
    cases.push(tc(
        "UNIQUE exactly_once=TRUE (BUG-04, filter cat)",
        &wrap_array("UNIQUE({1,2,2,3},FALSE,TRUE)", platform),
        "filter",
    ));
    cases.push(tc(
        "UNIQUE all values already unique",
        &wrap_array("UNIQUE({5,3,1,4,2})", platform),
        "filter",
    ));
    cases.push(tc(
        "UNIQUE with three duplicates exactly_once",
        &wrap_array("UNIQUE({1,1,1,2,3,3},FALSE,TRUE)", platform),
        "filter",
    ));
    cases.push(tc(
        "UNIQUE by col on 2D",
        &wrap_array("UNIQUE({1,2,1;3,4,3},TRUE,FALSE)", platform),
        "filter",
    ));

    // ── INDEX (filter category) ───────────────────────────────────────────────
    cases.push(tc_scalar(
        "INDEX 1D pick element 2 (filter cat)",
        "INDEX({10,20,30},1,2)",
        "filter",
    ));
    cases.push(tc_scalar(
        "INDEX 2D pick row 2 col 1 (filter cat)",
        "INDEX({1,2;3,4},2,1)",
        "filter",
    ));
    cases.push(tc(
        "INDEX return entire column 2",
        &wrap_array("INDEX({1,2;3,4;5,6},0,2)", platform),
        "filter",
    ));

    // ── ROWS / COLUMNS (filter category, BUG-20) ─────────────────────────────
    cases.push(tc_scalar(
        "ROWS 1D array (filter cat, BUG-20)",
        "ROWS({1,2,3,4,5})",
        "filter",
    ));
    cases.push(tc_scalar(
        "ROWS 2D array (filter cat)",
        "ROWS({1,2;3,4;5,6})",
        "filter",
    ));
    cases.push(tc_scalar(
        "ROWS column vector (filter cat, BUG-20)",
        "ROWS({1;2;3})",
        "filter",
    ));
    cases.push(tc_scalar(
        "COLUMNS 1D array (filter cat, BUG-20)",
        "COLUMNS({1,2,3,4,5})",
        "filter",
    ));
    cases.push(tc_scalar(
        "COLUMNS 2D array (filter cat)",
        "COLUMNS({1,2;3,4;5,6})",
        "filter",
    ));
    cases.push(tc_scalar(
        "COLUMNS column vector (filter cat, BUG-20)",
        "COLUMNS({1;2;3})",
        "filter",
    ));

    // ── SORTBY (filter category) ──────────────────────────────────────────────
    // BUG-10
    cases.push(tc(
        "SORTBY ascending (filter cat)",
        &wrap_array("SORTBY({3,1,2},{2,1,3},1)", platform),
        "filter",
    ));
    cases.push(tc(
        "SORTBY descending (BUG-10, filter cat)",
        &wrap_array("SORTBY({3,1,2},{2,1,3},-1)", platform),
        "filter",
    ));

    cases
}
