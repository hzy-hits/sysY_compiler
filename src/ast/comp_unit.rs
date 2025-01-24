use crate::{ir_builder::IRBuilder, traits::ToIr};

use super::refactor::{CompUnit, CompUnitItem};

impl ToIr for CompUnit {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        self.items
            .iter()
            .for_each(|item| item.to_ir(builder).unwrap());
        Ok(())
    }
}

impl ToIr for CompUnitItem {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        match self {
            CompUnitItem::FuncDef(func_def) => func_def.to_ir(builder),
            CompUnitItem::Decl(_) => unimplemented!(),
        }
    }
}
