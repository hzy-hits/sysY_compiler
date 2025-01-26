use koopa::ir::Value;

use crate::ir_builder::IRBuilder;
use anyhow::Result;
pub trait ToIr {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()>;
}

pub trait ExpToIr {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<Value>;
}
