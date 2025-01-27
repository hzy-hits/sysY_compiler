use crate::{ir_builder::IRBuilder, traits::ToIr};

use super::refactor::{CompUnit, CompUnitItem};
use super::Result;
impl ToIr for CompUnit {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        self.items.iter().try_for_each(|item| item.to_ir(builder))?;
        Ok(())
    }
}

impl ToIr for CompUnitItem {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        match self {
            CompUnitItem::FuncDef(func_def) => func_def.to_ir(builder),
            CompUnitItem::Decl(_) => unimplemented!(),
        }
    }
}
