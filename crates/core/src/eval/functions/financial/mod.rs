use super::super::{FunctionMeta, Registry};

pub mod bonds;
pub mod fv;
pub mod irr;
pub mod misc;
pub mod nper;
pub mod npv;
pub mod pmt;
pub mod pv;
pub mod rate;

pub fn register_financial(registry: &mut Registry) {
    registry.register_eager("PMT",  pmt::pmt_fn,  FunctionMeta { category: "financial", signature: "PMT(rate, nper, pv)",       description: "Periodic payment for a loan" });
    registry.register_eager("NPV",  npv::npv_fn,  FunctionMeta { category: "financial", signature: "NPV(rate, value1,...)",     description: "Net present value" });
    registry.register_eager("IRR",  irr::irr_fn,  FunctionMeta { category: "financial", signature: "IRR(values, [guess])",      description: "Internal rate of return" });
    registry.register_eager("PV",   pv::pv_fn,    FunctionMeta { category: "financial", signature: "PV(rate, nper, pmt)",       description: "Present value" });
    registry.register_eager("FV",   fv::fv_fn,    FunctionMeta { category: "financial", signature: "FV(rate, nper, pmt)",       description: "Future value" });
    registry.register_eager("RATE", rate::rate_fn,FunctionMeta { category: "financial", signature: "RATE(nper, pmt, pv)",       description: "Interest rate per period" });
    registry.register_eager("NPER", nper::nper_fn,FunctionMeta { category: "financial", signature: "NPER(rate, pmt, pv)",       description: "Number of payment periods" });

    // Bond / securities functions
    registry.register_eager("ACCRINT",    bonds::accrint_fn,    FunctionMeta { category: "financial", signature: "ACCRINT(issue, first_coupon, settlement, rate, par, frequency, [basis])", description: "Accrued interest for periodic coupon" });
    registry.register_eager("ACCRINTM",   bonds::accrintm_fn,   FunctionMeta { category: "financial", signature: "ACCRINTM(issue, settlement, rate, par, [basis])",                        description: "Accrued interest at maturity" });
    registry.register_eager("COUPDAYBS",  bonds::coupdaybs_fn,  FunctionMeta { category: "financial", signature: "COUPDAYBS(settlement, maturity, frequency, [basis])",                   description: "Days from coupon start to settlement" });
    registry.register_eager("COUPDAYS",   bonds::coupdays_fn,   FunctionMeta { category: "financial", signature: "COUPDAYS(settlement, maturity, frequency, [basis])",                    description: "Days in coupon period" });
    registry.register_eager("COUPDAYSNC", bonds::coupdaysnc_fn, FunctionMeta { category: "financial", signature: "COUPDAYSNC(settlement, maturity, frequency, [basis])",                  description: "Days from settlement to next coupon" });
    registry.register_eager("COUPNCD",    bonds::coupncd_fn,    FunctionMeta { category: "financial", signature: "COUPNCD(settlement, maturity, frequency, [basis])",                     description: "Next coupon date after settlement" });
    registry.register_eager("COUPNUM",    bonds::coupnum_fn,    FunctionMeta { category: "financial", signature: "COUPNUM(settlement, maturity, frequency, [basis])",                     description: "Number of coupons between settlement and maturity" });
    registry.register_eager("COUPPCD",    bonds::couppcd_fn,    FunctionMeta { category: "financial", signature: "COUPPCD(settlement, maturity, frequency, [basis])",                     description: "Previous coupon date before settlement" });
    registry.register_eager("DISC",       bonds::disc_fn,       FunctionMeta { category: "financial", signature: "DISC(settlement, maturity, pr, redemption, [basis])",                   description: "Discount rate for a security" });
    registry.register_eager("INTRATE",    bonds::intrate_fn,    FunctionMeta { category: "financial", signature: "INTRATE(settlement, maturity, investment, redemption, [basis])",        description: "Interest rate for fully invested security" });
    registry.register_eager("PRICE",      bonds::price_fn,      FunctionMeta { category: "financial", signature: "PRICE(settlement, maturity, rate, yld, redemption, frequency, [basis])", description: "Price per $100 face value" });
    registry.register_eager("PRICEDISC",  bonds::pricedisc_fn,  FunctionMeta { category: "financial", signature: "PRICEDISC(settlement, maturity, discount, redemption, [basis])",        description: "Price of discounted security" });
    registry.register_eager("PRICEMAT",   bonds::pricemat_fn,   FunctionMeta { category: "financial", signature: "PRICEMAT(settlement, maturity, issue, rate, yld, [basis])",             description: "Price at maturity" });
    registry.register_eager("RECEIVED",   bonds::received_fn,   FunctionMeta { category: "financial", signature: "RECEIVED(settlement, maturity, investment, discount, [basis])",         description: "Amount received at maturity" });
    registry.register_eager("TBILLEQ",    bonds::tbilleq_fn,    FunctionMeta { category: "financial", signature: "TBILLEQ(settlement, maturity, discount)",                               description: "T-bill bond-equivalent yield" });
    registry.register_eager("TBILLPRICE", bonds::tbillprice_fn, FunctionMeta { category: "financial", signature: "TBILLPRICE(settlement, maturity, discount)",                            description: "T-bill price" });
    registry.register_eager("TBILLYIELD", bonds::tbillyield_fn, FunctionMeta { category: "financial", signature: "TBILLYIELD(settlement, maturity, pr)",                                  description: "T-bill yield" });
    registry.register_eager("YIELD",      bonds::yield_fn,      FunctionMeta { category: "financial", signature: "YIELD(settlement, maturity, rate, pr, redemption, frequency, [basis])", description: "Yield on a coupon bond" });
    registry.register_eager("YIELDDISC",  bonds::yielddisc_fn,  FunctionMeta { category: "financial", signature: "YIELDDISC(settlement, maturity, pr, redemption, [basis])",             description: "Annual yield for discounted security" });
    registry.register_eager("YIELDMAT",   bonds::yieldmat_fn,   FunctionMeta { category: "financial", signature: "YIELDMAT(settlement, maturity, issue, rate, pr, [basis])",             description: "Annual yield at maturity" });

    // Miscellaneous financial functions
    registry.register_eager("IPMT",       misc::ipmt_fn,        FunctionMeta { category: "financial", signature: "IPMT(rate, per, nper, pv, [fv], [type])",   description: "Interest payment for a period" });
    registry.register_eager("PPMT",       misc::ppmt_fn,        FunctionMeta { category: "financial", signature: "PPMT(rate, per, nper, pv, [fv], [type])",   description: "Principal payment for a period" });
    registry.register_eager("CUMIPMT",    misc::cumipmt_fn,     FunctionMeta { category: "financial", signature: "CUMIPMT(rate, nper, pv, start, end, type)", description: "Cumulative interest paid" });
    registry.register_eager("CUMPRINC",   misc::cumprinc_fn,    FunctionMeta { category: "financial", signature: "CUMPRINC(rate, nper, pv, start, end, type)", description: "Cumulative principal paid" });
    registry.register_eager("ISPMT",      misc::ispmt_fn,       FunctionMeta { category: "financial", signature: "ISPMT(rate, per, nper, pv)",                description: "Interest paid for a period (straight line)" });
    registry.register_eager("SLN",        misc::sln_fn,         FunctionMeta { category: "financial", signature: "SLN(cost, salvage, life)",                  description: "Straight-line depreciation" });
    registry.register_eager("SYD",        misc::syd_fn,         FunctionMeta { category: "financial", signature: "SYD(cost, salvage, life, per)",             description: "Sum-of-years' digits depreciation" });
    registry.register_eager("DDB",        misc::ddb_fn,         FunctionMeta { category: "financial", signature: "DDB(cost, salvage, life, period, [factor])", description: "Double-declining balance depreciation" });
    registry.register_eager("DB",         misc::db_fn,          FunctionMeta { category: "financial", signature: "DB(cost, salvage, life, period, [month])",  description: "Fixed-declining balance depreciation" });
    registry.register_eager("VDB",        misc::vdb_fn,         FunctionMeta { category: "financial", signature: "VDB(cost, salvage, life, start, end, [factor], [no_switch])", description: "Variable-rate declining balance" });
    registry.register_eager("AMORLINC",   misc::amorlinc_fn,    FunctionMeta { category: "financial", signature: "AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])", description: "Linear depreciation (French system)" });
    registry.register_eager("DOLLARDE",   misc::dollarde_fn,    FunctionMeta { category: "financial", signature: "DOLLARDE(fractional_dollar, fraction)",     description: "Convert dollar price to decimal" });
    registry.register_eager("DOLLARFR",   misc::dollarfr_fn,    FunctionMeta { category: "financial", signature: "DOLLARFR(decimal_dollar, fraction)",        description: "Convert decimal dollar to fractional" });
    registry.register_eager("EFFECT",     misc::effect_fn,      FunctionMeta { category: "financial", signature: "EFFECT(nominal_rate, npery)",               description: "Effective annual interest rate" });
    registry.register_eager("NOMINAL",    misc::nominal_fn,     FunctionMeta { category: "financial", signature: "NOMINAL(effect_rate, npery)",               description: "Nominal annual interest rate" });
    registry.register_eager("PDURATION",  misc::pduration_fn,   FunctionMeta { category: "financial", signature: "PDURATION(rate, pv, fv)",                   description: "Periods required to reach a value" });
    registry.register_eager("RRI",        misc::rri_fn,         FunctionMeta { category: "financial", signature: "RRI(nper, pv, fv)",                         description: "Equivalent interest rate for growth" });
    registry.register_eager("DURATION",   misc::duration_fn,    FunctionMeta { category: "financial", signature: "DURATION(settlement, maturity, coupon, yld, frequency, [basis])", description: "Macaulay duration" });
    registry.register_eager("MDURATION",  misc::mduration_fn,   FunctionMeta { category: "financial", signature: "MDURATION(settlement, maturity, coupon, yld, frequency, [basis])", description: "Modified duration" });
    registry.register_eager("FVSCHEDULE", misc::fvschedule_fn,  FunctionMeta { category: "financial", signature: "FVSCHEDULE(principal, schedule)",            description: "Future value with variable rates" });
    registry.register_eager("MIRR",       misc::mirr_fn,        FunctionMeta { category: "financial", signature: "MIRR(values, finance_rate, reinvest_rate)", description: "Modified internal rate of return" });
    registry.register_eager("XNPV",       misc::xnpv_fn,        FunctionMeta { category: "financial", signature: "XNPV(rate, values, dates)",                 description: "NPV for irregular cash flows" });
    registry.register_eager("XIRR",       misc::xirr_fn,        FunctionMeta { category: "financial", signature: "XIRR(values, dates, [guess])",              description: "IRR for irregular cash flows" });
}
