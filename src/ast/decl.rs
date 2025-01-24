use crate::{ir_builder::IRBuilder, traits::ToIr};

use super::Decl;

impl ToIr for Decl {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<(), String> {
        match self {
            Decl::VarDecl(var_decl, vec_defs) => {
                for var_def in vec_defs {
                    var_def.to_ir(builder)?;
                }
                Ok(())
            }
            Decl::ConstDecl(const_decl, vec_defs) => {
                for const_def in vec_defs {
                    const_def.to_ir(builder)?;
                }
                Ok(())
            }
        }
    }
}
