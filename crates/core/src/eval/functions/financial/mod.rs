use super::super::Registry;

pub mod fv;
pub mod irr;
pub mod npv;
pub mod nper;
pub mod pmt;
pub mod pv;
pub mod rate;

pub fn register_financial(registry: &mut Registry) {
    registry.register_eager("PMT", pmt::pmt_fn);
    registry.register_eager("NPV", npv::npv_fn);
    registry.register_eager("IRR", irr::irr_fn);
    registry.register_eager("PV", pv::pv_fn);
    registry.register_eager("FV", fv::fv_fn);
    registry.register_eager("RATE", rate::rate_fn);
    registry.register_eager("NPER", nper::nper_fn);
}
