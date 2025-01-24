use crate::{ir_builder::IRBuilder, traits::ToIr};

use super::refactor::{Block, BlockItem};

impl ToIr for Block {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        for item in &self.items {
            item.to_ir(builder)?;
        }
        Ok(())
    }
}

impl ToIr for BlockItem {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        match self {
            BlockItem::Stmt(stmt) => stmt.to_ir(builder),
            BlockItem::Decl(decl) => decl.to_ir(builder),
        }
    }
}
