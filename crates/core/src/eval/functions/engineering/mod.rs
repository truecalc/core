use super::Registry;
use super::FunctionMeta;

pub mod bitand;
pub mod bitor;
pub mod bitxor;
pub mod bitlshift;
pub mod bitrshift;
pub mod delta;
pub mod gestep;

pub fn register_engineering(registry: &mut Registry) {
    registry.register_eager("BITAND",    bitand::bitand_fn,       FunctionMeta { category: "engineering", signature: "BITAND(number1, number2)",          description: "Bitwise AND of two integers" });
    registry.register_eager("BITOR",     bitor::bitor_fn,         FunctionMeta { category: "engineering", signature: "BITOR(number1, number2)",           description: "Bitwise OR of two integers" });
    registry.register_eager("BITXOR",    bitxor::bitxor_fn,       FunctionMeta { category: "engineering", signature: "BITXOR(number1, number2)",          description: "Bitwise XOR of two integers" });
    registry.register_eager("BITLSHIFT", bitlshift::bitlshift_fn, FunctionMeta { category: "engineering", signature: "BITLSHIFT(number, shift_amount)",   description: "Left-shift an integer by a number of bits" });
    registry.register_eager("BITRSHIFT", bitrshift::bitrshift_fn, FunctionMeta { category: "engineering", signature: "BITRSHIFT(number, shift_amount)",   description: "Right-shift an integer by a number of bits" });
    registry.register_eager("DELTA",     delta::delta_fn,         FunctionMeta { category: "engineering", signature: "DELTA(number1, [number2])",         description: "Test whether two values are equal" });
    registry.register_eager("GESTEP",    gestep::gestep_fn,       FunctionMeta { category: "engineering", signature: "GESTEP(number, [step])",            description: "Test whether a number is greater than or equal to a step value" });
}
