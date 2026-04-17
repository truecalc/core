use super::super::{FunctionMeta, Registry};

pub mod distributions;
pub mod distributions_impl;
use distributions_impl as di;

pub mod averagea;
pub mod averageifs;
pub mod count;
pub mod countblank;
pub mod geomean;
pub mod harmean;
pub mod kurt;
pub mod large;
pub mod max;
pub mod maxa;
pub mod maxifs;
pub mod median;
pub mod min;
pub mod mina;
pub mod minifs;
pub mod mode;
pub mod mode_mult;
pub mod mode_sngl;
pub mod percentile;
pub mod percentile_exc;
pub mod percentile_inc;
pub mod percentrank;
pub mod percentrank_exc;
pub mod percentrank_inc;
pub mod quartile;
pub mod quartile_exc;
pub mod quartile_inc;
pub mod rank;
pub mod rank_avg;
pub mod rank_eq;
pub mod skew;
pub mod skew_p;
pub mod small;
pub mod stat_helpers;
pub mod trimmean;

pub mod avedev;
pub mod covariance_p;
pub mod covariance_s;
pub mod devsq;
pub mod stdev;
pub mod stdev_p;
pub mod stdev_s;
pub mod stdeva;
pub mod stdevp;
pub mod stdevpa;
pub mod var;
pub mod var_p;
pub mod var_s;
pub mod vara;
pub mod varp;
pub mod varpa;

pub fn register_statistical(registry: &mut Registry) {
    registry.register_eager("AVERAGEA",  averagea::averagea_fn,   FunctionMeta { category: "statistical", signature: "AVERAGEA(value1,...)",                          description: "Average including booleans and text" });
    registry.register_eager("AVERAGEIFS",averageifs::averageifs_fn,FunctionMeta { category: "statistical", signature: "AVERAGEIFS(avg_range,criteria_range1,criteria1,...)", description: "Conditional average with multiple criteria" });
    registry.register_lazy("COUNT",  count::count_lazy_fn,  FunctionMeta { category: "statistical", signature: "COUNT(value1,...)",  description: "Count numeric values" });
    registry.register_lazy("COUNTA", count::counta_lazy_fn, FunctionMeta { category: "statistical", signature: "COUNTA(value1,...)", description: "Count non-empty values" });
    registry.register_eager("COUNTBLANK", countblank::countblank_fn, FunctionMeta { category: "statistical", signature: "COUNTBLANK(range)", description: "Count blank/empty cells" });
    registry.register_eager("GEOMEAN",  geomean::geomean_fn,  FunctionMeta { category: "statistical", signature: "GEOMEAN(value1,...)", description: "Geometric mean" });
    registry.register_eager("HARMEAN",  harmean::harmean_fn,  FunctionMeta { category: "statistical", signature: "HARMEAN(value1,...)", description: "Harmonic mean" });
    registry.register_eager("KURT",     kurt::kurt_fn,        FunctionMeta { category: "statistical", signature: "KURT(value1,...)",    description: "Excess kurtosis" });
    registry.register_eager("LARGE",  large::large_fn,  FunctionMeta { category: "statistical", signature: "LARGE(array,k)",   description: "k-th largest value" });
    registry.register_eager("MAX",   max::max_fn,           FunctionMeta { category: "statistical", signature: "MAX(value1,...)",    description: "Maximum value" });
    registry.register_eager("MAXA",  maxa::maxa_fn,         FunctionMeta { category: "statistical", signature: "MAXA(value1,...)",   description: "Maximum including booleans and text" });
    registry.register_eager("MAXIFS",maxifs::maxifs_fn,     FunctionMeta { category: "statistical", signature: "MAXIFS(max_range,criteria_range1,criteria1,...)", description: "Maximum with multiple criteria" });
    registry.register_eager("MEDIAN",median::median_fn,     FunctionMeta { category: "statistical", signature: "MEDIAN(value1,...)", description: "Median value" });
    registry.register_eager("MIN",   min::min_fn,           FunctionMeta { category: "statistical", signature: "MIN(value1,...)",    description: "Minimum value" });
    registry.register_eager("MINA",  mina::mina_fn,         FunctionMeta { category: "statistical", signature: "MINA(value1,...)",   description: "Minimum including booleans and text" });
    registry.register_eager("MINIFS",minifs::minifs_fn,     FunctionMeta { category: "statistical", signature: "MINIFS(min_range,criteria_range1,criteria1,...)", description: "Minimum with multiple criteria" });
    registry.register_eager("MODE",      mode::mode_fn,           FunctionMeta { category: "statistical", signature: "MODE(value1,...)",      description: "Most frequent value" });
    registry.register_eager("MODE.MULT", mode_mult::mode_mult_fn, FunctionMeta { category: "statistical", signature: "MODE.MULT(value1,...)", description: "All most frequent values" });
    registry.register_eager("MODE.SNGL", mode_sngl::mode_sngl_fn, FunctionMeta { category: "statistical", signature: "MODE.SNGL(value1,...)", description: "Most frequent value" });
    registry.register_eager("PERCENTILE",     percentile::percentile_fn,         FunctionMeta { category: "statistical", signature: "PERCENTILE(array,k)",     description: "k-th percentile (inclusive)" });
    registry.register_eager("PERCENTILE.EXC", percentile_exc::percentile_exc_fn, FunctionMeta { category: "statistical", signature: "PERCENTILE.EXC(array,k)", description: "Exclusive percentile" });
    registry.register_eager("PERCENTILE.INC", percentile_inc::percentile_inc_fn, FunctionMeta { category: "statistical", signature: "PERCENTILE.INC(array,k)", description: "Inclusive percentile" });
    registry.register_eager("PERCENTRANK",     percentrank::percentrank_fn,         FunctionMeta { category: "statistical", signature: "PERCENTRANK(array,x,[sig])",     description: "Percentile rank (inclusive)" });
    registry.register_eager("PERCENTRANK.EXC", percentrank_exc::percentrank_exc_fn, FunctionMeta { category: "statistical", signature: "PERCENTRANK.EXC(array,x,[sig])", description: "Exclusive percentile rank" });
    registry.register_eager("PERCENTRANK.INC", percentrank_inc::percentrank_inc_fn, FunctionMeta { category: "statistical", signature: "PERCENTRANK.INC(array,x,[sig])", description: "Inclusive percentile rank" });
    registry.register_eager("QUARTILE",     quartile::quartile_fn,         FunctionMeta { category: "statistical", signature: "QUARTILE(array,quart)",     description: "Quartile (inclusive)" });
    registry.register_eager("QUARTILE.EXC", quartile_exc::quartile_exc_fn, FunctionMeta { category: "statistical", signature: "QUARTILE.EXC(array,quart)", description: "Exclusive quartile" });
    registry.register_eager("QUARTILE.INC", quartile_inc::quartile_inc_fn, FunctionMeta { category: "statistical", signature: "QUARTILE.INC(array,quart)", description: "Inclusive quartile" });
    registry.register_eager("RANK",     rank::rank_fn,         FunctionMeta { category: "statistical", signature: "RANK(number,ref,[order])",     description: "Rank of number (ties=lowest)" });
    registry.register_eager("RANK.AVG", rank_avg::rank_avg_fn, FunctionMeta { category: "statistical", signature: "RANK.AVG(number,ref,[order])", description: "Rank with average for ties" });
    registry.register_eager("RANK.EQ",  rank_eq::rank_eq_fn,   FunctionMeta { category: "statistical", signature: "RANK.EQ(number,ref,[order])",  description: "Rank with equal (lowest) for ties" });
    registry.register_eager("SKEW",   skew::skew_fn,        FunctionMeta { category: "statistical", signature: "SKEW(value1,...)",   description: "Sample skewness" });
    registry.register_eager("SKEW.P", skew_p::skew_p_fn,   FunctionMeta { category: "statistical", signature: "SKEW.P(value1,...)", description: "Population skewness" });
    registry.register_eager("SMALL",  small::small_fn,  FunctionMeta { category: "statistical", signature: "SMALL(array,k)",   description: "k-th smallest value" });
    registry.register_eager("TRIMMEAN", trimmean::trimmean_fn, FunctionMeta { category: "statistical", signature: "TRIMMEAN(data,percent)", description: "Trimmed mean" });

    registry.register_eager("AVEDEV",      avedev::avedev_fn,           FunctionMeta { category: "statistical", signature: "AVEDEV(value1,...)",       description: "Average of absolute deviations from the mean" });
    registry.register_eager("DEVSQ",       devsq::devsq_fn,             FunctionMeta { category: "statistical", signature: "DEVSQ(value1,...)",        description: "Sum of squared deviations from the mean" });
    registry.register_eager("COVARIANCE.P",covariance_p::covariance_p_fn, FunctionMeta { category: "statistical", signature: "COVARIANCE.P(array1,array2)", description: "Population covariance" });
    registry.register_eager("COVARIANCE.S",covariance_s::covariance_s_fn, FunctionMeta { category: "statistical", signature: "COVARIANCE.S(array1,array2)", description: "Sample covariance" });
    registry.register_eager("STDEV",       stdev::stdev_fn,             FunctionMeta { category: "statistical", signature: "STDEV(value1,...)",         description: "Sample standard deviation" });
    registry.register_eager("STDEV.P",     stdev_p::stdev_p_fn,         FunctionMeta { category: "statistical", signature: "STDEV.P(value1,...)",       description: "Population standard deviation" });
    registry.register_eager("STDEV.S",     stdev_s::stdev_s_fn,         FunctionMeta { category: "statistical", signature: "STDEV.S(value1,...)",       description: "Sample standard deviation" });
    registry.register_eager("STDEVA",      stdeva::stdeva_fn,           FunctionMeta { category: "statistical", signature: "STDEVA(value1,...)",        description: "Sample standard deviation including text and logical values" });
    registry.register_eager("STDEVP",      stdevp::stdevp_fn,           FunctionMeta { category: "statistical", signature: "STDEVP(value1,...)",        description: "Population standard deviation" });
    registry.register_eager("STDEVPA",     stdevpa::stdevpa_fn,         FunctionMeta { category: "statistical", signature: "STDEVPA(value1,...)",       description: "Population standard deviation including text and logical values" });
    registry.register_eager("VAR",         var::var_fn,                 FunctionMeta { category: "statistical", signature: "VAR(value1,...)",           description: "Sample variance" });
    registry.register_eager("VAR.P",       var_p::var_p_fn,             FunctionMeta { category: "statistical", signature: "VAR.P(value1,...)",         description: "Population variance" });
    registry.register_eager("VAR.S",       var_s::var_s_fn,             FunctionMeta { category: "statistical", signature: "VAR.S(value1,...)",         description: "Sample variance" });
    registry.register_eager("VARA",        vara::vara_fn,               FunctionMeta { category: "statistical", signature: "VARA(value1,...)",          description: "Sample variance including text and logical values" });
    registry.register_eager("VARP",        varp::varp_fn,               FunctionMeta { category: "statistical", signature: "VARP(value1,...)",          description: "Population variance" });
    registry.register_eager("VARPA",       varpa::varpa_fn,             FunctionMeta { category: "statistical", signature: "VARPA(value1,...)",         description: "Population variance including text and logical values" });

    // M3 statistical distribution functions
    registry.register_eager("AVERAGE.WEIGHTED", di::average_weighted_fn, FunctionMeta { category: "statistical", signature: "AVERAGE.WEIGHTED(values,weights)", description: "Weighted average" });

    registry.register_eager("NORM.S.DIST", di::norm_s_dist_fn, FunctionMeta { category: "statistical", signature: "NORM.S.DIST(x,cumulative)", description: "Standard normal CDF/PDF" });
    registry.register_eager("NORMSDIST",   di::normsdist_fn,   FunctionMeta { category: "statistical", signature: "NORMSDIST(x)", description: "Standard normal CDF (legacy)" });
    registry.register_eager("NORM.S.INV",  di::norm_s_inv_fn,  FunctionMeta { category: "statistical", signature: "NORM.S.INV(p)", description: "Standard normal inverse CDF" });
    registry.register_eager("NORMSINV",    di::norm_s_inv_fn,  FunctionMeta { category: "statistical", signature: "NORMSINV(p)", description: "Standard normal inverse CDF (legacy)" });
    registry.register_eager("NORM.DIST",   di::norm_dist_fn,   FunctionMeta { category: "statistical", signature: "NORM.DIST(x,mean,stdev,cumulative)", description: "Normal CDF/PDF" });
    registry.register_eager("NORMDIST",    di::normdist_fn,    FunctionMeta { category: "statistical", signature: "NORMDIST(x,mean,stdev,cumulative)", description: "Normal CDF/PDF (legacy)" });
    registry.register_eager("NORM.INV",    di::norm_inv_fn,    FunctionMeta { category: "statistical", signature: "NORM.INV(p,mean,stdev)", description: "Normal inverse CDF" });
    registry.register_eager("NORMINV",     di::norm_inv_fn,    FunctionMeta { category: "statistical", signature: "NORMINV(p,mean,stdev)", description: "Normal inverse CDF (legacy)" });
    registry.register_eager("GAUSS",       di::gauss_fn,       FunctionMeta { category: "statistical", signature: "GAUSS(x)", description: "NORM.S.DIST(x,TRUE) - 0.5" });
    registry.register_eager("PHI",         di::phi_fn,         FunctionMeta { category: "statistical", signature: "PHI(x)", description: "Standard normal PDF" });
    registry.register_eager("STANDARDIZE", di::standardize_fn, FunctionMeta { category: "statistical", signature: "STANDARDIZE(x,mean,stdev)", description: "Standardize a value" });

    registry.register_eager("CONFIDENCE",       di::confidence_fn,   FunctionMeta { category: "statistical", signature: "CONFIDENCE(alpha,stdev,size)", description: "Confidence interval half-width (normal)" });
    registry.register_eager("CONFIDENCE.NORM",  di::confidence_fn,   FunctionMeta { category: "statistical", signature: "CONFIDENCE.NORM(alpha,stdev,size)", description: "Confidence interval half-width (normal)" });
    registry.register_eager("CONFIDENCE.T",     di::confidence_t_fn, FunctionMeta { category: "statistical", signature: "CONFIDENCE.T(alpha,stdev,size)", description: "Confidence interval half-width (t-dist)" });

    registry.register_eager("CORREL",  di::correl_fn,  FunctionMeta { category: "statistical", signature: "CORREL(array1,array2)", description: "Pearson correlation coefficient" });
    registry.register_eager("PEARSON", di::pearson_fn, FunctionMeta { category: "statistical", signature: "PEARSON(array1,array2)", description: "Pearson correlation coefficient" });

    registry.register_eager("SLOPE",            di::slope_fn,            FunctionMeta { category: "statistical", signature: "SLOPE(known_y,known_x)", description: "Slope of linear regression" });
    registry.register_eager("INTERCEPT",        di::intercept_fn,        FunctionMeta { category: "statistical", signature: "INTERCEPT(known_y,known_x)", description: "Intercept of linear regression" });
    registry.register_eager("RSQ",              di::rsq_fn,              FunctionMeta { category: "statistical", signature: "RSQ(known_y,known_x)", description: "R-squared of linear regression" });
    registry.register_eager("FORECAST",         di::forecast_fn,         FunctionMeta { category: "statistical", signature: "FORECAST(x,known_y,known_x)", description: "Forecast using linear regression" });
    registry.register_eager("FORECAST.LINEAR",  di::forecast_linear_fn,  FunctionMeta { category: "statistical", signature: "FORECAST.LINEAR(x,known_y,known_x)", description: "Forecast using linear regression" });
    registry.register_eager("STEYX",            di::steyx_fn,            FunctionMeta { category: "statistical", signature: "STEYX(known_y,known_x)", description: "Standard error of regression" });

    registry.register_eager("CHISQ.DIST",    di::chisq_dist_fn,    FunctionMeta { category: "statistical", signature: "CHISQ.DIST(x,df,cumulative)", description: "Chi-squared CDF/PDF" });
    registry.register_eager("CHISQ.DIST.RT", di::chisq_dist_rt_fn, FunctionMeta { category: "statistical", signature: "CHISQ.DIST.RT(x,df)", description: "Chi-squared right-tail CDF" });
    registry.register_eager("CHIDIST",       di::chidist_fn,       FunctionMeta { category: "statistical", signature: "CHIDIST(x,df)", description: "Chi-squared right-tail CDF (legacy)" });
    registry.register_eager("CHISQ.INV",     di::chisq_inv_fn,     FunctionMeta { category: "statistical", signature: "CHISQ.INV(p,df)", description: "Chi-squared inverse CDF" });
    registry.register_eager("CHISQ.INV.RT",  di::chisq_inv_rt_fn,  FunctionMeta { category: "statistical", signature: "CHISQ.INV.RT(p,df)", description: "Chi-squared right-tail inverse CDF" });
    registry.register_eager("CHIINV",        di::chiinv_fn,        FunctionMeta { category: "statistical", signature: "CHIINV(p,df)", description: "Chi-squared right-tail inverse CDF (legacy)" });
    registry.register_eager("CHISQ.TEST",    di::chisq_test_fn,    FunctionMeta { category: "statistical", signature: "CHISQ.TEST(observed,expected)", description: "Chi-squared goodness of fit test" });
    registry.register_eager("CHITEST",       di::chitest_fn,       FunctionMeta { category: "statistical", signature: "CHITEST(observed,expected)", description: "Chi-squared test (legacy)" });

    registry.register_eager("T.DIST",    di::t_dist_fn,    FunctionMeta { category: "statistical", signature: "T.DIST(x,df,cumulative)", description: "Student's t CDF/PDF" });
    registry.register_eager("T.DIST.RT", di::t_dist_rt_fn, FunctionMeta { category: "statistical", signature: "T.DIST.RT(x,df)", description: "Student's t right-tail CDF" });
    registry.register_eager("T.DIST.2T", di::t_dist_2t_fn, FunctionMeta { category: "statistical", signature: "T.DIST.2T(x,df)", description: "Student's t two-tailed CDF" });
    registry.register_eager("TDIST",     di::tdist_fn,     FunctionMeta { category: "statistical", signature: "TDIST(x,df,tails)", description: "Student's t distribution (legacy)" });
    registry.register_eager("T.INV",     di::t_inv_fn,     FunctionMeta { category: "statistical", signature: "T.INV(p,df)", description: "Student's t inverse CDF" });
    registry.register_eager("T.INV.2T",  di::t_inv_2t_fn,  FunctionMeta { category: "statistical", signature: "T.INV.2T(p,df)", description: "Student's t two-tailed inverse" });
    registry.register_eager("TINV",      di::tinv_fn,      FunctionMeta { category: "statistical", signature: "TINV(p,df)", description: "Student's t two-tailed inverse (legacy)" });
    registry.register_eager("T.TEST",    di::t_test_fn,    FunctionMeta { category: "statistical", signature: "T.TEST(array1,array2,tails,type)", description: "Student's t-test" });
    registry.register_eager("TTEST",     di::ttest_fn,     FunctionMeta { category: "statistical", signature: "TTEST(array1,array2,tails,type)", description: "Student's t-test (legacy)" });

    registry.register_eager("F.DIST",    di::f_dist_fn,    FunctionMeta { category: "statistical", signature: "F.DIST(x,df1,df2,cumulative)", description: "F CDF/PDF" });
    registry.register_eager("F.DIST.RT", di::f_dist_rt_fn, FunctionMeta { category: "statistical", signature: "F.DIST.RT(x,df1,df2)", description: "F right-tail CDF" });
    registry.register_eager("FDIST",     di::fdist_fn,     FunctionMeta { category: "statistical", signature: "FDIST(x,df1,df2)", description: "F right-tail CDF (legacy)" });
    registry.register_eager("F.INV",     di::f_inv_fn,     FunctionMeta { category: "statistical", signature: "F.INV(p,df1,df2)", description: "F inverse CDF" });
    registry.register_eager("F.INV.RT",  di::f_inv_rt_fn,  FunctionMeta { category: "statistical", signature: "F.INV.RT(p,df1,df2)", description: "F right-tail inverse CDF" });
    registry.register_eager("FINV",      di::finv_fn,      FunctionMeta { category: "statistical", signature: "FINV(p,df1,df2)", description: "F right-tail inverse CDF (legacy)" });
    registry.register_eager("F.TEST",    di::f_test_fn,    FunctionMeta { category: "statistical", signature: "F.TEST(array1,array2)", description: "F-test" });
    registry.register_eager("FTEST",     di::ftest_fn,     FunctionMeta { category: "statistical", signature: "FTEST(array1,array2)", description: "F-test (legacy)" });

    registry.register_eager("GAMMA",      di::gamma_fn_impl,   FunctionMeta { category: "statistical", signature: "GAMMA(x)", description: "Gamma function" });
    registry.register_eager("GAMMA.DIST", di::gamma_dist_fn,   FunctionMeta { category: "statistical", signature: "GAMMA.DIST(x,alpha,beta,cumulative)", description: "Gamma distribution CDF/PDF" });
    registry.register_eager("GAMMADIST",  di::gamma_dist_fn,   FunctionMeta { category: "statistical", signature: "GAMMADIST(x,alpha,beta,cumulative)", description: "Gamma distribution (legacy)" });
    registry.register_eager("GAMMA.INV",  di::gamma_inv_fn,    FunctionMeta { category: "statistical", signature: "GAMMA.INV(p,alpha,beta)", description: "Gamma inverse CDF" });
    registry.register_eager("GAMMAINV",   di::gamma_inv_fn,    FunctionMeta { category: "statistical", signature: "GAMMAINV(p,alpha,beta)", description: "Gamma inverse CDF (legacy)" });

    registry.register_eager("BETA.DIST", di::beta_dist_fn, FunctionMeta { category: "statistical", signature: "BETA.DIST(x,alpha,beta,cumulative,[lo],[hi])", description: "Beta distribution CDF/PDF" });
    registry.register_eager("BETADIST",  di::betadist_fn,  FunctionMeta { category: "statistical", signature: "BETADIST(x,alpha,beta,lo,hi)", description: "Beta distribution CDF (legacy)" });
    registry.register_eager("BETA.INV",  di::beta_inv_fn,  FunctionMeta { category: "statistical", signature: "BETA.INV(p,alpha,beta,lo,hi)", description: "Beta inverse CDF" });
    registry.register_eager("BETAINV",   di::beta_inv_fn,  FunctionMeta { category: "statistical", signature: "BETAINV(p,alpha,beta,lo,hi)", description: "Beta inverse CDF (legacy)" });

    registry.register_eager("BINOM.DIST", di::binom_dist_fn, FunctionMeta { category: "statistical", signature: "BINOM.DIST(k,n,p,cumulative)", description: "Binomial CDF/PMF" });
    registry.register_eager("BINOMDIST",  di::binom_dist_fn, FunctionMeta { category: "statistical", signature: "BINOMDIST(k,n,p,cumulative)", description: "Binomial CDF/PMF (legacy)" });
    registry.register_eager("BINOM.INV",  di::binom_inv_fn,  FunctionMeta { category: "statistical", signature: "BINOM.INV(n,p,alpha)", description: "Binomial inverse CDF" });
    registry.register_eager("CRITBINOM",  di::binom_inv_fn,  FunctionMeta { category: "statistical", signature: "CRITBINOM(n,p,alpha)", description: "Binomial inverse CDF (legacy)" });

    registry.register_eager("POISSON",      di::poisson_fn,      FunctionMeta { category: "statistical", signature: "POISSON(x,mean,cumulative)", description: "Poisson distribution (legacy)" });
    registry.register_eager("POISSON.DIST", di::poisson_dist_fn, FunctionMeta { category: "statistical", signature: "POISSON.DIST(x,mean,cumulative)", description: "Poisson CDF/PMF" });

    registry.register_eager("NEGBINOM.DIST", di::negbinom_dist_fn, FunctionMeta { category: "statistical", signature: "NEGBINOM.DIST(x,r,p,cumulative)", description: "Negative binomial CDF/PMF" });
    registry.register_eager("NEGBINOMDIST",  di::negbinomdist_fn,  FunctionMeta { category: "statistical", signature: "NEGBINOMDIST(x,r,p)", description: "Negative binomial PMF (legacy)" });

    registry.register_eager("HYPGEOM.DIST", di::hypgeom_dist_fn, FunctionMeta { category: "statistical", signature: "HYPGEOM.DIST(x,n,k,pop,cumulative)", description: "Hypergeometric distribution" });
    registry.register_eager("HYPGEOMDIST",  di::hypgeomdist_fn,  FunctionMeta { category: "statistical", signature: "HYPGEOMDIST(x,n,k,pop)", description: "Hypergeometric PMF (legacy)" });

    registry.register_eager("EXPON.DIST", di::expon_dist_fn, FunctionMeta { category: "statistical", signature: "EXPON.DIST(x,lambda,cumulative)", description: "Exponential distribution" });
    registry.register_eager("EXPONDIST",  di::expondist_fn,  FunctionMeta { category: "statistical", signature: "EXPONDIST(x,lambda,cumulative)", description: "Exponential distribution (legacy)" });

    registry.register_eager("WEIBULL",      di::weibull_fn,      FunctionMeta { category: "statistical", signature: "WEIBULL(x,alpha,beta,cumulative)", description: "Weibull distribution (legacy)" });
    registry.register_eager("WEIBULL.DIST", di::weibull_dist_fn, FunctionMeta { category: "statistical", signature: "WEIBULL.DIST(x,alpha,beta,cumulative)", description: "Weibull distribution" });

    registry.register_eager("LOGNORM.DIST", di::lognorm_dist_fn, FunctionMeta { category: "statistical", signature: "LOGNORM.DIST(x,mean,stdev,cumulative)", description: "Lognormal distribution" });
    registry.register_eager("LOGNORMDIST",  di::lognormdist_fn,  FunctionMeta { category: "statistical", signature: "LOGNORMDIST(x,mean,stdev)", description: "Lognormal CDF (legacy)" });
    registry.register_eager("LOGNORM.INV",  di::lognorm_inv_fn,  FunctionMeta { category: "statistical", signature: "LOGNORM.INV(p,mean,stdev)", description: "Lognormal inverse CDF" });
    registry.register_eager("LOGINV",       di::loginv_fn,       FunctionMeta { category: "statistical", signature: "LOGINV(p,mean,stdev)", description: "Lognormal inverse CDF (legacy)" });

    registry.register_eager("FISHER",    di::fisher_fn,     FunctionMeta { category: "statistical", signature: "FISHER(x)", description: "Fisher z-transform" });
    registry.register_eager("FISHERINV", di::fisher_inv_fn, FunctionMeta { category: "statistical", signature: "FISHERINV(y)", description: "Inverse Fisher z-transform" });

    registry.register_eager("PERMUT",       di::permut_fn,       FunctionMeta { category: "statistical", signature: "PERMUT(n,k)", description: "Number of permutations" });
    registry.register_eager("PERMUTATIONA", di::permutationa_fn, FunctionMeta { category: "statistical", signature: "PERMUTATIONA(n,k)", description: "Permutations with repetition" });

    registry.register_eager("PROB", di::prob_fn, FunctionMeta { category: "statistical", signature: "PROB(x_range,prob_range,lo,[hi])", description: "Probability from distribution" });

    registry.register_eager("Z.TEST", di::z_test_fn, FunctionMeta { category: "statistical", signature: "Z.TEST(data,mu,[sigma])", description: "Z-test p-value" });
    registry.register_eager("ZTEST",  di::ztest_fn,  FunctionMeta { category: "statistical", signature: "ZTEST(data,mu,[sigma])", description: "Z-test p-value (legacy)" });

    registry.register_eager("MARGINOFERROR", di::marginoferror_fn, FunctionMeta { category: "statistical", signature: "MARGINOFERROR(data,confidence)", description: "Margin of error" });

    registry.register_eager("COVAR", di::covar_fn, FunctionMeta { category: "statistical", signature: "COVAR(array1,array2)", description: "Population covariance (legacy)" });
}
