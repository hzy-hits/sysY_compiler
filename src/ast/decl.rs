use anyhow::ensure;
use anyhow::Context;
use koopa::ir::Type;

use super::BType;
use super::ConstDef;
use super::Decl;
use super::Result;
use super::VarDef;
use crate::semantic::SymbolKind;
use crate::traits::semantic::ConstEval;
use crate::traits::semantic::SymbolTable;
use crate::traits::to_ir::ExpToIr;
use crate::{ir_builder::IRBuilder, traits::ToIr};

impl ToIr for Decl {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        match self {
            Decl::VarDecl(_var_decl, vec_defs) => {
                for var_def in vec_defs {
                    var_def.to_ir(builder).with_context(|| {
                        format!("Failed to convert variable definition to IR: {:?}", var_def)
                    })?;
                }
                Ok(())
            }
            Decl::ConstDecl(_const_decl, vec_defs) => {
                for const_def in vec_defs {
                    const_def.to_ir(builder)?;
                }
                Ok(())
            }
        }
    }
}

impl ToIr for ConstDef {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        let val = self
            .value
            .eval_const(builder)
            .with_context(|| format!("Invalid const initializer for {}", self.id))?;
        if builder.lookup(&self.id).is_ok() {
            return Err(anyhow::anyhow!("Duplicate const definition: {}", self.id));
        }
        builder
            .add_symbol(&self.id, SymbolKind::Const(val))
            .with_context(|| format!("Failed to add const symbol {}", self.id))
    }
}

impl ToIr for VarDef {
    fn to_ir(&self, builder: &mut IRBuilder) -> Result<()> {
        let ty = match self.ty {
            BType::Int => Type::get_i32(),
        };
        if builder.lookup(&self.id).is_ok() {
            return Err(anyhow::anyhow!(
                "Duplicate variable definition: {}",
                self.id
            ));
        }
        let var_name = format!("@{}", self.id);

        let alloc = builder
            .create_alloc(ty.clone(), var_name)
            .with_context(|| format!("Failed to create alloc for {}", self.id))?;
        builder
            .add_symbol(&self.id, SymbolKind::Variable(alloc))
            .with_context(|| format!("Failed to add var symbol {}", self.id))?;

        if let Some(init) = &self.init_val {
            let init_val = init.exp.to_ir(builder)?;
            let init_ty = builder.value_type(init_val)?;
            ensure!(
                init_ty == ty,
                "Type mismatch: expected {:?}, got {:?}",
                ty,
                init_ty.clone()
            );
            builder
                .create_store(alloc, init_val)
                .with_context(|| format!("Failed to store initial value for {}", self.id))?;
        }

        Ok(())
    }
}
