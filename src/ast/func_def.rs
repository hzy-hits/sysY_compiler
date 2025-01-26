use std::vec;

use koopa::ir::Type;

use super::refactor::{FuncDef, FuncType};
use super::Result;
use crate::{ast::Block, ir_builder::IRBuilder, traits::ToIr};

impl ToIr for FuncDef {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        let name = format!("@{}", &self.id);
        builder.create_function(&name, vec![], self.func_type.to_koop());
        let entry = builder.create_bb("%entry")?;
        builder.set_current_bb(entry);

        self.block.to_ir(builder)?;
        Ok(())
    }
}

impl FuncType {
    pub fn to_koop(&self) -> Type {
        match self {
            FuncType::Int => Type::get_i32(),
        }
    }
}
