use crate::{
    ir_builder::IRBuilder,
    traits::{to_ir::ExpToIr, ToIr},
};

use super::refactor::Stmt;

impl ToIr for Stmt {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        match self {
            Stmt::Return(exp) => {
                let val = exp.to_ir(builder)?;
                builder.create_ret(Some(val));
            }
            Stmt::Assign(_, _) => unimplemented!(),
        }

        Ok(())
    }
}
