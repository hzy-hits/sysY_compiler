use crate::traits::semantic::SymbolTable;
use crate::{ir_builder::IRBuilder, traits::ToIr};

use super::refactor::{Block, BlockItem};
use super::Result;
impl ToIr for Block {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        builder.enter_scope()?;
        for item in &self.items {
            item.to_ir(builder)?;
        }
        builder.exit_scope()?;
        Ok(())
    }
}

impl ToIr for BlockItem {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        match self {
            BlockItem::Stmt(stmt) => stmt.to_ir(builder),
            BlockItem::Decl(decl) => decl.to_ir(builder),
        }
    }
}
