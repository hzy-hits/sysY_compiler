use koopa::ir::Value;

use crate::ir_builder::IRBuilder;

pub trait ToIr {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String>;
}

pub trait ExpToIr {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value, String>;
}
