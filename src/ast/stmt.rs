use crate::{
    ir_builder::IRBuilder,
    traits::{to_ir::ExpToIr, ToIr},
};

use super::refactor::Stmt;
use anyhow::Result;
impl ToIr for Stmt {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        match self {
            Stmt::Return(exp) => match exp {
                Some(exp) => {
                    let val = exp.to_ir(builder)?;
                    builder.create_ret(Some(val));
                }
                None => {
                    builder.create_ret(None);
                }
            },
            Stmt::Assign(lval, exp) => {
                let addr = lval.get_address(builder)?;
                let val = exp.to_ir(builder)?;

                builder.create_store(addr, val)?;
            }
            Stmt::Exp(exp) => match exp {
                Some(exp) => {
                    exp.to_ir(builder)?;
                }
                None => {}
            },
            Stmt::Block(block) => block.to_ir(builder)?,
        }

        Ok(())
    }
}
