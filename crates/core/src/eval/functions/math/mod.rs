use super::super::Registry;

pub mod abs;
pub mod average;
pub mod ceiling_floor;
pub mod exp;
pub mod int_fn;
pub mod log;
pub mod mod_fn;
pub mod power;
pub mod product;
pub mod quotient;
pub mod rand;
pub mod round;
pub mod sign;
pub mod sqrt;
pub mod sum;
pub mod trig;

pub fn register_math(registry: &mut Registry) {
    registry.register_eager("SUM", sum::sum_fn);
    registry.register_eager("AVERAGE", average::average_fn);
    registry.register_eager("PRODUCT", product::product_fn);
    registry.register_eager("ROUND", round::round_fn);
    registry.register_eager("ROUNDUP", round::roundup_fn);
    registry.register_eager("ROUNDDOWN", round::rounddown_fn);
    registry.register_eager("INT", int_fn::int_fn);
    registry.register_eager("ABS", abs::abs_fn);
    registry.register_eager("SIGN", sign::sign_fn);
    registry.register_eager("MOD", mod_fn::mod_fn);
    registry.register_eager("POWER", power::power_fn);
    registry.register_eager("SQRT", sqrt::sqrt_fn);
    registry.register_eager("LOG", log::log_fn);
    registry.register_eager("LOG10", log::log10_fn);
    registry.register_eager("LN", log::ln_fn);
    registry.register_eager("EXP", exp::exp_fn);
    registry.register_eager("CEILING", ceiling_floor::ceiling_fn);
    registry.register_eager("FLOOR", ceiling_floor::floor_fn);
    registry.register_eager("RAND", rand::rand_fn);
    registry.register_eager("RANDBETWEEN", rand::randbetween_fn);
    registry.register_eager("PI", trig::pi_fn);
    registry.register_eager("SIN", trig::sin_fn);
    registry.register_eager("COS", trig::cos_fn);
    registry.register_eager("TAN", trig::tan_fn);
    registry.register_eager("QUOTIENT", quotient::quotient_fn);
}
