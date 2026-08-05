use crate::types::{Platform, TestCase};

fn tc(description: &str, formula: &str, expected_type: &str) -> TestCase {
    TestCase::new(description, formula, "", "statistical", expected_type)
}

#[allow(clippy::vec_init_then_push)]
pub fn generate(_platform: Platform) -> Vec<TestCase> {
    let mut cases: Vec<TestCase> = Vec::new();

    // -------------------------------------------------------------------------
    // AVERAGE / AVERAGEA
    // -------------------------------------------------------------------------
    cases.push(tc("AVERAGE basic", "AVERAGE(1,2,3,4,5)", "number"));
    cases.push(tc("AVERAGE single value", "AVERAGE(7)", "number"));
    cases.push(tc("AVERAGE with floats", "AVERAGE(1.5,2.5,3.5)", "number"));
    cases.push(tc("AVERAGE array", "AVERAGE({10,20,30,40})", "number"));
    cases.push(tc("AVERAGEA basic", "AVERAGEA(1,2,3,4,5)", "number"));
    cases.push(tc("AVERAGEA with zero", "AVERAGEA(0,5,10)", "number"));

    // -------------------------------------------------------------------------
    // COUNT / COUNTA / COUNTBLANK
    // -------------------------------------------------------------------------
    cases.push(tc("COUNT numbers", "COUNT(1,2,3)", "number"));
    cases.push(tc("COUNT array", "COUNT({1,2,3,4,5})", "number"));
    cases.push(tc("COUNTA values", "COUNTA(1,\"a\",TRUE)", "number"));
    cases.push(tc("COUNTBLANK empty string", "COUNTBLANK(\"\")", "number"));

    // -------------------------------------------------------------------------
    // COUNTIF / COUNTIFS
    // -------------------------------------------------------------------------
    cases.push(tc("COUNTIF greater than", "COUNTIF({1,2,3,4,5},\">3\")", "number"));
    cases.push(tc("COUNTIF equals", "COUNTIF({1,2,2,3},2)", "number"));
    cases.push(tc("COUNTIF wildcard", "COUNTIF({\"apple\",\"apricot\",\"banana\"},\"a*\")", "number"));
    cases.push(tc("COUNTIFS single range", "COUNTIFS({1,2,3,4,5},\">2\")", "number"));

    // -------------------------------------------------------------------------
    // MAX / MAXA / MIN / MINA
    // -------------------------------------------------------------------------
    cases.push(tc("MAX basic", "MAX(3,1,4,1,5,9)", "number"));
    cases.push(tc("MAX array", "MAX({-5,-3,-1})", "number"));
    cases.push(tc("MAXA basic", "MAXA(3,1,4)", "number"));
    cases.push(tc("MIN basic", "MIN(3,1,4,1,5,9)", "number"));
    cases.push(tc("MIN array", "MIN({5,10,15})", "number"));
    cases.push(tc("MINA basic", "MINA(3,1,4)", "number"));

    // -------------------------------------------------------------------------
    // MEDIAN
    // -------------------------------------------------------------------------
    cases.push(tc("MEDIAN odd count", "MEDIAN(1,3,5)", "number"));
    cases.push(tc("MEDIAN even count", "MEDIAN(1,2,3,4)", "number"));
    cases.push(tc("MEDIAN array", "MEDIAN({2,4,6,8,10})", "number"));

    // -------------------------------------------------------------------------
    // MODE / MODE.SNGL / MODE.MULT
    // -------------------------------------------------------------------------
    cases.push(tc("MODE basic", "MODE(1,2,2,3)", "number"));
    cases.push(tc("MODE.SNGL basic", "MODE.SNGL(1,2,2,3)", "number"));
    cases.push(tc("MODE.SNGL array", "MODE.SNGL({4,4,5,6})", "number"));
    cases.push(tc("MODE.MULT basic", "MODE.MULT({1,2,2,3,3})", "number"));

    // -------------------------------------------------------------------------
    // LARGE / SMALL
    // -------------------------------------------------------------------------
    cases.push(tc("LARGE k=1", "LARGE({5,3,8,1,9},1)", "number"));
    cases.push(tc("LARGE k=2", "LARGE({5,3,8,1,9},2)", "number"));
    cases.push(tc("LARGE k=3", "LARGE({5,3,8,1,9},3)", "number"));
    cases.push(tc("SMALL k=1", "SMALL({5,3,8,1,9},1)", "number"));
    cases.push(tc("SMALL k=2", "SMALL({5,3,8,1,9},2)", "number"));
    cases.push(tc("SMALL k=3", "SMALL({5,3,8,1,9},3)", "number"));

    // -------------------------------------------------------------------------
    // PERCENTILE / PERCENTILE.INC / PERCENTILE.EXC
    // -------------------------------------------------------------------------
    cases.push(tc("PERCENTILE 0.25", "PERCENTILE({1,2,3,4,5},0.25)", "number"));
    cases.push(tc("PERCENTILE 0.5", "PERCENTILE({1,2,3,4,5},0.5)", "number"));
    cases.push(tc("PERCENTILE 0.75", "PERCENTILE({1,2,3,4,5},0.75)", "number"));
    cases.push(tc("PERCENTILE.INC 0.25", "PERCENTILE.INC({1,2,3,4,5},0.25)", "number"));
    cases.push(tc("PERCENTILE.INC 0.5", "PERCENTILE.INC({1,2,3,4,5},0.5)", "number"));
    cases.push(tc("PERCENTILE.INC 0.75", "PERCENTILE.INC({1,2,3,4,5},0.75)", "number"));
    cases.push(tc("PERCENTILE.EXC 0.25", "PERCENTILE.EXC({1,2,3,4,5},0.25)", "number"));
    cases.push(tc("PERCENTILE.EXC 0.5", "PERCENTILE.EXC({1,2,3,4,5},0.5)", "number"));
    cases.push(tc("PERCENTILE.EXC 0.75", "PERCENTILE.EXC({1,2,3,4,5},0.75)", "number"));

    // -------------------------------------------------------------------------
    // PERCENTRANK / PERCENTRANK.INC / PERCENTRANK.EXC
    // -------------------------------------------------------------------------
    cases.push(tc("PERCENTRANK basic", "PERCENTRANK({1,2,3,4,5},3)", "number"));
    cases.push(tc("PERCENTRANK with significance", "PERCENTRANK({1,2,3,4,5},3,3)", "number"));
    cases.push(tc("PERCENTRANK.INC basic", "PERCENTRANK.INC({1,2,3,4,5},3)", "number"));
    cases.push(tc("PERCENTRANK.EXC basic", "PERCENTRANK.EXC({1,2,3,4,5},3)", "number"));

    // -------------------------------------------------------------------------
    // QUARTILE / QUARTILE.INC / QUARTILE.EXC
    // -------------------------------------------------------------------------
    cases.push(tc("QUARTILE Q1", "QUARTILE({1,2,3,4,5},1)", "number"));
    cases.push(tc("QUARTILE Q2", "QUARTILE({1,2,3,4,5},2)", "number"));
    cases.push(tc("QUARTILE Q3", "QUARTILE({1,2,3,4,5},3)", "number"));
    cases.push(tc("QUARTILE.INC Q1", "QUARTILE.INC({1,2,3,4,5},1)", "number"));
    cases.push(tc("QUARTILE.INC Q2", "QUARTILE.INC({1,2,3,4,5},2)", "number"));
    cases.push(tc("QUARTILE.INC Q3", "QUARTILE.INC({1,2,3,4,5},3)", "number"));
    cases.push(tc("QUARTILE.EXC Q1", "QUARTILE.EXC({1,2,3,4,5},1)", "number"));
    cases.push(tc("QUARTILE.EXC Q2", "QUARTILE.EXC({1,2,3,4,5},2)", "number"));
    cases.push(tc("QUARTILE.EXC Q3", "QUARTILE.EXC({1,2,3,4,5},3)", "number"));

    // -------------------------------------------------------------------------
    // RANK / RANK.AVG / RANK.EQ
    // -------------------------------------------------------------------------
    cases.push(tc("RANK descending", "RANK(3,{1,2,3,4,5})", "number"));
    cases.push(tc("RANK ascending", "RANK(3,{1,2,3,4,5},1)", "number"));
    cases.push(tc("RANK tied value", "RANK(2,{1,2,2,4,5})", "number"));
    cases.push(tc("RANK.AVG descending", "RANK.AVG(3,{1,2,3,4,5})", "number"));
    cases.push(tc("RANK.AVG tied value", "RANK.AVG(2,{1,2,2,4,5})", "number"));
    cases.push(tc("RANK.EQ descending", "RANK.EQ(3,{1,2,3,4,5})", "number"));
    cases.push(tc("RANK.EQ tied value", "RANK.EQ(2,{1,2,2,4,5})", "number"));

    // -------------------------------------------------------------------------
    // STDEV / STDEV.S / STDEV.P / STDEVA / STDEVPA
    // -------------------------------------------------------------------------
    cases.push(tc("STDEV basic", "STDEV(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("STDEV array", "STDEV({2,4,4,4,5,5,7,9})", "number"));
    cases.push(tc("STDEV.S basic", "STDEV.S(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("STDEV.P basic", "STDEV.P(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("STDEVA basic", "STDEVA(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("STDEVPA basic", "STDEVPA(2,4,4,4,5,5,7,9)", "number"));

    // -------------------------------------------------------------------------
    // VAR / VAR.S / VAR.P / VARA / VARPA
    // -------------------------------------------------------------------------
    cases.push(tc("VAR basic", "VAR(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("VAR array", "VAR({2,4,4,4,5,5,7,9})", "number"));
    cases.push(tc("VAR.S basic", "VAR.S(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("VAR.P basic", "VAR.P(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("VARA basic", "VARA(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("VARPA basic", "VARPA(2,4,4,4,5,5,7,9)", "number"));

    // -------------------------------------------------------------------------
    // CORREL / COVAR / COVARIANCE.P / COVARIANCE.S
    // -------------------------------------------------------------------------
    cases.push(tc("CORREL positive", "CORREL({1,2,3,4,5},{2,4,6,8,10})", "number"));
    cases.push(tc("CORREL negative", "CORREL({1,2,3,4,5},{10,8,6,4,2})", "number"));
    cases.push(tc("CORREL zero", "CORREL({1,2,3,4,5},{5,5,5,5,5})", "number"));
    cases.push(tc("COVAR basic", "COVAR({1,2,3,4,5},{2,4,6,8,10})", "number"));
    cases.push(tc("COVARIANCE.P basic", "COVARIANCE.P({1,2,3,4,5},{2,4,6,8,10})", "number"));
    cases.push(tc("COVARIANCE.S basic", "COVARIANCE.S({1,2,3,4,5},{2,4,6,8,10})", "number"));

    // -------------------------------------------------------------------------
    // FORECAST / FORECAST.LINEAR
    // -------------------------------------------------------------------------
    cases.push(tc("FORECAST x=6", "FORECAST(6,{2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("FORECAST x=3", "FORECAST(3,{2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("FORECAST.LINEAR x=6", "FORECAST.LINEAR(6,{2,4,6,8,10},{1,2,3,4,5})", "number"));

    // -------------------------------------------------------------------------
    // SLOPE / INTERCEPT / RSQ / STEYX
    // -------------------------------------------------------------------------
    cases.push(tc("SLOPE basic", "SLOPE({2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("INTERCEPT basic", "INTERCEPT({2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("RSQ basic", "RSQ({2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("STEYX basic", "STEYX({2,4,6,8,10},{1,2,3,4,5})", "number"));

    // -------------------------------------------------------------------------
    // NORM.DIST / NORMDIST
    // -------------------------------------------------------------------------
    cases.push(tc("NORM.DIST cumulative mean=0 sd=1", "NORM.DIST(1,0,1,TRUE)", "number"));
    cases.push(tc("NORM.DIST pdf mean=0 sd=1", "NORM.DIST(1,0,1,FALSE)", "number"));
    cases.push(tc("NORM.DIST cumulative mean=5 sd=2", "NORM.DIST(5,5,2,TRUE)", "number"));
    cases.push(tc("NORM.DIST pdf mean=5 sd=2", "NORM.DIST(5,5,2,FALSE)", "number"));
    cases.push(tc("NORMDIST cumulative", "NORMDIST(1,0,1,TRUE)", "number"));
    cases.push(tc("NORMDIST pdf", "NORMDIST(1,0,1,FALSE)", "number"));
    cases.push(tc("NORMDIST mean=5 sd=0.5 cumulative", "NORMDIST(5,5,0.5,TRUE)", "number"));

    // -------------------------------------------------------------------------
    // NORM.INV / NORMINV
    // -------------------------------------------------------------------------
    cases.push(tc("NORM.INV p=0.1", "NORM.INV(0.1,0,1)", "number"));
    cases.push(tc("NORM.INV p=0.5", "NORM.INV(0.5,0,1)", "number"));
    cases.push(tc("NORM.INV p=0.9", "NORM.INV(0.9,0,1)", "number"));
    cases.push(tc("NORM.INV mean=5 sd=2 p=0.5", "NORM.INV(0.5,5,2)", "number"));
    cases.push(tc("NORMINV p=0.1", "NORMINV(0.1,0,1)", "number"));
    cases.push(tc("NORMINV p=0.5", "NORMINV(0.5,0,1)", "number"));
    cases.push(tc("NORMINV p=0.9", "NORMINV(0.9,0,1)", "number"));

    // -------------------------------------------------------------------------
    // NORM.S.DIST / NORMSDIST
    // -------------------------------------------------------------------------
    cases.push(tc("NORM.S.DIST z=0 cumulative", "NORM.S.DIST(0,TRUE)", "number"));
    cases.push(tc("NORM.S.DIST z=1 cumulative", "NORM.S.DIST(1,TRUE)", "number"));
    cases.push(tc("NORM.S.DIST z=-1 cumulative", "NORM.S.DIST(-1,TRUE)", "number"));
    cases.push(tc("NORM.S.DIST z=0 pdf", "NORM.S.DIST(0,FALSE)", "number"));
    cases.push(tc("NORMSDIST z=0", "NORMSDIST(0)", "number"));
    cases.push(tc("NORMSDIST z=1", "NORMSDIST(1)", "number"));
    cases.push(tc("NORMSDIST z=-1", "NORMSDIST(-1)", "number"));

    // -------------------------------------------------------------------------
    // NORM.S.INV / NORMSINV
    // -------------------------------------------------------------------------
    cases.push(tc("NORM.S.INV p=0.1", "NORM.S.INV(0.1)", "number"));
    cases.push(tc("NORM.S.INV p=0.5", "NORM.S.INV(0.5)", "number"));
    cases.push(tc("NORM.S.INV p=0.9", "NORM.S.INV(0.9)", "number"));
    cases.push(tc("NORMSINV p=0.1", "NORMSINV(0.1)", "number"));
    cases.push(tc("NORMSINV p=0.5", "NORMSINV(0.5)", "number"));
    cases.push(tc("NORMSINV p=0.9", "NORMSINV(0.9)", "number"));

    // -------------------------------------------------------------------------
    // T.DIST / T.DIST.2T / T.DIST.RT
    // -------------------------------------------------------------------------
    cases.push(tc("T.DIST df=1 cumulative", "T.DIST(1,1,TRUE)", "number"));
    cases.push(tc("T.DIST df=5 cumulative", "T.DIST(1,5,TRUE)", "number"));
    cases.push(tc("T.DIST df=10 cumulative", "T.DIST(1,10,TRUE)", "number"));
    cases.push(tc("T.DIST df=30 cumulative", "T.DIST(1,30,TRUE)", "number"));
    cases.push(tc("T.DIST df=5 pdf", "T.DIST(1,5,FALSE)", "number"));
    cases.push(tc("T.DIST.2T df=1", "T.DIST.2T(1,1)", "number"));
    cases.push(tc("T.DIST.2T df=5", "T.DIST.2T(1,5)", "number"));
    cases.push(tc("T.DIST.2T df=10", "T.DIST.2T(1,10)", "number"));
    cases.push(tc("T.DIST.2T df=30", "T.DIST.2T(1,30)", "number"));
    cases.push(tc("T.DIST.RT df=1", "T.DIST.RT(1,1)", "number"));
    cases.push(tc("T.DIST.RT df=5", "T.DIST.RT(1,5)", "number"));
    cases.push(tc("T.DIST.RT df=10", "T.DIST.RT(1,10)", "number"));
    cases.push(tc("T.DIST.RT df=30", "T.DIST.RT(1,30)", "number"));

    // -------------------------------------------------------------------------
    // T.INV / T.INV.2T / TINV
    // -------------------------------------------------------------------------
    cases.push(tc("T.INV p=0.1 df=1", "T.INV(0.1,1)", "number"));
    cases.push(tc("T.INV p=0.1 df=5", "T.INV(0.1,5)", "number"));
    cases.push(tc("T.INV p=0.1 df=10", "T.INV(0.1,10)", "number"));
    cases.push(tc("T.INV p=0.1 df=30", "T.INV(0.1,30)", "number"));
    cases.push(tc("T.INV p=0.5 df=5", "T.INV(0.5,5)", "number"));
    cases.push(tc("T.INV p=0.9 df=5", "T.INV(0.9,5)", "number"));
    cases.push(tc("T.INV.2T p=0.1 df=1", "T.INV.2T(0.1,1)", "number"));
    cases.push(tc("T.INV.2T p=0.1 df=5", "T.INV.2T(0.1,5)", "number"));
    cases.push(tc("T.INV.2T p=0.1 df=10", "T.INV.2T(0.1,10)", "number"));
    cases.push(tc("T.INV.2T p=0.1 df=30", "T.INV.2T(0.1,30)", "number"));
    cases.push(tc("T.INV.2T p=0.5 df=5", "T.INV.2T(0.5,5)", "number"));
    cases.push(tc("T.INV.2T p=0.9 df=5", "T.INV.2T(0.9,5)", "number"));
    cases.push(tc("TINV p=0.1 df=5", "TINV(0.1,5)", "number"));
    cases.push(tc("TINV p=0.5 df=5", "TINV(0.5,5)", "number"));
    cases.push(tc("TINV p=0.9 df=5", "TINV(0.9,5)", "number"));

    // -------------------------------------------------------------------------
    // T.TEST / TTEST — BUG-03: constant-difference paired test
    // -------------------------------------------------------------------------
    cases.push(tc(
        "T.TEST constant-difference paired BUG-03",
        "T.TEST({1,2,3},{4,5,6},2,1)",
        "number",
    ));
    cases.push(tc(
        "T.TEST non-constant paired should work",
        "T.TEST({1,2,3},{1,2,4},2,1)",
        "number",
    ));
    cases.push(tc(
        "T.TEST two-sample equal-variance tails=2 type=2",
        "T.TEST({1,2,3,4,5},{2,3,4,5,6},2,2)",
        "number",
    ));
    cases.push(tc(
        "T.TEST two-sample unequal-variance tails=2 type=3",
        "T.TEST({1,2,3,4,5},{2,3,4,5,6},2,3)",
        "number",
    ));
    cases.push(tc(
        "T.TEST one-tailed paired",
        "T.TEST({1,2,3},{4,5,6},1,1)",
        "number",
    ));
    cases.push(tc(
        "TTEST constant-difference paired alias BUG-03",
        "TTEST({1,2,3},{4,5,6},2,1)",
        "number",
    ));
    cases.push(tc(
        "TTEST two-sample equal-variance alias",
        "TTEST({1,2,3,4,5},{2,3,4,5,6},2,2)",
        "number",
    ));

    // -------------------------------------------------------------------------
    // F.DIST / F.DIST.RT / FDIST
    // -------------------------------------------------------------------------
    cases.push(tc("F.DIST cumulative df1=1 df2=5", "F.DIST(1,1,5,TRUE)", "number"));
    cases.push(tc("F.DIST cumulative df1=5 df2=10", "F.DIST(1,5,10,TRUE)", "number"));
    cases.push(tc("F.DIST pdf df1=5 df2=10", "F.DIST(1,5,10,FALSE)", "number"));
    cases.push(tc("F.DIST.RT df1=1 df2=5", "F.DIST.RT(1,1,5)", "number"));
    cases.push(tc("F.DIST.RT df1=5 df2=10", "F.DIST.RT(1,5,10)", "number"));
    cases.push(tc("F.DIST.RT df1=10 df2=30", "F.DIST.RT(2,10,30)", "number"));
    cases.push(tc("FDIST df1=5 df2=10", "FDIST(1,5,10)", "number"));
    cases.push(tc("FDIST df1=1 df2=5", "FDIST(1,1,5)", "number"));

    // -------------------------------------------------------------------------
    // F.INV / F.INV.RT / FINV
    // -------------------------------------------------------------------------
    cases.push(tc("F.INV p=0.1 df1=1 df2=5", "F.INV(0.1,1,5)", "number"));
    cases.push(tc("F.INV p=0.5 df1=5 df2=10", "F.INV(0.5,5,10)", "number"));
    cases.push(tc("F.INV p=0.9 df1=10 df2=30", "F.INV(0.9,10,30)", "number"));
    cases.push(tc("F.INV.RT p=0.1 df1=1 df2=5", "F.INV.RT(0.1,1,5)", "number"));
    cases.push(tc("F.INV.RT p=0.5 df1=5 df2=10", "F.INV.RT(0.5,5,10)", "number"));
    cases.push(tc("F.INV.RT p=0.9 df1=10 df2=30", "F.INV.RT(0.9,10,30)", "number"));
    cases.push(tc("FINV p=0.1 df1=5 df2=10", "FINV(0.1,5,10)", "number"));
    cases.push(tc("FINV p=0.5 df1=5 df2=10", "FINV(0.5,5,10)", "number"));

    // -------------------------------------------------------------------------
    // F.TEST / FTEST
    // -------------------------------------------------------------------------
    cases.push(tc("F.TEST basic", "F.TEST({1,2,3,4,5},{2,3,4,5,6})", "number"));
    cases.push(tc("F.TEST different variance", "F.TEST({1,2,3,4,5},{1,4,9,16,25})", "number"));
    cases.push(tc("FTEST basic", "FTEST({1,2,3,4,5},{2,3,4,5,6})", "number"));

    // -------------------------------------------------------------------------
    // CHISQ.DIST / CHISQ.DIST.RT / CHIDIST
    // -------------------------------------------------------------------------
    cases.push(tc("CHISQ.DIST cumulative df=1", "CHISQ.DIST(1,1,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST cumulative df=5", "CHISQ.DIST(1,5,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST cumulative df=10", "CHISQ.DIST(5,10,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST pdf df=5", "CHISQ.DIST(1,5,FALSE)", "number"));
    cases.push(tc("CHISQ.DIST.RT df=1", "CHISQ.DIST.RT(1,1)", "number"));
    cases.push(tc("CHISQ.DIST.RT df=5", "CHISQ.DIST.RT(1,5)", "number"));
    cases.push(tc("CHISQ.DIST.RT df=10", "CHISQ.DIST.RT(5,10)", "number"));
    cases.push(tc("CHISQ.DIST.RT df=30", "CHISQ.DIST.RT(10,30)", "number"));
    cases.push(tc("CHIDIST df=5", "CHIDIST(1,5)", "number"));
    cases.push(tc("CHIDIST df=10", "CHIDIST(5,10)", "number"));

    // -------------------------------------------------------------------------
    // CHISQ.INV / CHISQ.INV.RT / CHIINV
    // -------------------------------------------------------------------------
    cases.push(tc("CHISQ.INV p=0.1 df=1", "CHISQ.INV(0.1,1)", "number"));
    cases.push(tc("CHISQ.INV p=0.5 df=5", "CHISQ.INV(0.5,5)", "number"));
    cases.push(tc("CHISQ.INV p=0.9 df=10", "CHISQ.INV(0.9,10)", "number"));
    cases.push(tc("CHISQ.INV p=0.9 df=30", "CHISQ.INV(0.9,30)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.1 df=1", "CHISQ.INV.RT(0.1,1)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.5 df=5", "CHISQ.INV.RT(0.5,5)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.9 df=10", "CHISQ.INV.RT(0.9,10)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.9 df=30", "CHISQ.INV.RT(0.9,30)", "number"));
    cases.push(tc("CHIINV p=0.1 df=5", "CHIINV(0.1,5)", "number"));
    cases.push(tc("CHIINV p=0.5 df=10", "CHIINV(0.5,10)", "number"));

    // -------------------------------------------------------------------------
    // CHISQ.TEST / CHITEST
    // -------------------------------------------------------------------------
    cases.push(tc(
        "CHISQ.TEST basic",
        "CHISQ.TEST({10,20,30},{15,15,30})",
        "number",
    ));
    cases.push(tc(
        "CHITEST basic",
        "CHITEST({10,20,30},{15,15,30})",
        "number",
    ));

    // -------------------------------------------------------------------------
    // BETA.DIST / BETA.INV / BETADIST / BETAINV
    // -------------------------------------------------------------------------
    cases.push(tc("BETA.DIST cumulative a=2 b=2", "BETA.DIST(0.5,2,2,TRUE)", "number"));
    cases.push(tc("BETA.DIST cumulative a=2 b=5", "BETA.DIST(0.5,2,5,TRUE)", "number"));
    cases.push(tc("BETA.DIST pdf a=2 b=2", "BETA.DIST(0.5,2,2,FALSE)", "number"));
    cases.push(tc("BETA.DIST 5-arg cumulative", "BETA.DIST(0.5,2,2,TRUE,0,1)", "number"));
    cases.push(tc("BETA.DIST 5-arg pdf", "BETA.DIST(0.5,2,2,FALSE,0,1)", "number"));
    cases.push(tc("BETADIST 3-arg", "BETADIST(0.5,2,2)", "number"));
    cases.push(tc("BETADIST 5-arg", "BETADIST(0.5,2,2,0,1)", "number"));
    // BUG-19: BETA.INV 3-arg form with default lo/hi
    cases.push(tc(
        "BETA.INV 3-arg default lo/hi BUG-19",
        "BETA.INV(0.5,2,2)",
        "number",
    ));
    cases.push(tc(
        "BETA.INV 5-arg explicit should work",
        "BETA.INV(0.5,2,2,0,1)",
        "number",
    ));
    cases.push(tc("BETA.INV p=0.1 a=1 b=2", "BETA.INV(0.1,1,2)", "number"));
    cases.push(tc("BETA.INV p=0.9 a=5 b=2", "BETA.INV(0.9,5,2)", "number"));
    // BETAINV alias
    cases.push(tc("BETAINV alias 5-arg", "BETAINV(0.5,2,2,0,1)", "number"));
    cases.push(tc("BETAINV p=0.1 a=2 b=5", "BETAINV(0.1,2,5,0,1)", "number"));
    cases.push(tc("BETAINV p=0.9 a=5 b=1", "BETAINV(0.9,5,1,0,1)", "number"));

    // -------------------------------------------------------------------------
    // BINOM.DIST / BINOM.INV / BINOMDIST / CRITBINOM
    // -------------------------------------------------------------------------
    cases.push(tc("BINOM.DIST exact n=10 k=5 p=0.5", "BINOM.DIST(5,10,0.5,FALSE)", "number"));
    cases.push(tc("BINOM.DIST cumulative n=10 k=5 p=0.5", "BINOM.DIST(5,10,0.5,TRUE)", "number"));
    cases.push(tc("BINOM.DIST exact n=20 k=10 p=0.5", "BINOM.DIST(10,20,0.5,FALSE)", "number"));
    cases.push(tc("BINOM.DIST exact n=5 k=2 p=0.1", "BINOM.DIST(2,5,0.1,FALSE)", "number"));
    cases.push(tc("BINOM.DIST exact n=5 k=4 p=0.9", "BINOM.DIST(4,5,0.9,FALSE)", "number"));
    cases.push(tc("BINOM.INV n=10 p=0.5 alpha=0.1", "BINOM.INV(10,0.5,0.1)", "number"));
    cases.push(tc("BINOM.INV n=20 p=0.5 alpha=0.5", "BINOM.INV(20,0.5,0.5)", "number"));
    cases.push(tc("BINOM.INV n=10 p=0.1 alpha=0.9", "BINOM.INV(10,0.1,0.9)", "number"));
    cases.push(tc("BINOMDIST exact n=10 k=5 p=0.5", "BINOMDIST(5,10,0.5,FALSE)", "number"));
    cases.push(tc("BINOMDIST cumulative n=10 k=5 p=0.5", "BINOMDIST(5,10,0.5,TRUE)", "number"));
    cases.push(tc("CRITBINOM n=10 p=0.5 alpha=0.1", "CRITBINOM(10,0.5,0.1)", "number"));
    cases.push(tc("CRITBINOM n=20 p=0.5 alpha=0.9", "CRITBINOM(20,0.5,0.9)", "number"));

    // -------------------------------------------------------------------------
    // NEGBINOM.DIST / NEGBINOMDIST
    // -------------------------------------------------------------------------
    cases.push(tc("NEGBINOM.DIST exact k=5 r=1 p=0.5", "NEGBINOM.DIST(5,1,0.5,FALSE)", "number"));
    cases.push(tc("NEGBINOM.DIST cumulative k=5 r=1 p=0.5", "NEGBINOM.DIST(5,1,0.5,TRUE)", "number"));
    cases.push(tc("NEGBINOM.DIST exact k=10 r=5 p=0.1", "NEGBINOM.DIST(10,5,0.1,FALSE)", "number"));
    cases.push(tc("NEGBINOM.DIST exact k=5 r=2 p=0.9", "NEGBINOM.DIST(5,2,0.9,FALSE)", "number"));
    cases.push(tc("NEGBINOMDIST k=5 r=1 p=0.5", "NEGBINOMDIST(5,1,0.5)", "number"));
    cases.push(tc("NEGBINOMDIST k=10 r=5 p=0.1", "NEGBINOMDIST(10,5,0.1)", "number"));

    // -------------------------------------------------------------------------
    // POISSON.DIST / POISSON
    // -------------------------------------------------------------------------
    cases.push(tc("POISSON.DIST exact lambda=5 k=5", "POISSON.DIST(5,5,FALSE)", "number"));
    cases.push(tc("POISSON.DIST cumulative lambda=5 k=5", "POISSON.DIST(5,5,TRUE)", "number"));
    cases.push(tc("POISSON.DIST exact lambda=1 k=0", "POISSON.DIST(0,1,FALSE)", "number"));
    cases.push(tc("POISSON.DIST exact lambda=10 k=10", "POISSON.DIST(10,10,FALSE)", "number"));
    cases.push(tc("POISSON exact lambda=5 k=5", "POISSON(5,5,FALSE)", "number"));
    cases.push(tc("POISSON cumulative lambda=5 k=5", "POISSON(5,5,TRUE)", "number"));
    cases.push(tc("POISSON exact lambda=1 k=0", "POISSON(0,1,FALSE)", "number"));

    // -------------------------------------------------------------------------
    // HYPGEOM.DIST / HYPGEOMDIST
    // -------------------------------------------------------------------------
    cases.push(tc(
        "HYPGEOM.DIST exact s=2 ns=5 M=10 N=20",
        "HYPGEOM.DIST(2,5,10,20,FALSE)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOM.DIST cumulative s=2 ns=5 M=10 N=20",
        "HYPGEOM.DIST(2,5,10,20,TRUE)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOM.DIST exact s=1 ns=5 M=5 N=20",
        "HYPGEOM.DIST(1,5,5,20,FALSE)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOMDIST s=2 ns=5 M=10 N=20",
        "HYPGEOMDIST(2,5,10,20)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOMDIST s=1 ns=5 M=5 N=20",
        "HYPGEOMDIST(1,5,5,20)",
        "number",
    ));

    // -------------------------------------------------------------------------
    // EXPON.DIST / EXPONDIST
    // -------------------------------------------------------------------------
    cases.push(tc("EXPON.DIST cumulative lambda=1 x=1", "EXPON.DIST(1,1,TRUE)", "number"));
    cases.push(tc("EXPON.DIST pdf lambda=1 x=1", "EXPON.DIST(1,1,FALSE)", "number"));
    cases.push(tc("EXPON.DIST cumulative lambda=2 x=0.5", "EXPON.DIST(0.5,2,TRUE)", "number"));
    cases.push(tc("EXPON.DIST cumulative lambda=5 x=0.1", "EXPON.DIST(0.1,5,TRUE)", "number"));
    cases.push(tc("EXPONDIST cumulative lambda=1 x=1", "EXPONDIST(1,1,TRUE)", "number"));
    cases.push(tc("EXPONDIST pdf lambda=1 x=1", "EXPONDIST(1,1,FALSE)", "number"));

    // -------------------------------------------------------------------------
    // GAMMA.DIST / GAMMADIST
    // -------------------------------------------------------------------------
    cases.push(tc("GAMMA.DIST cumulative a=1 b=1", "GAMMA.DIST(1,1,1,TRUE)", "number"));
    cases.push(tc("GAMMA.DIST pdf a=1 b=1", "GAMMA.DIST(1,1,1,FALSE)", "number"));
    cases.push(tc("GAMMA.DIST cumulative a=2 b=1", "GAMMA.DIST(1,2,1,TRUE)", "number"));
    cases.push(tc("GAMMA.DIST cumulative a=5 b=2", "GAMMA.DIST(5,5,2,TRUE)", "number"));
    cases.push(tc("GAMMADIST cumulative a=1 b=1", "GAMMADIST(1,1,1,TRUE)", "number"));
    cases.push(tc("GAMMADIST pdf a=2 b=1", "GAMMADIST(1,2,1,FALSE)", "number"));

    // -------------------------------------------------------------------------
    // GAMMA.INV / GAMMAINV
    // -------------------------------------------------------------------------
    cases.push(tc("GAMMA.INV p=0.1 a=1 b=1", "GAMMA.INV(0.1,1,1)", "number"));
    cases.push(tc("GAMMA.INV p=0.5 a=2 b=1", "GAMMA.INV(0.5,2,1)", "number"));
    cases.push(tc("GAMMA.INV p=0.9 a=5 b=2", "GAMMA.INV(0.9,5,2)", "number"));
    cases.push(tc("GAMMAINV p=0.1 a=1 b=1", "GAMMAINV(0.1,1,1)", "number"));
    cases.push(tc("GAMMAINV p=0.5 a=2 b=1", "GAMMAINV(0.5,2,1)", "number"));

    // -------------------------------------------------------------------------
    // GAMMALN / GAMMALN.PRECISE
    // -------------------------------------------------------------------------
    cases.push(tc("GAMMALN x=1", "GAMMALN(1)", "number"));
    cases.push(tc("GAMMALN x=2", "GAMMALN(2)", "number"));
    cases.push(tc("GAMMALN x=5", "GAMMALN(5)", "number"));
    cases.push(tc("GAMMALN x=10", "GAMMALN(10)", "number"));
    cases.push(tc("GAMMALN.PRECISE x=1", "GAMMALN.PRECISE(1)", "number"));
    cases.push(tc("GAMMALN.PRECISE x=5", "GAMMALN.PRECISE(5)", "number"));

    // -------------------------------------------------------------------------
    // LOGNORM.DIST / LOGNORMDIST
    // -------------------------------------------------------------------------
    cases.push(tc("LOGNORM.DIST cumulative mu=0 sd=1", "LOGNORM.DIST(1,0,1,TRUE)", "number"));
    cases.push(tc("LOGNORM.DIST pdf mu=0 sd=1", "LOGNORM.DIST(1,0,1,FALSE)", "number"));
    cases.push(tc("LOGNORM.DIST cumulative mu=0 sd=0.5", "LOGNORM.DIST(1,0,0.5,TRUE)", "number"));
    cases.push(tc("LOGNORM.DIST cumulative mu=0 sd=2", "LOGNORM.DIST(1,0,2,TRUE)", "number"));
    cases.push(tc("LOGNORMDIST mu=0 sd=1", "LOGNORMDIST(1,0,1)", "number"));
    cases.push(tc("LOGNORMDIST mu=0 sd=0.5", "LOGNORMDIST(1,0,0.5)", "number"));

    // -------------------------------------------------------------------------
    // LOGNORM.INV / LOGINV
    // -------------------------------------------------------------------------
    cases.push(tc("LOGNORM.INV p=0.1 mu=0 sd=1", "LOGNORM.INV(0.1,0,1)", "number"));
    cases.push(tc("LOGNORM.INV p=0.5 mu=0 sd=1", "LOGNORM.INV(0.5,0,1)", "number"));
    cases.push(tc("LOGNORM.INV p=0.9 mu=0 sd=1", "LOGNORM.INV(0.9,0,1)", "number"));
    cases.push(tc("LOGINV p=0.1 mu=0 sd=1", "LOGINV(0.1,0,1)", "number"));
    cases.push(tc("LOGINV p=0.5 mu=0 sd=1", "LOGINV(0.5,0,1)", "number"));
    cases.push(tc("LOGINV p=0.9 mu=0 sd=1", "LOGINV(0.9,0,1)", "number"));

    // -------------------------------------------------------------------------
    // WEIBULL.DIST / WEIBULL
    // -------------------------------------------------------------------------
    cases.push(tc("WEIBULL.DIST cumulative a=1 b=1", "WEIBULL.DIST(1,1,1,TRUE)", "number"));
    cases.push(tc("WEIBULL.DIST pdf a=1 b=1", "WEIBULL.DIST(1,1,1,FALSE)", "number"));
    cases.push(tc("WEIBULL.DIST cumulative a=2 b=1", "WEIBULL.DIST(1,2,1,TRUE)", "number"));
    cases.push(tc("WEIBULL.DIST cumulative a=5 b=2", "WEIBULL.DIST(2,5,2,TRUE)", "number"));
    cases.push(tc("WEIBULL cumulative a=1 b=1", "WEIBULL(1,1,1,TRUE)", "number"));
    cases.push(tc("WEIBULL pdf a=2 b=1", "WEIBULL(1,2,1,FALSE)", "number"));

    // -------------------------------------------------------------------------
    // DEVSQ / AVEDEV
    // -------------------------------------------------------------------------
    cases.push(tc("DEVSQ basic", "DEVSQ(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("DEVSQ array", "DEVSQ({1,2,3,4,5})", "number"));
    cases.push(tc("AVEDEV basic", "AVEDEV(2,4,4,4,5,5,7,9)", "number"));
    cases.push(tc("AVEDEV array", "AVEDEV({1,2,3,4,5})", "number"));

    // -------------------------------------------------------------------------
    // GEOMEAN / HARMEAN
    // -------------------------------------------------------------------------
    cases.push(tc("GEOMEAN basic", "GEOMEAN(1,2,4,8)", "number"));
    cases.push(tc("GEOMEAN array", "GEOMEAN({2,8,18,32})", "number"));
    cases.push(tc("HARMEAN basic", "HARMEAN(1,2,4)", "number"));
    cases.push(tc("HARMEAN array", "HARMEAN({2,4,8})", "number"));

    // -------------------------------------------------------------------------
    // KURT / SKEW / SKEW.P
    // -------------------------------------------------------------------------
    cases.push(tc("KURT basic", "KURT(3,4,5,2,3,4,5,6,4,7)", "number"));
    cases.push(tc("KURT array", "KURT({3,4,5,2,3,4,5,6,4,7})", "number"));
    cases.push(tc("SKEW basic", "SKEW(3,4,5,2,3,4,5,6,4,7)", "number"));
    cases.push(tc("SKEW array", "SKEW({3,4,5,2,3,4,5,6,4,7})", "number"));
    cases.push(tc("SKEW.P basic", "SKEW.P(3,4,5,2,3,4,5,6,4,7)", "number"));
    cases.push(tc("SKEW.P array", "SKEW.P({3,4,5,2,3,4,5,6,4,7})", "number"));

    // -------------------------------------------------------------------------
    // TRIMMEAN
    // -------------------------------------------------------------------------
    cases.push(tc("TRIMMEAN 10%", "TRIMMEAN({1,2,3,4,5,6,7,8,9,10},0.1)", "number"));
    cases.push(tc("TRIMMEAN 20%", "TRIMMEAN({1,2,3,4,5,6,7,8,9,10},0.2)", "number"));
    cases.push(tc("TRIMMEAN 40%", "TRIMMEAN({1,2,3,4,5,6,7,8,9,10},0.4)", "number"));

    // -------------------------------------------------------------------------
    // PROB
    // -------------------------------------------------------------------------
    cases.push(tc(
        "PROB x=[1,2,3] prob=[0.2,0.5,0.3] lower=1 upper=2",
        "PROB({1,2,3},{0.2,0.5,0.3},1,2)",
        "number",
    ));
    cases.push(tc(
        "PROB x=[1,2,3] prob=[0.2,0.5,0.3] lower=2",
        "PROB({1,2,3},{0.2,0.5,0.3},2)",
        "number",
    ));

    // -------------------------------------------------------------------------
    // FREQUENCY
    // -------------------------------------------------------------------------
    cases.push(tc(
        "FREQUENCY basic",
        "FREQUENCY({1,2,3,4,5,6},{2,4})",
        "number",
    ));
    cases.push(tc(
        "FREQUENCY data=[79,85,78,85,50] bins=[70,79,89]",
        "FREQUENCY({79,85,78,85,50},{70,79,89})",
        "number",
    ));

    // -------------------------------------------------------------------------
    // GROWTH / LOGEST / LINEST
    // -------------------------------------------------------------------------
    cases.push(tc(
        "GROWTH basic",
        "GROWTH({2,4,8,16},{1,2,3,4})",
        "number",
    ));
    cases.push(tc(
        "LOGEST basic",
        "LOGEST({2,4,8,16},{1,2,3,4})",
        "number",
    ));
    cases.push(tc(
        "LINEST basic",
        "LINEST({2,4,6,8},{1,2,3,4})",
        "number",
    ));

    // -------------------------------------------------------------------------
    // ZTEST / Z.TEST
    // -------------------------------------------------------------------------
    cases.push(tc("ZTEST basic mu=3", "ZTEST({1,2,3,4,5},3)", "number"));
    cases.push(tc("ZTEST mu=3 sigma=1", "ZTEST({1,2,3,4,5},3,1)", "number"));
    cases.push(tc("ZTEST mu=3 sigma=2", "ZTEST({1,2,3,4,5},3,2)", "number"));
    cases.push(tc("Z.TEST basic mu=3", "Z.TEST({1,2,3,4,5},3)", "number"));
    cases.push(tc("Z.TEST mu=3 sigma=1", "Z.TEST({1,2,3,4,5},3,1)", "number"));
    cases.push(tc("Z.TEST mu=3 sigma=2", "Z.TEST({1,2,3,4,5},3,2)", "number"));

    // =========================================================================
    // Additional parameter sweep cases for systematic-D coverage
    // =========================================================================

    // AVERAGE additional data sets
    cases.push(tc("AVERAGE negative values", "AVERAGE(-3,-1,0,1,3)", "number"));
    cases.push(tc("AVERAGE large values", "AVERAGE(100,200,300,400,500)", "number"));
    cases.push(tc("AVERAGE decimal result", "AVERAGE(1,2)", "number"));
    cases.push(tc("AVERAGEA with boolean TRUE", "AVERAGEA(TRUE,3,5)", "number"));

    // COUNT/COUNTA additional
    cases.push(tc("COUNT no numbers in mix", "COUNT(\"a\",\"b\",\"c\")", "number"));
    cases.push(tc("COUNT large array", "COUNT({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("COUNTA with numbers and text", "COUNTA(1,2,\"hello\",\"world\")", "number"));

    // COUNTIF additional
    cases.push(tc("COUNTIF less than", "COUNTIF({1,2,3,4,5},\"<3\")", "number"));
    cases.push(tc("COUNTIF not equal", "COUNTIF({1,2,3,4,5},\"<>3\")", "number"));
    cases.push(tc("COUNTIF text match", "COUNTIF({\"a\",\"b\",\"a\",\"c\"},\"a\")", "number"));

    // MAX/MIN additional
    cases.push(tc("MAX single value", "MAX(42)", "number"));
    cases.push(tc("MAX negative values", "MAX(-5,-3,-1,-10)", "number"));
    cases.push(tc("MIN single value", "MIN(42)", "number"));
    cases.push(tc("MIN positive values", "MIN(5,3,1,10)", "number"));

    // MEDIAN additional
    cases.push(tc("MEDIAN two values", "MEDIAN(3,7)", "number"));
    cases.push(tc("MEDIAN large dataset", "MEDIAN({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("MEDIAN with duplicates", "MEDIAN({1,1,2,2,3})", "number"));

    // MODE additional
    cases.push(tc("MODE multiple modes returns first", "MODE(1,2,2,3,3,4)", "number"));
    cases.push(tc("MODE.SNGL with ties", "MODE.SNGL({5,5,6,6,7})", "number"));

    // LARGE/SMALL additional
    cases.push(tc("LARGE k=4", "LARGE({5,3,8,1,9,2,7},4)", "number"));
    cases.push(tc("LARGE k=5", "LARGE({5,3,8,1,9,2,7},5)", "number"));
    cases.push(tc("SMALL k=4", "SMALL({5,3,8,1,9,2,7},4)", "number"));
    cases.push(tc("SMALL k=5", "SMALL({5,3,8,1,9,2,7},5)", "number"));

    // PERCENTILE additional parameter sweep
    cases.push(tc("PERCENTILE 0.1", "PERCENTILE({1,2,3,4,5,6,7,8,9,10},0.1)", "number"));
    cases.push(tc("PERCENTILE 0.9", "PERCENTILE({1,2,3,4,5,6,7,8,9,10},0.9)", "number"));
    cases.push(tc("PERCENTILE.INC 0.1", "PERCENTILE.INC({1,2,3,4,5,6,7,8,9,10},0.1)", "number"));
    cases.push(tc("PERCENTILE.INC 0.9", "PERCENTILE.INC({1,2,3,4,5,6,7,8,9,10},0.9)", "number"));
    cases.push(tc("PERCENTILE.EXC 0.1", "PERCENTILE.EXC({1,2,3,4,5,6,7,8,9,10},0.1)", "number"));
    cases.push(tc("PERCENTILE.EXC 0.9", "PERCENTILE.EXC({1,2,3,4,5,6,7,8,9,10},0.9)", "number"));

    // PERCENTRANK additional
    cases.push(tc("PERCENTRANK.INC x=1", "PERCENTRANK.INC({1,2,3,4,5},1)", "number"));
    cases.push(tc("PERCENTRANK.INC x=5", "PERCENTRANK.INC({1,2,3,4,5},5)", "number"));
    cases.push(tc("PERCENTRANK.EXC x=1", "PERCENTRANK.EXC({1,2,3,4,5},1)", "number"));
    cases.push(tc("PERCENTRANK.EXC x=5", "PERCENTRANK.EXC({1,2,3,4,5},5)", "number"));

    // QUARTILE additional
    cases.push(tc("QUARTILE Q0 min", "QUARTILE({1,2,3,4,5},0)", "number"));
    cases.push(tc("QUARTILE Q4 max", "QUARTILE({1,2,3,4,5},4)", "number"));
    cases.push(tc("QUARTILE.INC Q0", "QUARTILE.INC({1,2,3,4,5},0)", "number"));
    cases.push(tc("QUARTILE.INC Q4", "QUARTILE.INC({1,2,3,4,5},4)", "number"));

    // RANK additional
    cases.push(tc("RANK value=1 ascending", "RANK(1,{1,2,3,4,5},1)", "number"));
    cases.push(tc("RANK value=5 ascending", "RANK(5,{1,2,3,4,5},1)", "number"));
    cases.push(tc("RANK.AVG ascending", "RANK.AVG(2,{1,2,2,4,5},1)", "number"));
    cases.push(tc("RANK.EQ ascending", "RANK.EQ(2,{1,2,2,4,5},1)", "number"));

    // STDEV additional datasets
    cases.push(tc("STDEV.S single spread", "STDEV.S(1,2,3,4,5)", "number"));
    cases.push(tc("STDEV.P single spread", "STDEV.P(1,2,3,4,5)", "number"));
    cases.push(tc("STDEV identical values", "STDEV(5,5,5,5,5)", "number"));

    // VAR additional
    cases.push(tc("VAR.S single spread", "VAR.S(1,2,3,4,5)", "number"));
    cases.push(tc("VAR.P single spread", "VAR.P(1,2,3,4,5)", "number"));

    // CORREL additional
    cases.push(tc("CORREL perfect negative", "CORREL({1,2,3},{3,2,1})", "number"));
    cases.push(tc("CORREL perfect positive", "CORREL({1,2,3},{2,4,6})", "number"));

    // COVARIANCE additional
    cases.push(tc("COVARIANCE.P negative", "COVARIANCE.P({1,2,3},{3,2,1})", "number"));
    cases.push(tc("COVARIANCE.S negative", "COVARIANCE.S({1,2,3},{3,2,1})", "number"));

    // FORECAST additional x values
    cases.push(tc("FORECAST x=1", "FORECAST(1,{2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("FORECAST x=10", "FORECAST(10,{2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("FORECAST.LINEAR x=1", "FORECAST.LINEAR(1,{2,4,6,8,10},{1,2,3,4,5})", "number"));
    cases.push(tc("FORECAST.LINEAR x=10", "FORECAST.LINEAR(10,{2,4,6,8,10},{1,2,3,4,5})", "number"));

    // NORM.DIST additional x/mean/sd sweeps
    cases.push(tc("NORM.DIST x=-1 cumulative", "NORM.DIST(-1,0,1,TRUE)", "number"));
    cases.push(tc("NORM.DIST x=0 cumulative", "NORM.DIST(0,0,1,TRUE)", "number"));
    cases.push(tc("NORM.DIST x=2 cumulative", "NORM.DIST(2,0,1,TRUE)", "number"));
    cases.push(tc("NORM.DIST sd=0.5 x=5", "NORM.DIST(5,5,0.5,TRUE)", "number"));
    cases.push(tc("NORM.DIST sd=2 x=5", "NORM.DIST(5,5,2,TRUE)", "number"));
    cases.push(tc("NORMDIST x=-1", "NORMDIST(-1,0,1,TRUE)", "number"));
    cases.push(tc("NORMDIST x=0", "NORMDIST(0,0,1,TRUE)", "number"));
    cases.push(tc("NORMDIST x=2", "NORMDIST(2,0,1,TRUE)", "number"));

    // NORM.INV additional
    cases.push(tc("NORM.INV p=0.5 mean=5 sd=0.5", "NORM.INV(0.5,5,0.5)", "number"));
    cases.push(tc("NORM.INV p=0.1 mean=10 sd=2", "NORM.INV(0.1,10,2)", "number"));
    cases.push(tc("NORM.INV p=0.9 mean=10 sd=2", "NORM.INV(0.9,10,2)", "number"));
    cases.push(tc("NORMINV p=0.5 mean=5 sd=2", "NORMINV(0.5,5,2)", "number"));

    // NORM.S.DIST additional
    cases.push(tc("NORM.S.DIST z=2 cumulative", "NORM.S.DIST(2,TRUE)", "number"));
    cases.push(tc("NORM.S.DIST z=-2 cumulative", "NORM.S.DIST(-2,TRUE)", "number"));
    cases.push(tc("NORMSDIST z=2", "NORMSDIST(2)", "number"));
    cases.push(tc("NORMSDIST z=-2", "NORMSDIST(-2)", "number"));

    // NORM.S.INV additional
    cases.push(tc("NORM.S.INV p=0.025", "NORM.S.INV(0.025)", "number"));
    cases.push(tc("NORM.S.INV p=0.975", "NORM.S.INV(0.975)", "number"));
    cases.push(tc("NORMSINV p=0.025", "NORMSINV(0.025)", "number"));
    cases.push(tc("NORMSINV p=0.975", "NORMSINV(0.975)", "number"));

    // T.DIST additional t values and df
    cases.push(tc("T.DIST t=0 df=5 cumulative", "T.DIST(0,5,TRUE)", "number"));
    cases.push(tc("T.DIST t=2 df=5 cumulative", "T.DIST(2,5,TRUE)", "number"));
    cases.push(tc("T.DIST t=-1 df=10 cumulative", "T.DIST(-1,10,TRUE)", "number"));
    cases.push(tc("T.DIST.2T t=2 df=5", "T.DIST.2T(2,5)", "number"));
    cases.push(tc("T.DIST.2T t=2 df=30", "T.DIST.2T(2,30)", "number"));
    cases.push(tc("T.DIST.RT t=0 df=5", "T.DIST.RT(0,5)", "number"));
    cases.push(tc("T.DIST.RT t=2 df=5", "T.DIST.RT(2,5)", "number"));

    // T.INV additional
    cases.push(tc("T.INV p=0.025 df=5", "T.INV(0.025,5)", "number"));
    cases.push(tc("T.INV p=0.975 df=5", "T.INV(0.975,5)", "number"));
    cases.push(tc("T.INV p=0.025 df=30", "T.INV(0.025,30)", "number"));
    cases.push(tc("T.INV.2T p=0.05 df=5", "T.INV.2T(0.05,5)", "number"));
    cases.push(tc("T.INV.2T p=0.05 df=30", "T.INV.2T(0.05,30)", "number"));
    cases.push(tc("TINV p=0.05 df=5", "TINV(0.05,5)", "number"));
    cases.push(tc("TINV p=0.05 df=30", "TINV(0.05,30)", "number"));

    // T.TEST additional type variations
    cases.push(tc("T.TEST paired one-tailed equal", "T.TEST({2,3,4},{2,3,5},1,1)", "number"));
    cases.push(tc("T.TEST equal-variance one-tailed", "T.TEST({1,2,3,4,5},{2,3,4,5,6},1,2)", "number"));
    cases.push(tc("T.TEST unequal-variance one-tailed", "T.TEST({1,2,3,4,5},{2,3,4,5,6},1,3)", "number"));
    cases.push(tc("TTEST paired one-tailed", "TTEST({2,3,4},{2,3,5},1,1)", "number"));
    cases.push(tc("TTEST unequal-variance two-tailed", "TTEST({1,2,3,4,5},{2,3,4,5,6},2,3)", "number"));

    // F.DIST additional x and df values
    cases.push(tc("F.DIST x=0.5 df1=5 df2=10 cumulative", "F.DIST(0.5,5,10,TRUE)", "number"));
    cases.push(tc("F.DIST x=2 df1=5 df2=10 cumulative", "F.DIST(2,5,10,TRUE)", "number"));
    cases.push(tc("F.DIST.RT x=0.5 df1=5 df2=10", "F.DIST.RT(0.5,5,10)", "number"));
    cases.push(tc("F.DIST.RT x=2 df1=5 df2=10", "F.DIST.RT(2,5,10)", "number"));
    cases.push(tc("FDIST x=2 df1=5 df2=10", "FDIST(2,5,10)", "number"));
    cases.push(tc("FDIST x=0.5 df1=10 df2=30", "FDIST(0.5,10,30)", "number"));

    // F.INV additional
    cases.push(tc("F.INV p=0.5 df1=1 df2=5", "F.INV(0.5,1,5)", "number"));
    cases.push(tc("F.INV p=0.1 df1=5 df2=10", "F.INV(0.1,5,10)", "number"));
    cases.push(tc("F.INV.RT p=0.5 df1=1 df2=5", "F.INV.RT(0.5,1,5)", "number"));
    cases.push(tc("FINV p=0.9 df1=5 df2=10", "FINV(0.9,5,10)", "number"));

    // F.TEST additional
    cases.push(tc("F.TEST identical arrays", "F.TEST({1,2,3,4,5},{1,2,3,4,5})", "number"));
    cases.push(tc("FTEST different variance", "FTEST({1,2,3,4,5},{1,4,9,16,25})", "number"));

    // CHISQ.DIST additional x values
    cases.push(tc("CHISQ.DIST x=0.5 df=1 cumulative", "CHISQ.DIST(0.5,1,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST x=10 df=5 cumulative", "CHISQ.DIST(10,5,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST x=20 df=10 cumulative", "CHISQ.DIST(20,10,TRUE)", "number"));
    cases.push(tc("CHISQ.DIST.RT x=0.5 df=1", "CHISQ.DIST.RT(0.5,1)", "number"));
    cases.push(tc("CHISQ.DIST.RT x=10 df=5", "CHISQ.DIST.RT(10,5)", "number"));
    cases.push(tc("CHIDIST x=0.5 df=1", "CHIDIST(0.5,1)", "number"));
    cases.push(tc("CHIDIST x=10 df=30", "CHIDIST(10,30)", "number"));

    // CHISQ.INV additional
    cases.push(tc("CHISQ.INV p=0.5 df=1", "CHISQ.INV(0.5,1)", "number"));
    cases.push(tc("CHISQ.INV p=0.9 df=5", "CHISQ.INV(0.9,5)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.5 df=1", "CHISQ.INV.RT(0.5,1)", "number"));
    cases.push(tc("CHISQ.INV.RT p=0.1 df=30", "CHISQ.INV.RT(0.1,30)", "number"));
    cases.push(tc("CHIINV p=0.9 df=5", "CHIINV(0.9,5)", "number"));
    cases.push(tc("CHIINV p=0.05 df=30", "CHIINV(0.05,30)", "number"));

    // CHISQ.TEST additional
    cases.push(tc(
        "CHISQ.TEST uniform vs non-uniform",
        "CHISQ.TEST({20,20,20},{10,20,30})",
        "number",
    ));
    cases.push(tc(
        "CHITEST uniform vs non-uniform",
        "CHITEST({20,20,20},{10,20,30})",
        "number",
    ));

    // BETA.DIST additional parameter sweeps
    cases.push(tc("BETA.DIST a=1 b=1 x=0.1 cumulative", "BETA.DIST(0.1,1,1,TRUE)", "number"));
    cases.push(tc("BETA.DIST a=1 b=1 x=0.9 cumulative", "BETA.DIST(0.9,1,1,TRUE)", "number"));
    cases.push(tc("BETA.DIST a=5 b=2 x=0.5 cumulative", "BETA.DIST(0.5,5,2,TRUE)", "number"));
    cases.push(tc("BETA.DIST a=2 b=5 x=0.5 cumulative", "BETA.DIST(0.5,2,5,TRUE)", "number"));
    cases.push(tc("BETADIST a=1 b=5 x=0.1", "BETADIST(0.1,1,5)", "number"));
    cases.push(tc("BETADIST a=5 b=1 x=0.9", "BETADIST(0.9,5,1)", "number"));

    // BETA.INV additional
    cases.push(tc("BETA.INV p=0.5 a=1 b=1", "BETA.INV(0.5,1,1)", "number"));
    cases.push(tc("BETA.INV p=0.5 a=5 b=5", "BETA.INV(0.5,5,5)", "number"));
    cases.push(tc("BETA.INV p=0.1 a=2 b=5 explicit", "BETA.INV(0.1,2,5,0,1)", "number"));
    cases.push(tc("BETA.INV p=0.9 a=5 b=2 explicit", "BETA.INV(0.9,5,2,0,1)", "number"));
    cases.push(tc("BETAINV p=0.5 a=1 b=1", "BETAINV(0.5,1,1,0,1)", "number"));
    cases.push(tc("BETAINV p=0.5 a=5 b=5", "BETAINV(0.5,5,5,0,1)", "number"));

    // BINOM.DIST additional sweeps
    cases.push(tc("BINOM.DIST exact n=5 k=0 p=0.5", "BINOM.DIST(0,5,0.5,FALSE)", "number"));
    cases.push(tc("BINOM.DIST exact n=5 k=5 p=0.5", "BINOM.DIST(5,5,0.5,FALSE)", "number"));
    cases.push(tc("BINOM.DIST exact n=10 k=0 p=0.1", "BINOM.DIST(0,10,0.1,FALSE)", "number"));
    cases.push(tc("BINOM.DIST exact n=10 k=9 p=0.9", "BINOM.DIST(9,10,0.9,FALSE)", "number"));
    cases.push(tc("BINOMDIST n=20 k=0 p=0.1", "BINOMDIST(0,20,0.1,FALSE)", "number"));
    cases.push(tc("BINOMDIST n=20 k=20 p=0.9", "BINOMDIST(20,20,0.9,FALSE)", "number"));
    cases.push(tc("CRITBINOM n=5 p=0.5 alpha=0.5", "CRITBINOM(5,0.5,0.5)", "number"));

    // BINOM.INV additional
    cases.push(tc("BINOM.INV n=5 p=0.5 alpha=0.5", "BINOM.INV(5,0.5,0.5)", "number"));
    cases.push(tc("BINOM.INV n=20 p=0.1 alpha=0.5", "BINOM.INV(20,0.1,0.5)", "number"));
    cases.push(tc("BINOM.INV n=20 p=0.9 alpha=0.5", "BINOM.INV(20,0.9,0.5)", "number"));

    // NEGBINOM.DIST additional
    cases.push(tc("NEGBINOM.DIST exact k=1 r=1 p=0.1", "NEGBINOM.DIST(1,1,0.1,FALSE)", "number"));
    cases.push(tc("NEGBINOM.DIST exact k=1 r=1 p=0.9", "NEGBINOM.DIST(1,1,0.9,FALSE)", "number"));
    cases.push(tc("NEGBINOMDIST k=1 r=2 p=0.5", "NEGBINOMDIST(1,2,0.5)", "number"));

    // POISSON.DIST additional lambda sweeps
    cases.push(tc("POISSON.DIST exact lambda=1 k=1", "POISSON.DIST(1,1,FALSE)", "number"));
    cases.push(tc("POISSON.DIST exact lambda=2 k=2", "POISSON.DIST(2,2,FALSE)", "number"));
    cases.push(tc("POISSON.DIST cumulative lambda=2 k=3", "POISSON.DIST(3,2,TRUE)", "number"));
    cases.push(tc("POISSON exact lambda=2 k=2", "POISSON(2,2,FALSE)", "number"));
    cases.push(tc("POISSON cumulative lambda=2 k=3", "POISSON(3,2,TRUE)", "number"));

    // HYPGEOM.DIST additional
    cases.push(tc(
        "HYPGEOM.DIST exact s=0 ns=5 M=5 N=20",
        "HYPGEOM.DIST(0,5,5,20,FALSE)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOM.DIST exact s=5 ns=5 M=10 N=20",
        "HYPGEOM.DIST(5,5,10,20,FALSE)",
        "number",
    ));
    cases.push(tc(
        "HYPGEOMDIST s=0 ns=5 M=5 N=20",
        "HYPGEOMDIST(0,5,5,20)",
        "number",
    ));

    // EXPON.DIST additional
    cases.push(tc("EXPON.DIST cumulative lambda=0.5 x=2", "EXPON.DIST(2,0.5,TRUE)", "number"));
    cases.push(tc("EXPON.DIST pdf lambda=2 x=0.5", "EXPON.DIST(0.5,2,FALSE)", "number"));
    cases.push(tc("EXPONDIST cumulative lambda=2 x=0.5", "EXPONDIST(0.5,2,TRUE)", "number"));
    cases.push(tc("EXPONDIST pdf lambda=0.5 x=2", "EXPONDIST(2,0.5,FALSE)", "number"));

    // GAMMA.DIST additional parameter sweep
    cases.push(tc("GAMMA.DIST cumulative a=1 b=2", "GAMMA.DIST(2,1,2,TRUE)", "number"));
    cases.push(tc("GAMMA.DIST cumulative a=2 b=2", "GAMMA.DIST(2,2,2,TRUE)", "number"));
    cases.push(tc("GAMMA.DIST pdf a=2 b=2", "GAMMA.DIST(2,2,2,FALSE)", "number"));
    cases.push(tc("GAMMADIST cumulative a=5 b=1", "GAMMADIST(5,5,1,TRUE)", "number"));

    // GAMMA.INV additional
    cases.push(tc("GAMMA.INV p=0.1 a=2 b=2", "GAMMA.INV(0.1,2,2)", "number"));
    cases.push(tc("GAMMA.INV p=0.9 a=1 b=1", "GAMMA.INV(0.9,1,1)", "number"));
    cases.push(tc("GAMMAINV p=0.9 a=5 b=2", "GAMMAINV(0.9,5,2)", "number"));

    // GAMMALN additional
    cases.push(tc("GAMMALN x=0.5", "GAMMALN(0.5)", "number"));
    cases.push(tc("GAMMALN x=100", "GAMMALN(100)", "number"));
    cases.push(tc("GAMMALN.PRECISE x=0.5", "GAMMALN.PRECISE(0.5)", "number"));
    cases.push(tc("GAMMALN.PRECISE x=10", "GAMMALN.PRECISE(10)", "number"));

    // LOGNORM.DIST additional
    cases.push(tc("LOGNORM.DIST x=2 mu=0 sd=1 cumulative", "LOGNORM.DIST(2,0,1,TRUE)", "number"));
    cases.push(tc("LOGNORM.DIST x=0.5 mu=0 sd=1 cumulative", "LOGNORM.DIST(0.5,0,1,TRUE)", "number"));
    cases.push(tc("LOGNORMDIST x=2 mu=0 sd=1", "LOGNORMDIST(2,0,1)", "number"));
    cases.push(tc("LOGNORMDIST x=0.5 mu=0 sd=1", "LOGNORMDIST(0.5,0,1)", "number"));

    // LOGNORM.INV additional
    cases.push(tc("LOGNORM.INV p=0.5 mu=0 sd=0.5", "LOGNORM.INV(0.5,0,0.5)", "number"));
    cases.push(tc("LOGNORM.INV p=0.5 mu=0 sd=2", "LOGNORM.INV(0.5,0,2)", "number"));
    cases.push(tc("LOGINV p=0.5 mu=0 sd=0.5", "LOGINV(0.5,0,0.5)", "number"));
    cases.push(tc("LOGINV p=0.5 mu=0 sd=2", "LOGINV(0.5,0,2)", "number"));

    // WEIBULL.DIST additional parameter sweeps
    cases.push(tc("WEIBULL.DIST cumulative a=1 b=2", "WEIBULL.DIST(2,1,2,TRUE)", "number"));
    cases.push(tc("WEIBULL.DIST cumulative a=2 b=2", "WEIBULL.DIST(2,2,2,TRUE)", "number"));
    cases.push(tc("WEIBULL.DIST pdf a=2 b=2", "WEIBULL.DIST(2,2,2,FALSE)", "number"));
    cases.push(tc("WEIBULL cumulative a=2 b=2", "WEIBULL(2,2,2,TRUE)", "number"));
    cases.push(tc("WEIBULL pdf a=5 b=2", "WEIBULL(2,5,2,FALSE)", "number"));

    // DEVSQ / AVEDEV additional
    cases.push(tc("DEVSQ small dataset", "DEVSQ(1,2,3)", "number"));
    cases.push(tc("AVEDEV small dataset", "AVEDEV(1,2,3)", "number"));
    cases.push(tc("DEVSQ large dataset", "DEVSQ({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("AVEDEV large dataset", "AVEDEV({1,2,3,4,5,6,7,8,9,10})", "number"));

    // GEOMEAN / HARMEAN additional
    cases.push(tc("GEOMEAN two values", "GEOMEAN(4,16)", "number"));
    cases.push(tc("GEOMEAN large dataset", "GEOMEAN({1,2,4,8,16,32})", "number"));
    cases.push(tc("HARMEAN two values", "HARMEAN(2,6)", "number"));
    cases.push(tc("HARMEAN large dataset", "HARMEAN({1,2,3,4,5,6})", "number"));

    // KURT / SKEW additional datasets
    cases.push(tc("KURT normal-like", "KURT(1,2,2,3,3,3,4,4,5)", "number"));
    cases.push(tc("SKEW positive", "SKEW(1,1,1,2,3,4,5)", "number"));
    cases.push(tc("SKEW negative", "SKEW(1,2,3,4,5,5,5)", "number"));
    cases.push(tc("SKEW.P positive", "SKEW.P(1,1,1,2,3,4,5)", "number"));
    cases.push(tc("SKEW.P negative", "SKEW.P(1,2,3,4,5,5,5)", "number"));

    // TRIMMEAN additional
    cases.push(tc("TRIMMEAN 0% no trim", "TRIMMEAN({1,2,3,4,5,6,7,8,9,10},0.0)", "number"));
    cases.push(tc("TRIMMEAN 50% heavy trim", "TRIMMEAN({1,2,3,4,5,6,7,8,9,10},0.5)", "number"));

    // PROB additional
    cases.push(tc(
        "PROB x=[1,2,3,4] prob=[0.1,0.2,0.3,0.4] lower=2 upper=3",
        "PROB({1,2,3,4},{0.1,0.2,0.3,0.4},2,3)",
        "number",
    ));
    cases.push(tc(
        "PROB single value",
        "PROB({1,2,3},{0.2,0.5,0.3},1,1)",
        "number",
    ));

    // FREQUENCY additional
    cases.push(tc(
        "FREQUENCY single bin",
        "FREQUENCY({1,2,3,4,5},{3})",
        "number",
    ));

    // GROWTH / LOGEST / LINEST additional
    cases.push(tc(
        "GROWTH with prediction",
        "GROWTH({2,4,8},{1,2,3},{4})",
        "number",
    ));
    cases.push(tc(
        "LINEST intercept=TRUE stats=FALSE",
        "LINEST({2,4,6,8},{1,2,3,4},TRUE,FALSE)",
        "number",
    ));

    // ZTEST additional datasets
    cases.push(tc("ZTEST large sample mu=5", "ZTEST({1,2,3,4,5,6,7,8,9,10},5)", "number"));
    cases.push(tc("Z.TEST large sample mu=5", "Z.TEST({1,2,3,4,5,6,7,8,9,10},5)", "number"));
    cases.push(tc("ZTEST mu=0 sigma=1", "ZTEST({-1,0,1},0,1)", "number"));
    cases.push(tc("Z.TEST mu=0 sigma=1", "Z.TEST({-1,0,1},0,1)", "number"));

    // =========================================================================
    // Extended parameter sweeps: distributions with all valid df/shape combos
    // =========================================================================

    // NORM.DIST x sweep over [-3,-2,-1,0,1,2,3] with mu=0, sd=1
    for (xi, x) in ["-3", "-2", "-1", "0", "1", "2", "3"].iter().enumerate() {
        cases.push(tc(
            &format!("NORM.DIST x={} std normal cumulative", x),
            &format!("NORM.DIST({},0,1,TRUE)", x),
            "number",
        ));
        cases.push(tc(
            &format!("NORM.DIST x={} std normal pdf", x),
            &format!("NORM.DIST({},0,1,FALSE)", x),
            "number",
        ));
        let _ = xi; // suppress unused warning
    }

    // NORM.S.DIST x sweep
    for x in ["-3", "-2", "-1", "0", "1", "2", "3"].iter() {
        cases.push(tc(
            &format!("NORM.S.DIST z={} cumulative", x),
            &format!("NORM.S.DIST({},TRUE)", x),
            "number",
        ));
    }

    // NORM.S.INV probability sweep: 0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99
    for p in ["0.01", "0.05", "0.25", "0.75", "0.95", "0.99"].iter() {
        cases.push(tc(
            &format!("NORM.S.INV p={}", p),
            &format!("NORM.S.INV({})", p),
            "number",
        ));
        cases.push(tc(
            &format!("NORMSINV p={}", p),
            &format!("NORMSINV({})", p),
            "number",
        ));
    }

    // T.DIST df sweep: [1,2,3,5,10,20,30] at t=1.5
    for df in ["1", "2", "3", "5", "10", "20", "30"].iter() {
        cases.push(tc(
            &format!("T.DIST t=1.5 df={} cumulative", df),
            &format!("T.DIST(1.5,{},TRUE)", df),
            "number",
        ));
        cases.push(tc(
            &format!("T.DIST.RT t=1.5 df={}", df),
            &format!("T.DIST.RT(1.5,{})", df),
            "number",
        ));
        cases.push(tc(
            &format!("T.DIST.2T t=1.5 df={}", df),
            &format!("T.DIST.2T(1.5,{})", df),
            "number",
        ));
    }

    // T.INV.2T p sweep over common alpha values, df=10
    for p in ["0.01", "0.025", "0.05", "0.1", "0.2", "0.5"].iter() {
        cases.push(tc(
            &format!("T.INV.2T p={} df=10", p),
            &format!("T.INV.2T({},10)", p),
            "number",
        ));
        cases.push(tc(
            &format!("TINV p={} df=10", p),
            &format!("TINV({},10)", p),
            "number",
        ));
    }

    // CHISQ.DIST df sweep: [1,2,3,5,10,20,30] at x=3
    for df in ["1", "2", "3", "5", "10", "20", "30"].iter() {
        cases.push(tc(
            &format!("CHISQ.DIST x=3 df={} cumulative", df),
            &format!("CHISQ.DIST(3,{},TRUE)", df),
            "number",
        ));
        cases.push(tc(
            &format!("CHISQ.DIST.RT x=3 df={}", df),
            &format!("CHISQ.DIST.RT(3,{})", df),
            "number",
        ));
    }

    // CHISQ.INV.RT p sweep over df=5 and df=10
    for p in ["0.01", "0.025", "0.05", "0.1", "0.25", "0.5", "0.75", "0.9", "0.95"].iter() {
        cases.push(tc(
            &format!("CHISQ.INV.RT p={} df=5", p),
            &format!("CHISQ.INV.RT({},5)", p),
            "number",
        ));
        cases.push(tc(
            &format!("CHIINV p={} df=5", p),
            &format!("CHIINV({},5)", p),
            "number",
        ));
    }

    // F.DIST df1,df2 combination sweep at x=2
    for (df1, df2) in [("1", "5"), ("2", "5"), ("5", "5"), ("5", "10"), ("10", "20"), ("20", "30")].iter() {
        cases.push(tc(
            &format!("F.DIST x=2 df1={} df2={} cumulative", df1, df2),
            &format!("F.DIST(2,{},{},TRUE)", df1, df2),
            "number",
        ));
        cases.push(tc(
            &format!("F.DIST.RT x=2 df1={} df2={}", df1, df2),
            &format!("F.DIST.RT(2,{},{})", df1, df2),
            "number",
        ));
    }

    // F.INV.RT p sweep over df1=5 df2=10
    for p in ["0.01", "0.025", "0.05", "0.1", "0.25", "0.5", "0.75", "0.9", "0.95"].iter() {
        cases.push(tc(
            &format!("F.INV.RT p={} df1=5 df2=10", p),
            &format!("F.INV.RT({},5,10)", p),
            "number",
        ));
        cases.push(tc(
            &format!("FINV p={} df1=5 df2=10", p),
            &format!("FINV({},5,10)", p),
            "number",
        ));
    }

    // BINOM.DIST n=10, k sweep [0,1,2,3,4,5,6,7,8,9,10], p=0.5
    for k in ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"].iter() {
        cases.push(tc(
            &format!("BINOM.DIST n=10 k={} p=0.5 exact", k),
            &format!("BINOM.DIST({},10,0.5,FALSE)", k),
            "number",
        ));
    }
    // BINOM.DIST n=20 cumulative at key k values
    for k in ["5", "10", "15"].iter() {
        cases.push(tc(
            &format!("BINOM.DIST n=20 k={} p=0.5 cumulative", k),
            &format!("BINOM.DIST({},20,0.5,TRUE)", k),
            "number",
        ));
    }

    // POISSON.DIST lambda sweep [1,2,5,10] at k=lambda
    for (k, lam) in [("1", "1"), ("2", "2"), ("5", "5"), ("10", "10")].iter() {
        cases.push(tc(
            &format!("POISSON.DIST lambda={} k={} exact", lam, k),
            &format!("POISSON.DIST({},{},FALSE)", k, lam),
            "number",
        ));
        cases.push(tc(
            &format!("POISSON.DIST lambda={} k={} cumulative", lam, k),
            &format!("POISSON.DIST({},{},TRUE)", k, lam),
            "number",
        ));
    }

    // EXPON.DIST lambda sweep [0.5,1,2,5] cumulative at x=1
    for lam in ["0.5", "1", "2", "5"].iter() {
        cases.push(tc(
            &format!("EXPON.DIST x=1 lambda={} cumulative", lam),
            &format!("EXPON.DIST(1,{},TRUE)", lam),
            "number",
        ));
        cases.push(tc(
            &format!("EXPON.DIST x=1 lambda={} pdf", lam),
            &format!("EXPON.DIST(1,{},FALSE)", lam),
            "number",
        ));
    }

    // GAMMA.DIST alpha sweep [1,2,5] x beta sweep [1,2] cumulative
    for alpha in ["1", "2", "5"].iter() {
        for beta in ["1", "2"].iter() {
            cases.push(tc(
                &format!("GAMMA.DIST a={} b={} x=2 cumulative", alpha, beta),
                &format!("GAMMA.DIST(2,{},{},TRUE)", alpha, beta),
                "number",
            ));
        }
    }

    // GAMMA.INV p sweep at alpha=2 beta=1
    for p in ["0.01", "0.05", "0.1", "0.5", "0.9", "0.95", "0.99"].iter() {
        cases.push(tc(
            &format!("GAMMA.INV p={} a=2 b=1", p),
            &format!("GAMMA.INV({},2,1)", p),
            "number",
        ));
    }

    // LOGNORM.DIST x sweep [0.5,1,2,5] at mu=0 sd=1
    for x in ["0.5", "1", "2", "5"].iter() {
        cases.push(tc(
            &format!("LOGNORM.DIST x={} mu=0 sd=1 cumulative", x),
            &format!("LOGNORM.DIST({},0,1,TRUE)", x),
            "number",
        ));
        cases.push(tc(
            &format!("LOGNORM.DIST x={} mu=0 sd=1 pdf", x),
            &format!("LOGNORM.DIST({},0,1,FALSE)", x),
            "number",
        ));
    }

    // LOGNORM.INV p sweep at mu=0 sd=1
    for p in ["0.01", "0.05", "0.1", "0.25", "0.5", "0.75", "0.9", "0.95", "0.99"].iter() {
        cases.push(tc(
            &format!("LOGNORM.INV p={} mu=0 sd=1", p),
            &format!("LOGNORM.INV({},0,1)", p),
            "number",
        ));
        cases.push(tc(
            &format!("LOGINV p={} mu=0 sd=1", p),
            &format!("LOGINV({},0,1)", p),
            "number",
        ));
    }

    // WEIBULL.DIST alpha sweep [1,2,5] beta=1 cumulative at x=1
    for alpha in ["1", "2", "5"].iter() {
        cases.push(tc(
            &format!("WEIBULL.DIST a={} b=1 x=1 cumulative", alpha),
            &format!("WEIBULL.DIST(1,{},1,TRUE)", alpha),
            "number",
        ));
        cases.push(tc(
            &format!("WEIBULL a={} b=1 x=1 cumulative", alpha),
            &format!("WEIBULL(1,{},1,TRUE)", alpha),
            "number",
        ));
    }

    // BETA.DIST alpha/beta matrix sweep at x=0.5
    for (alpha, beta) in [("1", "1"), ("1", "2"), ("2", "1"), ("2", "5"), ("5", "2"), ("5", "5")].iter() {
        cases.push(tc(
            &format!("BETA.DIST a={} b={} x=0.5 cumulative", alpha, beta),
            &format!("BETA.DIST(0.5,{},{},TRUE)", alpha, beta),
            "number",
        ));
    }

    // BETA.INV p sweep at alpha=2 beta=5
    for p in ["0.01", "0.05", "0.1", "0.25", "0.5", "0.75", "0.9", "0.95", "0.99"].iter() {
        cases.push(tc(
            &format!("BETA.INV p={} a=2 b=5 explicit", p),
            &format!("BETA.INV({},2,5,0,1)", p),
            "number",
        ));
    }

    // NEGBINOM.DIST r sweep [1,2,5] at k=5, p=0.5
    for r in ["1", "2", "5"].iter() {
        cases.push(tc(
            &format!("NEGBINOM.DIST k=5 r={} p=0.5 exact", r),
            &format!("NEGBINOM.DIST(5,{},0.5,FALSE)", r),
            "number",
        ));
    }

    // PERCENTILE / PERCENTILE.INC / PERCENTILE.EXC p sweep [0.1,0.2,...,0.9]
    for p in ["0.1", "0.2", "0.3", "0.4", "0.6", "0.7", "0.8"].iter() {
        cases.push(tc(
            &format!("PERCENTILE.INC p={}", p),
            &format!("PERCENTILE.INC({{1,2,3,4,5,6,7,8,9,10}},{})", p),
            "number",
        ));
        cases.push(tc(
            &format!("PERCENTILE.EXC p={}", p),
            &format!("PERCENTILE.EXC({{1,2,3,4,5,6,7,8,9,10}},{})", p),
            "number",
        ));
    }

    // STDEV.S and STDEV.P on different datasets
    for (label, data) in [
        ("small", "{1,2,3}"),
        ("medium", "{1,2,3,4,5,6,7}"),
        ("wide", "{1,5,10,15,20}"),
    ].iter() {
        cases.push(tc(
            &format!("STDEV.S {}", label),
            &format!("STDEV.S({})", data),
            "number",
        ));
        cases.push(tc(
            &format!("STDEV.P {}", label),
            &format!("STDEV.P({})", data),
            "number",
        ));
        cases.push(tc(
            &format!("VAR.S {}", label),
            &format!("VAR.S({})", data),
            "number",
        ));
        cases.push(tc(
            &format!("VAR.P {}", label),
            &format!("VAR.P({})", data),
            "number",
        ));
    }

    // RANK on larger datasets
    for val in ["1", "5", "10"].iter() {
        cases.push(tc(
            &format!("RANK.EQ val={} descending", val),
            &format!("RANK.EQ({},{{1,2,3,4,5,6,7,8,9,10}})", val),
            "number",
        ));
        cases.push(tc(
            &format!("RANK.AVG val={} descending", val),
            &format!("RANK.AVG({},{{1,2,3,4,5,6,7,8,9,10}})", val),
            "number",
        ));
    }

    // =========================================================================
    // Additional systematic coverage: CORREL/COVAR on more datasets
    // =========================================================================
    for (label, x, y) in [
        ("identical", "{1,2,3,4,5}", "{1,2,3,4,5}"),
        ("scaled", "{1,2,3,4,5}", "{3,6,9,12,15}"),
        ("shifted", "{1,2,3,4,5}", "{6,7,8,9,10}"),
    ].iter() {
        cases.push(tc(
            &format!("CORREL {}", label),
            &format!("CORREL({},{})", x, y),
            "number",
        ));
        cases.push(tc(
            &format!("COVARIANCE.P {}", label),
            &format!("COVARIANCE.P({},{})", x, y),
            "number",
        ));
        cases.push(tc(
            &format!("COVARIANCE.S {}", label),
            &format!("COVARIANCE.S({},{})", x, y),
            "number",
        ));
    }

    // HYPGEOM.DIST ns sweep at M=10, N=20, s=2
    for ns in ["5", "10", "15"].iter() {
        cases.push(tc(
            &format!("HYPGEOM.DIST ns={} s=2 M=10 N=20 exact", ns),
            &format!("HYPGEOM.DIST(2,{},10,20,FALSE)", ns),
            "number",
        ));
        cases.push(tc(
            &format!("HYPGEOM.DIST ns={} s=2 M=10 N=20 cumulative", ns),
            &format!("HYPGEOM.DIST(2,{},10,20,TRUE)", ns),
            "number",
        ));
    }

    // BINOM.INV sweep at various n and alpha
    for (n, p, alpha) in [("10", "0.5", "0.5"), ("20", "0.5", "0.5"), ("10", "0.9", "0.5"), ("10", "0.1", "0.5")].iter() {
        cases.push(tc(
            &format!("BINOM.INV n={} p={} alpha={}", n, p, alpha),
            &format!("BINOM.INV({},{},{})", n, p, alpha),
            "number",
        ));
    }

    // LARGE/SMALL on 10-element dataset
    for k in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"].iter() {
        cases.push(tc(
            &format!("LARGE k={} 10-elements", k),
            &format!("LARGE({{10,3,7,1,9,5,6,8,2,4}},{})", k),
            "number",
        ));
        cases.push(tc(
            &format!("SMALL k={} 10-elements", k),
            &format!("SMALL({{10,3,7,1,9,5,6,8,2,4}},{})", k),
            "number",
        ));
    }

    // MODE on various datasets
    cases.push(tc("MODE.SNGL no repeat returns first", "MODE.SNGL({3,1,4,1,5,9,2,6})", "number"));
    cases.push(tc("MODE array larger", "MODE({1,2,3,3,4,5,5,5,6})", "number"));

    // GAMMALN additional x values
    for x in ["3", "4", "6", "7", "8", "9", "20", "50"].iter() {
        cases.push(tc(
            &format!("GAMMALN x={}", x),
            &format!("GAMMALN({})", x),
            "number",
        ));
    }

    // AVERAGE / STDEV on large data
    cases.push(tc("AVERAGE 10 values", "AVERAGE({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("STDEV 10 values", "STDEV({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("MEDIAN 10 values", "MEDIAN({1,2,3,4,5,6,7,8,9,10})", "number"));
    cases.push(tc("VAR 10 values", "VAR({1,2,3,4,5,6,7,8,9,10})", "number"));

    cases
}
