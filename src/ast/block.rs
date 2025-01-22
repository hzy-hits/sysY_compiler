use crate::{ast::Stmt, ir_builder::IRBuilder, traits::ToIr};

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

impl ToIr for Block {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        self.stmt.to_ir(builder)?;
        Ok(())
    }
}
